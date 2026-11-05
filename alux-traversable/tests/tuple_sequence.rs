//! The laws of sequencing `Result` through a tuple, from `TUPLE_SEQUENCE.md`, at several widths.

use alux_traversable::{IterTraversableExt, TupleSequence};

type R<T> = Result<T, &'static str>;

/// Identity: a tuple of `Ok`s sequences to `Ok` of their values, whatever their types.
#[test]
fn a_tuple_of_oks_is_ok_of_its_values() {
    assert_eq!(<() as TupleSequence<&str>>::sequence(()), Ok(()));
    assert_eq!((R::Ok(7),).sequence(), Ok((7,)));
    assert_eq!((R::Ok(7), R::Ok(true), R::Ok(vec![1_u8, 22, 33])).sequence(), Ok((7, true, vec![1, 22, 33])));
}

/// Leftmost error: the first `Err` is the result, whatever comes after it.
#[test]
fn the_leftmost_error_wins() {
    assert_eq!((R::Ok(1), R::<bool>::Err("second"), R::<char>::Err("third")).sequence(), Err("second"));
    assert_eq!((R::<u8>::Err("first"), R::Ok(true)).sequence(), Err("first"));
}

/// Naturality: mapping every error, then sequencing, is sequencing, then mapping the error.
#[test]
fn mapping_errors_commutes_with_sequencing() {
    let f = |error: &'static str| error.len();
    for (a, b) in [(Ok(1), Ok('x')), (Err("ab"), Ok('x')), (Ok(1), Err("abc")), (Err("a"), Err("bc"))] {
        let (a, b): (R<u8>, R<char>) = (a, b);
        assert_eq!((a.map_err(f), b.map_err(f)).sequence(), (a, b).sequence().map_err(f));
    }
}

/// Agreement with the iterator instance: a tuple of one type sequences as an array of it does.
#[test]
fn a_tuple_of_one_type_agrees_with_the_iterator_instance() {
    for (a, b, c) in [(Ok(1), Ok(2), Ok(3)), (Ok(1), Err("b"), Err("c")), (Err("a"), Ok(2), Ok(3))] {
        let (a, b, c): (R<u8>, R<u8>, R<u8>) = (a, b, c);
        let tuple = (a, b, c).sequence().map(|(a, b, c)| vec![a, b, c]);
        assert_eq!(tuple, [a, b, c].into_iter().sequence());
    }
}

/// The widest tuple, sixteen elements of sixteen types, sequences too.
#[test]
fn sixteen_elements_of_sixteen_types_sequence() {
    let sixteen = (
        R::Ok(1_u8),
        R::Ok(2_u16),
        R::Ok(3_u32),
        R::Ok(4_u64),
        R::Ok(5_i8),
        R::Ok(6_i16),
        R::Ok(7_i32),
        R::Ok(8_i64),
        R::Ok('a'),
        R::Ok(true),
        R::Ok("s"),
        R::Ok(()),
        R::Ok(1.5_f32),
        R::Ok(2.5_f64),
        R::Ok(vec![1]),
        R::Ok(Some(16)),
    );
    assert_eq!(sixteen.sequence().map(|all| (all.0, all.15)), Ok((1, Some(16))));
}
