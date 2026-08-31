#![doc = include_str!("../README.md")]

mod channel;
mod message;
mod sender;

pub use crate::channel::{AlgebraStream, bounded_algebra_channel, interpret_stream};
pub use crate::message::{AlgebraMessage, AlgebraResponder};
pub use crate::sender::BoundedAlgebraSender;
