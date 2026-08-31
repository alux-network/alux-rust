//! The sorts of a data shape, and the primitives over them.

/// A name: the words it is made of, in order.
///
/// Spelling is not part of a name. An interpretation joins these words however it spells names, so
/// the same term reads `displayName` in one and `display_name` in another without either
/// re-segmenting anything — which is what makes the spelling exact rather than a conversion's guess.
pub type Words<'a> = &'a [&'a str];

/// The sorts of a data shape.
///
/// A pure carrier trait, shared by the classes below so the sorts stay linked with no equality
/// bounds and no per-class re-declaration.
pub trait Sorts {
    /// A shape.
    type Ty;
    /// One member of a product.
    type Field;
}

/// The shape sort: what data is, and how it is written.
pub trait ShapeAlg: Sorts {
    /// A boolean.
    fn truth(&self) -> Self::Ty;

    /// The empty value: `null`, or an absent member.
    fn unit(&self) -> Self::Ty;

    /// Text.
    fn text(&self) -> Self::Ty;

    /// One fixed piece of text, as a value: content that happens to be constant.
    fn literal(&self, text: &str) -> Self::Ty;

    /// A value whose content is a name, spelled however the interpretation spells names.
    ///
    /// What a discriminant carries, and what a choice between names alone is made of. Distinct from
    /// [`ShapeAlg::literal`] because a name is words and a literal is already text.
    fn name_word(&self, words: Words<'_>) -> Self::Ty;

    /// An integer of a stated width and signedness.
    fn int(&self, signed: bool, bits: u16) -> Self::Ty;

    /// A floating-point number of a stated width.
    fn float(&self, bits: u16) -> Self::Ty;

    /// Opaque bytes, of a stated length when fixed.
    fn bytes(&self, len: Option<usize>) -> Self::Ty;

    /// Written as `0x`-prefixed hexadecimal, rather than however the item would appear alone.
    fn hex(&self, item: Self::Ty) -> Self::Ty;

    /// Written as decimal digits in text, which is what a width beyond a JSON number requires.
    fn decimal(&self, item: Self::Ty) -> Self::Ty;

    /// Written as base64 text.
    fn base64(&self, item: Self::Ty) -> Self::Ty;

    /// A value that may be absent.
    fn opt(&self, item: Self::Ty) -> Self::Ty;

    /// An ordered sequence of one shape.
    fn seq(&self, item: Self::Ty) -> Self::Ty;

    /// An association from one shape to another.
    fn map(&self, key: Self::Ty, value: Self::Ty) -> Self::Ty;

    /// A product of members.
    fn product(&self, fields: Vec<Self::Field>) -> Self::Ty;

    /// A choice between alternatives, carrying no discriminant of its own.
    ///
    /// Every encoding that does write one is derived from this — see the extension in `derived`.
    fn choice(&self, alternatives: Vec<Self::Ty>) -> Self::Ty;

    /// Introduces a name for a shape, so an interpretation can state it once.
    ///
    /// A name is an identity, not an instruction: an interpretation that declares types emits one
    /// and refers to it, and one that types structurally may spell the shape at each use.
    fn named(&self, words: Words<'_>, body: Self::Ty) -> Self::Ty;

    /// Uses a name introduced elsewhere, which is what makes recursion expressible.
    fn reference(&self, words: Words<'_>) -> Self::Ty;
}

/// The member sort: the ways one member of a product arises.
pub trait FieldAlg: Sorts {
    /// A member under the name it is written with.
    fn field(&self, words: Words<'_>, shape: Self::Ty) -> Self::Field;

    /// Another product's members, observed as this product's own.
    ///
    /// Not a member, but two products seen as one. Taking a shape and answering with a member is
    /// what makes merging a non-product inexpressible.
    fn merge(&self, shape: Self::Ty) -> Self::Field;
}
