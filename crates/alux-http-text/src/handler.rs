//! Records what one endpoint means: the handler it names and the types it relates.

use alux_ext::{ApplyAlg, HandlerContextAlg};
use alux_http::{HandlerAlg, HandlerEndpointAlg, OutputAlg, OutputKindAlg};
use core::any::type_name;
use std::sync::Arc;

/// Carries interpreted endpoint type information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEndpoint {
    pub(crate) handler: &'static str,
    pub(crate) inputs: &'static str,
    pub(crate) args: &'static str,
    pub(crate) result: &'static str,
    pub(crate) transform: &'static str,
    pub(crate) output: &'static str,
}

/// Interprets typed APIs as text descriptions.
#[derive(Debug, Default)]
pub struct TextHandlerImpl;

impl HandlerAlg for TextHandlerImpl {
    type Endpoint = TextEndpoint;
}

impl<Context, Inputs, Args, Transform, Output> HandlerEndpointAlg<Context, Inputs, Args, Transform, Output>
    for TextHandlerImpl
where
    Transform: OutputKindAlg<TextHandlerImpl, Output>,
{
    fn finish_handler<Handler>(&self, _handler: Handler) -> TextEndpoint
    where
        Handler: ApplyAlg<Context, Args, Output = Output> + Send + Sync + 'static,
    {
        TextEndpoint {
            handler: type_name::<Handler>(),
            inputs: type_name::<Inputs>(),
            args: type_name::<Args>(),
            result: type_name::<Output>(),
            transform: type_name::<<Transform as OutputKindAlg<Self, Output>>::Transform>(),
            output: type_name::<<<Transform as OutputKindAlg<Self, Output>>::Transform as OutputAlg<Output>>::Output>(),
        }
    }
}

impl<Context> HandlerContextAlg<Context> for TextHandlerImpl
where
    Context: Send + Sync + 'static,
{
    type Handle = Arc<Context>;
}
