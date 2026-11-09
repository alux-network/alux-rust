//! The plain-value interpreter of the source algebra.

use alux_shape::{FieldAlg, ShapeAlg, Sorts, Words};
use alux_shape_term::{Member, Term, TermShape};
use alux_source::{DataAlg, SourceAlg};

/// Reads what source declares as a list of [`Declared`], names and types kept as written, and the
/// shapes data holds as `alux-shape-term` terms.
#[derive(Debug, Clone, Copy, Default)]
pub struct Index;

/// One declaration, and where it is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declared {
    /// The file it is written in; empty when no file placed it.
    pub file: String,
    /// The modules it sits in, outermost first.
    pub modules: Vec<String>,
    /// What it declares.
    pub declaration: Declaration,
}

/// What a declaration states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    /// A trait, the traits it extends, and its other bounds.
    Trait {
        /// The trait's name.
        name: String,
        /// The traits it extends.
        supertraits: Vec<String>,
        /// Every other bound its definition states.
        bounds: Vec<Bound>,
    },
    /// A type implementing a trait.
    Impl {
        /// The trait implemented, as written.
        trait_name: String,
        /// The type implementing it.
        self_ty: String,
        /// The bounds it is implemented under.
        bounds: Vec<Bound>,
    },
    /// A struct or an enum, and the shape it holds.
    Data {
        /// The type's name.
        name: String,
        /// Its type parameters.
        params: Vec<String>,
        /// What it holds, as a shape term.
        shape: Term,
    },
    /// An extension block.
    Extension {
        /// The extension's name.
        name: String,
        /// The carrier its methods are over.
        carrier: String,
        /// The bounds its methods depend on.
        bounds: Vec<Bound>,
        /// The backend reifying it, `laws` for `via = laws`.
        via: Option<String>,
        /// Its methods, in order.
        methods: Vec<String>,
    },
}

/// A type required to implement traits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bound {
    /// The bounded type, as written.
    pub bounded: String,
    /// The traits, as written.
    pub traits: Vec<String>,
}

impl SourceAlg for Index {
    type Name = String;
    type Type = String;
    type Bound = Bound;
    type Source = Vec<Declared>;

    fn name(&self, written: &str) -> String {
        written.to_owned()
    }

    fn ty(&self, written: &str) -> String {
        written.to_owned()
    }

    fn bound(&self, bounded: String, traits: Vec<String>) -> Bound {
        Bound { bounded, traits }
    }

    fn source_empty(&self) -> Vec<Declared> {
        Vec::new()
    }

    fn source_merge(&self, mut left: Vec<Declared>, right: Vec<Declared>) -> Vec<Declared> {
        left.extend(right);
        left
    }

    fn file(&self, path: &str, source: Vec<Declared>) -> Vec<Declared> {
        source.into_iter().map(|declared| Declared { file: path.to_owned(), ..declared }).collect()
    }

    fn module(&self, name: String, source: Vec<Declared>) -> Vec<Declared> {
        source
            .into_iter()
            .map(|mut declared| {
                declared.modules.insert(0, name.clone());
                declared
            })
            .collect()
    }

    fn trait_def(&self, name: String, supertraits: Vec<String>, bounds: Vec<Bound>) -> Vec<Declared> {
        declared(Declaration::Trait { name, supertraits, bounds })
    }

    fn trait_impl(&self, trait_name: String, self_ty: String, bounds: Vec<Bound>) -> Vec<Declared> {
        declared(Declaration::Impl { trait_name, self_ty, bounds })
    }

    fn extension(
        &self,
        name: String,
        carrier: String,
        bounds: Vec<Bound>,
        via: Option<String>,
        methods: Vec<String>,
    ) -> Vec<Declared> {
        declared(Declaration::Extension { name, carrier, bounds, via, methods })
    }
}

impl DataAlg for Index {
    fn data(&self, name: String, params: Vec<String>, shape: Term) -> Vec<Declared> {
        declared(Declaration::Data { name, params, shape })
    }
}

impl Sorts for Index {
    type Ty = Term;
    type Field = Member;
}

/// Shapes are the terms `alux-shape-term` writes down.
impl ShapeAlg for Index {
    fn truth(&self) -> Term {
        TermShape.truth()
    }

    fn unit(&self) -> Term {
        TermShape.unit()
    }

    fn text(&self) -> Term {
        TermShape.text()
    }

    fn literal(&self, text: &str) -> Term {
        TermShape.literal(text)
    }

    fn name_word(&self, words: Words<'_>) -> Term {
        TermShape.name_word(words)
    }

    fn int(&self, signed: bool, bits: u16) -> Term {
        TermShape.int(signed, bits)
    }

    fn float(&self, bits: u16) -> Term {
        TermShape.float(bits)
    }

    fn bytes(&self, len: Option<usize>) -> Term {
        TermShape.bytes(len)
    }

    fn hex(&self, item: Term) -> Term {
        TermShape.hex(item)
    }

    fn decimal(&self, item: Term) -> Term {
        TermShape.decimal(item)
    }

    fn base64(&self, item: Term) -> Term {
        TermShape.base64(item)
    }

    fn opt(&self, item: Term) -> Term {
        TermShape.opt(item)
    }

    fn seq(&self, item: Term) -> Term {
        TermShape.seq(item)
    }

    fn map(&self, key: Term, value: Term) -> Term {
        TermShape.map(key, value)
    }

    fn tuple(&self, items: Vec<Term>) -> Term {
        TermShape.tuple(items)
    }

    fn product(&self, fields: Vec<Member>) -> Term {
        TermShape.product(fields)
    }

    fn choice(&self, alternatives: Vec<Term>) -> Term {
        TermShape.choice(alternatives)
    }

    fn named(&self, words: Words<'_>, body: Term) -> Term {
        TermShape.named(words, body)
    }

    fn reference(&self, words: Words<'_>) -> Term {
        TermShape.reference(words)
    }
}

impl FieldAlg for Index {
    fn field(&self, words: Words<'_>, shape: Term) -> Member {
        TermShape.field(words, shape)
    }

    fn merge(&self, shape: Term) -> Member {
        TermShape.merge(shape)
    }
}

/// One declaration, placed nowhere yet.
fn declared(declaration: Declaration) -> Vec<Declared> {
    vec![Declared { file: String::new(), modules: Vec::new(), declaration }]
}
