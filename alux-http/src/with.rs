use alux_ext::for_tuples;

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
    ($($input:ident => $index:tt),*) => {
        impl<$($input),*> WithAlg for ($($input,)*) {
            type With<Input> = ($($input,)* Input,);
        }
    };
}

for_tuples!(with_products, 0..=16);
