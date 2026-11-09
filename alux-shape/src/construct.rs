//! How a value is made from the members its shape names, and taken apart into them.
//!
//! A shape describes a value; it never builds one. An interpretation that reads a value member by
//! member, from command-line words, named parameters, or a form, needs the other direction too: a
//! product is its fields taken together, and a sum is the one alternative it carries. A sum is an
//! isomorphism with a `SumN`. A product is one with a tuple unless a member is skipped: that member is
//! no field and is made with `Default`, so a product made from its fields answers those fields, and its members are named as identifiers, the way an
//! operation's argument names are, so `words_of` reads them as the words a shape names them by.

use alux_ext::{ext, for_tuples};

/// States that a value is its fields taken together.
///
/// `#[derive(Shape)]` states it for every struct it reads.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not stated as a product of its fields",
    note = "`#[derive(Shape)]` states `Product` for a struct with named members; a skipped member is made with `Default`, and a flattened one is one field"
)]
pub trait Product: Sized {
    /// The fields, in order, as a tuple.
    type Fields;
    /// The fields' names, in order, as identifiers.
    const FIELDS: &'static [&'static str];

    /// Makes the value from its fields.
    fn from_fields(fields: Self::Fields) -> Self;

    /// Takes the value apart into its fields.
    fn into_fields(self) -> Self::Fields;
}

/// States that a value is exactly one of its alternatives.
///
/// An alternative carrying no value is `()`.
///
/// `#[derive(Shape)]` states it for every enum it reads with two to sixteen alternatives.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not stated as a sum of its alternatives",
    note = "`#[derive(Shape)]` states `Sum` for an enum of two to sixteen alternatives, each carrying one value or none"
)]
pub trait Sum: Sized {
    /// The alternatives, in order, as a `Sum2` through `Sum16`.
    type Alternatives;
    /// The alternatives' names, in order, as identifiers.
    const ALTERNATIVES: &'static [&'static str];

    /// Makes the value from the alternative it carries.
    fn from_alternative(alternative: Self::Alternatives) -> Self;

    /// Answers the alternative the value carries.
    fn into_alternative(self) -> Self::Alternatives;
}

/// States `SumN`, the alternatives of one width.
macro_rules! sums {
    ($sum:ident, $width:tt; $(($variant:ident, $index:tt)),+) => {
        #[doc = concat!("Holds the one alternative of ", stringify!($width), " a value carries.")]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $sum<$($variant),+> {
            $(
                #[doc = concat!("The alternative at index ", stringify!($index), ".")]
                $variant($variant),
            )+
        }
    };
}

for_tuples!(sums, 2..=16, named = Sum, each = []);

/// States that a product made from its fields answers those fields.
///
/// A skipped member is no field, so the product made again holds its `Default`, not its value.
#[ext(name = ProductLawsExt)]
pub impl<This, Fields> This
where
    This: Product<Fields = Fields> + Clone,
    Fields: Clone + PartialEq,
{
    /// Taking apart a product made from this one's fields answers the same fields.
    fn law_product_round_trip(&self) -> bool {
        let fields = self.clone().into_fields();
        This::from_fields(fields.clone()).into_fields() == fields
    }
}

/// States that answering a sum's alternative and making it again changes nothing.
#[ext(name = SumLawsExt)]
pub impl<This> This
where
    This: Sum + Clone + PartialEq,
{
    /// Making a sum from the alternative it carries answers the same sum.
    fn law_sum_round_trip(&self) -> bool {
        This::from_alternative(self.clone().into_alternative()) == *self
    }
}
