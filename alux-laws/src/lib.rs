#![doc = include_str!("../README.md")]

extern crate self as alux_laws;

mod algebra;
mod definitions;
mod names;
mod op;
mod program;
mod verdict;

pub use algebra::*;
pub use alux_ext::macros::laws;
pub use definitions::*;
pub use names::*;
pub use op::*;
pub use program::*;
pub use verdict::*;
