#![doc = include_str!("../README.md")]

extern crate self as alux_shape;

mod algebra;
mod derived;
mod edge;
mod program;
mod spell;
mod std_shapes;

pub use algebra::*;
pub use alux_shape_macros::{Shape, shape_layout};
pub use derived::*;
pub use edge::*;
pub use program::*;
pub use spell::*;
