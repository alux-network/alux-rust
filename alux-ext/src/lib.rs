//! Reifies extension methods as first-order operations with explicit application meaning.

extern crate self as alux_ext;

/// Re-exports the procedural-macro implementation shared by ALUX program crates.
///
/// Program crates re-export their own backend attributes from this path so that authored code and
/// generated code refer to the same macro implementation without depending on it directly.
pub use alux_ext_macros as macros;
pub use alux_ext_macros::ext;

mod operation;

pub use operation::*;
