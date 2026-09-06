//! Shape program backend for extension defunctionalization.
//!
//! The backend states what a shape declaration means: the method names the shape, an identifier in
//! name position states the words that name a member, and the record the declaration opens is closed
//! for it. Unlike a transport declaration a shape body applies no handler — it calls the algebra
//! directly — so it needs no per-operation evidence. Everything shared with other backends lives in
//! [`crate::lower`] and [`crate::syntax`].

use crate::lower::{LoweredProgram, ProgramBackendAlg, expand_program};
use proc_macro2::TokenStream;
use quote::quote;
use syn::visit_mut::{self, VisitMut};
use syn::{Block, Expr, ExprMethodCall, Ident, ImplItemFn, Stmt, parse_quote};

/// Interprets the shared lowering as a shape program.
struct ShapeBackend;

/// Reads an identifier as the words it states, splitting on `_` and nothing else.
fn words(name: &Ident) -> Vec<String> {
    name.to_string().split('_').filter(|word| !word.is_empty()).map(str::to_owned).collect()
}

/// Writes a name as the word sequence a shape algebra reads.
fn stated(words: &[String]) -> Expr {
    parse_quote!(&[#(#words),*])
}

/// Lowers each name a declaration states as an identifier.
struct Names;

impl VisitMut for Names {
    fn visit_expr_method_call_mut(&mut self, call: &mut ExprMethodCall) {
        if call.method == "field" {
            let named = match call.args.first() {
                Some(Expr::Path(path)) => path.path.get_ident().map(words),
                // A name no identifier spells is written as a literal, and passes through as authored.
                _ => None,
            };
            if let (Some(named), Some(argument)) = (named, call.args.first_mut()) {
                *argument = stated(&named);
            }
        }
        visit_mut::visit_expr_method_call_mut(self, call);
    }
}

/// Names the record a declaration opens, which is the base of the body's own call chain.
///
/// Only that record is named. A record opened anywhere else states a shape of its own and would need
/// its own name, which a declaration gives by being a declaration.
///
/// So a member cannot be an inline anonymous record, and that turns out to be the point rather than a
/// limitation. A nested shape reaches the fold by name, which is what lets `stored_user` read as
/// `stored_user { ..user { … }, ..timestamps { … } }`: the parts a reader can look up are still named
/// in the result. An anonymous inline record would have flattened them into one shape stating nothing
/// about where its members came from.
fn name_record(block: &mut Block, named: &[String]) {
    let Some(Stmt::Expr(tail, _)) = block.stmts.last_mut() else { return };
    let mut current = tail;
    loop {
        let Expr::MethodCall(call) = current else { return };
        if call.method == "record" && call.args.is_empty() {
            call.args.push(stated(named));

            return;
        }
        current = &mut call.receiver;
    }
}

impl ProgramBackendAlg for ShapeBackend {
    /// A shape program states nothing once for all of its declarations.
    type Defaults = ();

    const NESTED_SUFFIX: &'static str = "_shape";
    const REJECTED_PARAM: &'static str = "shape programs currently support type parameters only";

    fn require_declarations(method: &mut ImplItemFn, (): &Self::Defaults) {
        // The method is the shape's name, and this is the only stage that reads it: the lowering
        // carries the derived program type from here on, not the name it came from.
        let mut named = words(&method.sig.ident);
        if named.last().is_some_and(|word| word == Self::NESTED_SUFFIX.trim_start_matches('_')) {
            named.pop();
        }
        Names.visit_block_mut(&mut method.block);
        name_record(&mut method.block, &named);
    }

    fn require_subprogram(program: &TokenStream) -> TokenStream {
        quote! {
            #program: ::alux_shape::ShapeProgramAlg<This, Ty = <This as ::alux_shape::Sorts>::Ty>
        }
    }

    fn compile_program(lowered: &LoweredProgram) -> TokenStream {
        let LoweredProgram { program_type, compiler_params, predicates, body } = lowered;
        quote! {
            impl<This, #compiler_params> ::alux_shape::ShapeProgramAlg<This> for #program_type
            where
                #(#predicates,)*
            {
                type Ty = <This as ::alux_shape::Sorts>::Ty;

                fn compile_shape(self, alg: &This) -> Self::Ty {
                    let _ = self;
                    // Every `self` a shape body states is the algebra, so that is what it becomes.
                    let builder = alg;
                    let record = #body;

                    record.into_shape()
                }
            }
        }
    }
}

/// Expands the facade macro after converting compiler token streams into testable tokens.
pub(crate) fn shape_program_defunc_internal(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    expand_program::<ShapeBackend>(attr, item, &())
}

#[cfg(test)]
mod tests {
    use super::shape_program_defunc_internal;
    use quote::quote;

    #[test]
    fn states_a_member_name_as_the_words_its_identifier_spells() {
        let output = shape_program_defunc_internal(
            quote!(name = UserShapeExt),
            quote! {
                pub impl<This> This
                where
                    This: ShapeAlg + FieldAlg,
                {
                    /// A user, as a surface answers one.
                    fn user_shape(&self) {
                        self.record().field(display_name, self.text()).field(email, self.opt(self.text()))
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains(r#"field (& ["display" , "name"] ,"#));
        assert!(output.contains(r#"field (& ["email"] ,"#));
        // The body reaches the algebra through the same `self` the author wrote.
        assert!(output.contains("builder . text ()"), "the shape body lost its algebra");
        assert!(output.contains("let builder = alg ;"));
    }

    #[test]
    fn keeps_a_name_no_identifier_spells() {
        let output = shape_program_defunc_internal(
            quote!(name = UserShapeExt),
            quote! {
                pub impl<This> This
                where
                    This: ShapeAlg + FieldAlg,
                {
                    /// A user whose member is named by a word Rust cannot spell.
                    fn user_shape(&self) {
                        self.record().field("2fa-enabled", self.truth())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains(r#"field ("2fa-enabled" ,"#));
    }

    #[test]
    fn names_the_shape_after_the_method_without_its_suffix() {
        let output = shape_program_defunc_internal(
            quote!(name = StoredUserShapeExt),
            quote! {
                pub impl<This> This
                where
                    This: ShapeAlg + FieldAlg,
                {
                    /// A user as it is stored.
                    fn stored_user_shape(&self) {
                        self.record().field(id, self.text())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains(r#"record (& ["stored" , "user"])"#));
        assert!(output.contains("struct StoredUserShapeProgram"));
        // The closing call is the backend's, so no declaration writes one.
        assert!(output.contains("record . into_shape ()"));
    }

    #[test]
    fn reads_a_suffixed_call_as_a_nested_shape() {
        let output = shape_program_defunc_internal(
            quote!(name = StoredUserShapeExt),
            quote! {
                pub impl<This> This
                where
                    This: ShapeAlg + FieldAlg,
                {
                    /// A user as it is stored: a profile and its stamps, observed as one.
                    fn stored_user_shape(&self) {
                        self.record().merge(self.profile_shape()).merge(self.timestamps_shape())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("ProfileShapeProgram : :: alux_shape :: ShapeProgramAlg < This"));
        assert!(output.contains("TimestampsShapeProgram : :: alux_shape :: ShapeProgramAlg < This"));
        assert!(output.contains("builder . program (builder . profile_shape"));
    }

    #[test]
    fn rejects_non_type_program_parameters() {
        let error = shape_program_defunc_internal(
            quote!(name = InvalidShapeExt),
            quote! {
                pub impl<This> This {
                    fn invalid_shape<const N: usize>(&self) {
                        self.record()
                    }
                }
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("type parameters only"));
    }
}
