//! Composes the token and stake specifications into chains whose stake is their native token.
//!
//! A variant is a marker type. Every chain shares the token ledger, epochs, snapshots, and
//! exclusion; escrow variants share bonds, unbonding, and custody; each variant answers only the
//! primitives where it differs.

mod cardano;
mod chain;
mod cosmos;
mod escrow;
mod ethereum;

#[cfg(test)]
mod tests;

pub use cardano::*;
pub use chain::*;
pub use cosmos::*;
pub use escrow::*;
pub use ethereum::*;
