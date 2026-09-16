#![doc = include_str!("../README.md")]

//! Serves an [`alux-http`](https://docs.rs/alux-http) surface over hyper, with no web framework.
//!
//! [`alux-http-direct`](https://docs.rs/alux-http-direct) answers a request with values and carries
//! no transport, so something has to move the bytes. This is that something, and nothing more: it
//! reads a hyper request into a `DirectRequest`, hands it to the surface, and writes the answer back
//! as a hyper response.
//!
//! It states a [`hyper::service::Service`] and stops there. Which runtime listens, and how
//! connections are driven, stays the caller's, so this crate names no runtime either.

mod message;
mod service;

pub use message::*;
pub use service::*;
