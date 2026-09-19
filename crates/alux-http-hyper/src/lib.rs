#![doc = include_str!("../README.md")]

//! Serves an [`alux-http`](https://docs.rs/alux-http) surface over hyper, with no web framework.
//!
//! [`alux-http-direct`](https://docs.rs/alux-http-direct) answers a request with values and carries
//! no transport, so something has to move the bytes. This is that something, and nothing more: it
//! reads a hyper request into a `DirectRequest`, hands it to the surface, and writes the answer back
//! as a hyper response.
//!
//! It also states the accepting itself, in [`HyperConnections`]. A framework that hands out a
//! service rather than a loop needs nothing more than that to serve, so the interpreters for those
//! frameworks accept through this one rather than each writing the same loop.

mod connections;
mod message;
mod route;
mod server;

pub use connections::*;
pub use message::*;
pub use route::*;
pub use server::*;
