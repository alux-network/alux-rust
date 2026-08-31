#![doc = include_str!("../README.md")]

extern crate self as alux_sdk;

mod call;
mod collection;
mod conversion;
mod iterator;
mod mapping;
mod option;
mod result;

pub use crate::call::AlgebraCall;
pub use crate::collection::*;
pub use crate::conversion::*;
pub use crate::iterator::*;
pub use crate::option::*;
pub use crate::result::*;
pub use alux_sdk_macros::trait_algebra;
pub use alux_traversable::*;

// Re-exported so the mapping macros reach it without every caller depending on it.
#[doc(hidden)]
pub use paste;
