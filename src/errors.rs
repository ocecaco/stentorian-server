use jsonrpc_core::Error as RpcError;
use jsonrpc_core::ErrorCode;
use error_chain::ChainedError;
use serde_json;

error_chain! {
    links {
        Stentorian(::stentorian::errors::Error, ::stentorian::errors::ErrorKind);
    }

    foreign_links {
        Io(::std::io::Error);
        Serde(::serde_json::Error);
    }
}

impl From<Error> for RpcError {
    fn from(e: Error) -> RpcError {
        RpcError {
            code: ErrorCode::ServerError(-1),
            message: e.display().to_string(),
            data: e.backtrace().map(|b| serde_json::to_value(b).unwrap()),
        }
    }
}
