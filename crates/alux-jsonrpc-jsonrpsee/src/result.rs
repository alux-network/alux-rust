use alux_ext::ext;
use alux_jsonrpc::RpcErrorAlg;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use serde::Serialize;
use std::fmt::Display;

/// Constructs an owned JSON-RPC error.
pub fn rpc_error<Data>(code: i32, message: impl Into<String>, data: Option<Data>) -> ErrorObjectOwned
where
    Data: Serialize,
{
    ErrorObjectOwned::owned(code, message, data)
}

/// Constructs an internal JSON-RPC error without implementation-specific data.
pub fn internal_rpc_error(message: impl Into<String>) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, message, None::<()>)
}

/// Reads a domain failure as the JSON-RPC error it denotes.
#[ext(name = RpcErrorExt)]
pub impl<This> This
where
    This: RpcErrorAlg,
{
    /// Builds the owned JSON-RPC error this failure states.
    fn to_rpc_error(&self) -> ErrorObjectOwned {
        rpc_error(self.rpc_code(), self.rpc_message(), None::<()>)
    }
}

/// Converts semantic results into JSON-RPC boundary results.
#[ext(name = ResultToRpcExt)]
pub impl<Output, Error> Result<Output, Error> {
    /// Maps a displayable semantic error to an internal JSON-RPC error.
    fn to_rpc_result(self) -> RpcResult<Output>
    where
        Error: Display,
    {
        self.map_err(|error| internal_rpc_error(error.to_string()))
    }

    /// Maps a semantic error with a caller-provided JSON-RPC conversion.
    fn map_rpc_error<F>(self, convert: F) -> RpcResult<Output>
    where
        F: FnOnce(Error) -> ErrorObjectOwned,
    {
        self.map_err(convert)
    }

    /// Maps a semantic error to an internal JSON-RPC error message.
    fn map_internal_rpc_error<F, Message>(self, message: F) -> RpcResult<Output>
    where
        F: FnOnce(Error) -> Message,
        Message: Into<String>,
    {
        self.map_err(|error| internal_rpc_error(message(error)))
    }
}

#[cfg(test)]
mod tests {
    use super::{ResultToRpcExt, RpcErrorExt};
    use alux_jsonrpc::RpcErrorAlg;
    use jsonrpsee::types::error::INTERNAL_ERROR_CODE;

    struct NoHistory;

    impl RpcErrorAlg for NoHistory {
        fn rpc_code(&self) -> i32 {
            -32000
        }

        fn rpc_message(&self) -> String {
            "the domain keeps no history".to_owned()
        }
    }

    #[test]
    fn reads_a_domain_failure_as_the_error_it_denotes() {
        let error = NoHistory.to_rpc_error();

        assert_eq!(error.code(), -32000);
        assert_eq!(error.message(), "the domain keeps no history");
    }

    #[test]
    fn converts_a_semantic_error_at_the_rpc_boundary() {
        let result: Result<(), _> = Err("not available");
        let error = result.to_rpc_result().unwrap_err();

        assert_eq!(error.code(), INTERNAL_ERROR_CODE);
        assert_eq!(error.message(), "not available");
    }
}
