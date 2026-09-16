#![doc = include_str!("../README.md")]

//! Reads a body arriving as parts, for interpretations whose framework does not.
//!
//! Poem, axum, and Salvo each read parts themselves. The rest have the bytes and the media type a
//! caller sent and nothing that reads them, so what states the sequence is here rather than copied
//! into each of them. Nothing about a framework is named: what arrives is bytes, and what comes out
//! is [`alux_http::ChunksAlg`] of [`alux_http::PartAlg`], the same as everywhere else.

mod read;

pub use read::*;
