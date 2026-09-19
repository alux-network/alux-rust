//! Records the selectors a route composes, and renders the surface they describe.

use crate::{TextEndpoint, TextHandlerImpl};
use alux_http::{HttpMethod, HttpSelectorAlg, RouteAlg, RoutePath, SelectorAlg, describe_path};

#[derive(Debug, Clone, PartialEq, Eq)]
enum TextSelectorPart {
    Method(HttpMethod),
    Path(RoutePath),
    Prefix(RoutePath),
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
        describe_path(self.parts.iter().filter_map(|part| match part {
            TextSelectorPart::Path(path) | TextSelectorPart::Prefix(path) => Some(path),
            TextSelectorPart::Method(_) => None,
        }))
    }

    /// Returns the selected method and path, using `*` when no method is selected.
    pub fn label(&self) -> String {
        let method = self
            .parts
            .iter()
            .rev()
            .find_map(|part| match part {
                TextSelectorPart::Method(method) => Some(method.label()),
                TextSelectorPart::Path(_) | TextSelectorPart::Prefix(_) => None,
            })
            .unwrap_or("*");

        format!("{method} {}", self.path())
    }
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

impl HttpSelectorAlg for TextHandlerImpl {
    type Selector = TextSelector;

    fn http_method(&self, method: HttpMethod) -> TextSelector {
        TextSelector { parts: vec![TextSelectorPart::Method(method)] }
    }

    fn http_path(&self, path: &RoutePath) -> TextSelector {
        TextSelector { parts: vec![TextSelectorPart::Path(path.clone())] }
    }

    fn http_prefix(&self, prefix: &RoutePath) -> TextSelector {
        TextSelector { parts: vec![TextSelectorPart::Prefix(prefix.clone())] }
    }
}
