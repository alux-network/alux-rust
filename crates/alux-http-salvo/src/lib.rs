#![doc = include_str!("../README.md")]

//! Interprets typed HTTP programs as executable Salvo routes.
//!
//! The interpretation chooses Salvo's readers for input roles, its response for output kinds, and
//! `Arc` for the runtime handle of a semantic context. A Salvo handler writes into the response it
//! is given, so every output kind here answers with one.

#![allow(async_fn_in_trait)]

mod handler;
mod input;
mod output;
mod route;

pub use handler::*;
pub use input::*;
pub use output::*;
pub use route::*;
