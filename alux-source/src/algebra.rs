//! The algebra of what Rust source declares.

use alux_ext::ext;
use alux_shape::{FieldAlg, ShapeAlg, Sorts};

/// Combines what several sources declare.
#[ext(name = SourceFoldExt)]
pub impl<This, Source> This
where
    This: SourceAlg<Source = Source>,
{
    /// Returns what every source of `all` declares, in order.
    fn source_all(&self, all: impl IntoIterator<Item = Source>) -> Source {
        all.into_iter().fold(self.source_empty(), |left, right| self.source_merge(left, right))
    }
}

/// States what Rust source declares: traits, which types implement them, and extension blocks with
/// what they require, each where it is written. A reader of source builds a term of it, and an
/// interpreter chooses what names, types, bounds and sources are.
pub trait SourceAlg {
    /// A name as the source writes it: a trait, a module, an extension, a method, a backend.
    type Name;
    /// A type as the source writes it.
    type Type;
    /// A requirement that a type implement traits.
    type Bound;
    /// What some source declares.
    type Source;

    /// Introduces the name `written`, a path as the source spells it.
    fn name(&self, written: &str) -> Self::Name;

    /// Introduces the type `written`, as the source spells it.
    fn ty(&self, written: &str) -> Self::Type;

    /// Requires `bounded` to implement each of `traits`.
    fn bound(&self, bounded: Self::Type, traits: Vec<Self::Name>) -> Self::Bound;

    /// Declares nothing.
    fn source_empty(&self) -> Self::Source;

    /// Declares what `left` declares, then what `right` declares.
    fn source_merge(&self, left: Self::Source, right: Self::Source) -> Self::Source;

    /// Places `source` in the file at `path`.
    fn file(&self, path: &str, source: Self::Source) -> Self::Source;

    /// Places `source` in the module `name`.
    fn module(&self, name: Self::Name, source: Self::Source) -> Self::Source;

    /// Declares the trait `name`, extending `supertraits` and stating `bounds` beside them: on its
    /// parameters, in its `where` clause, on its methods and its associated types.
    fn trait_def(&self, name: Self::Name, supertraits: Vec<Self::Name>, bounds: Vec<Self::Bound>) -> Self::Source;

    /// Declares that `self_ty` implements the trait `trait_name` under `bounds`.
    fn trait_impl(&self, trait_name: Self::Name, self_ty: Self::Type, bounds: Vec<Self::Bound>) -> Self::Source;

    /// Declares the extension `name`: `methods` over `carrier`, depending on `bounds`, lowered by
    /// the backend `via` when the block is reified.
    fn extension(
        &self,
        name: Self::Name,
        carrier: Self::Type,
        bounds: Vec<Self::Bound>,
        via: Option<Self::Name>,
        methods: Vec<Self::Name>,
    ) -> Self::Source;
}

/// Declares data: a struct or an enum, whose meaning is a shape. A struct holds a product and an
/// enum a choice, so one declaration states both; the shape is an `alux-shape` term, in the same
/// vocabulary every other reading of data uses.
pub trait DataAlg: SourceAlg + ShapeAlg + FieldAlg {
    /// Declares the data type `name`, with type parameters `params`, holding `shape`.
    fn data(&self, name: Self::Name, params: Vec<Self::Name>, shape: <Self as Sorts>::Ty) -> Self::Source;
}
