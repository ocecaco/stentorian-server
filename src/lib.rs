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
use jsonrpc_core::MetaIoHandler;
use rustlink::engine::Engine;
use std::sync::{Arc, Mutex};
use std::io;
use std::collections::HashMap;
use rpc::Rpc;
use errors::*;

struct ConnectionState {
    grammar_count: u64,
    engine_count: u64,
    loaded_grammars: HashMap<u64, GrammarControl>,
    engine_registrations: HashMap<u64, EngineRegistration>,
}

struct RpcImpl {
    engine: Arc<Engine>,
    state: Mutex<ConnectionState>,
}

impl Rpc for RpcImpl {
    fn rpc_grammar_load(&self, grammar: Grammar, all_recognitions: bool) -> RpcResult<u64> {
        Ok(self.grammar_load(grammar, all_recognitions)?)
    }

    fn rpc_grammar_unload(&self, id: u64) -> RpcResult<()> {
        Ok(self.grammar_unload(id)?)
    }
}

impl RpcImpl {
    fn grammar_load(&self, grammar: Grammar, all_recognitions: bool) -> Result<u64> {
        Ok(0)
    }

    fn grammar_unload(&self, id: u64) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.loaded_grammars.remove(&id);
        Ok(())
    }
}

pub fn main() {
    env_logger::init().unwrap();
}
