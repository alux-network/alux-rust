//! Records the selectors a route composes, and renders the surface they describe.

use crate::{TextEndpoint, TextHandlerImpl};
use alux_http::{HttpSelectorAlg, RouteAlg, SelectorAlg, append_path};

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
