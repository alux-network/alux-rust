#![doc = include_str!("../README.md")]

//! Interprets typed HTTP programs as executable actix-web routes.
//!
//! The interpretation chooses actix-web's extractors for input roles, its responses for output
//! kinds, and `Arc` for the runtime handle of a semantic context. It also states an endpoint as what
//! makes one, because actix-web configures its routes again for every worker.

#![allow(async_fn_in_trait)]

mod handler;
mod input;
mod output;
mod route;

pub use handler::*;
pub use input::*;
pub use output::*;
pub use route::*;
