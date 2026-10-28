//! How a value is made from the members its shape names, and taken apart into them.
//!
//! A shape describes a value; it never builds one. An interpretation that reads a value member by
//! member, from command-line words, named parameters, or a form, needs the other direction too: a
//! product is its fields taken together, and a sum is the one alternative it carries. Each is an
//! isomorphism with a tuple or a `SumN`, and its members are named as identifiers, the way an
//! operation's argument names are, so `words_of` reads them as the words a shape names them by.

use alux_ext::ext;

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

/// Holds the one alternative of two a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum2<A, B> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
}

/// Holds the one alternative of three a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum3<A, B, C> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
}

/// Holds the one alternative of four a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum4<A, B, C, D> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
}

/// Holds the one alternative of five a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum5<A, B, C, D, E> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
    /// The fifth alternative.
    E(E),
}

/// Holds the one alternative of six a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum6<A, B, C, D, E, F> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
    /// The fifth alternative.
    E(E),
    /// The sixth alternative.
    F(F),
}

/// Holds the one alternative of seven a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum7<A, B, C, D, E, F, G> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
    /// The fifth alternative.
    E(E),
    /// The sixth alternative.
    F(F),
    /// The seventh alternative.
    G(G),
}

/// Holds the one alternative of eight a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum8<A, B, C, D, E, F, G, H> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
    /// The fifth alternative.
    E(E),
    /// The sixth alternative.
    F(F),
    /// The seventh alternative.
    G(G),
    /// The eighth alternative.
    H(H),
}

/// Holds the one alternative of nine a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum9<A, B, C, D, E, F, G, H, I> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
    /// The fifth alternative.
    E(E),
    /// The sixth alternative.
    F(F),
    /// The seventh alternative.
    G(G),
    /// The eighth alternative.
    H(H),
    /// The ninth alternative.
    I(I),
}

/// Holds the one alternative of ten a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum10<A, B, C, D, E, F, G, H, I, J> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
    /// The fifth alternative.
    E(E),
    /// The sixth alternative.
    F(F),
    /// The seventh alternative.
    G(G),
    /// The eighth alternative.
    H(H),
    /// The ninth alternative.
    I(I),
    /// The tenth alternative.
    J(J),
}

/// Holds the one alternative of eleven a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum11<A, B, C, D, E, F, G, H, I, J, K> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
    /// The fifth alternative.
    E(E),
    /// The sixth alternative.
    F(F),
    /// The seventh alternative.
    G(G),
    /// The eighth alternative.
    H(H),
    /// The ninth alternative.
    I(I),
    /// The tenth alternative.
    J(J),
    /// The eleventh alternative.
    K(K),
}

/// Holds the one alternative of twelve a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum12<A, B, C, D, E, F, G, H, I, J, K, L> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
    /// The fifth alternative.
    E(E),
    /// The sixth alternative.
    F(F),
    /// The seventh alternative.
    G(G),
    /// The eighth alternative.
    H(H),
    /// The ninth alternative.
    I(I),
    /// The tenth alternative.
    J(J),
    /// The eleventh alternative.
    K(K),
    /// The twelfth alternative.
    L(L),
}

/// Holds the one alternative of thirteen a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum13<A, B, C, D, E, F, G, H, I, J, K, L, M> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
    /// The fifth alternative.
    E(E),
    /// The sixth alternative.
    F(F),
    /// The seventh alternative.
    G(G),
    /// The eighth alternative.
    H(H),
    /// The ninth alternative.
    I(I),
    /// The tenth alternative.
    J(J),
    /// The eleventh alternative.
    K(K),
    /// The twelfth alternative.
    L(L),
    /// The thirteenth alternative.
    M(M),
}

/// Holds the one alternative of fourteen a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum14<A, B, C, D, E, F, G, H, I, J, K, L, M, N> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
    /// The fifth alternative.
    E(E),
    /// The sixth alternative.
    F(F),
    /// The seventh alternative.
    G(G),
    /// The eighth alternative.
    H(H),
    /// The ninth alternative.
    I(I),
    /// The tenth alternative.
    J(J),
    /// The eleventh alternative.
    K(K),
    /// The twelfth alternative.
    L(L),
    /// The thirteenth alternative.
    M(M),
    /// The fourteenth alternative.
    N(N),
}

/// Holds the one alternative of fifteen a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum15<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
    /// The fifth alternative.
    E(E),
    /// The sixth alternative.
    F(F),
    /// The seventh alternative.
    G(G),
    /// The eighth alternative.
    H(H),
    /// The ninth alternative.
    I(I),
    /// The tenth alternative.
    J(J),
    /// The eleventh alternative.
    K(K),
    /// The twelfth alternative.
    L(L),
    /// The thirteenth alternative.
    M(M),
    /// The fourteenth alternative.
    N(N),
    /// The fifteenth alternative.
    O(O),
}

/// Holds the one alternative of sixteen a value carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sum16<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P> {
    /// The first alternative.
    A(A),
    /// The second alternative.
    B(B),
    /// The third alternative.
    C(C),
    /// The fourth alternative.
    D(D),
    /// The fifth alternative.
    E(E),
    /// The sixth alternative.
    F(F),
    /// The seventh alternative.
    G(G),
    /// The eighth alternative.
    H(H),
    /// The ninth alternative.
    I(I),
    /// The tenth alternative.
    J(J),
    /// The eleventh alternative.
    K(K),
    /// The twelfth alternative.
    L(L),
    /// The thirteenth alternative.
    M(M),
    /// The fourteenth alternative.
    N(N),
    /// The fifteenth alternative.
    O(O),
    /// The sixteenth alternative.
    P(P),
}

/// States that taking a product apart and making it again changes nothing.
#[ext(name = ProductLawsExt)]
pub impl<This> This
where
    This: Product + Clone + PartialEq,
{
    /// Making a product from the fields it is taken apart into answers the same product.
    fn law_product_round_trip(&self) -> bool {
        This::from_fields(self.clone().into_fields()) == *self
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
