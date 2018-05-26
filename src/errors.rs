use jsonrpc_core::Error as RpcError;
use jsonrpc_core::ErrorCode;

use failure::Error;

pub type Result<T> = ::std::result::Result<T, MyError>;

pub struct MyError(pub Error);

impl<T: Into<Error>> From<T> for MyError {
    fn from(e: T) -> MyError {
        MyError(e.into())
    }
}

impl From<MyError> for RpcError {
    fn from(e: MyError) -> RpcError {
        RpcError {
            code: ErrorCode::ServerError(-1),
            message: e.0.to_string(),
            data: None,
        }
    }
}
