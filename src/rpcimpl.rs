use errors::*;
use futures::sync::mpsc;
use notifications::{create_notification, EngineNotification};
use rpc::Rpc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use stentorian::engine::{CatchallGrammarControl, CommandGrammarControl, DictationGrammarControl,
                         EngineRegistration, MicrophoneState, SelectGrammarControl};
use stentorian::engine::Engine;
use stentorian::grammar::Grammar;

struct ConnectionState {
    counter: u64,
    command_grammars: HashMap<u64, CommandGrammarControl>,
    select_grammars: HashMap<u64, SelectGrammarControl>,
    dictation_grammars: HashMap<u64, DictationGrammarControl>,
    catchall_grammars: HashMap<u64, CatchallGrammarControl>,
    engine_registrations: HashMap<u64, EngineRegistration>,
}

impl ConnectionState {
    fn command_grammar(&self, id: u64) -> Result<&CommandGrammarControl> {
        Ok(self.command_grammars
            .get(&id)
            .ok_or(ErrorKind::UnknownEntityId(id))?)
    }

    fn select_grammar(&self, id: u64) -> Result<&SelectGrammarControl> {
        Ok(self.select_grammars
            .get(&id)
            .ok_or(ErrorKind::UnknownEntityId(id))?)
    }

    fn dictation_grammar(&self, id: u64) -> Result<&DictationGrammarControl> {
        Ok(self.dictation_grammars
            .get(&id)
            .ok_or(ErrorKind::UnknownEntityId(id))?)
    }

    fn catchall_grammar(&self, id: u64) -> Result<&CatchallGrammarControl> {
        Ok(self.catchall_grammars
            .get(&id)
            .ok_or(ErrorKind::UnknownEntityId(id))?)
    }

    fn new_id(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }
}

pub struct RpcImpl {
    engine: Arc<Engine>,
    notifications: mpsc::UnboundedSender<Result<String>>,
    state: Mutex<ConnectionState>,
}

impl RpcImpl {
    pub fn new(engine: Arc<Engine>, notifications: mpsc::UnboundedSender<Result<String>>) -> Self {
        RpcImpl {
            engine: engine,
            notifications: notifications,
            state: Mutex::new(ConnectionState {
                counter: 0,
                command_grammars: HashMap::new(),
                select_grammars: HashMap::new(),
                dictation_grammars: HashMap::new(),
                catchall_grammars: HashMap::new(),
                engine_registrations: HashMap::new(),
            }),
        }
    }

    fn state(&self) -> MutexGuard<ConnectionState> {
        self.state.lock().expect("attempt to lock poisoned mutex")
    }
}

static GRAMMAR_NOTIFICATION: &'static str = "grammar_notification";

impl Rpc for RpcImpl {
    fn command_grammar_load(&self, grammar: Grammar) -> Result<u64> {
        let mut state = self.state();
        let id = state.new_id();
        let notifications = self.notifications.clone();

        let callback = move |e| {
            let result = create_notification(id, GRAMMAR_NOTIFICATION, &e);
            notifications.unbounded_send(result).unwrap();
        };

        let control = self.engine.command_grammar_load(&grammar, callback)?;
        state.command_grammars.insert(id, control);

        Ok(id)
    }

    fn select_grammar_load(
        &self,
        start_words: Vec<String>,
        through_words: Vec<String>,
    ) -> Result<u64> {
        let mut state = self.state();
        let id = state.new_id();
        let notifications = self.notifications.clone();

        let callback = move |e| {
            let result = create_notification(id, GRAMMAR_NOTIFICATION, &e);
            notifications.unbounded_send(result).unwrap();
        };

        let control = self.engine
            .select_grammar_load(&start_words, &through_words, callback)?;
        state.select_grammars.insert(id, control);

        Ok(id)
    }

    fn command_grammar_unload(&self, id: u64) -> Result<()> {
        let mut state = self.state();
        state.command_grammars.remove(&id);
        Ok(())
    }

    fn command_grammar_rule_activate(&self, id: u64, name: String) -> Result<()> {
        let state = self.state();
        state.command_grammar(id)?.rule_activate(&name)?;
        Ok(())
    }

    fn command_grammar_rule_deactivate(&self, id: u64, name: String) -> Result<()> {
        let state = self.state();
        state.command_grammar(id)?.rule_deactivate(&name)?;
        Ok(())
    }

    fn command_grammar_list_append(&self, id: u64, name: String, word: String) -> Result<()> {
        let state = self.state();
        state.command_grammar(id)?.list_append(&name, &word)?;
        Ok(())
    }

    fn command_grammar_list_remove(&self, id: u64, name: String, word: String) -> Result<()> {
        let state = self.state();
        state.command_grammar(id)?.list_remove(&name, &word)?;
        Ok(())
    }

    fn command_grammar_list_clear(&self, id: u64, name: String) -> Result<()> {
        let state = self.state();
        state.command_grammar(id)?.list_clear(&name)?;
        Ok(())
    }

    fn select_grammar_unload(&self, id: u64) -> Result<()> {
        let mut state = self.state();
        state.select_grammars.remove(&id);
        Ok(())
    }

    fn select_grammar_activate(&self, id: u64) -> Result<()> {
        let state = self.state();
        state.select_grammar(id)?.activate()?;
        Ok(())
    }

    fn select_grammar_deactivate(&self, id: u64) -> Result<()> {
        let state = self.state();
        state.select_grammar(id)?.deactivate()?;
        Ok(())
    }

    fn select_grammar_text_set(&self, id: u64, text: String) -> Result<()> {
        let state = self.state();
        state.select_grammar(id)?.text_set(&text)?;
        Ok(())
    }

    fn select_grammar_text_change(
        &self,
        id: u64,
        start: u32,
        stop: u32,
        text: String,
    ) -> Result<()> {
        let state = self.state();
        state.select_grammar(id)?.text_change(start, stop, &text)?;
        Ok(())
    }

    fn select_grammar_text_delete(&self, id: u64, start: u32, stop: u32) -> Result<()> {
        let state = self.state();
        state.select_grammar(id)?.text_delete(start, stop)?;
        Ok(())
    }

    fn select_grammar_text_insert(&self, id: u64, start: u32, text: String) -> Result<()> {
        let state = self.state();
        state.select_grammar(id)?.text_insert(start, &text)?;
        Ok(())
    }

    fn select_grammar_text_get(&self, id: u64) -> Result<String> {
        let state = self.state();
        let text = state.select_grammar(id)?.text_get()?;
        Ok(text)
    }

    fn dictation_grammar_load(&self) -> Result<u64> {
        let mut state = self.state();
        let id = state.new_id();
        let notifications = self.notifications.clone();

        let callback = move |e| {
            let result = create_notification(id, GRAMMAR_NOTIFICATION, &e);
            notifications.unbounded_send(result).unwrap();
        };

        let control = self.engine.dictation_grammar_load(callback)?;
        state.dictation_grammars.insert(id, control);

        Ok(id)
    }

    fn dictation_grammar_unload(&self, id: u64) -> Result<()> {
        let mut state = self.state();
        state.dictation_grammars.remove(&id);
        Ok(())
    }

    fn dictation_grammar_activate(&self, id: u64) -> Result<()> {
        let state = self.state();
        state.dictation_grammar(id)?.activate()?;
        Ok(())
    }

    fn dictation_grammar_deactivate(&self, id: u64) -> Result<()> {
        let state = self.state();
        state.dictation_grammar(id)?.deactivate()?;
        Ok(())
    }

    fn dictation_grammar_context_set(&self, id: u64, context: String) -> Result<()> {
        let state = self.state();
        state.dictation_grammar(id)?.context_set(&context)?;
        Ok(())
    }

    fn catchall_grammar_load(&self) -> Result<u64> {
        let mut state = self.state();
        let id = state.new_id();
        let notifications = self.notifications.clone();

        let callback = move |e| {
            let result = create_notification(id, GRAMMAR_NOTIFICATION, &e);
            notifications.unbounded_send(result).unwrap();
        };

        let control = self.engine.catchall_grammar_load(callback)?;
        state.catchall_grammars.insert(id, control);

        Ok(id)
    }

    fn catchall_grammar_unload(&self, id: u64) -> Result<()> {
        let mut state = self.state();
        state.catchall_grammars.remove(&id);
        Ok(())
    }

    fn catchall_grammar_activate(&self, id: u64) -> Result<()> {
        let state = self.state();
        state.catchall_grammar(id)?.activate()?;
        Ok(())
    }

    fn catchall_grammar_deactivate(&self, id: u64) -> Result<()> {
        let state = self.state();
        state.catchall_grammar(id)?.deactivate()?;
        Ok(())
    }

    fn engine_register(&self) -> Result<u64> {
        let mut state = self.state();
        let id = state.new_id();

        let notifications = self.notifications.clone();
        let engine = self.engine.clone();

        let convert_event = move |e| {
            let n = EngineNotification::from_event(&engine, e)?;
            Ok(create_notification(id, "engine_notification", &n)?)
        };

        let callback = move |e| {
            let result = convert_event(e);
            notifications.unbounded_send(result).unwrap();
        };

        let registration = self.engine.register(callback)?;
        state.engine_registrations.insert(id, registration);

        Ok(id)
    }

    fn engine_unregister(&self, id: u64) -> Result<()> {
        let mut state = self.state();
        state.engine_registrations.remove(&id);
        Ok(())
    }

    fn microphone_set_state(&self, state: MicrophoneState) -> Result<()> {
        Ok(self.engine.microphone_set_state(state)?)
    }

    fn microphone_get_state(&self) -> Result<MicrophoneState> {
        Ok(self.engine.microphone_get_state()?)
    }

    fn get_current_user(&self) -> Result<Option<String>> {
        Ok(self.engine.get_current_user()?)
    }
}
