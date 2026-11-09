//! Token-level expansion checks of the `laws` backend: what a block lowers to, and what it refuses.

use super::laws_program_defunc_internal;
use proc_macro2::TokenStream;
use quote::quote;

/// The expansion of a block, or the error it is refused with, as text.
fn expand(attr: TokenStream, item: TokenStream) -> String {
    laws_program_defunc_internal(attr, item).map_or_else(|error| error.to_string(), |tokens| tokens.to_string())
}

/// Each law becomes `Op_<law>` with its name, documentation, argument names and check, and the
/// block states the suite function returning the program.
#[test]
fn lowers_each_law_to_an_operation_and_adds_the_suite() {
    let output = expand(
        quote!(name = CounterLaws),
        quote! {
            pub impl<This> This where This: CounterAlg {
                /// Bumping adds one.
                fn bump_adds_one(&mut self) {}
                fn add_is_repeated_bump(&mut self, steps: &Steps) {}
            }
        },
    );
    assert!(output.contains("struct Op_bump_adds_one < This >"), "{output}");
    assert!(output.contains("const DOC : & 'static str = \"Bumping adds one.\""), "{output}");
    assert!(output.contains("const ARG_NAMES : & 'static [& 'static str] = & [stringify ! (steps)]"), "{output}");
    assert!(output.contains(":: alux_laws :: LawOp < This > for Op_add_is_repeated_bump < This >"), "{output}");
    assert!(output.contains("const NAME : & 'static str = stringify ! (bump_adds_one)"), "{output}");
    assert!(output.contains("fn counter_laws < This > () -> Program_counter_laws < This >"), "{output}");
}

/// The program is tagless-final: it folds each law through `LawAlg` in the order the block states
/// them, and asks one `LawAlg` per input type, not per law.
#[test]
fn folds_each_law_through_the_compiler_in_block_order() {
    let output = expand(
        quote!(name = PoolLaws),
        quote! {
            pub impl<This> This where This: PoolAlg {
                fn debts_net(&mut self, pair: &DebtPair) {}
                fn groups_split(&mut self, pair: &GroupPair) {}
                fn debts_clear(&self, pair: &DebtPair) {}
            }
        },
    );
    assert!(output.contains(":: alux_laws :: LawsProgramAlg < Compiler > for Program_pool_laws < This >"), "{output}");
    // One `LawAlg` per input type, not per law.
    assert_eq!(output.matches("Compiler : :: alux_laws :: LawAlg < This , DebtPair >").count(), 1, "{output}");
    assert_eq!(output.matches("Compiler : :: alux_laws :: LawAlg < This , GroupPair >").count(), 1, "{output}");
    let debts_net = output.find("Op_debts_net < This > as").expect("folds debts_net");
    let groups_split = output.find("Op_groups_split < This > as").expect("folds groups_split");
    let debts_clear = output.find("Op_debts_clear < This > as").expect("folds debts_clear");
    assert!(debts_net < groups_split && groups_split < debts_clear, "{output}");
}

/// One suite holds laws over different input types; each operation states its own.
#[test]
fn laws_of_one_suite_may_take_different_inputs() {
    let output = expand(
        quote!(name = PoolLaws),
        quote! {
            pub impl<This> This where This: PoolAlg {
                fn debts_net(&mut self, pair: &DebtPair) {}
                fn groups_split(&mut self, pair: &GroupPair) {}
            }
        },
    );
    assert!(output.contains("type Input = DebtPair"), "{output}");
    assert!(output.contains("type Input = GroupPair"), "{output}");
}

#[test]
fn refuses_what_is_no_law() {
    let block = |method| quote!(pub impl<This> This where This: A { #method });
    let cases = [
        (
            quote!(
                async fn a(&self) {}
            ),
            "a law is not async",
        ),
        (
            quote!(
                fn a(self) {}
            ),
            "a law borrows its subject",
        ),
        (
            quote!(
                fn a(&self, x: u32) {}
            ),
            "a law's input is borrowed",
        ),
        (
            quote!(
                fn a(&self, x: &u32, y: &u32) {}
            ),
            "a law takes at most one input",
        ),
        (
            quote!(
                fn a<T>(&self) {}
            ),
            "a law takes no generic parameters",
        ),
    ];
    for (method, why) in cases {
        let output = expand(quote!(name = ALaws), block(method));
        assert!(output.contains(why), "{why}: {output}");
    }
}

/// A suite is no other program, so `via = laws` does not chain with other backends.
#[test]
fn refuses_to_be_lowered_with_another_backend() {
    let output = expand(
        quote!(name = ALaws, __alux_then(jsonrpc)),
        quote!(
            pub impl<This> This where This: A {}
        ),
    );
    assert!(output.contains("lowers a block alone"), "{output}");
}

/// Every refused law of a block is reported in one compile, not one per fix.
#[test]
fn every_refusal_of_a_block_is_reported_at_once() {
    let error = laws_program_defunc_internal(
        quote!(name = ALaws),
        quote!(
            pub impl<This> This
            where
                This: A,
            {
                async fn a(&self) {}
                fn b(self) {}
            }
        ),
    )
    .expect_err("both laws are refused");
    let messages = error.into_iter().map(|error| error.to_string()).collect::<Vec<_>>();
    assert_eq!(messages, ["`a`: a law is not async", "`b`: a law borrows its subject: take `&self` or `&mut self`"]);
}

/// A suite is named by its block, because reports and the suite function read the name.
#[test]
fn refuses_a_suite_without_a_name() {
    let output = expand(
        quote!(),
        quote!(
            pub impl<This> This where This: A {}
        ),
    );
    assert!(output.contains("a laws suite needs a name"), "{output}");
}

/// A suite takes only type parameters, which its operations and program can carry.
#[test]
fn refuses_a_lifetime_parameter() {
    let output = expand(
        quote!(name = ALaws),
        quote!(
            pub impl<'a, This> This where This: A<'a> {}
        ),
    );
    assert!(output.contains("takes only type parameters"), "{output}");
}

/// A `?Sized` parameter stays `?Sized` on the generated operation and suite function.
#[test]
fn keeps_an_unsized_parameter_unsized() {
    let output = expand(
        quote!(name = UnsizedLaws),
        quote!(
            pub impl<T, U> ()
            where
                T: ?Sized + PartialEq,
                U: PartialEq,
            {
                fn a(&self, at: &(Box<T>, U)) {}
            }
        ),
    );
    assert!(output.contains("struct Op_a < T : ? Sized , U >"), "{output}");
    assert!(output.contains("fn unsized_laws < T : ? Sized , U > ()"), "{output}");
}

/// `Self` in an input type is the subject, written out, since operations are declared outside the
/// block.
#[test]
fn names_self_in_an_input_by_the_subject() {
    let output = expand(
        quote!(name = KeyedLaws),
        quote!(
            pub impl<This> This
            where
                This: Keyed,
            {
                fn a(&self, key: &Self::Key) {}
            }
        ),
    );
    assert!(output.contains("type Input = This :: Key"), "{output}");
}
