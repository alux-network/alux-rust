//! Runs programs whose commands and sources `clap` must keep apart, through `clap`.

use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_cli_clap::{CliClap, UNSUPPORTED_STATUS};
use alux_cli_direct::Exit;
use alux_ext::ext;
use std::{ffi::OsString, sync::Arc};

trait Tool {}

#[ext(name = ToolExt, defunc)]
impl<This> This
where
    This: Tool,
{
    /// Runs with a configuration.
    fn serve(&self, config: String) -> String {
        format!("serve {config}")
    }

    /// Checks a file.
    fn check(&self, file: String) -> String {
        format!("check {file}")
    }

    /// Copies a path to a named path.
    // `Debug` shows a word's bytes that are not text, which is what is tested.
    #[allow(clippy::unnecessary_debug_formatting)]
    fn copy(&self, from: OsString, to: OsString) -> String {
        format!("copy {from:?} {to:?}")
    }

    /// Answers nothing.
    fn fail(&self) {}
}

#[ext(name = ToolCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn tool_cli<Machine>(&self) {
        self.commands()
            .cmd(self.op(Machine::serve).arg(self.named("Configuration")))
            .command("check", self.op(Machine::check).arg(self.positional("File")))
            .command(
                "copy",
                self.op(Machine::copy)
                    .arg_for("from", self.positional("From"))
                    .arg_for("to", self.named("To").long("from")),
            )
    }

    fn false_cli<Machine>(&self) {
        self.commands().command("false", self.op(Machine::fail).verbatim())
    }
}

struct Local;

impl Tool for Local {}

fn tool<Word: Into<OsString>>(words: impl IntoIterator<Item = Word>) -> Exit {
    CliClap::new(Arc::new(Local), CliProgramBuilder.tool_cli::<Local>()).run_blocking(words)
}

#[test]
fn selects_a_command_without_the_programs_own_arguments() {
    assert_eq!(tool(["tool", "check", "a.toml"]).stdout, "check a.toml\n");
    assert_eq!(tool(["tool", "--config", "c"]).stdout, "serve c\n");
    assert_ne!(tool(["tool"]).status, 0);
}

#[test]
fn keeps_an_operand_and_an_option_of_one_name_apart() {
    assert_eq!(tool(["tool", "copy", "a", "--from", "b"]).stdout, "copy \"a\" \"b\"\n");
}

#[cfg(unix)]
#[test]
fn reads_a_value_whatever_its_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let raw = OsString::from_vec(b"a\xff".to_vec());
    let exit = tool([OsString::from("tool"), "copy".into(), raw, "--from".into(), "b".into()]);
    assert_eq!(exit.stdout, "copy \"a\\xFF\" \"b\"\n", "{exit:?}");
}

#[test]
fn refuses_conventions_it_does_not_read() {
    let exit =
        CliClap::new(Arc::new(Local), CliProgramBuilder.false_cli::<Local>()).run_blocking(["box", "false", "--help"]);
    assert_eq!(exit.status, UNSUPPORTED_STATUS);
    assert!(exit.stderr.contains("`false`"), "{exit:?}");
}
