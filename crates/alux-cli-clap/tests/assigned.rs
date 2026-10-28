//! Runs a command whose arguments are assigned by operands, `dd if=FILE`, through `clap`.

use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_cli_clap::CliClap;
use alux_cli_direct::{CliDirect, Exit};
use alux_ext::ext;
use std::sync::Arc;

trait Blocks {}

#[ext(name = BlocksExt, defunc)]
impl<This> This
where
    This: Blocks,
{
    /// Copies blocks between files.
    fn dd(&self, input: Option<String>, count: Option<u64>, conversions: Vec<String>, extra: Vec<String>) -> String {
        format!("if={input:?} count={count:?} conv={conversions:?} extra={extra:?}")
    }
}

#[ext(name = BlocksCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn blocks_cli<Machine>(&self) {
        self.commands().command(
            "dd",
            self.op(Machine::dd)
                .arg_for("input", self.named("Input file").assigned("if").optional())
                .arg_for("count", self.named("Blocks to copy").assigned("count").optional())
                .arg_for("conversions", self.named("Conversions").assigned("conv").many().delimiter(','))
                .arg_for("extra", self.positional("Other operands").many()),
        )
    }
}

struct Files;

impl Blocks for Files {}

fn clap(line: &str) -> Exit {
    CliClap::new(Arc::new(Files), CliProgramBuilder.blocks_cli::<Files>()).run_blocking(line.split_whitespace())
}

fn direct(line: &str) -> Exit {
    CliDirect::new(Arc::new(Files)).run_blocking(CliProgramBuilder.blocks_cli::<Files>(), line.split_whitespace())
}

#[test]
fn reads_assigned_operands_as_the_direct_interpretation_does() {
    for line in ["box dd if=a count=3 conv=x,y conv=z", "box dd a if= b", "box dd x=y", "box dd"] {
        assert_eq!(clap(line).stdout, direct(line).stdout, "{line}");
    }
    assert_eq!(clap("box dd count=x").status, direct("box dd count=x").status);
}
