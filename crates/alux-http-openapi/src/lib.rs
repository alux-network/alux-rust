#![doc = include_str!("../README.md")]

//! Interprets typed HTTP programs as the `OpenAPI` document that describes them.
//!
//! This interpretation applies nothing. It reads each endpoint for what a caller states and what
//! they are answered with, which is why it asks for shapes where an executing interpretation asks
//! for extractors, and why it reads a failure's statuses from the type rather than from a value it
//! never holds.

mod handler;
mod input;
mod output;
mod route;

pub use handler::*;
pub use input::*;
pub use output::*;
pub use route::*;
