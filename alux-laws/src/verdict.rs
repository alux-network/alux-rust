//! What a run says about each law: structured, so any reader can render or compare it.

use core::fmt;

/// Why a law broke on one subject and input: the message, and where a caught panic happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// What the law reported, or the panic's message.
    pub message: String,
    /// Where a caught panic happened, as `file:line`.
    pub at: Option<String>,
}

impl Failure {
    /// Returns a failure with `message` and no location.
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), at: None }
    }

    /// Returns the same failure, located at `at` (`file:line`).
    #[must_use]
    pub fn at(self, at: impl Into<String>) -> Self {
        Self { at: Some(at.into()), ..self }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.at {
            Some(at) => write!(f, "panicked at {at}: {}", self.message),
            None => f.write_str(&self.message),
        }
    }
}

/// The first subject and input a law broke on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Broke {
    /// The case the subject was built from, when the run has more than one.
    pub subject: Option<String>,
    /// The input, as `Debug` shows it; `None` for a law over the subject alone.
    pub input: Option<String>,
    /// Whether the input was sampled rather than pinned.
    pub sampled: bool,
    /// Why it broke.
    pub failure: Failure,
}

/// What a run found for one law: how many checks reached it, and whether it broke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// The suite the law belongs to.
    pub suite: &'static str,
    /// The law's name.
    pub law: &'static str,
    /// How many checks of the law ran.
    pub reached: usize,
    /// The first break, when the law broke.
    pub broke: Option<Broke>,
}

impl Verdict {
    /// Reports whether `name` names this law: by its own name, or as `Suite::law`.
    pub fn is(&self, name: &str) -> bool {
        name == self.law || name.split_once("::").is_some_and(|(suite, law)| suite == self.suite && law == self.law)
    }

    /// Adds another subject's verdict on the same law; the first break is kept.
    pub fn merge(&mut self, other: Verdict) {
        self.reached += other.reached;
        if self.broke.is_none() {
            self.broke = other.broke;
        }
    }
}
