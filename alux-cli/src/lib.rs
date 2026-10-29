#![doc = include_str!("../README.md")]

extern crate self as alux_cli;

mod algebra;
mod argument;
mod compound;
mod description;
mod laws;
mod outcome;
mod position;
mod program;
mod source;

pub use algebra::*;
pub use alux_ext::macros::cli;
pub use argument::*;
pub use compound::*;
pub use description::*;
pub use laws::*;
pub use outcome::*;
pub use position::*;
pub use program::*;
pub use source::*;
