//! Interprets typed JSON-RPC programs as `jsonrpsee` method collections.
//!
//! The interpretation owns parameter decoding, method registration, context sharing, serialization,
//! and boundary error conversion. Those are jsonrpsee mechanics rather than JSON-RPC program
//! meaning, so the program compiled here is the same value any other interpreter folds.

mod args;
mod interpreter;
mod methods;
mod result;
mod rpc_ctx;

pub use args::*;
pub use interpreter::*;
pub use methods::*;
pub use result::*;
pub use rpc_ctx::*;
