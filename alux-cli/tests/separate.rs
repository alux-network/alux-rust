//! Composes programs declared in separate extension blocks.

use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_ext::ext;

trait Files {
    fn files(&self) -> usize;
}

#[ext(name = FilesExt, defunc)]
impl<This> This
where
    This: Files,
{
    /// Concatenates files.
    fn cat(&self, unbuffered: bool) -> usize {
        self.files() + usize::from(unbuffered)
    }

    /// Writes words.
    fn echo(&self, words: Vec<String>) -> String {
        words.join(" ")
    }
}

#[ext(name = CatCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn cat_commands<Machine>(&self) {
        self.commands().command("cat", self.op(Machine::cat).arg(self.named("Unbuffered").short('u')))
    }
}

#[ext(name = EchoCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn echo_commands<Machine>(&self) {
        self.commands().command("echo", self.op(Machine::echo).arg(self.positional("Words")))
    }
}

#[ext(name = CommandsExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn surface<Machine>(&self) {
        self.commands().merge(self.cat_commands::<Machine>()).merge(self.echo_commands::<Machine>())
    }
}

#[test]
fn merges_programs_declared_in_other_blocks_by_position_not_name() {
    let _program = CliProgramBuilder.surface::<()>();
}
