//! Composes the calls a program states into the module that makes them.

use crate::TsArgument;
use alux_http::{HttpMethod, HttpSelectorAlg, RouteAlg, RoutePath, SelectorAlg, describe_path};
use alux_shape_typescript::TsType;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TsSelectorPart {
    Method(HttpMethod),
    Path(RoutePath),
    Prefix(RoutePath),
}

/// Carries route-selection meaning, which a call reads as a method and a path template.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TsSelector {
    parts: Vec<TsSelectorPart>,
}

impl TsSelector {
    /// Returns the composed path this selector matches.
    ///
    /// A described path is already a template, so a call carries it unchanged and the runtime fills
    /// each slot with the argument whose role said it goes there.
    pub fn path(&self) -> String {
        describe_path(self.parts.iter().filter_map(|part| match part {
            TsSelectorPart::Path(path) | TsSelectorPart::Prefix(path) => Some(path),
            TsSelectorPart::Method(_) => None,
        }))
    }

    /// Returns the selected method and path, using `*` when no method is selected.
    pub fn label(&self) -> String {
        let method = self.method().map_or("*", HttpMethod::label);
        format!("{method} {}", self.path())
    }

    fn method(&self) -> Option<HttpMethod> {
        self.parts.iter().rev().find_map(|part| match part {
            TsSelectorPart::Method(method) => Some(*method),
            TsSelectorPart::Path(_) | TsSelectorPart::Prefix(_) => None,
        })
    }
}

/// One call, before the method and path it is made under are composed around it.
#[derive(Debug, Clone)]
pub struct TsCall {
    pub(crate) name: String,
    pub(crate) doc: &'static str,
    pub(crate) parameters: Vec<(String, TsArgument)>,
    pub(crate) answer: TsType,
}

impl TsCall {
    /// Returns the name a caller writes this call under.
    pub fn name(&self) -> &str {
        &self.name
    }

    fn declarations(&self) -> BTreeMap<String, String> {
        let shapes = self.parameters.iter().map(|(_, argument)| &argument.shape).chain([&self.answer]);

        shapes.flat_map(|shape| shape.declarations().map(|(name, text)| (name.to_owned(), text.to_owned()))).collect()
    }

    /// Writes what the operation was documented as, as the comment a caller reads.
    fn documented(&self) -> String {
        let doc = self.doc.trim();
        if doc.is_empty() {
            return String::new();
        }
        if let Some(line) = doc.lines().next().filter(|_| doc.lines().count() == 1) {
            return format!("/** {line} */\n");
        }

        let lines = doc.lines().map(|line| format!(" * {line}").trim_end().to_owned()).collect::<Vec<_>>();

        format!("/**\n{}\n */\n", lines.join("\n"))
    }

    fn entry(&self, method: HttpMethod, path: &str) -> String {
        let parameters = self
            .parameters
            .iter()
            .map(|(name, argument)| format!("{name}: {}", argument.shape.expr()))
            .collect::<Vec<_>>();
        let roles =
            self.parameters.iter().map(|(_, argument)| format!("\"{}\"", argument.source.label())).collect::<Vec<_>>();

        format!(
            "{}{}: endpoint<[{}], {}>(\"{}\", \"{path}\", [{}])",
            self.documented(),
            self.name,
            parameters.join(", "),
            self.answer.expr(),
            method.label(),
            roles.join(", "),
        )
    }
}

/// A client module: the declarations its calls depend on, and the calls themselves.
#[derive(Debug, Clone, Default)]
pub struct TsHttpModule {
    declarations: BTreeMap<String, String>,
    entries: BTreeMap<String, String>,
    calls: Vec<(TsSelector, TsCall)>,
}

impl TsHttpModule {
    /// Writes the module: every declaration a call depends on, then the program the calls form.
    pub fn render(&self) -> String {
        let declarations = self.declarations.values().map(String::as_str).collect::<Vec<_>>();
        // An entry states its own documentation above it, so indenting is a matter of every line.
        let entries = self
            .entries
            .values()
            .map(|entry| {
                let mut lines = entry.lines().map(|line| format!("  {line}").trim_end().to_owned()).collect::<Vec<_>>();
                if let Some(last) = lines.last_mut() {
                    last.push(',');
                }

                lines.join("\n")
            })
            .collect::<Vec<_>>();
        let program = format!("export const program = {{\n{}\n}} as const", entries.join("\n"));

        if declarations.is_empty() { program } else { format!("{}\n\n{program}", declarations.join("\n\n")) }
    }

    /// Returns the name each call is written under, in name order.
    pub fn call_names(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }

    /// Returns each composed selector as `METHOD /path`, in declaration order.
    pub fn labels(&self) -> Vec<String> {
        self.calls.iter().map(|(selector, _)| selector.label()).collect()
    }

    /// Returns each composed path template, in declaration order.
    pub fn paths(&self) -> Vec<String> {
        self.calls.iter().map(|(selector, _)| selector.path()).collect()
    }

    /// Writes every call whose method and path are now composed.
    ///
    /// A call composed without a method states nothing a caller could make, so it is left out.
    fn written(mut self) -> Self {
        self.entries = self
            .calls
            .iter()
            .filter_map(|(selector, call)| {
                let method = selector.method()?;

                Some((call.name.clone(), call.entry(method, &selector.path())))
            })
            .collect();

        self
    }
}

/// Composes route selection as the method and path a call is made under.
#[derive(Debug, Default)]
pub struct TsRouteImpl;

impl SelectorAlg for TsRouteImpl {
    type Selector = TsSelector;

    fn identity(&self) -> TsSelector {
        TsSelector::default()
    }

    fn compose(&self, mut first: TsSelector, second: TsSelector) -> TsSelector {
        first.parts.extend(second.parts);
        first
    }
}

impl RouteAlg for TsRouteImpl {
    type Route = TsHttpModule;
    type Selector = TsSelector;
    type Endpoint = TsCall;

    fn initial(&self) -> TsHttpModule {
        TsHttpModule::default()
    }

    fn coproduct(&self, mut left: TsHttpModule, right: TsHttpModule) -> TsHttpModule {
        left.declarations.extend(right.declarations);
        left.calls.extend(right.calls);

        left.written()
    }

    fn precompose(&self, selector: TsSelector, mut route: TsHttpModule) -> TsHttpModule {
        for (composed, _) in &mut route.calls {
            *composed = self.compose(selector.clone(), core::mem::take(composed));
        }

        route.written()
    }

    fn lift(&self, call: TsCall) -> TsHttpModule {
        let module = TsHttpModule {
            declarations: call.declarations(),
            entries: BTreeMap::new(),
            calls: vec![(TsSelector::default(), call)],
        };

        module.written()
    }
}

impl HttpSelectorAlg for TsRouteImpl {
    type Selector = TsSelector;

    fn http_method(&self, method: HttpMethod) -> TsSelector {
        TsSelector { parts: vec![TsSelectorPart::Method(method)] }
    }

    fn http_path(&self, path: &RoutePath) -> TsSelector {
        TsSelector { parts: vec![TsSelectorPart::Path(path.clone())] }
    }

    fn http_prefix(&self, prefix: &RoutePath) -> TsSelector {
        TsSelector { parts: vec![TsSelectorPart::Prefix(prefix.clone())] }
    }
}
