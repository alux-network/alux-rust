//! Composes the endpoints a program states into the document that describes them.

use crate::{OpenApiAnswer, OpenApiArgument, OpenApiSource, OpenApiStated};
use alux_http::{HttpMethod, HttpSelectorAlg, RouteAlg, RoutePath, SelectorAlg, describe_path, write_header_name};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
enum OpenApiSelectorPart {
    Method(HttpMethod),
    Path(RoutePath),
    Prefix(RoutePath),
}

/// Carries route-selection meaning, which a document reads as a path and a method.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenApiSelector {
    parts: Vec<OpenApiSelectorPart>,
}

impl OpenApiSelector {
    /// Returns the composed path this selector matches.
    ///
    /// A document templates a path exactly as a described surface states one, so nothing is spelled
    /// a second way here.
    pub fn path(&self) -> String {
        describe_path(self.parts.iter().filter_map(|part| match part {
            OpenApiSelectorPart::Path(path) | OpenApiSelectorPart::Prefix(path) => Some(path),
            OpenApiSelectorPart::Method(_) => None,
        }))
    }

    /// Returns the selected method and path, using `*` when no method is selected.
    pub fn label(&self) -> String {
        let method = self.method().map_or("*", HttpMethod::label);
        format!("{method} {}", self.path())
    }

    fn method(&self) -> Option<HttpMethod> {
        self.parts.iter().rev().find_map(|part| match part {
            OpenApiSelectorPart::Method(method) => Some(*method),
            OpenApiSelectorPart::Path(_) | OpenApiSelectorPart::Prefix(_) => None,
        })
    }
}

/// One endpoint, as a document describes it.
#[derive(Debug, Clone)]
pub struct OpenApiEndpoint {
    pub(crate) operation: &'static str,
    pub(crate) doc: &'static str,
    pub(crate) arguments: Vec<(&'static str, OpenApiArgument)>,
    pub(crate) answers: Vec<OpenApiAnswer>,
}

impl OpenApiEndpoint {
    /// Returns the name the operation was declared under, which the document states as its id.
    pub fn operation(&self) -> &'static str {
        self.operation
    }

    /// Returns what the operation was documented as, split the way a doc comment already reads.
    ///
    /// The first line is the summary a reader sees beside the operation, and whatever follows is the
    /// description they see when they open it.
    fn documented(&self) -> (Option<&str>, Option<&str>) {
        let doc = self.doc.trim();
        let (summary, description) = doc.split_once('\n').unwrap_or((doc, ""));

        (Some(summary.trim()).filter(|text| !text.is_empty()), Some(description.trim()).filter(|text| !text.is_empty()))
    }

    fn parameters(&self) -> Vec<Value> {
        self.arguments
            .iter()
            .filter_map(|(name, argument)| {
                let location = match argument.source {
                    OpenApiSource::Path => "path",
                    OpenApiSource::Query => "query",
                    OpenApiSource::Header => "header",
                    OpenApiSource::Cookie => "cookie",
                    OpenApiSource::Body(_) | OpenApiSource::Unstated => return None,
                };

                Some(match &argument.stated {
                    // What a path binds is the argument itself, and it binds every segment it names.
                    OpenApiStated::Whole(schema) => {
                        vec![json!({ "name": name, "in": location, "required": location == "path", "schema": schema })]
                    }
                    // A collection of names and values states one parameter for each name it carries.
                    OpenApiStated::Named(named) => named
                        .iter()
                        .map(|named| {
                            json!({
                                "name": stated_as(argument.source, &named.name),
                                "in": location,
                                "required": named.required,
                                "schema": named.schema,
                            })
                        })
                        .collect(),
                })
            })
            .flatten()
            .collect()
    }

    fn request_body(&self) -> Option<Value> {
        self.arguments.iter().find_map(|(_, argument)| match (argument.source, &argument.stated) {
            (OpenApiSource::Body(content_type), OpenApiStated::Whole(schema)) => Some(json!({
                "required": true,
                "content": { content_type: { "schema": schema } },
            })),
            _ => None,
        })
    }

    fn responses(&self) -> Value {
        // A response object states a description of its own. Only a successful answer is what the
        // operation was documented as; what a failure answers with the program never says in words.
        let mut responses = Map::new();
        for answer in &self.answers {
            let stated = if answer.status.is_success() { self.documented().0.unwrap_or_default() } else { "" };
            let mut described = match (answer.content_type, &answer.schema) {
                (Some(content_type), Some(schema)) => json!({
                    "description": stated,
                    "content": { content_type: { "schema": schema } },
                }),
                _ => json!({ "description": stated }),
            };
            // A header an answer carries is one a caller reads, so a document states each of them.
            if !answer.headers.is_empty()
                && let Some(described) = described.as_object_mut()
            {
                let carried =
                    answer.headers.iter().map(|name| ((*name).to_owned(), json!({ "schema": { "type": "string" } })));
                described.insert("headers".into(), Value::Object(carried.collect()));
            }
            responses.insert(answer.status.code().to_string(), described);
        }

        Value::Object(responses)
    }

    fn operation_object(&self) -> Value {
        let mut described = Map::new();
        described.insert("operationId".into(), self.operation.into());
        let (summary, description) = self.documented();
        if let Some(summary) = summary {
            described.insert("summary".into(), summary.into());
        }
        if let Some(description) = description {
            described.insert("description".into(), description.into());
        }
        let parameters = self.parameters();
        if !parameters.is_empty() {
            described.insert("parameters".into(), Value::Array(parameters));
        }
        if let Some(body) = self.request_body() {
            described.insert("requestBody".into(), body);
        }
        described.insert("responses".into(), self.responses());

        Value::Object(described)
    }
}

/// Writes a name the way the collection it was read from states one.
///
/// A header name is words the wire spells with `-`; everywhere else a name reaches a caller as the
/// member stated it.
fn stated_as(source: OpenApiSource, name: &str) -> String {
    match source {
        OpenApiSource::Header => write_header_name(name),
        _ => name.to_owned(),
    }
}

#[derive(Debug, Clone)]
struct OpenApiRouteEntry {
    selector: OpenApiSelector,
    endpoint: OpenApiEndpoint,
}

/// Carries every endpoint a program states, in declaration order.
#[derive(Debug, Clone, Default)]
pub struct OpenApiRoute {
    entries: Vec<OpenApiRouteEntry>,
}

impl OpenApiRoute {
    /// Returns each composed selector as `METHOD /path`, in declaration order.
    pub fn labels(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.label()).collect()
    }

    /// Returns each composed route path, in declaration order.
    pub fn paths(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.path()).collect()
    }

    /// Returns the name each endpoint's operation was declared under, in declaration order.
    pub fn operations(&self) -> Vec<&'static str> {
        self.entries.iter().map(|entry| entry.endpoint.operation).collect()
    }

    /// Returns the paths object this surface states, keyed by path and then by method.
    ///
    /// An endpoint composed without a method states nothing a document can key, so it is left out.
    pub fn paths_object(&self) -> Value {
        let mut paths = BTreeMap::<String, Map<String, Value>>::new();
        for entry in &self.entries {
            let Some(method) = entry.selector.method() else { continue };
            let methods = paths.entry(entry.selector.path()).or_default();
            methods.insert(method.label().to_lowercase(), entry.endpoint.operation_object());
        }

        Value::Object(paths.into_iter().map(|(path, methods)| (path, Value::Object(methods))).collect())
    }
}

/// Composes route selection as the path and method a document keys an operation by.
#[derive(Debug, Default)]
pub struct OpenApiRouteImpl;

impl SelectorAlg for OpenApiRouteImpl {
    type Selector = OpenApiSelector;

    fn identity(&self) -> OpenApiSelector {
        OpenApiSelector::default()
    }

    fn compose(&self, mut first: OpenApiSelector, second: OpenApiSelector) -> OpenApiSelector {
        first.parts.extend(second.parts);
        first
    }
}

impl RouteAlg for OpenApiRouteImpl {
    type Route = OpenApiRoute;
    type Selector = OpenApiSelector;
    type Endpoint = OpenApiEndpoint;

    fn initial(&self) -> OpenApiRoute {
        OpenApiRoute::default()
    }

    fn coproduct(&self, mut left: OpenApiRoute, right: OpenApiRoute) -> OpenApiRoute {
        left.entries.extend(right.entries);
        left
    }

    fn precompose(&self, selector: OpenApiSelector, mut route: OpenApiRoute) -> OpenApiRoute {
        for entry in &mut route.entries {
            entry.selector = self.compose(selector.clone(), core::mem::take(&mut entry.selector));
        }
        route
    }

    fn lift(&self, endpoint: OpenApiEndpoint) -> OpenApiRoute {
        OpenApiRoute { entries: vec![OpenApiRouteEntry { selector: self.identity(), endpoint }] }
    }
}

impl HttpSelectorAlg for OpenApiRouteImpl {
    type Selector = OpenApiSelector;

    fn http_method(&self, method: HttpMethod) -> OpenApiSelector {
        OpenApiSelector { parts: vec![OpenApiSelectorPart::Method(method)] }
    }

    fn http_path(&self, path: &RoutePath) -> OpenApiSelector {
        OpenApiSelector { parts: vec![OpenApiSelectorPart::Path(path.clone())] }
    }

    fn http_prefix(&self, prefix: &RoutePath) -> OpenApiSelector {
        OpenApiSelector { parts: vec![OpenApiSelectorPart::Prefix(prefix.clone())] }
    }
}
