#![doc = include_str!("../README.md")]

//! Interprets typed HTTP programs as executable Rocket routes.
//!
//! Rocket states a method on every route and builds a response that borrows the request it answers,
//! so this interpretation mounts one route per endpoint and states what an endpoint answers with as
//! a value the route hands over.

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
