#![doc = include_str!("../README.md")]

//! One declared HTTP surface, and the scenario every interpretation of it must satisfy.
//!
//! An interpretation agreeing with another is only evidence when both were held to the same thing.
//! What lives here is that thing: one declaration, compiled by whoever is under test, and one set of
//! exchanges naming what answering it must produce. The scenario knows methods, paths, media types
//! and statuses, and nothing of the domain or of any framework.

#![allow(async_fn_in_trait)]

mod scenario;
mod surface;

pub use scenario::*;
pub use surface::*;
