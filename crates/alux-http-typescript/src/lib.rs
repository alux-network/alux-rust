#![doc = include_str!("../README.md")]

//! Interprets typed HTTP programs as the TypeScript client that calls them.
//!
//! The same program an executing interpretation answers is read here from the other side: what a
//! caller states, where each argument goes, and what comes back. Nothing about the surface is
//! written a second time in another language.

mod handler;
mod input;
mod output;
mod route;

pub use handler::*;
pub use input::*;
pub use output::*;
pub use route::*;
