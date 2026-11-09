//! What an index answers.

use crate::{Bound, Declaration, Declared};
use alux_ext::ext;

/// Answers questions about what source declares. A trait is asked for by its name, the last segment
/// of a path without generic arguments, as every spelling of it shares.
#[ext(name = IndexQuestionsExt)]
pub impl [Declared] {
    /// Returns the law suites over the trait `capability`: the extensions lowered `via = laws`
    /// whose carrier must implement it.
    fn laws_of(&self, capability: &str) -> Vec<&Declared> {
        self.iter()
            .filter(|declared| match &declared.declaration {
                Declaration::Extension { carrier, bounds, via, .. } => {
                    via.as_deref() == Some("laws") && requires(bounds, carrier, capability)
                }
                _ => false,
            })
            .collect()
    }

    /// Returns the implementations of the trait `capability`.
    fn impls_of(&self, capability: &str) -> Vec<&Declared> {
        self.iter()
            .filter(|declared| {
                matches!(&declared.declaration, Declaration::Impl { trait_name, .. } if name_of(trait_name) == capability)
            })
            .collect()
    }

    /// Returns the trait definitions stating bounds beyond the traits they extend, other than the
    /// ones a definition may state ([`JUSTIFIED`]): a definition states meaning, and the rest of its
    /// bounds belong at the use sites, unless the definition means nothing without them.
    fn bounded_trait_defs(&self) -> Vec<&Declared> {
        self.iter()
            .filter(|declared| match &declared.declaration {
                Declaration::Trait { bounds, .. } => {
                    bounds.iter().any(|bound| bound.traits.iter().any(|path| !JUSTIFIED.contains(&name_of(path))))
                }
                _ => false,
            })
            .collect()
    }

    /// Returns the struct and enum declarations.
    fn data(&self) -> Vec<&Declared> {
        self.iter().filter(|declared| matches!(declared.declaration, Declaration::Data { .. })).collect()
    }

    /// Returns the extension blocks.
    fn extensions(&self) -> Vec<&Declared> {
        self.iter().filter(|declared| matches!(declared.declaration, Declaration::Extension { .. })).collect()
    }
}

impl Declared {
    /// Returns the declared item's name: the trait, the extension, or for an impl the type
    /// implementing.
    pub fn name(&self) -> &str {
        match &self.declaration {
            Declaration::Trait { name, .. } | Declaration::Data { name, .. } | Declaration::Extension { name, .. } => {
                name
            }
            Declaration::Impl { self_ty, .. } => self_ty,
        }
    }
}

/// The bounds any definition may state: what the language, sharing across threads, copying, and
/// showing a value ask of it.
pub const JUSTIFIED: &[&str] = &["Sized", "Send", "Sync", "Clone", "Debug"];

/// `a::b::Col<K>` named `Col`.
pub fn name_of(path: &str) -> &str {
    let bare = path.split('<').next().unwrap_or(path);
    bare.rsplit("::").next().unwrap_or(bare)
}

/// Whether `bounds` require `carrier` to implement `capability`.
fn requires(bounds: &[Bound], carrier: &str, capability: &str) -> bool {
    bounds.iter().any(|bound| bound.bounded == carrier && bound.traits.iter().any(|path| name_of(path) == capability))
}
