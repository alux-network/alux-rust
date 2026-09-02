//! The interpreter: a program, read as a client module.

use crate::TsParams;
use alux_ext::{ApplyAlg, HandlerContextAlg};
use alux_jsonrpc::{JsonRpcAlg, JsonRpcFallibleAlg, JsonRpcMethodAlg, OutcomeAlg};
use alux_shape::{ShapeOf, Spelling, words_of};
use alux_shape_typescript::{TsShape, TsType};
use std::collections::BTreeMap;

/// The package that interprets a program: what turns one into a client, installed once.
///
/// A generated module imports `method` from here rather than restating it, so a surface and what
/// reads a surface are upgraded separately — which is the whole reason a program is a value.
pub const RUNTIME_PACKAGE: &str = "@alux-network/api";

/// Owns a shape's declarations, so a module can keep them.
fn owned(shape: &TsType) -> Vec<(String, String)> {
    shape.declarations().map(|(name, declaration)| (name.to_owned(), declaration.to_owned())).collect()
}

/// A client module: the declarations its calls depend on, and the calls themselves.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TsModule {
    declarations: BTreeMap<String, String>,
    entries: BTreeMap<String, String>,
}

impl TsModule {
    /// Writes the module: every declaration a call depends on, then the program the calls form.
    #[must_use]
    pub fn render(&self) -> String {
        let declarations: Vec<&str> = self.declarations.values().map(String::as_str).collect();
        let entries: Vec<String> = self.entries.values().map(|entry| format!("  {entry},")).collect();
        let program = format!("export const program = {{\n{}\n}} as const", entries.join("\n"));

        if declarations.is_empty() { program } else { format!("{}\n\n{program}", declarations.join("\n\n")) }
    }

    /// The calls this module states, by the method name each answers to.
    #[must_use]
    pub fn method_names(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }
}

/// Interprets a JSON-RPC program as a TypeScript client module.
#[derive(Debug, Clone, Copy)]
pub struct TsClient {
    shapes: TsShape,
    members: Spelling,
}

impl TsClient {
    /// Emits a client whose member names are spelled this way.
    #[must_use]
    pub fn new(members: Spelling) -> Self {
        Self { shapes: TsShape::new(members), members }
    }

    /// States one call: its name, the parameters it takes, and what it answers with.
    /// Renders one method entry.
    ///
    /// `labels` name the parameters for whoever writes the call, and every method has them. `wire`
    /// names them for the request document, which only a method decoded from a parameter object does.
    fn method(
        &self,
        name: &'static str,
        params: &[TsType],
        labels: &[&str],
        wire: &[&str],
        answer: &TsType,
    ) -> TsModule {
        let mut declarations = BTreeMap::new();
        let mut written = Vec::new();

        for (index, param) in params.iter().enumerate() {
            declarations.extend(owned(param));

            // Every parameter the operation named is labelled, however the wire carries it.
            written.push(match labels.get(index) {
                // An argument name arrives as it was authored, so its words are read out of it.
                Some(label) => format!("{}: {}", self.members.spell(&words_of(label)), param.expr()),
                None => param.expr().to_owned(),
            });
        }

        declarations.extend(owned(answer));

        // The name a method answers to is already an identifier, so a caller writes `api.{name}(…)`
        // rather than indexing a quoted key. Two namespaces cannot collide, since the whole name is
        // kept rather than a stem of it.
        let entry = format!(
            "{name}: method<[{}], {}>(\"{name}\", [{}])",
            written.join(", "),
            answer.expr(),
            wire.iter().map(|name| format!("\"{name}\"")).collect::<Vec<_>>().join(", "),
        );

        TsModule { declarations, entries: BTreeMap::from([(name.to_owned(), entry)]) }
    }
}

impl JsonRpcAlg for TsClient {
    type Methods = TsModule;

    fn jsonrpc_empty(&self) -> TsModule {
        TsModule::default()
    }

    fn jsonrpc_merge(&self, left: TsModule, right: TsModule) -> TsModule {
        let mut merged = left;
        merged.declarations.extend(right.declarations);
        merged.entries.extend(right.entries);

        merged
    }
}

impl<Context> HandlerContextAlg<Context> for TsClient
where
    Context: Send + Sync + 'static,
{
    // A client applies nothing, so the handle it names is only what the program's obligation asks
    // for: an owned reference to the domain the operations read.
    type Handle = std::sync::Arc<Context>;
}

impl<Handle, Args, Output> JsonRpcMethodAlg<Handle, Args, Output> for TsClient
where
    Args: TsParams,
    Output: ShapeOf<TsShape, Shape = TsType>,
{
    fn finish_jsonrpc_positional_method<Handler>(
        &self,
        name: &'static str,
        arg_names: &'static [&'static str],
        _handler: Handler,
    ) -> TsModule
    where
        Handler: ApplyAlg<Handle, Args, Output = Output> + Send + Sync + 'static,
    {
        // The request document carries an array, so nothing names the parameters on the wire. The
        // caller still gets the names, because the operation carries them either way.
        self.method(name, &Args::params(&self.shapes), arg_names, &[], &Output::shape_of(&self.shapes))
    }

    fn finish_jsonrpc_named_method<Handler>(
        &self,
        name: &'static str,
        arg_names: &'static [&'static str],
        _handler: Handler,
    ) -> TsModule
    where
        Handler: ApplyAlg<Handle, Args, Output = Output> + Send + Sync + 'static,
    {
        self.method(name, &Args::params(&self.shapes), arg_names, arg_names, &Output::shape_of(&self.shapes))
    }
}

impl<Handle, Args, Output> JsonRpcFallibleAlg<Handle, Args, Output> for TsClient
where
    Args: TsParams,
    Output: OutcomeAlg,
    Output::Value: ShapeOf<TsShape, Shape = TsType>,
{
    fn finish_jsonrpc_positional_fallible<Handler>(
        &self,
        name: &'static str,
        arg_names: &'static [&'static str],
        _handler: Handler,
    ) -> TsModule
    where
        Handler: ApplyAlg<Handle, Args, Output = Output> + Send + Sync + 'static,
    {
        // A failure reaches a caller as a rejected call, so only the value it answers with is a type.
        self.method(name, &Args::params(&self.shapes), arg_names, &[], &Output::Value::shape_of(&self.shapes))
    }

    fn finish_jsonrpc_named_fallible<Handler>(
        &self,
        name: &'static str,
        arg_names: &'static [&'static str],
        _handler: Handler,
    ) -> TsModule
    where
        Handler: ApplyAlg<Handle, Args, Output = Output> + Send + Sync + 'static,
    {
        self.method(name, &Args::params(&self.shapes), arg_names, arg_names, &Output::Value::shape_of(&self.shapes))
    }
}
