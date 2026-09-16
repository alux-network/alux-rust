/// Describes type-level accumulation of one more input in a declaration's product.
///
/// Each recorded input role extends the endpoint's extractor product and its handler argument
/// product in the same position, so declaration order is the application order.
///
/// A declaration reaching the widest product stated here can still state what it answers with,
/// because accumulation is what the output declarations are written over. One role beyond it
/// accumulates into a product nothing interprets, which is where a declaration stops.
pub trait WithAlg {
    /// The product formed by appending `Input` to this product.
    type With<Input>;
}

/// States the accumulation for one product width.
macro_rules! with_products {
    ($($input:ident),* $(,)?) => {
        impl<$($input),*> WithAlg for ($($input,)*) {
            type With<Input> = ($($input,)* Input,);
        }
    };
}

with_products!();
with_products!(A1);
with_products!(A1, A2);
with_products!(A1, A2, A3);
with_products!(A1, A2, A3, A4);
with_products!(A1, A2, A3, A4, A5);
with_products!(A1, A2, A3, A4, A5, A6);
with_products!(A1, A2, A3, A4, A5, A6, A7);
with_products!(A1, A2, A3, A4, A5, A6, A7, A8);
with_products!(A1, A2, A3, A4, A5, A6, A7, A8, A9);
with_products!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
with_products!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
with_products!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
with_products!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13);
with_products!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14);
with_products!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15);
with_products!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16);
