use alux_jsonrpc::RpcErrorAlg;
use core::fmt::{self, Display};
use derive_new::new as New;
use serde::Serialize;

/// The code a malformed JSON document carries.
pub const PARSE_ERROR: i32 = -32700;
/// The code a document that is not a JSON-RPC request carries.
pub const INVALID_REQUEST: i32 = -32600;
/// The code an unknown method name carries.
pub const METHOD_NOT_FOUND: i32 = -32601;
/// The code a parameter the method cannot read carries.
pub const INVALID_PARAMS: i32 = -32602;
/// The code a failure of the interpretation itself carries.
pub const INTERNAL_ERROR: i32 = -32603;

/// What a JSON-RPC response states in its `error` member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, New)]
pub struct RpcError {
    /// The code this failure carries.
    pub code: i32,
    /// The message this failure states.
    #[new(into)]
    pub message: String,
}

impl RpcError {
    /// Reads the error a domain failure denotes.
    pub fn denoted<Failure>(failure: &Failure) -> Self
    where
        Failure: RpcErrorAlg,
    {
        Self { code: failure.rpc_code(), message: failure.rpc_message() }
    }

    /// States that a document is not JSON.
    pub fn parse_error() -> Self {
        Self::new(PARSE_ERROR, "invalid JSON")
    }

    /// States that a document is JSON but not a JSON-RPC request.
    pub fn invalid_request() -> Self {
        Self::new(INVALID_REQUEST, "invalid request")
    }

    /// States that no method answers to a name.
    pub fn method_not_found(method: &str) -> Self {
        Self::new(METHOD_NOT_FOUND, format!("method `{method}` not found"))
    }
}

impl Display for RpcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.message, self.code)
    }
}

impl core::error::Error for RpcError {}

/// A domain can answer with this error directly, stating its own code and message.
impl RpcErrorAlg for RpcError {
    fn rpc_code(&self) -> i32 {
        self.code
    }

    fn rpc_message(&self) -> String {
        self.message.clone()
    }
}
