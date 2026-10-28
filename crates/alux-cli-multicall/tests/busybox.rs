//! Runs a small BusyBox-like multicall executable, by its own name and by each command's name.

use alux_cli::{CliCommandsAlg, CliFailureAlg, CliProgramBuilder, cli};
use alux_cli_direct::{CliDirect, Exit, FailurePrefix};
use alux_cli_multicall::Multicall;
use alux_ext::ext;
use core::{fmt, num::NonZeroU8};
use std::{collections::BTreeMap, ffi::OsString};

/// Writes bytes as hexadecimal.
fn hex(bytes: &[u8]) -> String {
    use fmt::Write;
    bytes.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

/// A failure already written as it happened, so only its status remains.
#[derive(Debug)]
struct Reported(u8);

impl fmt::Display for Reported {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl CliFailureAlg for Reported {
    fn cli_status(&self) -> NonZeroU8 {
        NonZeroU8::new(self.0).unwrap_or(NonZeroU8::MIN)
    }

    fn cli_reported(&self) -> bool {
        true
    }
}

trait Unix {
    fn read(&self, name: &str) -> Option<String>;
    fn report(&mut self, line: String);
}

#[ext(name = UnixExt, defunc)]
impl<This> This
where
    This: Unix,
{
    /// Concatenates files, reporting each missing one and reading on.
    fn cat(&mut self, files: Vec<String>) -> Result<String, Reported> {
        let mut out = String::new();
        let mut missing = false;
        for file in files {
            if let Some(contents) = self.read(&file) {
                out.push_str(&contents);
            } else {
                self.report(format!("cat: {file}: No such file or directory"));
                missing = true;
            }
        }
        if missing { Err(Reported(1)) } else { Ok(out) }
    }

    /// Answers the switches it was given and its words, each word as the hexadecimal of its bytes.
    fn echo(&mut self, no_newline: bool, escapes: bool, no_escapes: bool, words: Vec<OsString>) -> String {
        let switches = [(no_newline, 'n'), (escapes, 'e'), (no_escapes, 'E')];
        let switches = switches.iter().filter(|(given, _)| *given).map(|(_, letter)| *letter).collect::<String>();
        let words = words.iter().map(|word| hex(word.as_encoded_bytes()));
        format!("[{switches}] {}", words.collect::<Vec<_>>().join(" "))
    }

    /// Succeeds, whatever its words.
    fn truth(&mut self, ignored: Vec<OsString>) {
        drop(ignored);
    }

    /// Fails with status 1, whatever its words, and says nothing.
    fn falsity(&mut self, ignored: Vec<OsString>) -> Result<(), Reported> {
        drop(ignored);
        Err(Reported(1))
    }

    /// Lists the commands.
    fn list(&mut self) -> String {
        Multicall::new(CliProgramBuilder.applets::<Self>()).names().join("\n")
    }
}

#[ext(name = BoxCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    /// Declares the applets.
    fn applets<Machine>(&self) {
        self.commands()
            .command("cat", self.op(Machine::cat).arg(self.positional("Files").many()).fallible())
            .command(
                "echo",
                self.op(Machine::echo)
                    .arg(self.named("No newline").switch().short('n'))
                    .arg(self.named("Escapes").switch().short('e'))
                    .arg(self.named("No escapes").switch().short('E'))
                    .arg(self.positional("Words").many())
                    .options_first()
                    .unknown_options_as_operands()
                    .no_end_of_options(),
            )
            .command("true", self.op(Machine::truth).arg(self.positional("Ignored").many()).verbatim())
            .command("false", self.op(Machine::falsity).arg(self.positional("Ignored").many()).verbatim().fallible())
    }

    /// Declares the executable: the applets, and `--list` at its root.
    fn executable<Machine>(&self) {
        self.commands().merge(self.applets::<Machine>()).command("--list", self.op(Machine::list))
    }
}

/// Owns its files and what it reports, so it cannot be cloned.
#[derive(Default)]
struct System {
    files: BTreeMap<String, String>,
    reported: Vec<String>,
}

impl Unix for System {
    fn read(&self, name: &str) -> Option<String> {
        self.files.get(name).cloned()
    }

    fn report(&mut self, line: String) {
        self.reported.push(line);
    }
}

fn run(words: Vec<OsString>) -> Exit {
    let system =
        System { files: BTreeMap::from([("a".into(), "A".into()), ("b".into(), "B".into())]), ..System::default() };
    let invocation = Multicall::new(CliProgramBuilder.executable::<System>()).invocation(words);
    CliDirect::new(system)
        .named(invocation.name)
        .failures_after(FailurePrefix::Command)
        .unknown_command(127, |word| format!("{word}: applet not found"))
        .selecting(invocation.command)
        .run_once_blocking(CliProgramBuilder.executable::<System>(), invocation.words)
}

fn line(text: &str) -> Exit {
    run(text.split(' ').map(OsString::from).collect())
}

#[test]
fn runs_a_command_by_its_own_name_as_beneath_the_executable() {
    assert_eq!(line("cat a b"), line("box cat a b"));
    assert_eq!(line("/bin/cat a b").stdout, "AB\n");
    assert_eq!(
        Multicall::new(CliProgramBuilder.executable::<System>()).names(),
        ["cat", "echo", "true", "false", "--list"]
    );
    assert_eq!(line("box --list").stdout, "cat\necho\ntrue\nfalse\n");
}

#[test]
fn ends_an_unknown_command_with_its_own_status() {
    assert_eq!(
        line("box nope"),
        Exit { status: 127, stdout: String::new(), stderr: "nope: applet not found\n".into() }
    );
}

#[test]
fn leaves_a_reported_failure_its_status_alone() {
    assert_eq!(line("cat a missing b"), Exit { status: 1, stdout: String::new(), stderr: String::new() });
    assert_eq!(line("false --help ignored"), Exit { status: 1, stdout: String::new(), stderr: String::new() });
    assert_eq!(line("true --help ignored"), Exit { status: 0, stdout: String::new(), stderr: String::new() });
}

/// What `echo` answers: its switches and each word's bytes in hexadecimal.
fn echoed(switches: &str, words: &[&str]) -> String {
    let words = words.iter().map(|word| hex(word.as_bytes()));
    format!("[{switches}] {}\n", words.collect::<Vec<_>>().join(" "))
}

#[test]
fn reads_words_as_each_command_declares() {
    // Options end at the first operand.
    assert_eq!(line("echo hello -n").stdout, echoed("", &["hello", "-n"]));
    // A word naming no option is an operand.
    assert_eq!(line("echo -x").stdout, echoed("", &["-x"]));
    // `--` is an operand.
    assert_eq!(line("echo -- -n").stdout, echoed("", &["--", "-n"]));
    assert_eq!(line("echo -enE a b").stdout, echoed("neE", &["a", "b"]));
}

#[cfg(unix)]
#[test]
fn passes_a_word_that_is_not_utf8_unchanged() {
    use std::os::unix::ffi::OsStringExt;
    let echoed = run(vec!["echo".into(), OsString::from_vec(vec![0xff, 0xfe])]);
    assert_eq!(echoed.stdout, "[] fffe\n");
}
