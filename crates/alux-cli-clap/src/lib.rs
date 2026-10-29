#![doc = include_str!("../README.md")]

mod command;
mod interpreter;
mod words;

pub use interpreter::{CliClap, UNSUPPORTED_STATUS};
pub use words::{ClapWord, ClapWords};

#[doc(hidden)]
pub use clap as __clap;
