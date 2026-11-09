//! Token-level checks of the tuple-width generators.

use super::{for_tuple_positions, for_tuples};
use quote::quote;

/// One invocation per width, each naming its type parameters with their positions; width 0 states
/// none.
#[test]
fn invokes_the_callback_once_per_width() {
    let output = for_tuples(quote!(sequence_at, 0..=2)).unwrap().to_string();
    assert_eq!(output, "sequence_at ! () ; sequence_at ! (A1 => 0) ; sequence_at ! (A1 => 0 , A2 => 1) ;");
}

/// One invocation per position of each width, the position and the type at it first, which a
/// `macro_rules!` callback could not pick out of the list itself.
#[test]
fn invokes_the_callback_once_per_position_of_each_width() {
    let output = for_tuple_positions(quote!(arg_at, 1..=2)).unwrap().to_string();
    assert_eq!(
        output,
        "arg_at ! (0 => A1 ; A1 => 0) ; arg_at ! (0 => A1 ; A1 => 0 , A2 => 1) ; arg_at ! (1 => A2 ; A1 => 0 , A2 => 1) ;"
    );
}

/// A range that is not inclusive, or runs backwards, is refused with what it should be.
#[test]
fn refuses_a_range_it_cannot_enumerate() {
    let open = for_tuples(quote!(m, 0..16)).unwrap_err().to_string();
    assert!(open.contains("inclusive range"), "{open}");
    let backwards = for_tuples(quote!(m, 3..=1)).unwrap_err().to_string();
    assert!(backwards.contains("narrower to the wider"), "{backwards}");
}

/// `named` passes each width's type name and number, and `each` gives every position identifiers
/// of its own, its letter and its index: what a family of types such as `Sum2` through `Sum16` needs.
#[test]
fn names_each_width_and_each_position_when_asked() {
    let output = for_tuples(quote!(sum, 2..=2, named = Sum, each = [S, a])).unwrap().to_string();
    assert_eq!(output, "sum ! (Sum2 , 2 ; (S0 , a0 , A , 0) , (S1 , a1 , B , 1)) ;");
    let unnamed = for_tuples(quote!(sources, 1..=1, each = [S])).unwrap().to_string();
    assert_eq!(unnamed, "sources ! ((S0 , A , 0)) ;");
}
