#![cfg(windows)]
#![cfg(target_arch = "x86")]
#![cfg(target_env = "msvc")]
mod errors;
mod linecodec;
mod notifications;
mod rpc;
mod rpcimpl;

use crate::errors::*;
use crate::linecodec::LineCodec;
use crate::rpc::*;
use crate::rpcimpl::*;
use futures::stream;
use futures::sync::mpsc;
use futures::Future;
use futures::Stream;
use jsonrpc_core::IoHandler;
use log::{error, info};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::{thread, time};
use stentorian::engine::Engine;
use structopt::StructOpt;
use tokio_codec::Framed;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

#[derive(StructOpt, Debug)]
#[structopt(name = "server")]
struct Opt {
    #[structopt(short = "H", long = "host")]
    host: IpAddr,
    #[structopt(short = "p", long = "port")]
    port: u16,
    #[structopt(short = "w", long = "wait")]
    wait_seconds: Option<u64>,
}

fn create_handler(
    engine: Arc<Engine>,
    notifications: mpsc::UnboundedSender<Result<String>>,
) -> IoHandler {
    let mut handler = IoHandler::new();
    let rpc_command = RpcCommandImpl(RpcHelper::new(engine.clone(), notifications.clone()));
    let rpc_select = RpcSelectImpl(RpcHelper::new(engine.clone(), notifications.clone()));
    let rpc_dictation = RpcDictationImpl(RpcHelper::new(engine.clone(), notifications.clone()));
    let rpc_catchall = RpcCatchallImpl(RpcHelper::new(engine.clone(), notifications.clone()));
    let rpc_engine = RpcEngineImpl(RpcHelper::new(engine.clone(), notifications));

    handler.extend_with(rpc_command.to_delegate());
    handler.extend_with(rpc_select.to_delegate());
    handler.extend_with(rpc_dictation.to_delegate());
    handler.extend_with(rpc_catchall.to_delegate());
    handler.extend_with(rpc_engine.to_delegate());

    handler
}

fn run_server(options: Opt) -> Result<()> {
    let mut core = Core::new()?;
    let handle = core.handle();

    let addr = SocketAddr::new(options.host, options.port);
    let listener = TcpListener::bind(&addr, &handle)?;

    info!("listening for connections on {}", addr);

    let engine = Arc::new(Engine::connect()?);

    let server = listener.incoming().for_each(|(sock, _)| {
        info!("new connection");
        // let framed = AsyncRead::framed(sock, LineCodec);
        let framed = Framed::new(sock, LineCodec);
        let (responses, requests) = framed.split();

        let (notifications_tx, notifications_rx) = mpsc::unbounded();
        let notifications_rx = notifications_rx
            .map_err(|()| panic!("channel receive should never fail"))
            .and_then(|r| r)
            .map(|x| Some(x))
            .chain(stream::once(Ok(None)));

        let handler = create_handler(engine.clone(), notifications_tx);

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
                    error!("{}", e.0);
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
    env_logger::init();
    let options = Opt::from_args();

    if let Some(s) = options.wait_seconds {
        info!("sleeping for {} seconds before connecting to Dragon", s);
        thread::sleep(time::Duration::from_secs(s))
    }

    stentorian::initialize()?;
    run_server(options)
}

pub fn main() {
    if let Err(e) = serve() {
        println!("{}", e.0);
    }
}
