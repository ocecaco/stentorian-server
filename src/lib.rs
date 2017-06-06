extern crate rustlink;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate jsonrpc_core;

#[macro_use]
extern crate jsonrpc_macros;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate error_chain;

mod rpc;
mod errors;

use rustlink::grammar::Grammar;
use rustlink::engine::{GrammarControl, EngineRegistration, GrammarEvent, Recognition, EngineEvent,
                       Attribute, MicrophoneState};
use rustlink::resultparser::{Matcher, Match};
use jsonrpc_core::{Notification, Version, IoHandler, Params};
use rustlink::engine::Engine;
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use rpc::Rpc;
use errors::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use responsesink::ResponseSink;
use serde::Serialize;

mod responsesink {
    use std::net::TcpStream;
    use std::sync::{Arc, Mutex};
    use errors::*;
    use std::io::Write;

    #[derive(Debug, Clone)]
    pub struct ResponseSink {
        stream: Arc<Mutex<TcpStream>>,
    }

    impl ResponseSink {
        pub fn new(stream: TcpStream) -> Self {
            let s = Arc::new(Mutex::new(stream));

            ResponseSink { stream: s }
        }

        pub fn send(&self, response: &str) -> Result<()> {
            let mut s = self.stream.lock().unwrap();
            s.write_all(response.as_bytes())?;
            s.write_all(b"\n")?;
            Ok(())
        }
    }
}

struct ConnectionState {
    grammar_count: u64,
    engine_count: u64,
    loaded_grammars: HashMap<u64, GrammarControl>,
    engine_registrations: HashMap<u64, EngineRegistration>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(unknown_lints, enum_variant_names)]
enum GrammarNotification<'a, 'c: 'b, 'b> {
    PhraseFinish {
        foreign_grammar: bool,
        words: &'b [&'c str],
        parse: Option<Match<'a>>,
    },
    PhraseRecognitionFailure,
    PhraseStart,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EngineNotification {
    Paused,
    MicrophoneStateChanged { state: MicrophoneState },
}

fn create_notification<E>(id: u64, method: &str, event: &E) -> Result<Notification>
    where E: Serialize
{
    let v_event = serde_json::to_value(event)?;
    let v_id = serde_json::to_value(&id)?;
    let p = Params::Array(vec![v_id, v_event]);
    let n = Notification {
        jsonrpc: Some(Version::V2),
        method: method.to_owned(),
        params: Some(p),
    };

    Ok(n)
}

struct RpcImpl {
    engine: Arc<Engine>,
    responses: ResponseSink,
    state: Mutex<ConnectionState>,
}

impl Rpc for RpcImpl {
    fn grammar_load(&self, grammar: Grammar, all_recognitions: bool) -> Result<u64> {
        let mut state = self.state.lock().unwrap();
        state.grammar_count += 1;
        let id = state.grammar_count;

        let matcher = Matcher::new(&grammar);
        let responses = self.responses.clone();

        let callback = move |e| {
            static METHOD: &str = "grammar_notification";

            match e {
                GrammarEvent::PhraseFinish(Some(Recognition {
                                                    words: words_with_id,
                                                    foreign,
                                                })) => {
                    let parse = if !foreign {
                        matcher.perform_match(&words_with_id)
                    } else {
                        None
                    };

                    let words_only = words_with_id
                        .iter()
                        .map(|&(ref w, _)| w as &str)
                        .collect::<Vec<_>>();

                    let g = GrammarNotification::PhraseFinish {
                        foreign_grammar: foreign,
                        words: &words_only,
                        parse: parse,
                    };

                    let n = create_notification(id, METHOD, &g).unwrap();
                    responses
                        .send(&serde_json::to_string(&n).unwrap())
                        .unwrap();
                }
                GrammarEvent::PhraseFinish(None) => {
                    let g = GrammarNotification::PhraseRecognitionFailure;
                    let n = create_notification(id, METHOD, &g).unwrap();
                    responses
                        .send(&serde_json::to_string(&n).unwrap())
                        .unwrap();
                }
                GrammarEvent::PhraseStart => {
                    let g = GrammarNotification::PhraseStart;
                    let n = create_notification(id, METHOD, &g).unwrap();
                    responses
                        .send(&serde_json::to_string(&n).unwrap())
                        .unwrap();
                }
            }
        };

        let control = self.engine
            .grammar_load(&grammar, all_recognitions, callback)?;

        state.loaded_grammars.insert(id, control);

        Ok(id)
    }

    fn grammar_unload(&self, id: u64) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.loaded_grammars.remove(&id);
        Ok(())
    }

    fn grammar_rule_activate(&self, id: u64, name: String) -> Result<()> {
        let state = self.state.lock().unwrap();
        state.loaded_grammars[&id].rule_activate(&name)?;
        Ok(())
    }

    fn grammar_rule_deactivate(&self, id: u64, name: String) -> Result<()> {
        let state = self.state.lock().unwrap();
        state.loaded_grammars[&id].rule_deactivate(&name)?;
        Ok(())
    }

    fn grammar_list_append(&self, id: u64, name: String, word: String) -> Result<()> {
        let state = self.state.lock().unwrap();
        state.loaded_grammars[&id].list_append(&name, &word)?;
        Ok(())
    }

    fn grammar_list_remove(&self, id: u64, name: String, word: String) -> Result<()> {
        let state = self.state.lock().unwrap();
        state.loaded_grammars[&id].list_remove(&name, &word)?;
        Ok(())
    }

    fn grammar_list_clear(&self, id: u64, name: String) -> Result<()> {
        let state = self.state.lock().unwrap();
        state.loaded_grammars[&id].list_clear(&name)?;
        Ok(())
    }

    fn engine_register(&self) -> Result<u64> {
        let mut state = self.state.lock().unwrap();
        state.engine_count += 1;
        let id = state.engine_count;

        let responses = self.responses.clone();
        let engine = self.engine.clone();

        let callback = move |e| {
            static METHOD: &str = "engine_notification";

            match e {
                EngineEvent::Paused(cookie) => {
                    engine.resume(cookie).unwrap();

                    let event = EngineNotification::Paused;

                    let n = create_notification(id, METHOD, &event).unwrap();
                    responses
                        .send(&serde_json::to_string(&n).unwrap())
                        .unwrap();
                }
                EngineEvent::AttributeChanged(a) => {
                    let event = match a {
                        Attribute::MicrophoneState => {
                            let state = engine.microphone_get_state().unwrap();
                            EngineNotification::MicrophoneStateChanged { state }
                        }
                    };

                    let n = create_notification(id, METHOD, &event).unwrap();
                    responses
                        .send(&serde_json::to_string(&n).unwrap())
                        .unwrap();
                }
            }
        };

        let registration = self.engine.register(callback)?;
        state.engine_registrations.insert(id, registration);

        Ok(id)
    }

    fn engine_unregister(&self, id: u64) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.engine_registrations.remove(&id);
        Ok(())
    }

    fn microphone_set_state(&self, state: MicrophoneState) -> Result<()> {
        Ok(self.engine.microphone_set_state(state)?)
    }

    fn microphone_get_state(&self) -> Result<MicrophoneState> {
        Ok(self.engine.microphone_get_state()?)
    }
}

fn handle_connection(s: TcpStream, engine: Arc<Engine>) -> Result<()> {
    rustlink::initialize()?;

    let mut handler = IoHandler::new();

    let stream = s.try_clone()?;
    let responses = ResponseSink::new(stream);

    let rpc = RpcImpl {
        engine: engine,
        responses: responses.clone(),
        state: Mutex::new(ConnectionState {
                              grammar_count: 0,
                              engine_count: 0,
                              loaded_grammars: HashMap::new(),
                              engine_registrations: HashMap::new(),
                          }),
    };

    handler.extend_with(rpc.to_delegate());

    let reader = BufReader::new(s);
    for request in reader.lines() {
        let response = handler.handle_request_sync(&request?);

        match response {
            None => {}
            Some(r) => {
                responses.send(&r)?;
            }
        }
    }

    Ok(())
}

fn serve() -> Result<()> {
    rustlink::initialize()?;

    let listener = TcpListener::bind("0.0.0.0:1337")?;
    info!("Listening on 0.0.0.0:1337");
    let engine = Arc::new(Engine::connect()?);

    for client in listener.incoming() {
        info!("Accepted new connection");
        let engine_clone = engine.clone();
        let stream = client?;
        thread::spawn(move || handle_connection(stream, engine_clone));
    }

    Ok(())
}

pub fn main() {
    env_logger::init().unwrap();

    serve().unwrap();
}
