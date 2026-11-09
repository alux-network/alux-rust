//! How much sampling a run does.

use std::sync::OnceLock;

/// How many sampled cases a run draws: all authored ones, or one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// One case per sampled source: writing code is the common case.
    Dev,
    /// Every authored case: what CI runs.
    Strict,
}

impl Tier {
    /// The tier this process runs at, read once from `ALUX_TEST_TIER` (`dev` or `strict`), or
    /// `Strict` when it is unset and `CI` is set.
    ///
    /// # Panics
    ///
    /// Panics when `ALUX_TEST_TIER` names another tier.
    pub fn from_env() -> Tier {
        static TIER: OnceLock<Tier> = OnceLock::new();
        *TIER.get_or_init(|| match std::env::var("ALUX_TEST_TIER") {
            Ok(tier) => match tier.as_str() {
                "dev" => Tier::Dev,
                "strict" => Tier::Strict,
                other => panic!("unknown ALUX_TEST_TIER `{other}`: expected `dev` or `strict`"),
            },
            Err(_) if std::env::var_os("CI").is_some() => Tier::Strict,
            Err(_) => Tier::Dev,
        })
    }

    /// How many of `authored` cases run under this tier.
    pub fn samples(self, authored: usize) -> usize {
        match self {
            Tier::Strict => authored,
            Tier::Dev => authored.min(1),
        }
    }
}
