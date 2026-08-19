#![doc = include_str!("../README.md")]

extern crate self as alux_jsonrpc;

mod algebra;
mod program;

pub use algebra::*;
pub use alux_ext::macros::jsonrpc;
pub use program::*;
