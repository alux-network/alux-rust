#![doc = include_str!("../README.md")]

//! One declared HTTP surface, and the scenario every interpretation of it must satisfy.
//!
//! An interpretation agreeing with another is only evidence when both were held to the same thing.
//! What lives here is that thing: one declaration, compiled by whoever is under test, and one set of
//! exchanges naming what answering it must produce. The scenario knows methods, paths, media types
//! and statuses, and nothing of the domain or of any framework.
//!
//! [`ExpectLifecycleExt`] states the other half for interpretations that serve rather than answer:
//! what opening and closing one bound address must do, and [`MeasureLifecycleExt`] what it costs.

mod lifecycle;
mod measure;
mod scenario;
mod surface;

pub use lifecycle::*;
pub use measure::*;
pub use scenario::*;
pub use surface::*;
