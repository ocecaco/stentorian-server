use stentorian::grammar::Grammar;
use stentorian::engine::{GrammarControl, EngineRegistration, GrammarEvent, MicrophoneState};
use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::HashMap;
use futures::sync::mpsc;
use stentorian::engine::Engine;
use errors::*;
use notifications::{GrammarNotification, EngineNotification, create_notification};
use rpc::Rpc;
use stentorian::resultparser::Matcher;

struct ConnectionState {
    grammar_count: u64,
    engine_count: u64,
    loaded_grammars: HashMap<u64, GrammarControl>,
    engine_registrations: HashMap<u64, EngineRegistration>,
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
                grammar_count: 0,
                engine_count: 0,
                loaded_grammars: HashMap::new(),
                engine_registrations: HashMap::new(),
            }),
        }
    }

    fn state(&self) -> MutexGuard<ConnectionState> {
        self.state.lock().expect("attempt to lock poisoned mutex")
    }
}

impl Rpc for RpcImpl {
    fn grammar_load(&self, grammar: Grammar, all_recognitions: bool) -> Result<u64> {
        let mut state = self.state();
        state.grammar_count += 1;
        let id = state.grammar_count;

        let notifications = self.notifications.clone();
        let matcher = Matcher::new(&grammar);

        let convert_event = move |e: &GrammarEvent| {
            let n = GrammarNotification::from_event(&matcher, e);
            Ok(create_notification(id, "grammar_notification", &n)?)
        };

        let callback = move |e| {
            let result = convert_event(&e);
            notifications.send(result).unwrap();
        };

        let control = self.engine.grammar_load(&grammar, all_recognitions, callback)?;
        state.loaded_grammars.insert(id, control);

        Ok(id)
    }

    fn grammar_unload(&self, id: u64) -> Result<()> {
        let mut state = self.state();
        state.loaded_grammars.remove(&id);
        Ok(())
    }

    fn grammar_rule_activate(&self, id: u64, name: String) -> Result<()> {
        let state = self.state();
        state.loaded_grammars[&id].rule_activate(&name)?;
        Ok(())
    }

    fn grammar_rule_deactivate(&self, id: u64, name: String) -> Result<()> {
        let state = self.state();
        state.loaded_grammars[&id].rule_deactivate(&name)?;
        Ok(())
    }

    fn grammar_list_append(&self, id: u64, name: String, word: String) -> Result<()> {
        let state = self.state();
        state.loaded_grammars[&id].list_append(&name, &word)?;
        Ok(())
    }

    fn grammar_list_remove(&self, id: u64, name: String, word: String) -> Result<()> {
        let state = self.state();
        state.loaded_grammars[&id].list_remove(&name, &word)?;
        Ok(())
    }

    fn grammar_list_clear(&self, id: u64, name: String) -> Result<()> {
        let state = self.state();
        state.loaded_grammars[&id].list_clear(&name)?;
        Ok(())
    }

    fn engine_register(&self) -> Result<u64> {
        let mut state = self.state();
        state.engine_count += 1;
        let id = state.engine_count;

        let notifications = self.notifications.clone();
        let engine = self.engine.clone();

        let convert_event = move |e| {
            let n = EngineNotification::from_event(&engine, e)?;
            Ok(create_notification(id, "engine_notification", &n)?)
        };

        let callback = move |e| {
            let result = convert_event(e);
            notifications.send(result).unwrap();
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