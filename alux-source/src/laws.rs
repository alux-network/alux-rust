//! The laws every interpreter of the source algebra obeys.

use crate::SourceAlg;
use alux_ext::ext;
use alux_laws::laws;
use core::fmt::Debug;

/// How declarations compose: in order, with nothing declared as the identity, and placing them in
/// a module or a file placing each of them there.
#[ext(name = SourceLaws, defunc(via = laws))]
pub impl<This, Name, Source> This
where
    This: SourceAlg<Name = Name, Source = Source>,
    Name: Clone,
    Source: Clone + PartialEq + Debug,
{
    /// Declaring nothing before or after a source changes nothing.
    fn empty_is_the_identity(&self, source: &Source) -> Result<(), String> {
        let before = self.source_merge(self.source_empty(), source.clone());
        let after = self.source_merge(source.clone(), self.source_empty());
        (before == *source && after == *source).then_some(()).ok_or_else(|| format!("{before:?} / {after:?}"))
    }

    /// Merging is associative: only the order of declarations counts.
    fn merge_is_associative(&self, sources: &(Source, Source, Source)) -> Result<(), String> {
        let (a, b, c) = sources.clone();
        let left = self.source_merge(self.source_merge(a.clone(), b.clone()), c.clone());
        let right = self.source_merge(a, self.source_merge(b, c));
        (left == right).then_some(()).ok_or_else(|| format!("{left:?} != {right:?}"))
    }

    /// A module holding two sources holds each of them.
    fn module_distributes_over_merge(&self, placed: &(Name, Source, Source)) -> Result<(), String> {
        let (name, a, b) = placed.clone();
        let whole = self.module(name.clone(), self.source_merge(a.clone(), b.clone()));
        let parts = self.source_merge(self.module(name.clone(), a), self.module(name, b));
        (whole == parts).then_some(()).ok_or_else(|| format!("{whole:?} != {parts:?}"))
    }

    /// A module holding nothing declares nothing.
    fn an_empty_module_declares_nothing(&self, name: &Name) -> Result<(), String> {
        let module = self.module(name.clone(), self.source_empty());
        (module == self.source_empty()).then_some(()).ok_or_else(|| format!("{module:?}"))
    }
}
