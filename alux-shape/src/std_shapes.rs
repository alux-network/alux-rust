//! What the standard types describe.
//!
//! Each states what `serde` writes for that type and nothing more: an integer is a number of its
//! width, not a number written some particular way, because how a quantity should appear is a
//! domain's statement rather than a width's. A domain that writes a `u128` as decimal text says so
//! in its own shape.

use crate::{ShapeAlg, ShapeOf};
use std::collections::{BTreeMap, HashMap};

/// States a shape for one integer width.
macro_rules! int_shapes {
    ($($ty:ty => ($signed:literal, $bits:literal)),* $(,)?) => {
        $(
            impl<A> ShapeOf<A> for $ty
            where
                A: ShapeAlg,
            {
                type Shape = A::Ty;

                fn shape_of(alg: &A) -> A::Ty {
                    alg.int($signed, $bits)
                }
            }
        )*
    };
}

int_shapes! {
    u8 => (false, 8), u16 => (false, 16), u32 => (false, 32), u64 => (false, 64), u128 => (false, 128),
    i8 => (true, 8), i16 => (true, 16), i32 => (true, 32), i64 => (true, 64), i128 => (true, 128),
    usize => (false, 64), isize => (true, 64),
}

impl<A> ShapeOf<A> for bool
where
    A: ShapeAlg,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        alg.truth()
    }
}

impl<A> ShapeOf<A> for f32
where
    A: ShapeAlg,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        alg.float(32)
    }
}

impl<A> ShapeOf<A> for f64
where
    A: ShapeAlg,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        alg.float(64)
    }
}

impl<A> ShapeOf<A> for String
where
    A: ShapeAlg,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        alg.text()
    }
}

impl<A> ShapeOf<A> for str
where
    A: ShapeAlg,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        alg.text()
    }
}

impl<A> ShapeOf<A> for ()
where
    A: ShapeAlg,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        alg.unit()
    }
}

impl<A, T> ShapeOf<A> for Option<T>
where
    A: ShapeAlg,
    T: ShapeOf<A, Shape = A::Ty>,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        alg.opt(T::shape_of(alg))
    }
}

impl<A, T> ShapeOf<A> for Vec<T>
where
    A: ShapeAlg,
    T: ShapeOf<A, Shape = A::Ty>,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        alg.seq(T::shape_of(alg))
    }
}

impl<A, T> ShapeOf<A> for [T]
where
    A: ShapeAlg,
    T: ShapeOf<A, Shape = A::Ty>,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        alg.seq(T::shape_of(alg))
    }
}

impl<A, T, const N: usize> ShapeOf<A> for [T; N]
where
    A: ShapeAlg,
    T: ShapeOf<A, Shape = A::Ty>,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        alg.seq(T::shape_of(alg))
    }
}

impl<A, T> ShapeOf<A> for Box<T>
where
    A: ShapeAlg,
    T: ShapeOf<A, Shape = A::Ty>,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        T::shape_of(alg)
    }
}

impl<A, K, V> ShapeOf<A> for HashMap<K, V>
where
    A: ShapeAlg,
    K: ShapeOf<A, Shape = A::Ty>,
    V: ShapeOf<A, Shape = A::Ty>,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        alg.map(K::shape_of(alg), V::shape_of(alg))
    }
}

impl<A, K, V> ShapeOf<A> for BTreeMap<K, V>
where
    A: ShapeAlg,
    K: ShapeOf<A, Shape = A::Ty>,
    V: ShapeOf<A, Shape = A::Ty>,
{
    type Shape = A::Ty;

    fn shape_of(alg: &A) -> A::Ty {
        alg.map(K::shape_of(alg), V::shape_of(alg))
    }
}
