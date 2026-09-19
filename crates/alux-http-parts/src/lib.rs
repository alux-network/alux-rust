#![doc = include_str!("../README.md")]

//! States the parts a request and an answer are made of, for interpretations that need them.
//!
//! A request and an answer are values here, naming no transport and no framework, so a crate
//! stating requests needs no interpreter to state them with.
//!
//! Poem, axum, and Salvo each read parts themselves. The rest have the bytes and the media type a
//! caller sent and nothing that reads them, so what states the sequence is here rather than copied
//! into each of them. Nothing about a framework is named: what arrives is bytes, and what comes out
//! is [`alux_http::ChunksAlg`] of [`alux_http::PartAlg`], the same as everywhere else.

mod message;
mod read;

pub use message::*;
pub use read::*;
