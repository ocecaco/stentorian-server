extern crate stentorian;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate jsonrpc_core;

#[macro_use]
extern crate jsonrpc_macros;

extern crate env_logger;
#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

extern crate bytes;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;

mod rpc;
mod rpcimpl;
mod errors;
mod linecodec;
mod notifications;

use errors::*;
use futures::Future;
use futures::Stream;
use futures::stream;
use futures::sync::mpsc;
use jsonrpc_core::IoHandler;
use linecodec::LineCodec;
use rpc::Rpc;
use rpcimpl::RpcImpl;
use std::env;
use std::sync::Arc;
use stentorian::engine::Engine;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_io::AsyncRead;

fn run_server(host_port: &str) -> Result<()> {
    let mut core = Core::new()?;
    let handle = core.handle();

    let addr = host_port.parse().unwrap();
    let listener = TcpListener::bind(&addr, &handle)?;

    info!("listening for connections");

    let engine = Arc::new(Engine::connect()?);

    let server = listener.incoming().for_each(|(sock, _)| {
        info!("new connection");
        let framed = AsyncRead::framed(sock, LineCodec);
        let (responses, requests) = framed.split();

        let (notifications_tx, notifications_rx) = mpsc::unbounded();
        let notifications_rx = notifications_rx
            .map_err(|()| panic!("channel receive should never fail"))
            .and_then(|r| r)
            .map(|x| Some(x))
            .chain(stream::once(Ok(None)));

        let mut handler = IoHandler::new();
        let rpc = RpcImpl::new(engine.clone(), notifications_tx);
        handler.extend_with(rpc.to_delegate());

        let request_results = requests
            .and_then(move |r| {
                handler
                    .handle_request(&r)
                    .map_err(|()| panic!("handle_request should never fail"))
            })
            .filter_map(|x| x)
            .from_err()
            .map(|x| Some(x))
            .chain(stream::once(Ok(None)));

        let merged = request_results
            .select(notifications_rx)
            .take_while(|x| Ok(x.is_some()))
            .filter_map(|x| x);

        let handle_connection = merged.forward(responses).then(|r: Result<_>| {
            match r {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                }
            }

            info!("connection closed");
            Ok(())
        });

        handle.spawn(handle_connection);
        Ok(())
    });

    core.run(server)?;

    Ok(())
}

fn serve() -> Result<()> {
    stentorian::initialize()?;
    let args: Vec<String> = env::args().collect();
    run_server(&args[1])
}

quick_main!(serve);

pub fn lib_main() {
    env_logger::init().expect("logger initialization failed");

    main();
}
