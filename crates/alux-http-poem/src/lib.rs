//! Interprets typed HTTP programs as executable Poem routes.
//!
//! The interpretation chooses Poem's extractors for input roles, its responses for output kinds, and
//! `Arc` for the runtime handle of a semantic context. Those choices stay inside this crate: the
//! program compiled here is the same value any other interpreter folds.

#![allow(async_fn_in_trait)]

mod handler;
mod input;
mod output;
mod route;

pub use handler::*;
pub use input::*;
pub use output::*;
pub use route::*;
