use crate::args::DirectArgs;
use crate::error::{INTERNAL_ERROR, RpcError};
use crate::table::{Answer, DuplicateMethod, MethodTable};
use alux_ext::{ApplyAlg, HandlerContextAlg};
use alux_jsonrpc::{JsonRpcAlg, JsonRpcFallibleAlg, JsonRpcMethodAlg, OutcomeAlg, RpcErrorAlg};
use derive_new::new as New;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

/// Interprets typed JSON-RPC programs as a dispatchable JSON-RPC 2.0 surface.
///
/// The interpretation owns parameter decoding, name dispatch, serialization, and the protocol's own
/// errors. It owns no transport: a surface answers one request document and says nothing about how
/// that document arrived.
#[derive(Debug, New)]
pub struct DirectImpl<Context> {
    #[new(into)]
    context: Arc<Context>,
}

impl<Context> DirectImpl<Context> {
    /// Creates an interpreter from an already shared semantic context.
    pub fn from_arc(context: Arc<Context>) -> Self {
        Self { context }
    }

    /// Registers one method, decoding its parameters and reading its output as an answer.
    fn register<Args, Output, Handler, Decode, Finish>(
        &self,
        name: &'static str,
        handler: Handler,
        decode: Decode,
        finish: Finish,
    ) -> Result<MethodTable, DuplicateMethod>
    where
        Context: Send + Sync + 'static,
        Args: Send + 'static,
        Output: Send + 'static,
        Handler: ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
        Decode: Fn(Option<Value>) -> Result<Args, RpcError> + Copy + Send + Sync + 'static,
        Finish: Fn(Output) -> Result<Value, RpcError> + Copy + Send + Sync + 'static,
    {
        let context = Arc::clone(&self.context);
        let handler = Arc::new(handler);
        let mut table = MethodTable::default();
        table.insert(
            name,
            Arc::new(move |params| {
                let context = Arc::clone(&context);
                let handler = Arc::clone(&handler);
                let answer: Answer = Box::pin(async move {
                    let args = decode(params)?;

                    finish(handler.apply(context, args).await)
                });

                answer
            }),
        )?;

        Ok(table)
    }
}

/// Reads a value the domain answered with as the JSON it denotes.
fn answered<Output>(output: Output) -> Result<Value, RpcError>
where
    Output: Serialize,
{
    serde_json::to_value(output).map_err(|error| RpcError::new(INTERNAL_ERROR, error.to_string()))
}

impl<Context> HandlerContextAlg<Context> for DirectImpl<Context>
where
    Context: Send + Sync + 'static,
{
    type Handle = Arc<Context>;
}

impl<Context> JsonRpcAlg for DirectImpl<Context> {
    type Methods = Result<MethodTable, DuplicateMethod>;

    fn jsonrpc_empty(&self) -> Self::Methods {
        Ok(MethodTable::default())
    }

    fn jsonrpc_merge(&self, left: Self::Methods, right: Self::Methods) -> Self::Methods {
        left?.merge(right?)
    }
}

impl<Context, Args, Output> JsonRpcMethodAlg<Arc<Context>, Args, Output> for DirectImpl<Context>
where
    Context: Send + Sync + 'static,
    Args: DirectArgs + Send + 'static,
    Output: Serialize + Send + 'static,
{
    fn finish_jsonrpc_positional_method<Handler>(
        &self,
        name: &'static str,
        _arg_names: &'static [&'static str],
        handler: Handler,
    ) -> <Self as JsonRpcAlg>::Methods
    where
        Handler: ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        self.register(name, handler, Args::from_positional, answered)
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
        self.register(name, handler, move |params| Args::from_named(params, arg_names), answered)
    }
}

/// Reads an outcome the domain answered with as a value or the protocol error its failure denotes.
fn converted<Output>(output: Output) -> Result<Value, RpcError>
where
    Output: OutcomeAlg,
    Output::Value: Serialize,
    Output::Error: RpcErrorAlg,
{
    match output.outcome() {
        Ok(value) => answered(value),
        Err(failure) => Err(RpcError::denoted(&failure)),
    }
}

impl<Context, Args, Output> JsonRpcFallibleAlg<Arc<Context>, Args, Output> for DirectImpl<Context>
where
    Context: Send + Sync + 'static,
    Args: DirectArgs + Send + 'static,
    Output: OutcomeAlg + Send + 'static,
    Output::Value: Serialize + Send + 'static,
    Output::Error: RpcErrorAlg + Send + 'static,
{
    fn finish_jsonrpc_positional_fallible<Handler>(
        &self,
        name: &'static str,
        _arg_names: &'static [&'static str],
        handler: Handler,
    ) -> <Self as JsonRpcAlg>::Methods
    where
        Handler: ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        self.register(name, handler, Args::from_positional, converted)
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
        self.register(name, handler, move |params| Args::from_named(params, arg_names), converted)
    }
}
