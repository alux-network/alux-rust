//! The operations a shape's primitives derive, and the laws relating them.
//!
//! Nothing here is observed by an interpretation. Every tagged encoding of a choice is a choice of
//! products over a name-valued discriminant, so an interpretation implements the primitives and gets
//! all of them — and cannot disagree with another interpretation about what internal tagging means,
//! because neither of them decides it.

use crate::{FieldAlg, ShapeAlg, Sorts, Words};
use alux_ext::ext;

/// The shape operations that follow from the primitives alone.
#[ext(name = ShapeExt, supertraits = Sorts)]
pub impl<This> This
where
    This: ShapeAlg,
{
    /// Bytes written as hexadecimal.
    fn bytes_hex(&self, len: Option<usize>) -> Self::Ty {
        self.hex(self.bytes(len))
    }

    /// Bytes written as an array of JSON numbers, which is a sequence of octets and nothing more.
    fn bytes_array(&self) -> Self::Ty {
        self.seq(self.int(false, 8))
    }

    /// An integer written as decimal digits, for a width a JSON number cannot hold.
    fn int_decimal(&self, signed: bool, bits: u16) -> Self::Ty {
        self.decimal(self.int(signed, bits))
    }

    /// An integer written as a `0x` quantity, as the Ethereum JSON-RPC specification states one.
    fn int_hex(&self, signed: bool, bits: u16) -> Self::Ty {
        self.hex(self.int(signed, bits))
    }

    /// A named product: the shape most types have.
    fn named_product(&self, words: Words<'_>, fields: Vec<Self::Field>) -> Self::Ty {
        self.named(words, self.product(fields))
    }

    /// A choice written as the alternative's value alone, with no discriminant on the wire.
    fn sum_untagged<'w>(&self, alternatives: Vec<(Words<'w>, Self::Ty)>) -> Self::Ty {
        self.choice(alternatives.into_iter().map(|(_, shape)| shape).collect())
    }

    /// A choice between names alone, which is a choice between name-valued constants.
    fn sum_of_names<'w>(&self, names: Vec<Words<'w>>) -> Self::Ty {
        self.choice(names.into_iter().map(|words| self.name_word(words)).collect())
    }
}

/// The encodings of a choice that state a discriminant, which is what needs a member to state it in.
#[ext(name = ShapeTaggedExt, supertraits = Sorts)]
pub impl<This> This
where
    This: ShapeAlg + FieldAlg,
{
    /// A choice written as one object per alternative, keyed by its name.
    fn sum_external<'w>(&self, alternatives: Vec<(Words<'w>, Self::Ty)>) -> Self::Ty {
        let alternatives =
            alternatives.into_iter().map(|(words, shape)| self.product(vec![self.field(words, shape)])).collect();

        self.choice(alternatives)
    }

    /// A choice written as one object carrying its name under `tag`, beside its own members.
    fn sum_internal<'w>(&self, tag: Words<'w>, alternatives: Vec<(Words<'w>, Self::Ty)>) -> Self::Ty {
        let alternatives = alternatives
            .into_iter()
            .map(|(words, shape)| self.product(vec![self.field(tag, self.name_word(words)), self.merge(shape)]))
            .collect();

        self.choice(alternatives)
    }

    /// A choice written as one object carrying its name under `tag` and its value under `content`.
    fn sum_adjacent<'w>(
        &self,
        tag: Words<'w>,
        content: Words<'w>,
        alternatives: Vec<(Words<'w>, Self::Ty)>,
    ) -> Self::Ty {
        let alternatives = alternatives
            .into_iter()
            .map(|(words, shape)| {
                self.product(vec![self.field(tag, self.name_word(words)), self.field(content, shape)])
            })
            .collect();

        self.choice(alternatives)
    }
}
