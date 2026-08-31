//! Interprets typed HTTP programs as readable route and type descriptions.
//!
//! The text interpretation executes no handler. It records the selectors, extractor roles, argument
//! product, handler result, and output conversion each endpoint denotes, which makes it the neutral
//! witness that an HTTP program means a surface rather than a framework callback.

mod handler;
mod input;
mod output;
mod route;

pub use handler::*;
pub use input::*;
pub use output::*;
pub use route::*;
