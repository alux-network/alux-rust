//! A layout's shape as a term: what each member's type states, read from how it is spelled.
//!
//! The `Shape` derive asks the compiler which `ShapeOf` a member's type has. A reader of source
//! cannot, so it states the shapes the standard types state by their spelling, as `alux-shape` gives
//! them, and refers to any other type by its name.

use alux_ext::ext;
use alux_shape::{FieldAlg, ShapeAlg, ShapeExt, ShapeTaggedExt};
use alux_shape_syn::{Body, Form, Layout, Member, Tagging, Written, of_camel};
use syn::{GenericArgument, PathArguments, Type};

/// States a layout's shape through the shape algebra.
#[ext(name = LayoutShapeExt)]
pub(crate) impl<This> This
where
    This: ShapeAlg + FieldAlg,
{
    /// Returns the shape `layout` states, under its name.
    fn layout_shape(&self, layout: &Layout) -> This::Ty {
        let body = match &layout.body {
            Body::Product(members) => {
                let fields = members
                    .iter()
                    .map(|member| match member {
                        Member::Field(words, form) => self.field(&words_of(words), self.form_shape(form)),
                        Member::Merge(form) => self.merge(self.form_shape(form)),
                    })
                    .collect();
                self.product(fields)
            }
            Body::Positional(forms) => {
                let mut shapes: Vec<_> = forms.iter().map(|form| self.form_shape(form)).collect();
                match shapes.len() {
                    0 => self.unit(),
                    1 => shapes.remove(0),
                    _ => self.tuple(shapes),
                }
            }
            Body::Choice(tagging, alternatives) => {
                let names: Vec<Vec<&str>> =
                    alternatives.iter().map(|alternative| words_of(&alternative.words)).collect();
                if layout.body.names_only() {
                    self.sum_of_names(names.iter().map(Vec::as_slice).collect())
                } else {
                    let carried = alternatives
                        .iter()
                        .zip(&names)
                        .map(|(alternative, name)| {
                            let shape =
                                alternative.carries.as_ref().map_or_else(|| self.unit(), |form| self.form_shape(form));
                            (name.as_slice(), shape)
                        })
                        .collect();
                    match tagging {
                        Tagging::External => self.sum_external(carried),
                        Tagging::Untagged => self.sum_untagged(carried),
                        Tagging::Internal(tag) => self.sum_internal(&words_of(tag), carried),
                        Tagging::Adjacent(tag, content) => {
                            self.sum_adjacent(&words_of(tag), &words_of(content), carried)
                        }
                    }
                }
            }
        };
        self.named(&words_of(&layout.words), body)
    }

    /// Returns the shape a member's form states.
    fn form_shape(&self, form: &Form) -> This::Ty {
        match form {
            Form::Type(ty) => self.type_shape(ty),
            Form::Written(written) => self.written_shape(written),
        }
    }

    /// Returns the shape a member writes itself.
    fn written_shape(&self, written: &Written) -> This::Ty {
        match written {
            Written::Truth => self.truth(),
            Written::Text => self.text(),
            Written::Hex(len) => self.bytes_hex(*len),
            Written::Decimal(bits) => self.int_decimal(false, *bits),
            Written::Int(bits) => self.int(false, *bits),
            Written::Named(words, item) => self.named(&words_of(words), self.written_shape(item)),
            Written::Seq(item) => self.seq(self.written_shape(item)),
            Written::Opt(item) => self.opt(self.written_shape(item)),
        }
    }

    /// Returns the shape a type states by its spelling: a standard type as `alux-shape` states it, any
    /// other type a reference to its name.
    fn type_shape(&self, ty: &Type) -> This::Ty {
        match ty {
            Type::Reference(borrowed) => self.type_shape(&borrowed.elem),
            Type::Paren(inner) => self.type_shape(&inner.elem),
            Type::Slice(slice) => self.seq(self.type_shape(&slice.elem)),
            Type::Array(array) => self.seq(self.type_shape(&array.elem)),
            Type::Tuple(tuple) if tuple.elems.is_empty() => self.unit(),
            Type::Tuple(tuple) => self.tuple(tuple.elems.iter().map(|item| self.type_shape(item)).collect()),
            Type::Path(path) if path.qself.is_none() => {
                let Some(last) = path.path.segments.last() else { return self.unit() };
                let arguments = type_arguments(&last.arguments);
                let name = last.ident.to_string();
                match (name.as_str(), arguments.as_slice()) {
                    ("bool", []) => self.truth(),
                    ("String" | "str", []) => self.text(),
                    ("f32", []) => self.float(32),
                    ("f64", []) => self.float(64),
                    ("u8", []) => self.int(false, 8),
                    ("u16", []) => self.int(false, 16),
                    ("u32", []) => self.int(false, 32),
                    ("u64" | "usize", []) => self.int(false, 64),
                    ("u128", []) => self.int(false, 128),
                    ("i8", []) => self.int(true, 8),
                    ("i16", []) => self.int(true, 16),
                    ("i32", []) => self.int(true, 32),
                    ("i64" | "isize", []) => self.int(true, 64),
                    ("i128", []) => self.int(true, 128),
                    ("Option", [item]) => self.opt(self.type_shape(item)),
                    ("Vec", [item]) => self.seq(self.type_shape(item)),
                    ("Box", [item]) => self.type_shape(item),
                    ("HashMap" | "BTreeMap", [key, value]) => self.map(self.type_shape(key), self.type_shape(value)),
                    _ => self.reference(&words_of(&of_camel(&name))),
                }
            }
            _ => self.reference(&[]),
        }
    }
}

/// A name's words, as the algebra takes them.
fn words_of(words: &[String]) -> Vec<&str> {
    words.iter().map(String::as_str).collect()
}

/// The type arguments of a path segment.
fn type_arguments(arguments: &PathArguments) -> Vec<&Type> {
    match arguments {
        PathArguments::AngleBracketed(angled) => angled
            .args
            .iter()
            .filter_map(|argument| match argument {
                GenericArgument::Type(ty) => Some(ty),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}
