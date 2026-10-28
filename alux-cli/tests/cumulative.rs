//! Declares a CLI program and a JSON-RPC program in one extension, `defunc(via = cli, via = jsonrpc)`.

use alux_cli::{CliCommandsAlg, CliOperationAlg, CliProgramBuilder, CliProgramExt, Conventions, cli};
use alux_ext::{ApplyAlg, HandlerContextAlg, OperationAlg, ext};
use alux_jsonrpc::{JsonRpcAlg, JsonRpcApiAlg, JsonRpcMethodAlg, JsonRpcProgramBuilder, JsonRpcProgramExt, jsonrpc};
use std::sync::Arc;

trait Reading {
    fn total(&self) -> u16;
}

#[ext(name = ReadingExt, defunc)]
impl<This> This
where
    This: Reading,
{
    /// Sums every value.
    fn sum(&self) -> u16 {
        self.total()
    }
}

#[ext(name = ReadingSurfacesExt, defunc(via = cli, via = jsonrpc))]
impl<This> This {
    /// Declares the commands.
    fn reading_cli<Machine>(&self)
    where
        This: CliCommandsAlg,
    {
        self.commands().command("sum", self.op(Machine::sum))
    }

    /// Declares the methods.
    fn reading_rpc<Machine>(&self)
    where
        This: JsonRpcApiAlg,
    {
        self.methods().method("reading_sum", self.op(Machine::sum))
    }
}

struct Memory;

impl Reading for Memory {
    fn total(&self) -> u16 {
        99
    }
}

/// Lists what each surface names, with the operation it applies.
struct Names;

impl CliCommandsAlg for Names {
    type Commands = Vec<String>;

    fn cli_empty(&self) -> Vec<String> {
        Vec::new()
    }

    fn cli_merge(&self, mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
        left.extend(right);
        left
    }
}

impl<Operation: OperationAlg, Inputs, Mode> CliOperationAlg<Operation, Inputs, Mode> for Names {
    type Commands = Vec<String>;

    fn cli_operation(&self, name: &'static str, _: Operation, _: Inputs, _: Mode, _: Conventions) -> Vec<String> {
        vec![format!("{name} -> {}", Operation::NAME)]
    }
}

impl JsonRpcAlg for Names {
    type Methods = Vec<String>;

    fn jsonrpc_empty(&self) -> Vec<String> {
        Vec::new()
    }

    fn jsonrpc_merge(&self, mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
        left.extend(right);
        left
    }
}

impl HandlerContextAlg<Memory> for Names {
    type Handle = Arc<Memory>;
}

impl<Args, Output> JsonRpcMethodAlg<Arc<Memory>, Args, Output> for Names {
    fn finish_jsonrpc_positional_method<Handler>(
        &self,
        name: &'static str,
        _: &'static [&'static str],
        _: Handler,
    ) -> Vec<String>
    where
        Handler: ApplyAlg<Arc<Memory>, Args, Output = Output> + Send + Sync + 'static,
    {
        vec![name.to_owned()]
    }

    fn finish_jsonrpc_named_method<Handler>(
        &self,
        name: &'static str,
        _: &'static [&'static str],
        _: Handler,
    ) -> Vec<String>
    where
        Handler: ApplyAlg<Arc<Memory>, Args, Output = Output> + Send + Sync + 'static,
    {
        vec![name.to_owned()]
    }
}

#[test]
fn each_backend_adds_its_own_program_to_one_extension() {
    assert_eq!(Names.compile_cli(CliProgramBuilder.reading_cli::<Memory>()), ["sum -> sum"]);
    assert_eq!(Names.compile_jsonrpc(JsonRpcProgramBuilder.reading_rpc::<Memory>()), ["reading_sum"]);
}
