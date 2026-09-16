#![doc = include_str!("../README.md")]

//! Interprets typed HTTP programs as a surface that answers requests itself.
//!
//! This interpretation implements the specification rather than delegating to a framework: it
//! matches a request against the paths a program states, binds what those paths capture, reads each
//! argument from the role it was declared under, applies the operation, and renders the answer,
//! including the failures routing itself produces. It carries no transport and no runtime, so
//! whatever moves bytes decides how a request arrives.
//!
//! Because it answers with values rather than with a framework's types, it is the reference another
//! interpretation can be held to.

mod handler;
mod input;
mod message;
mod output;
mod route;

pub use handler::*;
pub use input::*;
pub use message::*;
pub use output::*;
pub use route::*;
