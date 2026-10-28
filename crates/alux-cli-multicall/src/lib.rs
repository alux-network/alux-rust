#![doc = include_str!("../README.md")]

mod interpreter;

pub use interpreter::{Multicall, MulticallCompiler, MulticallInvocation};
