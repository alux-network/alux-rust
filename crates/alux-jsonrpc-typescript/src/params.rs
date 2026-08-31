//! An argument product, as the parameter types a call takes.
//!
//! A tuple is not a shape: it has no members of its own, only an order. So its elements' shapes are
//! read one at a time, and whether a call writes them by position or by name is the transport's
//! statement rather than this one's.

use alux_shape::ShapeOf;
use alux_shape_typescript::{TsShape, TsType};

/// States the parameter types an argument product carries.
pub trait TsParams {
    /// Reads each element's shape, in the order the product states them.
    fn params(alg: &TsShape) -> Vec<TsType>;
}

/// States `TsParams` for one tuple width.
macro_rules! params {
    ($($arg:ident),*) => {
        impl<$($arg),*> TsParams for ($($arg,)*)
        where
            $($arg: ShapeOf<TsShape, Shape = TsType>),*
        {
            fn params(alg: &TsShape) -> Vec<TsType> {
                vec![$($arg::shape_of(alg)),*]
            }
        }
    };
}

impl TsParams for () {
    fn params(_alg: &TsShape) -> Vec<TsType> {
        Vec::new()
    }
}

params!(A0);
params!(A0, A1);
params!(A0, A1, A2);
params!(A0, A1, A2, A3);
params!(A0, A1, A2, A3, A4);
params!(A0, A1, A2, A3, A4, A5);
params!(A0, A1, A2, A3, A4, A5, A6);
params!(A0, A1, A2, A3, A4, A5, A6, A7);
