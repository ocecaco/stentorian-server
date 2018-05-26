use errors::{Result};
use futures::sync::mpsc;
use notifications::{create_notification, EngineNotification};
use rpc::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use stentorian::engine::{CatchallGrammarControl, CommandGrammarControl, CommandGrammarEvent,
                         DictationGrammarControl, Engine, EngineRegistration, MicrophoneState,
                         SelectGrammarControl};
use stentorian::grammar::Grammar;
use stentorian::resultparser::Matcher;

struct ConnectionState<T> {
    counter: u64,
    items: HashMap<u64, T>,
}

impl<T> ConnectionState<T> {
    fn new() -> Self {
        ConnectionState {
            counter: 0,
            items: HashMap::new(),
        }
    }

    fn new_id(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    fn insert(&mut self, id: u64, item: T) {
        self.items.insert(id, item);
    }

    fn remove(&mut self, id: u64) -> Result<()> {
        // TODO: error if it doesn't exist
        self.items.remove(&id);
        Ok(())
    }

    fn lookup(&self, id: u64) -> Result<&T> {
        // TODO: error if it doesn't exist
        Ok(&self.items[&id])
    }
}

pub struct RpcHelper<T> {
    engine: Arc<Engine>,
    notifications: mpsc::UnboundedSender<Result<String>>,
    state: Mutex<ConnectionState<T>>,
}

impl<T> RpcHelper<T> {
    pub fn new(engine: Arc<Engine>, notifications: mpsc::UnboundedSender<Result<String>>) -> Self {
        RpcHelper {
            engine: engine,
            notifications: notifications,
            state: Mutex::new(ConnectionState::new()),
        }
    }

    fn state(&self) -> MutexGuard<ConnectionState<T>> {
        self.state.lock().expect("attempt to lock poisoned mutex")
    }
}

pub struct RpcCommandImpl(pub RpcHelper<CommandGrammarControl>);
pub struct RpcSelectImpl(pub RpcHelper<SelectGrammarControl>);
pub struct RpcDictationImpl(pub RpcHelper<DictationGrammarControl>);
pub struct RpcCatchallImpl(pub RpcHelper<CatchallGrammarControl>);
pub struct RpcEngineImpl(pub RpcHelper<EngineRegistration>);

impl RpcCommand for RpcCommandImpl {
    fn load(&self, grammar: Grammar) -> Result<u64> {
        let mut state = self.0.state();
        let id = state.new_id();
        let notifications = self.0.notifications.clone();
        let matcher = Matcher::new(&grammar);

        let callback = move |e: CommandGrammarEvent| {
            let with_matches = e.map(|words| {
                let matches = matcher.perform_match(&words);
                (words, matches)
            });
            let result = create_notification(id, "command_grammar_notification", &with_matches);
            notifications.unbounded_send(result).unwrap();
        };

        let control = self.0.engine.command_grammar_load(&grammar, callback)?;
        state.insert(id, control);

        Ok(id)
    }

    fn unload(&self, id: u64) -> Result<()> {
        let mut state = self.0.state();
        state.remove(id)?;
        Ok(())
    }

    fn rule_activate(&self, id: u64, name: String) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.rule_activate(&name)?;
        Ok(())
    }

    fn rule_deactivate(&self, id: u64, name: String) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.rule_deactivate(&name)?;
        Ok(())
    }

    fn list_append(&self, id: u64, name: String, word: String) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.list_append(&name, &word)?;
        Ok(())
    }

    fn list_remove(&self, id: u64, name: String, word: String) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.list_remove(&name, &word)?;
        Ok(())
    }

    fn list_clear(&self, id: u64, name: String) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.list_clear(&name)?;
        Ok(())
    }
}

impl RpcSelect for RpcSelectImpl {
    fn load(&self, start_words: Vec<String>, through_words: Vec<String>) -> Result<u64> {
        let mut state = self.0.state();
        let id = state.new_id();
        let notifications = self.0.notifications.clone();

        let callback = move |e| {
            let result = create_notification(id, "select_grammar_notification", &e);
            notifications.unbounded_send(result).unwrap();
        };

        let control = self.0
            .engine
            .select_grammar_load(&start_words, &through_words, callback)?;
        state.insert(id, control);

        Ok(id)
    }

    fn unload(&self, id: u64) -> Result<()> {
        let mut state = self.0.state();
        state.remove(id)?;
        Ok(())
    }

    fn activate(&self, id: u64) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.activate()?;
        Ok(())
    }

    fn deactivate(&self, id: u64) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.deactivate()?;
        Ok(())
    }

    fn text_set(&self, id: u64, text: String) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.text_set(&text)?;
        Ok(())
    }

    fn text_change(&self, id: u64, start: u32, stop: u32, text: String) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.text_change(start, stop, &text)?;
        Ok(())
    }

    fn text_delete(&self, id: u64, start: u32, stop: u32) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.text_delete(start, stop)?;
        Ok(())
    }

    fn text_insert(&self, id: u64, start: u32, text: String) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.text_insert(start, &text)?;
        Ok(())
    }

    fn text_get(&self, id: u64) -> Result<String> {
        let state = self.0.state();
        let text = state.lookup(id)?.text_get()?;
        Ok(text)
    }
}

impl RpcDictation for RpcDictationImpl {
    fn load(&self) -> Result<u64> {
        let mut state = self.0.state();
        let id = state.new_id();
        let notifications = self.0.notifications.clone();

        let callback = move |e| {
            let result = create_notification(id, "dictation_grammar_notification", &e);
            notifications.unbounded_send(result).unwrap();
        };

        let control = self.0.engine.dictation_grammar_load(callback)?;
        state.insert(id, control);

        Ok(id)
    }

    fn unload(&self, id: u64) -> Result<()> {
        let mut state = self.0.state();
        state.remove(id)?;
        Ok(())
    }

    fn activate(&self, id: u64) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.activate()?;
        Ok(())
    }

    fn deactivate(&self, id: u64) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.deactivate()?;
        Ok(())
    }

    fn context_set(&self, id: u64, context: String) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.context_set(&context)?;
        Ok(())
    }
}

impl RpcCatchall for RpcCatchallImpl {
    fn load(&self) -> Result<u64> {
        let mut state = self.0.state();
        let id = state.new_id();
        let notifications = self.0.notifications.clone();

        let callback = move |e| {
            let result = create_notification(id, "catchall_grammar_notification", &e);
            notifications.unbounded_send(result).unwrap();
        };

        let control = self.0.engine.catchall_grammar_load(callback)?;
        state.insert(id, control);

        Ok(id)
    }

    fn unload(&self, id: u64) -> Result<()> {
        let mut state = self.0.state();
        state.remove(id)?;
        Ok(())
    }

    fn activate(&self, id: u64) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.activate()?;
        Ok(())
    }

    fn deactivate(&self, id: u64) -> Result<()> {
        let state = self.0.state();
        state.lookup(id)?.deactivate()?;
        Ok(())
    }
}

impl RpcEngine for RpcEngineImpl {
    fn register(&self) -> Result<u64> {
        let mut state = self.0.state();
        let id = state.new_id();

        let notifications = self.0.notifications.clone();
        let engine = self.0.engine.clone();

        let convert_event = move |e| {
            let n = EngineNotification::from_event(&engine, e)?;
            Ok(create_notification(id, "engine_notification", &n)?)
        };

        let callback = move |e| {
            let result = convert_event(e);
            notifications.unbounded_send(result).unwrap();
        };

        let registration = self.0.engine.register(callback)?;
        state.insert(id, registration);

        Ok(id)
    }

    fn unregister(&self, id: u64) -> Result<()> {
        let mut state = self.0.state();
        state.remove(id)?;
        Ok(())
    }

    fn microphone_set_state(&self, state: MicrophoneState) -> Result<()> {
        Ok(self.0.engine.microphone_set_state(state)?)
    }

    fn microphone_get_state(&self) -> Result<MicrophoneState> {
        Ok(self.0.engine.microphone_get_state()?)
    }

    fn get_current_user(&self) -> Result<Option<String>> {
        Ok(self.0.engine.get_current_user()?)
    }
}
