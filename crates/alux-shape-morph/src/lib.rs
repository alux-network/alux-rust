#![doc = include_str!("../README.md")]

pub(crate) mod delegate;

mod patch;
mod prefixed;

pub use patch::*;
pub use prefixed::*;
