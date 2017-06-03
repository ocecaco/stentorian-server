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
use rustlink::engine::{GrammarControl, EngineRegistration, GrammarEvent, Recognition};
use jsonrpc_core::IoHandler;
use rustlink::engine::Engine;
use std::sync::{Arc, Mutex};
use std::io::{self, Read, Write, BufRead, BufReader};
use std::collections::HashMap;
use rpc::Rpc;
use errors::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

struct ConnectionState {
    grammar_count: u64,
    engine_count: u64,
    loaded_grammars: HashMap<u64, GrammarControl>,
    engine_registrations: HashMap<u64, EngineRegistration>,
}

struct RpcImpl {
    engine: Arc<Engine>,
    stream: Arc<Mutex<TcpStream>>,
    state: Mutex<ConnectionState>,
}

impl Rpc for RpcImpl {
    fn grammar_load(&self, grammar: Grammar, all_recognitions: bool) -> Result<u64> {
        let mut state = self.state.lock().unwrap();
        state.grammar_count += 1;
        let id = state.grammar_count;

        let callback = move |e| {
            match e {
                GrammarEvent::PhraseFinish(Some(Recognition { words, .. })) => {
                    let w = words
                        .iter()
                        .map(|&(ref w, _)| w as &str)
                        .collect::<Vec<_>>();

                    println!("{}", w.join(" "));
                }
                _ => {}
            }
        };

        let control = self.engine.grammar_load(&grammar, all_recognitions, callback)?;
        control.rule_activate("mapping")?;

        state.loaded_grammars.insert(id, control);

        Ok(id)
    }

    fn grammar_unload(&self, id: u64) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.loaded_grammars.remove(&id);
        Ok(())
    }
}

fn handle_connection(s: TcpStream, engine: Arc<Engine>) -> Result<()> {
    rustlink::initialize()?;

    let mut handler = IoHandler::new();

    let stream = Arc::new(Mutex::new(s.try_clone()?));

    let rpc = RpcImpl {
        engine: engine,
        stream: stream.clone(),
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
            Some(mut r) => {
                r.push('\n');
                let mut s = stream.lock().unwrap();
                s.write_all(r.as_bytes())?;
            }
        }
    }

    Ok(())
}

fn serve() -> Result<()> {
    rustlink::initialize()?;

    let listener = TcpListener::bind("0.0.0.0:1337")?;
    let engine = Arc::new(Engine::connect()?);

    for client in listener.incoming() {
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
