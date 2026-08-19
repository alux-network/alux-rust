//! Interprets typed HTTP programs as readable route and type descriptions.
//!
//! The text interpretation executes no handler. It records the selectors, extractor roles, argument
//! product, handler result, and output conversion each endpoint denotes, which makes it the neutral
//! witness that an HTTP program means a surface rather than a framework callback.

use alux_ext::{ApplyAlg, HandlerContextAlg};
use alux_http::{
    FileOutAlg, HandlerAlg, HandlerEndpointAlg, HttpInputAlg, HttpSelectorAlg, JsonOutAlg, OutputAlg, OutputKindAlg,
    RouteAlg, SelectorAlg, append_path,
};
use core::any::type_name;
use core::marker::PhantomData;
use std::sync::Arc;

/// Carries interpreted endpoint type information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEndpoint {
    handler: &'static str,
    inputs: &'static str,
    args: &'static str,
    result: &'static str,
    transform: &'static str,
    output: &'static str,
}

/// Identifies an HTTP input role in text output.
pub struct TextInputRole<Role, Input>(PhantomData<fn(Role) -> Input>);

/// Identifies path extraction in text descriptions.
pub struct PathRole;
/// Identifies query extraction in text descriptions.
pub struct QueryRole;
/// Identifies request-body extraction in text descriptions.
pub struct BodyRole;
/// Identifies header extraction in text descriptions.
pub struct HeaderRole;
/// Identifies authentication extraction in text descriptions.
pub struct AuthRole;
/// Identifies request-context extraction in text descriptions.
pub struct ContextRole;

/// Interprets JSON output selection in text descriptions.
pub struct TextJsonOutput;

impl<From> OutputAlg<From> for TextJsonOutput {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

/// Interprets streamed-file output selection in text descriptions.
pub struct TextFileOutput;

impl<From> OutputAlg<From> for TextFileOutput {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TextSelectorPart {
    Method(&'static str),
    Path(String),
    Prefix(String),
}

/// Carries interpreted selector data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextSelector {
    parts: Vec<TextSelectorPart>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextRouteEntry {
    selector: TextSelector,
    endpoint: TextEndpoint,
}

/// Carries an interpreted route composition.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextRoute {
    entries: Vec<TextRouteEntry>,
}

impl TextRoute {
    /// Returns each interpreted selector as `METHOD /path`, in declaration order.
    pub fn labels(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.label()).collect()
    }

    /// Returns each interpreted route path, in declaration order.
    pub fn paths(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.path()).collect()
    }

    /// Renders each route as a Markdown description.
    pub fn lines(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|entry| {
                format!(
                    concat!(
                        "### {}\n",
                        "- `handler`: `{}`\n",
                        "- `inputs`: `{}`\n",
                        "- `args`: `{}`\n",
                        "- `result`: `{}`\n",
                        "- `transform`: `{}`\n",
                        "- `output`: `{}`",
                    ),
                    entry.selector.label(),
                    entry.endpoint.handler,
                    entry.endpoint.inputs,
                    entry.endpoint.args,
                    entry.endpoint.result,
                    entry.endpoint.transform,
                    entry.endpoint.output,
                )
            })
            .collect()
    }
}

impl TextSelector {
    /// Returns the composed absolute path this selector matches.
    pub fn path(&self) -> String {
        let mut path = String::new();
        for part in &self.parts {
            if let TextSelectorPart::Path(value) | TextSelectorPart::Prefix(value) = part {
                append_path(&mut path, value);
            }
        }

        if path.is_empty() {
            path.push('/');
        }

        path
    }

    /// Returns the selected method and path, using `*` when no method is selected.
    pub fn label(&self) -> String {
        let method = self
            .parts
            .iter()
            .rev()
            .find_map(|part| match part {
                TextSelectorPart::Method(value) => Some(*value),
                TextSelectorPart::Path(_) | TextSelectorPart::Prefix(_) => None,
            })
            .unwrap_or("*");

        format!("{method} {}", self.path())
    }
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

impl SelectorAlg for TextHandlerImpl {
    type Selector = TextSelector;

    fn identity(&self) -> TextSelector {
        TextSelector::default()
    }

    fn compose(&self, mut first: TextSelector, second: TextSelector) -> TextSelector {
        first.parts.extend(second.parts);
        first
    }
}

impl RouteAlg for TextHandlerImpl {
    type Route = TextRoute;
    type Selector = TextSelector;
    type Endpoint = TextEndpoint;

    fn initial(&self) -> TextRoute {
        TextRoute::default()
    }

    fn coproduct(&self, mut left: TextRoute, right: TextRoute) -> TextRoute {
        left.entries.extend(right.entries);
        left
    }

    fn precompose(&self, selector: TextSelector, mut route: TextRoute) -> TextRoute {
        for entry in &mut route.entries {
            entry.selector = self.compose(selector.clone(), core::mem::take(&mut entry.selector));
        }
        route
    }

    fn lift(&self, endpoint: TextEndpoint) -> TextRoute {
        TextRoute { entries: vec![TextRouteEntry { selector: self.identity(), endpoint }] }
    }
}

impl HttpInputAlg for TextHandlerImpl {
    type Path<Input> = TextInputRole<PathRole, Input>;
    type Query<Input> = TextInputRole<QueryRole, Input>;
    type Body<Input> = TextInputRole<BodyRole, Input>;
    type Header<Input> = TextInputRole<HeaderRole, Input>;
    type Auth<Input> = TextInputRole<AuthRole, Input>;
    type Context<Input> = TextInputRole<ContextRole, Input>;
}

impl JsonOutAlg for TextHandlerImpl {
    type Json<From> = TextJsonOutput;
}

impl FileOutAlg for TextHandlerImpl {
    type File<From> = TextFileOutput;
}

impl HttpSelectorAlg for TextHandlerImpl {
    type Selector = TextSelector;

    fn http_get(&self) -> TextSelector {
        TextSelector { parts: vec![TextSelectorPart::Method("GET")] }
    }

    fn http_post(&self) -> TextSelector {
        TextSelector { parts: vec![TextSelectorPart::Method("POST")] }
    }

    fn http_path(&self, path: &str) -> TextSelector {
        TextSelector { parts: vec![TextSelectorPart::Path(path.into())] }
    }

    fn http_prefix(&self, prefix: &str) -> TextSelector {
        TextSelector { parts: vec![TextSelectorPart::Prefix(prefix.into())] }
    }
}
