//! Interprets an [`alux_jsonrpc`] program as a TypeScript client module.
//!
//! The program states a name, an argument product and an answer; the shapes of that product and that
//! answer state their types. Folding the two together is the whole of a client, so nothing about the
//! surface is written a second time in another language.

mod client;
mod params;

pub use client::*;
pub use params::*;
