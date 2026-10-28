//! Runs utilities that read their words as POSIX scripts write them: `head -5` and `sed -i.bak`.

use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_cli_direct::{CliDirect, Exit};
use alux_ext::ext;
use std::num::NonZeroUsize;

trait Text {}

#[ext(name = TextExt, defunc)]
impl<This> This
where
    This: Text,
{
    /// Writes the first lines of a file.
    fn head(&self, lines: Option<usize>, file: String) -> String {
        format!("{} of {file}", lines.unwrap_or(10))
    }

    /// Edits a file, in place when asked, keeping a backup with the suffix given.
    // Absent, given bare, or given a suffix: three meanings, which `Option<Option<_>>` states.
    #[allow(clippy::option_option)]
    fn sed(&self, in_place: Option<Option<String>>, script: String) -> String {
        format!("{script} in_place={in_place:?}")
    }

    /// Copies files, recursing into directories when asked.
    fn cp(&self, recursive: bool, files: Vec<String>) -> String {
        format!("recursive={recursive} {files:?}")
    }

    /// Copies blocks between files, as `dd` does.
    fn dd(
        &self,
        input: Option<String>,
        block: Option<NonZeroUsize>,
        skip: usize,
        conversions: Vec<String>,
        extra: Vec<String>,
    ) -> String {
        format!("if={input:?} bs={block:?} skip={skip} conv={conversions:?} extra={extra:?}")
    }

    /// Splits a file into pieces of a positive number of lines.
    fn split(&self, lines: NonZeroUsize) -> String {
        format!("{lines}")
    }
}

#[ext(name = TextCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn text_cli<Machine>(&self) {
        self.commands()
            .command(
                "head",
                self.op(Machine::head)
                    .arg(self.named("Lines to write").optional().short('n'))
                    .arg(self.positional("The file"))
                    .numeric_option("lines"),
            )
            .command(
                "sed",
                self.op(Machine::sed)
                    .arg(self.named("Edit in place, keeping a backup with this suffix").optional_value().short('i'))
                    .arg(self.positional("The script")),
            )
            .command(
                "cp",
                self.op(Machine::cp)
                    .arg(self.named("Copy directories recursively").switch().short('r').short_aliases(&['R']))
                    .arg(self.positional("Files").many()),
            )
            .command(
                "dd",
                self.op(Machine::dd)
                    .arg_for("input", self.named("Input file").assigned("if").optional())
                    .arg_for("block", self.named("Bytes per block").assigned("bs").optional())
                    .arg_for("skip", self.named("Input blocks to skip").assigned("skip").default("0"))
                    .arg_for("conversions", self.named("Conversions").assigned("conv").many().delimiter(','))
                    .arg_for("extra", self.positional("Other operands").many()),
            )
            .command(
                "ddo",
                self.op(Machine::dd)
                    .arg_for("input", self.named("Input file").long("if").optional())
                    .arg_for("block", self.named("Bytes per block").long("bs").optional())
                    .arg_for("skip", self.named("Input blocks to skip").default("0"))
                    .arg_for("conversions", self.named("Conversions").long("conv").many().delimiter(','))
                    .arg_for("extra", self.positional("Other operands").many()),
            )
            .command("split", self.op(Machine::split).arg(self.named("Lines per piece").short('l')))
    }
}

#[ext(name = TwiceCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn head_cli<Machine>(&self) {
        self.commands().command(
            "head",
            self.op(Machine::head).arg(self.named("Lines").optional().short('n')).arg(self.positional("File")),
        )
    }

    fn split_cli<Machine>(&self) {
        self.commands().command("head", self.op(Machine::split).arg(self.named("Lines per piece").short('n')))
    }

    /// Declares `head` twice, through programs declared apart, which only running can see.
    fn twice_cli<Machine>(&self) {
        self.commands().merge(self.head_cli::<Machine>()).merge(self.split_cli::<Machine>())
    }
}

struct Files;

impl Text for Files {}

fn run(line: &str) -> Exit {
    CliDirect::new(Files).run_once_blocking(CliProgramBuilder.text_cli::<Files>(), line.split_whitespace())
}

#[test]
fn reads_a_dash_and_digits_as_the_numeric_option() {
    assert_eq!(run("box head -5 f").stdout, "5 of f\n");
    assert_eq!(run("box head f -20").stdout, "20 of f\n");
    assert_eq!(run("box head -n 3 f").stdout, "3 of f\n");
    assert_eq!(run("box head f").stdout, "10 of f\n");
    // Only a command stating the convention reads it.
    assert_eq!(run("box sed -5 p").status, 2);
}

#[test]
fn reads_an_optional_value_from_the_option_s_own_word() {
    assert_eq!(run("box sed p").stdout, "p in_place=None\n");
    assert_eq!(run("box sed -i p").stdout, "p in_place=Some(None)\n");
    assert_eq!(run("box sed -i.bak p").stdout, "p in_place=Some(Some(\".bak\"))\n");
    assert_eq!(run("box sed --in-place=.orig p").stdout, "p in_place=Some(Some(\".orig\"))\n");
    assert!(run("box sed --help").stdout.contains("-i, --in-place[=<in-place>]"));
}

#[test]
fn reads_each_short_spelling_of_a_flag_alike() {
    assert_eq!(run("box cp -r a b").stdout, "recursive=true [\"a\", \"b\"]\n");
    assert_eq!(run("box cp -R a b"), run("box cp -r a b"));
    assert_eq!(run("box cp a -R b"), run("box cp --recursive a b"));
    assert!(run("box cp --help").stdout.contains("-r, -R, --recursive"));
}

#[test]
fn reads_a_value_the_type_states_as_not_zero() {
    assert_eq!(run("box split -l 3").stdout, "3\n");
    let zero = run("box split -l 0");
    assert_eq!(zero.status, 2);
    assert!(zero.stderr.starts_with("error: invalid <lines>"), "{}", zero.stderr);
}

#[test]
fn reads_an_operand_assigning_an_argument_as_the_option_given_its_value() {
    let same = |assigned: &str, options: &str| {
        assert_eq!(run(&format!("box dd {assigned}")), run(&format!("box ddo {options}")));
    };
    same("if=a bs=4 skip=2 conv=notrunc,sync conv=x", "--if=a --bs=4 --skip=2 --conv=notrunc,sync --conv=x");
    same("", "");
    // The value is everything after the first `=`, and may be empty.
    same("if=", "--if=");
    same("if=a=b", "--if=a=b");
    // Assigning operands take no positional place, wherever they are written.
    assert_eq!(run("box dd a if=x b"), run("box dd if=x a b"));
    assert_eq!(run("box dd a if=x b").stdout, "if=Some(\"x\") bs=None skip=0 conv=[] extra=[\"a\", \"b\"]\n");
    // `--` ends options, not assignments.
    assert_eq!(run("box dd -- if=x"), run("box dd if=x"));
}

#[test]
fn rejects_what_an_assigned_argument_does_not_take() {
    // A single argument assigned twice, and a value its type rejects.
    assert_eq!(run("box dd if=a if=b").status, 2);
    assert_eq!(run("box dd bs=0").status, 2);
    // A name no argument is assigned is an operand like any other.
    assert_eq!(run("box dd x=y").stdout, "if=None bs=None skip=0 conv=[] extra=[\"x=y\"]\n");
    assert_eq!(run("box split x=y -l 2").stderr.lines().next(), Some("error: unexpected argument `x=y`"));
    // Matching is exact and case-sensitive.
    assert!(run("box dd IF=a").stdout.contains("extra=[\"IF=a\"]"));
}

#[test]
fn helps_with_assigned_arguments_in_their_own_group() {
    let help = run("box dd --help").stdout;
    assert!(help.contains("Usage: box dd [if=VALUE] [bs=VALUE] [skip=VALUE] [conv=VALUE]... [extra]..."), "{help}");
    assert!(help.contains("\nAssignments:\n"), "{help}");
    assert!(help.contains("[conv=VALUE]...  Conversions"), "{help}");
}

/// The check is a debug build's, as in `clap`.
#[cfg(debug_assertions)]
#[test]
fn runs_nothing_from_a_program_declaring_a_command_twice() {
    use alux_cli_direct::DUPLICATE_STATUS;
    let twice =
        CliDirect::new(Files).run_once_blocking(CliProgramBuilder.twice_cli::<Files>(), ["box", "head", "-n", "3"]);
    assert_eq!(twice.status, DUPLICATE_STATUS);
    assert_eq!(twice.stderr, "error: commands declared more than once: `head`\n");
    // Whatever the words: the program fails on every start.
    for words in [&["box"][..], &["box", "--help"], &["box", "other"]] {
        let start =
            CliDirect::new(Files).run_once_blocking(CliProgramBuilder.twice_cli::<Files>(), words.iter().copied());
        assert_eq!(start.status, DUPLICATE_STATUS, "{words:?}");
    }
}
