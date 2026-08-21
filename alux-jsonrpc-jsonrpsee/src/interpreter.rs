use crate::args::JsonrpseeArgs;
use crate::result::RpcErrorExt;
use alux_ext::{ApplyAlg, HandlerContextAlg};
use alux_jsonrpc::{JsonRpcAlg, JsonRpcFallibleAlg, JsonRpcMethodAlg, OutcomeAlg, RpcErrorAlg};
use jsonrpsee::Methods;
use jsonrpsee::core::RegisterMethodError;
use jsonrpsee::core::server::ResponsePayload;
use jsonrpsee::server::RpcModule;
use jsonrpsee::types::{ErrorObjectOwned, Params};
use serde::Serialize;
use std::sync::Arc;

/// Interprets typed JSON-RPC programs as `jsonrpsee` method collections.
pub struct JsonrpseeImpl<Context> {
    context: Arc<Context>,
}

impl<Context> JsonrpseeImpl<Context> {
    /// Creates an interpreter around a semantic context.
    pub fn new(context: Context) -> Self {
        Self { context: Arc::new(context) }
    }

    /// Creates an interpreter from an already shared semantic context.
    pub fn from_arc(context: Arc<Context>) -> Self {
        Self { context }
    }

    fn finish_method<Args, Output, Handler, Parse>(
        &self,
        name: &'static str,
        handler: Handler,
        parse: Parse,
    ) -> Result<Methods, RegisterMethodError>
    where
        Context: Send + Sync + 'static,
        Args: Send + 'static,
        Output: Serialize + Clone + Send + 'static,
        Handler: ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
        Parse: Fn(Params<'static>) -> Result<Args, ErrorObjectOwned> + Send + Sync + 'static,
    {
        let mut module = RpcModule::from_arc(Arc::clone(&self.context));
        let handler = Arc::new(handler);
        let parse = Arc::new(parse);
        module.register_async_method(name, move |params, context, _| {
            let handler = Arc::clone(&handler);
            let parse = Arc::clone(&parse);
            async move {
                let args = parse(params)?;
                Ok::<Output, ErrorObjectOwned>(handler.apply(context, args).await)
            }
        })?;

        Ok(module.into())
    }

    fn finish_fallible_method<Args, Output, Handler, Parse>(
        &self,
        name: &'static str,
        handler: Handler,
        parse: Parse,
    ) -> Result<Methods, RegisterMethodError>
    where
        Context: Send + Sync + 'static,
        Args: Send + 'static,
        Output: OutcomeAlg + Send + 'static,
        Output::Value: Serialize + Clone + Send + 'static,
        Output::Error: RpcErrorAlg + Send + 'static,
        Handler: ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
        Parse: Fn(Params<'static>) -> Result<Args, ErrorObjectOwned> + Send + Sync + 'static,
    {
        let mut module = RpcModule::from_arc(Arc::clone(&self.context));
        let handler = Arc::new(handler);
        let parse = Arc::new(parse);
        module.register_async_method(name, move |params, context, _| {
            let handler = Arc::clone(&handler);
            let parse = Arc::clone(&parse);
            async move {
                match parse(params) {
                    Ok(args) => match handler.apply(context, args).await.outcome() {
                        Ok(value) => ResponsePayload::success(value),
                        Err(failure) => ResponsePayload::error(failure.to_rpc_error()),
                    },
                    Err(error) => ResponsePayload::error(error),
                }
            }
        })?;

        Ok(module.into())
    }
}

impl<Context> HandlerContextAlg<Context> for JsonrpseeImpl<Context>
where
    Context: Send + Sync + 'static,
{
    type Handle = Arc<Context>;
}

impl<Context> JsonRpcAlg for JsonrpseeImpl<Context> {
    type Methods = Result<Methods, RegisterMethodError>;

    fn jsonrpc_empty(&self) -> Self::Methods {
        Ok(Methods::new())
    }

    fn jsonrpc_merge(&self, left: Self::Methods, right: Self::Methods) -> Self::Methods {
        let mut left = left?;
        left.merge(right?)?;
        Ok(left)
    }
}

impl<Context, Args, Output> JsonRpcMethodAlg<Arc<Context>, Args, Output> for JsonrpseeImpl<Context>
where
    Context: Send + Sync + 'static,
    Args: JsonrpseeArgs + Send + 'static,
    Output: Serialize + Clone + Send + 'static,
{
    fn finish_jsonrpc_positional_method<Handler>(
        &self,
        name: &'static str,
        handler: Handler,
    ) -> <Self as JsonRpcAlg>::Methods
    where
        Handler: ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        self.finish_method(name, handler, Args::from_positional)
    }

    fn finish_jsonrpc_named_method<Handler>(
        &self,
        name: &'static str,
        arg_names: &'static [&'static str],
        handler: Handler,
    ) -> <Self as JsonRpcAlg>::Methods
    where
        Handler: ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        self.finish_method(name, handler, move |params| Args::from_named(params, arg_names))
    }
}

impl<Context, Args, Output> JsonRpcFallibleAlg<Arc<Context>, Args, Output> for JsonrpseeImpl<Context>
where
    Context: Send + Sync + 'static,
    Args: JsonrpseeArgs + Send + 'static,
    Output: OutcomeAlg + Send + 'static,
    Output::Value: Serialize + Clone + Send + 'static,
    Output::Error: RpcErrorAlg + Send + 'static,
{
    fn finish_jsonrpc_positional_fallible<Handler>(
        &self,
        name: &'static str,
        handler: Handler,
    ) -> <Self as JsonRpcAlg>::Methods
    where
        Handler: ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        self.finish_fallible_method(name, handler, Args::from_positional)
    }

    fn finish_jsonrpc_named_fallible<Handler>(
        &self,
        name: &'static str,
        arg_names: &'static [&'static str],
        handler: Handler,
    ) -> <Self as JsonRpcAlg>::Methods
    where
        Handler: ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        self.finish_fallible_method(name, handler, move |params| Args::from_named(params, arg_names))
    }
}
