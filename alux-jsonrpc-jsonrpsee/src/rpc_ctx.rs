use derive_more::{Deref, DerefMut, From};
use derive_new::new;

/// Carries a semantic context while interpreting it as a JSON-RPC service.
///
/// The transparent newtype provides a local target for implementations of
/// generated `jsonrpsee` server traits. The inner context remains the actual
/// interpreter of the domain algebra.
#[derive(Debug, Clone, Deref, DerefMut, From, new)]
#[repr(transparent)]
pub struct RpcCtx<Context>(Context);

impl<Context> RpcCtx<Context> {
    /// Consumes the carrier and returns its semantic context.
    pub fn into_inner(self) -> Context {
        self.0
    }
}

impl<Context> AsRef<Context> for RpcCtx<Context> {
    fn as_ref(&self) -> &Context {
        &self.0
    }
}

impl<Context> AsMut<Context> for RpcCtx<Context> {
    fn as_mut(&mut self) -> &mut Context {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::RpcCtx;

    #[test]
    fn transparently_carries_a_semantic_context() {
        let mut rpc = RpcCtx::new(41);
        *rpc += 1;

        assert_eq!(*rpc.as_ref(), 42);
        assert_eq!(rpc.into_inner(), 42);
    }
}
