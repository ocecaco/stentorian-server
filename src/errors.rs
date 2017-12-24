use error_chain::ChainedError;
use jsonrpc_core::Error as RpcError;
use jsonrpc_core::ErrorCode;
use serde_json;

error_chain! {
    links {
        Stentorian(::stentorian::errors::Error, ::stentorian::errors::ErrorKind);
    }

    foreign_links {
        Io(::std::io::Error);
        Serde(::serde_json::Error);
    }

    errors {
        UnknownEntityId(id: u64) {
            description("unknown entity ID")
            display("unknown entity ID: {}", id)
        }

        WrongGrammarType(id: u64, expected_type: String) {
            description("wrong grammar type")
            display("wrong grammar type '{}' for grammar ID {}", expected_type, id)
        }
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
