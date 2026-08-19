use alux_ext::ext;
use jsonrpsee::{Methods, core::RegisterMethodError};

/// Provides fallible composition of independently declared JSON-RPC methods.
#[ext(name = IntoRpcMethodsExt)]
pub impl<This, Item> This
where
    This: IntoIterator<Item = Item>,
    Item: Into<Methods>,
{
    /// Merges every method collection into a single JSON-RPC surface.
    fn into_rpc_methods(self) -> Result<Methods, RegisterMethodError> {
        self.into_iter().try_fold(Methods::new(), |mut methods, item| {
            methods.merge(item)?;
            Ok(methods)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::IntoRpcMethodsExt;
    use jsonrpsee::RpcModule;

    #[test]
    fn composes_independent_method_collections() {
        let mut first = RpcModule::new(());
        first.register_method("first", |_, (), _| "one").unwrap();

        let mut second = RpcModule::new(());
        second.register_method("second", |_, (), _| "two").unwrap();

        let methods = [first, second].into_rpc_methods().unwrap();

        assert!(methods.method("first").is_some());
        assert!(methods.method("second").is_some());
    }
}
