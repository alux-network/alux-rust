#![doc = include_str!("../README.md")]

mod command;
mod interpreter;
mod words;

pub use interpreter::CliClap;
pub use words::{ClapWord, ClapWords};

#[doc(hidden)]
pub use clap as __clap;
