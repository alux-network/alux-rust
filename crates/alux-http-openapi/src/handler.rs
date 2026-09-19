//! Reads one endpoint as what a document says about it.

use crate::input::{
    OpenApiBodyInput, OpenApiCookieInput, OpenApiFormInput, OpenApiHeaderInput, OpenApiInputsAlg,
    OpenApiMultipartInput, OpenApiPathInput, OpenApiQueryInput, OpenApiRawBodyInput, OpenApiUnstatedInput,
};
use crate::output::OpenApiOutputAlg;
use crate::route::{OpenApiEndpoint, OpenApiRoute, OpenApiRouteImpl, OpenApiSelector};
use alux_ext::{ApplyAlg, HandlerContextAlg, OperationAlg};
use alux_http::{
    HandlerAlg, HandlerEndpointAlg, HttpInputAlg, HttpMethod, HttpSelectorAlg, OutputKindAlg, RouteAlg, RoutePath,
    SelectorAlg,
};
use alux_shape::Spelling;
use alux_shape_jsonschema::JsonSchemaShape;
use serde_json::{Value, json};
use std::sync::Arc;

/// Interprets typed HTTP programs as the `OpenAPI` document that describes them.
///
/// Nothing is applied. Every endpoint is read for what a caller would have to state and what it
/// would be answered with, which is why this interpretation asks for shapes where the others ask
/// for extractors.
pub struct OpenApiHandlerImpl<Context> {
    schema: JsonSchemaShape,
    context: core::marker::PhantomData<fn() -> Context>,
}

impl<Context> Default for OpenApiHandlerImpl<Context> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Context> OpenApiHandlerImpl<Context> {
    /// Describes a surface, naming shapes where an `OpenAPI` document keeps them.
    pub fn new() -> Self {
        Self {
            schema: JsonSchemaShape::new(Spelling::Snake, "#/components/schemas/"),
            context: core::marker::PhantomData,
        }
    }

    /// Returns the whole document for a surface, under a title and a version.
    pub fn document(&self, title: &str, version: &str, route: &OpenApiRoute) -> Value {
        let paths = route.paths_object();
        let schemas = self.schema.definitions().into_iter().collect::<serde_json::Map<_, _>>();
        let mut document = json!({
            "openapi": "3.1.0",
            "info": { "title": title, "version": version },
            "paths": paths,
        });
        if !schemas.is_empty()
            && let Some(document) = document.as_object_mut()
        {
            document.insert("components".into(), json!({ "schemas": Value::Object(schemas) }));
        }

        document
    }
}

impl<Context> HandlerContextAlg<Context> for OpenApiHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
{
    type Handle = Arc<Context>;
}

impl<Context> HandlerAlg for OpenApiHandlerImpl<Context> {
    type Endpoint = OpenApiEndpoint;
}

impl<Context> HttpInputAlg for OpenApiHandlerImpl<Context> {
    type Path<I> = OpenApiPathInput<I>;
    type Query<I> = OpenApiQueryInput<I>;
    type Body<I> = OpenApiBodyInput<I>;
    type Form<I> = OpenApiFormInput<I>;
    type Multipart<I> = OpenApiMultipartInput<I>;
    type RawBody<I> = OpenApiRawBodyInput<I>;
    type Header<I> = OpenApiHeaderInput<I>;
    type Cookie<I> = OpenApiCookieInput<I>;
    // A caller states an authentication value in a header; an endpoint context it never states.
    type Auth<I> = OpenApiHeaderInput<I>;
    type Context<I> = OpenApiUnstatedInput<I>;
}

impl<Context, Inputs, Args, Transform, Answering, Output>
    HandlerEndpointAlg<Arc<Context>, Inputs, Args, Transform, Output> for OpenApiHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
    Inputs: OpenApiInputsAlg,
    Transform: OutputKindAlg<Self, Output, Transform = Answering>,
    Answering: OpenApiOutputAlg<Output>,
{
    fn finish_handler<Handler>(&self, _handler: Handler) -> <Self as HandlerAlg>::Endpoint
    where
        Handler: OperationAlg + ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        // Input roles accumulate in the same order as the handler's arguments, so a role and the
        // name it was declared under are the same position of two products.
        let arguments = Handler::ARG_NAMES.iter().copied().zip(Inputs::describe(&self.schema)).collect();

        OpenApiEndpoint {
            operation: Handler::NAME,
            doc: Handler::DOC,
            arguments,
            answers: Answering::answers(&self.schema),
        }
    }
}

impl<Context> SelectorAlg for OpenApiHandlerImpl<Context> {
    type Selector = OpenApiSelector;

    fn identity(&self) -> Self::Selector {
        SelectorAlg::identity(&OpenApiRouteImpl)
    }

    fn compose(&self, first: Self::Selector, second: Self::Selector) -> Self::Selector {
        SelectorAlg::compose(&OpenApiRouteImpl, first, second)
    }
}

impl<Context> RouteAlg for OpenApiHandlerImpl<Context> {
    type Route = OpenApiRoute;
    type Selector = OpenApiSelector;
    type Endpoint = OpenApiEndpoint;

    fn initial(&self) -> Self::Route {
        RouteAlg::initial(&OpenApiRouteImpl)
    }

    fn coproduct(&self, left: Self::Route, right: Self::Route) -> Self::Route {
        RouteAlg::coproduct(&OpenApiRouteImpl, left, right)
    }

    fn precompose(&self, selector: Self::Selector, route: Self::Route) -> Self::Route {
        RouteAlg::precompose(&OpenApiRouteImpl, selector, route)
    }

    fn lift(&self, endpoint: Self::Endpoint) -> Self::Route {
        RouteAlg::lift(&OpenApiRouteImpl, endpoint)
    }
}

impl<Context> HttpSelectorAlg for OpenApiHandlerImpl<Context> {
    type Selector = OpenApiSelector;

    fn http_method(&self, method: HttpMethod) -> Self::Selector {
        HttpSelectorAlg::http_method(&OpenApiRouteImpl, method)
    }

    fn http_path(&self, path: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_path(&OpenApiRouteImpl, path)
    }

    fn http_prefix(&self, prefix: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_prefix(&OpenApiRouteImpl, prefix)
    }
}
