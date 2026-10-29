use crate::command_line::{Cursor, Read, Skipped, read_word};
use crate::{DirectRender, ParseError, ParseErrorKind, Parsed, help};
use alux_cli::{
    ArgumentInfo, CliArgumentError, CliCommandsAlg, CliContextAlg, CliDecodeSources, CliFailureAlg, CliNestAlg,
    CliOperationAlg, CliOutcome, CliOutcomeOf, CliProgramAlg, CliSources, CommandDescription, Conventions, SourceInfo,
    USAGE_STATUS,
};
use alux_ext::{ApplyAlg, OperationAlg};
use core::{fmt::Display, future::Future, marker::PhantomData, pin::Pin};
use std::{
    ffi::{OsStr, OsString},
    io::Write,
    path::Path,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread::{self, Thread},
};

/// What one run of a command line shows: its exit status and what it writes to each stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exit {
    /// The status the process exits with; zero is success.
    pub status: u8,
    /// The text written to standard output.
    pub stdout: String,
    /// The text written to standard error.
    pub stderr: String,
}

impl Exit {
    fn out(status: u8, stdout: String) -> Self {
        Self { status, stdout, stderr: String::new() }
    }

    fn err(status: u8, stderr: String) -> Self {
        Self { status, stdout: String::new(), stderr }
    }

    /// Writes both streams and ends the process with the status.
    pub fn exit(self) -> ! {
        print!("{}", self.stdout);
        eprint!("{}", self.stderr);
        let _ = std::io::stdout().flush();
        std::process::exit(i32::from(self.status))
    }
}

/// A command running on its handle, ending in the exit it renders.
pub type Running = Pin<Box<dyn Future<Output = Exit> + Send>>;

/// A command whose arguments are decoded, waiting for the handle it is applied to.
pub type Ready<Handle> = Box<dyn FnOnce(Handle) -> Running + Send>;

/// Decodes one command's arguments, its global ones included, or says which its words do not make.
///
/// It reads the arguments the command was described as: the global arguments of the contexts
/// around it, outermost first, and then its own. A failure it renders is prefixed with the name it
/// is given.
type Decode<Handle, Decoder> = Arc<
    dyn Fn(&Decoder, &[ArgumentInfo], &[ArgumentInfo], &str) -> Result<Ready<Handle>, CliArgumentError<String>>
        + Send
        + Sync,
>;

/// One command of a compiled program: what it is, and how to run it on a handle once its words,
/// read by a `Decoder`, are decoded.
pub struct Entry<Handle, Decoder = Parsed> {
    description: CommandDescription,
    decode: Decode<Handle, Decoder>,
}

impl<Handle, Decoder> Clone for Entry<Handle, Decoder> {
    fn clone(&self) -> Self {
        Self { description: self.description.clone(), decode: Arc::clone(&self.decode) }
    }
}

impl<Handle, Decoder> Entry<Handle, Decoder> {
    /// Describes the command: its path, documentation, and arguments, global ones included.
    pub fn description(&self) -> &CommandDescription {
        &self.description
    }

    /// Decodes the command's arguments from what `decoder` read, ready to apply to a handle; a
    /// failure it renders is prefixed with `prefix`.
    ///
    /// # Errors
    ///
    /// Returns the argument the words do not make.
    pub fn decode(&self, decoder: &Decoder, prefix: &str) -> Result<Ready<Handle>, CliArgumentError<String>> {
        (self.decode)(decoder, &self.description.context, &self.description.arguments, prefix)
    }
}

/// Compiles a CLI program into the commands run on a `Handle`, their words read by a `Decoder`.
///
/// `CliDirect` reads words with `Parsed`; another interpretation that parses its own way compiles
/// with its own decoder. A context compiles its subtree with the compiler for the handle the
/// context answers.
pub struct DirectCompiler<Handle, Decoder = Parsed> {
    /// The only first-layer name whose commands are compiled, when not every command is.
    selected: Option<&'static str>,
    marker: PhantomData<fn() -> (Handle, Decoder)>,
}

impl<Handle, Decoder> DirectCompiler<Handle, Decoder> {
    /// Compiles only the commands beneath the first-layer name `name`, so no other command is
    /// described or decoded.
    pub fn selecting(name: &'static str) -> Self {
        Self { selected: Some(name), marker: PhantomData }
    }

    /// Whether the first-layer name `name` is compiled.
    fn compiles(&self, name: &str) -> bool {
        self.selected.is_none_or(|selected| selected == name)
    }
}

impl<Handle, Decoder> Default for DirectCompiler<Handle, Decoder> {
    fn default() -> Self {
        Self { selected: None, marker: PhantomData }
    }
}

/// Interprets a CLI program as a command line, applying its operations to `Handle`.
///
/// It parses the words, selects the command by its path, decodes the arguments through their
/// sources, applies the operation, and renders the outcome. `--help` or `-h` prints help built
/// from the same program.
pub struct CliDirect<Handle> {
    handle: Handle,
    name: Option<String>,
    prefix: FailurePrefix,
    unknown: Option<Unknown>,
    usage: u8,
    selected: Option<&'static str>,
    version: Option<&'static str>,
}

/// How a command line naming no command ends: its status, and its message for the word.
type Unknown = (u8, Arc<dyn Fn(&str) -> String + Send + Sync>);

/// The status a program declaring a command path more than once exits with, in a debug build:
/// `EX_SOFTWARE`, as it is the program's fault, not the command line's.
pub const DUPLICATE_STATUS: u8 = 70;

/// Names what a failure is written after: `error: reason`, `cat: reason`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FailurePrefix {
    /// `error`.
    #[default]
    Error,
    /// The name the program is invoked as, the file name of the first word.
    Invocation,
    /// The name of the command that failed, or the invocation name when no command was selected.
    Command,
}

impl<Handle> CliDirect<Handle> {
    /// Interprets programs against `handle`, the context every operation is applied to.
    pub fn new(handle: Handle) -> Self {
        Self {
            handle,
            name: None,
            prefix: FailurePrefix::Error,
            unknown: None,
            usage: USAGE_STATUS.get(),
            selected: None,
            version: None,
        }
    }

    /// Names the program in help instead of the first word of the command line; an empty name
    /// shows each command beneath no program name.
    #[must_use]
    pub fn named(self, name: impl Into<String>) -> Self {
        Self { name: Some(name.into()), ..self }
    }

    /// Writes failures after `prefix` instead of `error`.
    #[must_use]
    pub fn failures_after(self, prefix: FailurePrefix) -> Self {
        Self { prefix, ..self }
    }

    /// Ends words that do not make a command with `status` instead of `USAGE_STATUS`, as utilities
    /// that state their own do: `BusyBox` `test` exits 2, most of its others 1.
    #[must_use]
    pub fn usage_status(self, status: u8) -> Self {
        Self { usage: status, ..self }
    }

    /// Compiles only the commands beneath the first-layer name `name`, as a multicall executable
    /// invoked by that name runs no other; help then lists only those. `None` compiles every
    /// command.
    #[must_use]
    pub fn selecting(self, name: Option<&'static str>) -> Self {
        Self { selected: name, ..self }
    }

    /// Answers `--version` or `-V`, written before any command, with the program's name and
    /// `version`, and lists it in the program's help.
    #[must_use]
    pub fn version(self, version: &'static str) -> Self {
        Self { version: Some(version), ..self }
    }

    /// Ends a command line that names no command with `status` and the message `message` writes
    /// for the word, instead of a usage error with the program's help.
    #[must_use]
    pub fn unknown_command(self, status: u8, message: impl Fn(&str) -> String + Send + Sync + 'static) -> Self {
        Self { unknown: Some((status, Arc::new(message))), ..self }
    }

    /// Compiles `program` and selects the command the words name, decoded and ready for a handle.
    fn start<Program, Words>(&self, program: Program, words: Words) -> Result<Ready<Handle>, Exit>
    where
        Program: CliProgramAlg<DirectCompiler<Handle>, Commands = Vec<Entry<Handle>>>,
        Words: IntoIterator,
        Words::Item: Into<OsString>,
    {
        let mut words = words.into_iter().map(Into::into);
        let first = words.next().unwrap_or_default();
        let invoked = Path::new(&first).file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let words = words.collect::<Vec<_>>();
        let compiler = DirectCompiler { selected: self.selected, marker: PhantomData };
        let entries = program.compile_cli(&compiler);
        let name = self.name.clone().unwrap_or_else(|| invoked.clone());
        let invocation = Invocation {
            name: &name,
            invoked: &invoked,
            prefix: self.prefix,
            unknown: self.unknown.as_ref(),
            usage: self.usage,
            version: self.version,
        };
        select(&invocation, &entries, &words)
    }

    /// Runs `program` on a full command line, applying the selected command to the handle itself.
    ///
    /// It takes the handle, so a handle that owns what it reads, and cannot be cloned, runs too.
    pub fn run_once<Program, Words>(self, program: Program, words: Words) -> Running
    where
        Program: CliProgramAlg<DirectCompiler<Handle>, Commands = Vec<Entry<Handle>>>,
        Words: IntoIterator,
        Words::Item: Into<OsString>,
    {
        match self.start(program, words) {
            Ok(ready) => ready(self.handle),
            Err(exit) => Box::pin(async move { exit }),
        }
    }

    /// Runs `program` on a full command line with the handle itself, waiting on the calling thread.
    pub fn run_once_blocking<Program, Words>(self, program: Program, words: Words) -> Exit
    where
        Program: CliProgramAlg<DirectCompiler<Handle>, Commands = Vec<Entry<Handle>>>,
        Words: IntoIterator,
        Words::Item: Into<OsString>,
    {
        block_on(self.run_once(program, words))
    }
}

impl<Handle> CliDirect<Handle>
where
    Handle: Clone,
{
    /// Runs `program` on a full command line, its first word the program's name.
    pub fn run<Program, Words>(
        &self,
        program: Program,
        words: Words,
    ) -> impl Future<Output = Exit> + use<Handle, Program, Words>
    where
        Program: CliProgramAlg<DirectCompiler<Handle>, Commands = Vec<Entry<Handle>>>,
        Words: IntoIterator,
        Words::Item: Into<OsString>,
    {
        let selected = self.start(program, words).map(|ready| ready(self.handle.clone()));

        async move {
            match selected {
                Ok(running) => running.await,
                Err(exit) => exit,
            }
        }
    }

    /// Runs `program` on a full command line and waits for its exit, on the calling thread.
    pub fn run_blocking<Program, Words>(&self, program: Program, words: Words) -> Exit
    where
        Program: CliProgramAlg<DirectCompiler<Handle>, Commands = Vec<Entry<Handle>>>,
        Words: IntoIterator,
        Words::Item: Into<OsString>,
    {
        block_on(self.run(program, words))
    }
}

/// How the program was invoked: the name its help shows, the name it was invoked as, what a
/// failure is written after, how a command line naming no command ends, and the status of a usage
/// error.
struct Invocation<'a> {
    name: &'a str,
    invoked: &'a str,
    prefix: FailurePrefix,
    unknown: Option<&'a Unknown>,
    usage: u8,
    version: Option<&'static str>,
}

impl Invocation<'_> {
    /// Names what a failure of `command` is written after.
    fn prefix(&self, command: Option<&CommandDescription>) -> String {
        match self.prefix {
            FailurePrefix::Error => "error".to_owned(),
            FailurePrefix::Invocation => self.invoked.to_owned(),
            FailurePrefix::Command => command
                .and_then(|command| command.path.last())
                .map_or_else(|| self.invoked.to_owned(), |name| (*name).to_owned()),
        }
    }
}

/// Whether `word` is the text `text`.
fn is(word: &OsStr, text: &str) -> bool {
    word.to_str() == Some(text)
}

/// Selects the command the words name, by its longest path, and decodes its arguments.
fn select<Handle>(
    invocation: &Invocation<'_>,
    entries: &[Entry<Handle>],
    words: &[OsString],
) -> Result<Ready<Handle>, Exit> {
    // A program declaring one path more than once runs nothing, whatever the words: it fails on
    // every start, `--version` and `--help` included. As in `clap`, the check is the program's own,
    // not the command line's, so a debug build makes it and a release build saves its cost.
    #[cfg(debug_assertions)]
    if let Some(duplicated) = duplicated(entries) {
        return Err(Exit::err(DUPLICATE_STATUS, format!("error: commands declared more than once: {duplicated}\n")));
    }
    let name = invocation.name;
    let descriptions = entries.iter().map(|entry| &entry.description).collect::<Vec<_>>();
    if let Some(version) = invocation.version
        && words.first().is_some_and(|word| is(word, "--version") || is(word, "-V"))
    {
        return Err(Exit::out(0, format!("{name} {version}\n")));
    }
    let selected = entries
        .iter()
        .filter_map(|entry| path_words(&entry.description, words).map(|words| (entry, words)))
        .max_by_key(|(entry, _)| entry.description.path.len());
    let Some((entry, words)) = selected else {
        let prefix = invocation.prefix(None);
        let help = |word: &OsString| is(word, "--help") || is(word, "-h");
        let first = words.iter().find(|word| !help(word) && word.as_encoded_bytes().starts_with(b"-"));
        if let Some(option) = first.filter(|_| !words.iter().any(help)) {
            let option = option.to_string_lossy();
            let spelled = option.split_once('=').map_or(&*option, |(spelled, _)| spelled);
            return Err(Exit::err(
                invocation.usage,
                format!(
                    "{prefix}: unknown option `{spelled}`\n\n{}",
                    help::program(name, &descriptions, invocation.version)
                ),
            ));
        }
        return Err(match words.first().map(|word| word.to_string_lossy()) {
            None => Exit::err(invocation.usage, help::program(name, &descriptions, invocation.version)),
            Some(word) if matches!(&*word, "--help" | "-h" | "help") => {
                Exit::out(0, help::program(name, &descriptions, invocation.version))
            }
            Some(word) => match invocation.unknown {
                Some((status, message)) => Exit::err(*status, format!("{}\n", message(&word))),
                None => Exit::err(
                    invocation.usage,
                    format!(
                        "{prefix}: unknown command `{word}`\n\n{}",
                        help::program(name, &descriptions, invocation.version)
                    ),
                ),
            },
        });
    };
    let description = &entry.description;
    let conventions = description.conventions;
    let prefix = invocation.prefix(Some(description));
    let rest = &words[..];
    let arguments = description.context.iter().chain(&description.arguments).cloned().collect::<Vec<_>>();
    let declares_help = arguments.iter().any(|argument| {
        matches!(&argument.source, SourceInfo::Named { long, short, short_aliases } if long == "help" || *short == Some('h') || short_aliases.contains(&'h'))
    });
    // A verbatim command answers no help; another answers it among its options.
    let options = rest.iter().take_while(|word| !(conventions.double_dash_ends_options && is(word, "--")));
    if !declares_help && !conventions.verbatim && options.into_iter().any(|word| is(word, "--help") || is(word, "-h")) {
        // The program's own command, beside named ones, is helped with them.
        if description.path.is_empty() {
            return Err(Exit::out(0, help::program(name, &descriptions, invocation.version)));
        }
        return Err(Exit::out(0, help::command(name, description)));
    }
    let usage =
        |error: String| Exit::err(invocation.usage, format!("{prefix}: {error}\n\n{}", help::usage(name, description)));
    let parsed = Parsed::new(&arguments, rest, conventions).map_err(|error| usage(parse_error(&error, rest)))?;

    entry.decode(&parsed, &prefix).map_err(|error| usage(argument_error(&error)))
}

/// Names each command path declared more than once, in declaration order, or nothing when every path
/// is declared once.
#[cfg(debug_assertions)]
fn duplicated<Handle>(entries: &[Entry<Handle>]) -> Option<String> {
    let paths = entries.iter().map(|entry| &entry.description.path).collect::<Vec<_>>();
    let mut duplicated = Vec::new();
    for (at, path) in paths.iter().enumerate() {
        if paths[..at].contains(path) && !duplicated.contains(path) {
            duplicated.push(*path);
        }
    }
    let named = duplicated.iter().map(|path| format!("`{}`", path.join(" "))).collect::<Vec<_>>();
    (!named.is_empty()).then(|| named.join(", "))
}

/// Matches a command's path against the words, skipping the global options written before it.
///
/// Answers the words that remain for the command's arguments, the skipped global options first, or
/// nothing when the words do not name this command. A global option before or within the path is
/// read as after it, with the value word it takes.
fn path_words(description: &CommandDescription, words: &[OsString]) -> Option<Vec<OsString>> {
    let mut cursor = Cursor::new(words);
    let mut global = Vec::new();
    for segment in &description.path {
        loop {
            let start = cursor.at();
            if is(words.get(start)?, segment) {
                cursor.bump();
                break;
            }
            // Only global options come before or within the path, so `--` ends options nowhere here.
            let conventions = Conventions { double_dash_ends_options: false, ..Conventions::default() };
            let read = read_word(&description.context, conventions, &mut cursor, &mut Skipped).ok()??;
            if read != Read::Options {
                return None;
            }
            global.extend_from_slice(&words[start..cursor.at()]);
        }
    }
    global.extend_from_slice(&words[cursor.at()..]);

    Some(global)
}

/// Says what is wrong with `words`, naming the word it is in when that is more than the option.
fn parse_error(error: &ParseError, words: &[OsString]) -> String {
    let said = match &error.kind {
        ParseErrorKind::Unknown(spelling) => format!("unknown option `{spelling}`"),
        ParseErrorKind::MissingValue(spelling) => format!("`{spelling}` needs a value"),
        ParseErrorKind::SwitchValue(spelling) => format!("`{spelling}` takes no value"),
        ParseErrorKind::Unexpected(word) => return format!("unexpected argument `{word}`"),
    };
    match words.get(error.at.word).map(|word| word.to_string_lossy()) {
        // A letter inside a cluster is placed in the word that clusters it.
        Some(word) if error.at.byte > 1 => format!("{said} in `{word}`"),
        _ => said,
    }
}

fn argument_error(error: &CliArgumentError<String>) -> String {
    // An argument is named as the command line spells its value.
    let spelled = |name: &str| name.replace('_', "-");
    match error {
        CliArgumentError::Missing(name) => format!("missing <{}>", spelled(name)),
        CliArgumentError::Repeated(name) => format!("<{}> given more than once", spelled(name)),
        CliArgumentError::Word(name, reason) => format!("invalid <{}>: {reason}", spelled(name)),
        CliArgumentError::Conflict(first, second) => {
            format!("<{}> and <{}> cannot be given together", spelled(first), spelled(second))
        }
    }
}

/// Renders an outcome as the exit it ends with, or answers the value it succeeded with.
fn outcome<Value, Error>(outcome: CliOutcome<Value, Error>, prefix: &str) -> Result<Value, Exit>
where
    Error: Display + CliFailureAlg,
{
    let status = outcome.cli_status();
    match outcome {
        CliOutcome::Success(value) => Ok(value),
        // A failure already reported as it happened leaves only its status to the exit.
        CliOutcome::Failure(error) if error.cli_reported() => Err(Exit::err(status, String::new())),
        CliOutcome::Failure(error) => Err(Exit::err(status, format!("{prefix}: {error}\n"))),
    }
}

impl<Handle, Decoder> CliCommandsAlg for DirectCompiler<Handle, Decoder> {
    type Commands = Vec<Entry<Handle, Decoder>>;

    fn cli_empty(&self) -> Self::Commands {
        Vec::new()
    }

    fn cli_merge(&self, mut left: Self::Commands, right: Self::Commands) -> Self::Commands {
        left.extend(right);
        left
    }
}

impl<Handle, Decoder> CliNestAlg for DirectCompiler<Handle, Decoder> {
    type Commands = Vec<Entry<Handle, Decoder>>;

    fn cli_nest(&self, name: &'static str, mut commands: Self::Commands) -> Self::Commands {
        for entry in &mut commands {
            entry.description.path.insert(0, name);
        }
        commands
    }

    /// A selected name's subtree compiles in full; any other compiles to nothing.
    fn cli_nest_with(&self, name: &'static str, compile: impl FnOnce(&Self) -> Self::Commands) -> Self::Commands {
        if !self.compiles(name) {
            return Vec::new();
        }
        self.cli_nest(name, compile(&Self::default()))
    }
}

impl<Handle, Decoder, Operation, Sources, Mode> CliOperationAlg<Operation, Sources, Mode>
    for DirectCompiler<Handle, Decoder>
where
    Handle: Send + 'static,
    Decoder: 'static,
    Operation: OperationAlg + ApplyAlg<Handle, Operation::Args> + Send + Sync + 'static,
    Operation::Args: Send,
    Sources: CliSources<Operation::Args> + CliDecodeSources<Operation::Args, Decoder, String> + Send + Sync + 'static,
    Mode: CliOutcomeOf<Operation::Output> + Send + Sync + 'static,
    Mode::Value: DirectRender,
    Mode::Error: Display + CliFailureAlg,
{
    type Commands = Vec<Entry<Handle, Decoder>>;

    fn cli_operation(
        &self,
        name: &'static str,
        operation: Operation,
        sources: Sources,
        mode: Mode,
        conventions: Conventions,
    ) -> Self::Commands {
        if !self.compiles(name) {
            return Vec::new();
        }
        let mut description = CommandDescription::of::<Operation, Sources>(name, &sources);
        description.conventions = conventions;
        let (operation, mode) = (Arc::new(operation), Arc::new(mode));
        let decode: Decode<Handle, Decoder> = Arc::new(move |parsed, _, arguments, prefix| {
            let args = sources.cli_decode(arguments, parsed)?;
            let (operation, mode, prefix) = (Arc::clone(&operation), Arc::clone(&mode), prefix.to_owned());
            Ok(Box::new(move |handle| {
                Box::pin(async move {
                    match outcome(mode.cli_outcome(operation.apply(handle, args).await), &prefix) {
                        Ok(value) => {
                            let mut out = String::new();
                            value.render(&mut out);
                            if !out.is_empty() {
                                out.push('\n');
                            }
                            Exit::out(0, out)
                        }
                        Err(exit) => exit,
                    }
                }) as Running
            }) as Ready<Handle>)
        });
        vec![Entry { description, decode }]
    }
}

/// Applies a subtree's commands to the handle the context operation answers.
///
/// The global arguments are decoded with the command's own, so a usage error is reported before
/// the context is opened; a context that fails ends the run with its status.
impl<Handle, Decoder, Operation, Sources, Mode> CliContextAlg<Operation, Sources, Mode>
    for DirectCompiler<Handle, Decoder>
where
    Handle: Send + 'static,
    Decoder: 'static,
    Operation: OperationAlg + ApplyAlg<Handle, Operation::Args> + Send + Sync + 'static,
    Operation::Args: Send,
    Sources: CliSources<Operation::Args> + CliDecodeSources<Operation::Args, Decoder, String> + Send + Sync + 'static,
    Mode: CliOutcomeOf<Operation::Output> + Send + Sync + 'static,
    Mode::Value: Send + 'static,
    Mode::Error: Display + CliFailureAlg,
{
    type Inner = DirectCompiler<Mode::Value, Decoder>;
    type InnerCommands = Vec<Entry<Mode::Value, Decoder>>;
    type Commands = Vec<Entry<Handle, Decoder>>;

    /// A context adds no path, so its subtree's first layer is this one's, selected alike.
    fn cli_inner(&self) -> Self::Inner {
        DirectCompiler { selected: self.selected, marker: PhantomData }
    }

    fn cli_within(
        &self,
        operation: Operation,
        sources: Sources,
        mode: Mode,
        commands: Self::InnerCommands,
    ) -> Self::Commands {
        let global = sources.cli_arguments(Operation::ARG_NAMES);
        let count = global.len();
        let (operation, sources, mode) = (Arc::new(operation), Arc::new(sources), Arc::new(mode));
        commands
            .into_iter()
            .map(|Entry { mut description, decode: inner }| {
                description.context.splice(0..0, global.iter().cloned());
                let (operation, sources, mode) = (Arc::clone(&operation), Arc::clone(&sources), Arc::clone(&mode));
                let decode: Decode<Handle, Decoder> = Arc::new(move |parsed, context, arguments, prefix| {
                    // This context's global arguments come first; the rest are the subtree's.
                    let (global, inner_context) = context.split_at(count);
                    let args = sources.cli_decode(global, parsed)?;
                    let command = inner(parsed, inner_context, arguments, prefix)?;
                    let (operation, mode, prefix) = (Arc::clone(&operation), Arc::clone(&mode), prefix.to_owned());
                    Ok(Box::new(move |handle| {
                        Box::pin(async move {
                            match outcome(mode.cli_outcome(operation.apply(handle, args).await), &prefix) {
                                Ok(context) => command(context).await,
                                Err(exit) => exit,
                            }
                        }) as Running
                    }) as Ready<Handle>)
                });
                Entry { description, decode }
            })
            .collect()
    }
}

/// Wakes the thread waiting on a future.
struct Unpark(Thread);

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// Waits for `future` on the calling thread.
fn block_on<Output>(future: impl Future<Output = Output>) -> Output {
    let mut future = core::pin::pin!(future);
    let waker = Waker::from(Arc::new(Unpark(thread::current())));
    let mut context = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return output;
        }
        thread::park();
    }
}
