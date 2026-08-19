/// Describes type-level accumulation of one more input in a declaration's product.
///
/// Each recorded input role extends the endpoint's extractor product and its handler argument
/// product in the same position, so declaration order is the application order.
pub trait WithAlg {
    /// The product formed by appending `Input` to this product.
    type With<Input>;
}

impl WithAlg for () {
    type With<Input> = (Input,);
}

impl<A0> WithAlg for (A0,) {
    type With<Input> = (A0, Input);
}

impl<A0, B0> WithAlg for (A0, B0) {
    type With<Input> = (A0, B0, Input);
}

impl<A0, B0, C0> WithAlg for (A0, B0, C0) {
    type With<Input> = (A0, B0, C0, Input);
}

impl<A0, B0, C0, D0> WithAlg for (A0, B0, C0, D0) {
    type With<Input> = (A0, B0, C0, D0, Input);
}

impl<A0, B0, C0, D0, E0> WithAlg for (A0, B0, C0, D0, E0) {
    type With<Input> = (A0, B0, C0, D0, E0, Input);
}

impl<A0, B0, C0, D0, E0, F0> WithAlg for (A0, B0, C0, D0, E0, F0) {
    type With<Input> = (A0, B0, C0, D0, E0, F0, Input);
}

impl<A0, B0, C0, D0, E0, F0, G0> WithAlg for (A0, B0, C0, D0, E0, F0, G0) {
    type With<Input> = (A0, B0, C0, D0, E0, F0, G0, Input);
}
