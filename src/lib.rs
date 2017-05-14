extern crate rustlink;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate jsonrpc_core;

#[macro_use]
extern crate jsonrpc_macros;

extern crate tokio_core;
extern crate tokio_service;
extern crate tokio_proto;
extern crate tokio_io;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate error_chain;

extern crate futures;
extern crate bytes;

mod linecodec;
mod rpc;
mod errors;

use rustlink::grammar::Grammar;
use rustlink::engine::{GrammarControl, EngineRegistration};
use jsonrpc_core::MetaIoHandler;
use rustlink::engine::Engine;
use std::sync::{Arc, Mutex};
use futures::{BoxFuture, Future, Stream, Sink};
use futures::sync::mpsc::{self, Sender};
use std::io;
use std::collections::HashMap;
use tokio_core::reactor::{Core, Remote};
use tokio_core::net::TcpListener;
use tokio_io::AsyncRead;
use rpc::Rpc;
use errors::*;
use linecodec::LineCodec;

fn serve() -> Result<()>
{
    rustlink::initialize()?;

    let mut core = Core::new()?;
    let handle = core.handle();
    let remote = core.remote();

    let address = "0.0.0.0:12345".parse().unwrap();
    let listener = TcpListener::bind(&address, &handle)?;

    let engine = Arc::new(Engine::connect()?);

    let connections = listener.incoming();
    let server = connections.for_each(move |(socket, _peer_addr)| {
        let (writer, reader) = socket.framed(LineCodec).split();

        let (tx, rx) = mpsc::channel(512);

        let rpc = RpcImpl {
            engine: engine.clone(),
            remote: remote.clone(),
            notifications: tx,
            state: Mutex::new(ConnectionState {
                grammar_count: 0,
                engine_count: 0,
                loaded_grammars: HashMap::new(),
                engine_registrations: HashMap::new(),
            }),
        };

        let mut io = MetaIoHandler::<()>::default();
        io.extend_with(rpc.to_delegate());

        let responses = reader
            .and_then(move |req| {
                io.handle_request(&req, ())
                    .map_err(|_| panic!("handle_request should never fail"))
            })
            .filter_map(|maybe_response| maybe_response);

        let notifications = rx.map_err(|_| panic!("should not error"));

        let merged = responses.select(notifications);

        let server = writer.send_all(merged)
            .then(|_| Ok(()));
        handle.spawn(server);

        Ok(())
    });

    core.run(server)?;

    Ok(())
}

struct ConnectionState {
    grammar_count: u64,
    engine_count: u64,
    loaded_grammars: HashMap<u64, GrammarControl>,
    engine_registrations: HashMap<u64, EngineRegistration>,
}


struct RpcImpl {
    engine: Arc<Engine>,
    remote: Remote,
    notifications: Sender<String>,
    state: Mutex<ConnectionState>,
}

impl Rpc for RpcImpl {
    fn rpc_grammar_load(&self, grammar: Grammar, all_recognitions: bool) -> RpcResult<String> {
        Ok(self.grammar_load(grammar, all_recognitions)?)
    }

    fn rpc_grammar_unload(&self, id: String) -> RpcResult<()> {
        Ok(self.grammar_unload(id)?)
    }
}

impl RpcImpl {
    fn grammar_load(&self, grammar: Grammar, all_recognitions: bool) -> Result<String> {
        let (control, receiver) = self.engine.grammar_load(&grammar, all_recognitions)?;
        let mut state = self.state.lock().unwrap();
        state.grammar_count += 1;
        state.loaded_grammars.insert(state.grammar_count, control);

        let notifications = receiver.and_then(|event| {

        });
    }

    fn grammar_unload(&self, id: String) -> Result<()> {
        Ok(())
    }
}

pub fn main() {
    env_logger::init().unwrap();

    serve().unwrap();
}
