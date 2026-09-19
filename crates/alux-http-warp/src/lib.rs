#![doc = include_str!("../README.md")]

//! Interprets typed HTTP programs as an executable warp filter.
//!
//! warp composes filters rather than holding a route table, so a coproduct of routes is `or` and a
//! selector is the filters a request must pass. What reaches an endpoint is what those filters
//! gathered, which is why the roles here read a gathered request rather than a framework extractor.

mod handler;
mod input;
mod message;
mod output;
mod route;
mod server;

pub use handler::*;
pub use input::*;
pub use message::*;
pub use output::*;
pub use route::*;
pub use server::*;
