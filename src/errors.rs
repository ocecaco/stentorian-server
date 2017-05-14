use jsonrpc_core::Error as RpcError;
use jsonrpc_core::ErrorCode;

error_chain! {
    links {
        RustLink(::rustlink::errors::Error, ::rustlink::errors::ErrorKind);
    }

    foreign_links {
        Io(::std::io::Error);
    }
}

pub type RpcResult<T> = ::std::result::Result<T, RpcError>;

impl From<Error> for RpcError {
    fn from(_e: Error) -> RpcError {
        RpcError {
            code: ErrorCode::ServerError(-2),
            message: "an error occurred".to_owned(),
            data: None,
        }
    }
}
