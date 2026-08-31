//! Interprets typed JSON-RPC programs as a JSON-RPC 2.0 message handler.
//!
//! This interpretation implements the specification itself rather than delegating to a framework: it
//! decodes a request document, dispatches by method name, decodes parameters, and renders the
//! response — including the protocol's own errors for a malformed document, an unknown method, and a
//! parameter a method cannot read. It carries no transport and no runtime, so whatever moves bytes
//! decides how a request arrives.

mod args;
mod dispatch;
mod error;
mod interpreter;
mod table;

pub use args::*;
pub use error::*;
pub use interpreter::*;
pub use table::*;
