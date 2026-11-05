//! Reading source: every declaration of a file, as a term over the source algebra.

use crate::shape::LayoutShapeExt;
use crate::text::text;
use alux_ext::ext;
use alux_shape_syn::{Layout, read_layout};
use alux_source::{DataAlg, SourceFoldExt};
use std::path::Path;
use std::{fmt, fs, io};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    Attribute, DeriveInput, Generics, ImplItem, Item, ItemImpl, ItemTrait, Meta, Token, TraitItem, TypeParamBound,
    Visibility, WherePredicate,
};

/// Reads Rust source as a term over the source algebra.
#[ext(name = ReadSourceExt)]
pub impl<This, Source> This
where
    This: DataAlg<Source = Source>,
{
    /// Returns what `source`, the text of `file`, declares, placed in `file`.
    ///
    /// # Errors
    ///
    /// Returns the parse error when `source` is not Rust.
    fn read_source(&self, file: &str, source: &str) -> Result<Source, syn::Error> {
        let parsed: SourceFile = syn::parse_str(source)?;
        let mut found = Vec::new();
        written(&[], &parsed.0, &mut found);
        let bounds = |bounds: Vec<WrittenBound>| {
            let bound = |(bounded, traits): WrittenBound| {
                self.bound(self.ty(&bounded), traits.iter().map(|path| self.name(path)).collect())
            };
            bounds.into_iter().map(bound).collect::<Vec<_>>()
        };
        let names = |names: Vec<String>| names.iter().map(|name| self.name(name)).collect::<Vec<_>>();
        let declared = found.into_iter().map(|(modules, declaration)| {
            let declared = match declaration {
                Written::Trait { name, supertraits, bounds: stated } => {
                    self.trait_def(self.name(&name), names(supertraits), bounds(stated))
                }
                Written::Impl { trait_name, self_ty, bounds: stated } => {
                    self.trait_impl(self.name(&trait_name), self.ty(&self_ty), bounds(stated))
                }
                Written::Data { name, params, layout } => {
                    self.data(self.name(&name), names(params), self.layout_shape(&layout))
                }
                Written::Extension { name, carrier, bounds: stated, via, methods } => self.extension(
                    self.name(&name),
                    self.ty(&carrier),
                    bounds(stated),
                    via.map(|via| self.name(&via)),
                    names(methods),
                ),
            };
            modules.iter().rev().fold(declared, |inner, module| self.module(self.name(module), inner))
        });
        Ok(self.file(file, self.source_all(declared.collect::<Vec<_>>())))
    }

    /// Returns what every `.rs` file under `root` declares, in path order, skipping `target`.
    ///
    /// # Errors
    ///
    /// Returns the first file that cannot be read or parsed.
    fn read_tree(&self, root: &Path) -> Result<Source, ReadError> {
        let mut files = Vec::new();
        rust_files(root, &mut files).map_err(|why| ReadError::Io(root.display().to_string(), why))?;
        files.sort();
        let mut all = Vec::new();
        for file in files {
            let name = file.display().to_string();
            let source = fs::read_to_string(&file).map_err(|why| ReadError::Io(name.clone(), why))?;
            all.push(self.read_source(&name, &source).map_err(|why| ReadError::Parse(name, why))?);
        }
        Ok(self.source_all(all))
    }
}

/// Why a tree could not be read.
#[derive(Debug)]
pub enum ReadError {
    /// A file or directory could not be read.
    Io(String, io::Error),
    /// A file is not Rust.
    Parse(String, syn::Error),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::Io(at, why) => write!(f, "cannot read {at}: {why}"),
            ReadError::Parse(at, why) => write!(f, "cannot parse {at}: {why}"),
        }
    }
}

impl std::error::Error for ReadError {}

/// A source file, item by item. `syn` refuses `pub impl`, which an extension block writes and its
/// attribute removes before the compiler sees it, so the reader parses such a block, and the inline
/// modules that may hold one, itself.
struct SourceFile(Vec<Read>);

/// One item as the reader keeps it.
enum Read {
    /// An item `syn` parses.
    Item(Item),
    /// `pub impl ..`, an extension block.
    Impl(ItemImpl),
    /// An inline module and its items.
    Mod(String, Vec<Read>),
}

impl Parse for SourceFile {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        input.call(Attribute::parse_inner)?;
        Ok(SourceFile(items(input)?))
    }
}

/// The items of `input`, to its end.
fn items(input: ParseStream<'_>) -> syn::Result<Vec<Read>> {
    let mut read = Vec::new();
    while !input.is_empty() {
        let ahead = input.fork();
        let attrs = ahead.call(Attribute::parse_outer)?;
        let visibility: Visibility = ahead.parse()?;
        let public = !matches!(visibility, Visibility::Inherited);
        if public && ahead.peek(Token![impl]) {
            input.call(Attribute::parse_outer)?;
            input.parse::<Visibility>()?;
            let mut imp: ItemImpl = input.parse()?;
            imp.attrs = attrs;
            read.push(Read::Impl(imp));
        } else if ahead.peek(Token![mod]) && ahead.peek2(syn::Ident) && ahead.peek3(syn::token::Brace) {
            input.call(Attribute::parse_outer)?;
            input.parse::<Visibility>()?;
            input.parse::<Token![mod]>()?;
            let name: syn::Ident = input.parse()?;
            let content;
            syn::braced!(content in input);
            content.call(Attribute::parse_inner)?;
            read.push(Read::Mod(name.to_string(), items(&content)?));
        } else {
            read.push(Read::Item(input.parse()?));
        }
    }
    Ok(read)
}

/// A bound as written: the bounded type and its traits.
type WrittenBound = (String, Vec<String>);

/// One declaration as written, before it is stated through the algebra.
enum Written {
    Trait { name: String, supertraits: Vec<String>, bounds: Vec<WrittenBound> },
    Impl { trait_name: String, self_ty: String, bounds: Vec<WrittenBound> },
    Data { name: String, params: Vec<String>, layout: Layout },
    Extension { name: String, carrier: String, bounds: Vec<WrittenBound>, via: Option<String>, methods: Vec<String> },
}

/// Collects into `found` each declaration of `items`, with the modules it sits in below `modules`.
fn written(modules: &[String], items: &[Read], found: &mut Vec<(Vec<String>, Written)>) {
    for item in items {
        match item {
            Read::Item(Item::Trait(def)) => found.push((modules.to_vec(), trait_def(def))),
            Read::Item(Item::Struct(data)) => {
                found.extend(data_declaration(&data.clone().into()).map(|declared| (modules.to_vec(), declared)));
            }
            Read::Item(Item::Enum(data)) => {
                found.extend(data_declaration(&data.clone().into()).map(|declared| (modules.to_vec(), declared)));
            }
            Read::Item(Item::Impl(imp)) | Read::Impl(imp) => {
                found.extend(impl_declaration(imp).map(|declaration| (modules.to_vec(), declaration)));
            }
            Read::Item(_) => {}
            Read::Mod(name, inner) => {
                let mut inside = modules.to_vec();
                inside.push(name.clone());
                written(&inside, inner, found);
            }
        }
    }
}

/// A struct or an enum, with the shape it states; nothing for one no shape is read from, such as a
/// tuple struct.
fn data_declaration(input: &DeriveInput) -> Option<Written> {
    let layout = read_layout(input).ok()?;
    let params = input.generics.type_params().map(|param| param.ident.to_string()).collect();
    Some(Written::Data { name: input.ident.to_string(), params, layout })
}

/// An `impl` block: an extension when `#[ext]` marks it, a trait implementation when it names a
/// trait, nothing otherwise.
fn impl_declaration(imp: &ItemImpl) -> Option<Written> {
    let ext = imp.attrs.iter().find(|attr| attr.path().segments.last().is_some_and(|last| last.ident == "ext"));
    if let Some(ext) = ext {
        return Some(extension(ext, imp));
    }
    let (trait_path, _) = imp.trait_.as_ref()?;
    Some(Written::Impl { trait_name: text(trait_path), self_ty: text(&imp.self_ty), bounds: bounds(&imp.generics) })
}

/// An `#[ext(..)]` block.
fn extension(attr: &Attribute, imp: &ItemImpl) -> Written {
    let arguments = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap_or_default();
    let mut name = String::new();
    let mut via = None;
    for argument in &arguments {
        match argument {
            Meta::NameValue(pair) if pair.path.is_ident("name") => name = text(&pair.value),
            Meta::List(list) if list.path.is_ident("defunc") => {
                let inner = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap_or_default();
                via = inner.iter().find_map(|meta| match meta {
                    Meta::NameValue(pair) if pair.path.is_ident("via") => Some(text(&pair.value)),
                    _ => None,
                });
            }
            _ => {}
        }
    }
    let methods = imp
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Fn(method) => Some(method.sig.ident.to_string()),
            _ => None,
        })
        .collect();
    Written::Extension { name, carrier: text(&imp.self_ty), bounds: bounds(&imp.generics), via, methods }
}

/// A trait definition: its supertraits apart from every other bound it states.
fn trait_def(def: &ItemTrait) -> Written {
    let mut stated = bounds(&def.generics);
    for item in &def.items {
        match item {
            TraitItem::Fn(method) => stated.extend(bounds(&method.sig.generics)),
            TraitItem::Type(assoc) => {
                stated.extend(bounds(&assoc.generics));
                let traits = traits(&assoc.bounds);
                if !traits.is_empty() {
                    stated.push((format!("Self::{}", assoc.ident), traits));
                }
            }
            _ => {}
        }
    }
    Written::Trait { name: def.ident.to_string(), supertraits: traits(&def.supertraits), bounds: stated }
}

/// The bounds `generics` state on types, on parameters and in the `where` clause, each with at
/// least one trait.
fn bounds(generics: &Generics) -> Vec<WrittenBound> {
    let on_params = generics.type_params().map(|param| (param.ident.to_string(), traits(&param.bounds)));
    let in_where =
        generics.where_clause.iter().flat_map(|clause| &clause.predicates).filter_map(|predicate| match predicate {
            WherePredicate::Type(bound) => Some((text(&bound.bounded_ty), traits(&bound.bounds))),
            _ => None,
        });
    on_params.chain(in_where).filter(|(_, traits)| !traits.is_empty()).collect()
}

/// The traits of `bounds`, as written, `?Sized` and lifetimes left out.
fn traits(bounds: &Punctuated<TypeParamBound, Token![+]>) -> Vec<String> {
    bounds
        .iter()
        .filter_map(|bound| match bound {
            TypeParamBound::Trait(bound) if bound.maybe.is_none() => Some(text(&bound.path)),
            _ => None,
        })
        .collect()
}

/// Collects the `.rs` files under `dir`, skipping `target` and hidden directories.
fn rust_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        if path.is_dir() {
            if name != "target" && !name.starts_with('.') {
                rust_files(&path, files)?;
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(())
}
