//! A shape term as a value, so that a term can be written down and read back.
//!
//! Every other interpretation answers with something that is not a shape — text, a judgement, a
//! declaration. This one answers with the term itself, which is what lets a shape leave the host
//! that stated it: written here, read there, and folded into whichever algebra that host carries.

use alux_shape::{FieldAlg, ShapeAlg, Sorts, Words};
use serde::{Deserialize, Serialize};

/// A shape, as a value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Term {
    Truth,
    Unit,
    Text,
    Literal(String),
    NameWord(Vec<String>),
    Int { signed: bool, bits: u16 },
    Float { bits: u16 },
    Bytes { len: Option<usize> },
    Hex(Box<Term>),
    Decimal(Box<Term>),
    Base64(Box<Term>),
    Opt(Box<Term>),
    Seq(Box<Term>),
    Map(Box<Term>, Box<Term>),
    Product(Vec<Member>),
    Choice(Vec<Term>),
    Named { words: Vec<String>, body: Box<Term> },
    Reference(Vec<String>),
}

/// One member of a product, as a value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Member {
    /// A member under the name it is written with.
    Field { words: Vec<String>, shape: Term },
    /// Another product's members, observed as this product's own.
    Merge(Term),
}

/// Borrows a name's words in the form the algebra states them.
fn borrow(words: &[String]) -> Vec<&str> {
    words.iter().map(String::as_str).collect()
}

/// Owns a name's words, to keep them in a term.
fn own(words: Words<'_>) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}

impl Term {
    /// Folds this term into whichever algebra an interpretation provides.
    ///
    /// This is the reader: a term written by one host is interpreted by another without either
    /// naming the other's types. Reading and writing are checked against each other, since folding a
    /// written term with any interpretation answers what interpreting the original directly does.
    pub fn fold<A>(&self, alg: &A) -> A::Ty
    where
        A: ShapeAlg + FieldAlg,
    {
        match self {
            Self::Truth => alg.truth(),
            Self::Unit => alg.unit(),
            Self::Text => alg.text(),
            Self::Literal(text) => alg.literal(text),
            Self::NameWord(words) => alg.name_word(&borrow(words)),
            Self::Int { signed, bits } => alg.int(*signed, *bits),
            Self::Float { bits } => alg.float(*bits),
            Self::Bytes { len } => alg.bytes(*len),
            Self::Hex(item) => alg.hex(item.fold(alg)),
            Self::Decimal(item) => alg.decimal(item.fold(alg)),
            Self::Base64(item) => alg.base64(item.fold(alg)),
            Self::Opt(item) => alg.opt(item.fold(alg)),
            Self::Seq(item) => alg.seq(item.fold(alg)),
            Self::Map(key, value) => alg.map(key.fold(alg), value.fold(alg)),
            Self::Product(members) => alg.product(members.iter().map(|member| member.fold(alg)).collect()),
            Self::Choice(alternatives) => alg.choice(alternatives.iter().map(|item| item.fold(alg)).collect()),
            Self::Named { words, body } => alg.named(&borrow(words), body.fold(alg)),
            Self::Reference(words) => alg.reference(&borrow(words)),
        }
    }

    /// Answers with this term, with every reference replaced by the shape its name introduces.
    ///
    /// A fold cannot do this: it builds from the leaves upward, so a reference reaches it before the
    /// name it uses has been seen. A term holds the whole shape at once, which is what makes a name
    /// resolvable — and is why an interpretation that must decide a value, rather than describe one,
    /// reads a resolved term.
    ///
    /// A name that would resolve into itself is left as a reference, since expanding it never ends.
    #[must_use]
    pub fn resolved(&self) -> Self {
        let mut definitions = Vec::new();
        self.definitions(&mut definitions);

        self.expand(&definitions, &mut Vec::new())
    }

    /// Collects every name this term introduces, with the shape it names.
    fn definitions<'a>(&'a self, found: &mut Vec<(&'a [String], &'a Self)>) {
        if let Self::Named { words, body } = self {
            found.push((words, body));
        }

        self.children().for_each(|child| child.definitions(found));
    }

    /// Replaces the references in this term, refusing to expand a name already being expanded.
    fn expand(&self, definitions: &[(&[String], &Self)], expanding: &mut Vec<Vec<String>>) -> Self {
        match self {
            Self::Reference(words) => {
                let known = definitions.iter().find(|(name, _)| *name == words.as_slice());

                match known {
                    Some((_, body)) if !expanding.iter().any(|name| name == words) => {
                        expanding.push(words.clone());
                        let expanded = body.expand(definitions, expanding);
                        expanding.pop();

                        Self::Named { words: words.clone(), body: Box::new(expanded) }
                    }
                    _ => self.clone(),
                }
            }
            Self::Hex(item) => Self::Hex(Box::new(item.expand(definitions, expanding))),
            Self::Decimal(item) => Self::Decimal(Box::new(item.expand(definitions, expanding))),
            Self::Base64(item) => Self::Base64(Box::new(item.expand(definitions, expanding))),
            Self::Opt(item) => Self::Opt(Box::new(item.expand(definitions, expanding))),
            Self::Seq(item) => Self::Seq(Box::new(item.expand(definitions, expanding))),
            Self::Map(key, value) => {
                Self::Map(Box::new(key.expand(definitions, expanding)), Box::new(value.expand(definitions, expanding)))
            }
            Self::Product(members) => {
                Self::Product(members.iter().map(|member| member.expand(definitions, expanding)).collect())
            }
            Self::Choice(alternatives) => {
                Self::Choice(alternatives.iter().map(|item| item.expand(definitions, expanding)).collect())
            }
            Self::Named { words, body } => {
                expanding.push(words.clone());
                let body = body.expand(definitions, expanding);
                expanding.pop();

                Self::Named { words: words.clone(), body: Box::new(body) }
            }
            leaf => leaf.clone(),
        }
    }

    /// The terms this one is built from.
    fn children(&self) -> Box<dyn Iterator<Item = &Self> + '_> {
        match self {
            Self::Hex(item)
            | Self::Decimal(item)
            | Self::Base64(item)
            | Self::Opt(item)
            | Self::Seq(item)
            | Self::Named { body: item, .. } => Box::new(std::iter::once(&**item)),
            Self::Map(key, value) => Box::new([&**key, &**value].into_iter()),
            Self::Product(members) => Box::new(members.iter().map(Member::shape)),
            Self::Choice(alternatives) => Box::new(alternatives.iter()),
            _ => Box::new(std::iter::empty()),
        }
    }
}

impl Member {
    /// The shape this member carries, whether as a member of its own or as a merge.
    fn shape(&self) -> &Term {
        match self {
            Self::Field { shape, .. } | Self::Merge(shape) => shape,
        }
    }

    /// Replaces the references in this member.
    fn expand(&self, definitions: &[(&[String], &Term)], expanding: &mut Vec<Vec<String>>) -> Self {
        match self {
            Self::Field { words, shape } => {
                Self::Field { words: words.clone(), shape: shape.expand(definitions, expanding) }
            }
            Self::Merge(shape) => Self::Merge(shape.expand(definitions, expanding)),
        }
    }

    /// Folds this member into whichever algebra an interpretation provides.
    pub fn fold<A>(&self, alg: &A) -> A::Field
    where
        A: ShapeAlg + FieldAlg,
    {
        match self {
            Self::Field { words, shape } => alg.field(&borrow(words), shape.fold(alg)),
            Self::Merge(shape) => alg.merge(shape.fold(alg)),
        }
    }
}

/// Interprets a shape as the term it is.
#[derive(Debug, Clone, Copy, Default)]
pub struct TermShape;

impl Sorts for TermShape {
    type Ty = Term;
    type Field = Member;
}

impl ShapeAlg for TermShape {
    fn truth(&self) -> Term {
        Term::Truth
    }

    fn unit(&self) -> Term {
        Term::Unit
    }

    fn text(&self) -> Term {
        Term::Text
    }

    fn literal(&self, text: &str) -> Term {
        Term::Literal(text.to_owned())
    }

    fn name_word(&self, words: Words<'_>) -> Term {
        Term::NameWord(own(words))
    }

    fn int(&self, signed: bool, bits: u16) -> Term {
        Term::Int { signed, bits }
    }

    fn float(&self, bits: u16) -> Term {
        Term::Float { bits }
    }

    fn bytes(&self, len: Option<usize>) -> Term {
        Term::Bytes { len }
    }

    fn hex(&self, item: Term) -> Term {
        Term::Hex(Box::new(item))
    }

    fn decimal(&self, item: Term) -> Term {
        Term::Decimal(Box::new(item))
    }

    fn base64(&self, item: Term) -> Term {
        Term::Base64(Box::new(item))
    }

    fn opt(&self, item: Term) -> Term {
        Term::Opt(Box::new(item))
    }

    fn seq(&self, item: Term) -> Term {
        Term::Seq(Box::new(item))
    }

    fn map(&self, key: Term, value: Term) -> Term {
        Term::Map(Box::new(key), Box::new(value))
    }

    fn product(&self, fields: Vec<Member>) -> Term {
        Term::Product(fields)
    }

    fn choice(&self, alternatives: Vec<Term>) -> Term {
        Term::Choice(alternatives)
    }

    fn named(&self, words: Words<'_>, body: Term) -> Term {
        Term::Named { words: own(words), body: Box::new(body) }
    }

    fn reference(&self, words: Words<'_>) -> Term {
        Term::Reference(own(words))
    }
}

impl FieldAlg for TermShape {
    fn field(&self, words: Words<'_>, shape: Term) -> Member {
        Member::Field { words: own(words), shape }
    }

    fn merge(&self, shape: Term) -> Member {
        Member::Merge(shape)
    }
}
