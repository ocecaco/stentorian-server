extern crate stentorian;

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

extern crate bytes;
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;

mod rpc;
mod rpcimpl;
mod errors;
mod linecodec;
mod notifications;

use jsonrpc_core::IoHandler;
use stentorian::engine::Engine;
use std::sync::Arc;
use rpc::Rpc;
use rpcimpl::RpcImpl;
use errors::*;
use std::str;
use tokio_io::AsyncRead;
use futures::Future;
use futures::Stream;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use linecodec::LineCodec;
use futures::sync::mpsc;


fn run_server() -> Result<()> {
    let mut core = Core::new()?;
    let handle = core.handle();

    let addr = "0.0.0.0:1337".parse().unwrap();
    let listener = TcpListener::bind(&addr, &handle)?;

    let engine = Arc::new(Engine::connect()?);

    let server = listener.incoming().for_each(|(sock, _)| {
        let framed = AsyncRead::framed(sock, LineCodec);
        let (responses, requests) = framed.split();

        let (notifications_tx, notifications_rx) = mpsc::unbounded();
        let notifications_rx = notifications_rx
            .map_err(|()| panic!("channel receive should never fail"))
            .and_then(|r| r);

        let mut handler = IoHandler::new();
        let rpc = RpcImpl::new(engine.clone(), notifications_tx);
        handler.extend_with(rpc.to_delegate());

        let request_results = requests
            .and_then(move |r| {
                handler.handle_request(&r)
                    .map_err(|()| panic!("handle_request should never fail"))
            })
            .filter_map(|x| x)
            .from_err();

        let merged = request_results
            .select(notifications_rx);

        let handle_connection = merged.forward(responses)
            .map(|_| ())
            .map_err(|e| error!("{}", e));

        handle.spawn(handle_connection);
        Ok(())
    });

    core.run(server)?;

    Ok(())
}





fn serve() -> Result<()> {
    stentorian::initialize()?;
    run_server()
}

quick_main!(serve);

pub fn lib_main() {
    env_logger::init().expect("logger initialization failed");

    main();
}
