use crate::{
    ClapWords,
    command::{assigned_long, command},
};
use alux_cli::{Arity, CliArgumentError, CliProgramAlg, CommandDescription, Conventions, SourceInfo, USAGE_STATUS};
use alux_cli_direct::{DirectCompiler, Entry, Exit, Running};
use clap::{ArgMatches, Command, error::ErrorKind};
use core::future::Future;
use std::{
    ffi::OsString,
    path::Path,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread::{self, Thread},
};

/// The status a run exits with when the program declares conventions `clap` does not read, as
/// `EX_SOFTWARE`: no command line could run it as declared.
pub const UNSUPPORTED_STATUS: u8 = 70;

/// Interprets a CLI program through `clap`, applying its operations to `Handle`.
///
/// `clap` parses the words and renders help and usage errors from a command built from the program;
/// the program decodes the arguments through their sources, applies the operation, and renders the
/// outcome as `alux-cli-direct` does.
pub struct CliClap<Handle> {
    handle: Handle,
    name: Option<String>,
    version: Option<&'static str>,
    entries: Vec<Entry<Handle, ClapWords>>,
}

impl<Handle> CliClap<Handle> {
    /// Compiles `program` to apply its operations to `handle`.
    pub fn new<Program>(handle: Handle, program: Program) -> Self
    where
        Program: CliProgramAlg<DirectCompiler<Handle, ClapWords>, Commands = Vec<Entry<Handle, ClapWords>>>,
    {
        Self { handle, name: None, version: None, entries: program.compile_cli(&DirectCompiler::default()) }
    }

    /// Names the program in help instead of the first word of the command line.
    #[must_use]
    pub fn named(self, name: impl Into<String>) -> Self {
        Self { name: Some(name.into()), ..self }
    }

    /// Answers `--version` or `-V` with the program's name and `version`, as `clap` does.
    #[must_use]
    pub fn version(self, version: &'static str) -> Self {
        Self { version: Some(version), ..self }
    }

    /// Builds the `clap` command the program declares, to parse with or to merge into another.
    ///
    /// It is named as `named` states, or `program`.
    pub fn command(&self) -> Command {
        self.command_named(self.name.clone().unwrap_or_else(|| "program".to_owned()))
    }

    fn command_named(&self, name: String) -> Command {
        let command = command(name, &self.descriptions(), &[]);
        match self.version {
            Some(version) => command.version(version),
            None => command,
        }
    }

    /// The commands the program declares.
    fn descriptions(&self) -> Vec<&CommandDescription> {
        self.entries.iter().map(Entry::description).collect()
    }
}

impl<Handle> CliClap<Handle>
where
    Handle: Clone,
{
    /// Runs the command `matches` names, when it is one of the program's.
    ///
    /// Answers nothing for matches the program declares no command for, so an application moving
    /// to the program one command at a time dispatches the rest itself.
    pub fn dispatch(&self, matches: &ArgMatches) -> Option<Running> {
        self.dispatch_named(self.command(), matches)
    }

    /// Runs the command `matches` names, reporting a usage error against `command`.
    fn dispatch_named(&self, command: Command, matches: &ArgMatches) -> Option<Running> {
        let mut path = Vec::new();
        let mut matches = matches;
        while let Some((name, beneath)) = matches.subcommand() {
            path.push(name.to_owned());
            matches = beneath;
        }
        let entry = self.entries.iter().find(|entry| entry.description().path == path)?;
        Some(match entry.decode(&ClapWords(matches.clone()), "error") {
            Ok(ready) => ready(self.handle.clone()),
            Err(error) => {
                let exit = usage(command, &path, &error);
                Box::pin(async move { exit })
            }
        })
    }

    /// Runs the command `matches` names, when it is one of the program's, and waits for its exit on
    /// the calling thread.
    pub fn dispatch_blocking(&self, matches: &ArgMatches) -> Option<Exit> {
        self.dispatch(matches).map(block_on)
    }

    /// Runs `program` on a full command line, its first word the program's name.
    pub fn run<Words>(&self, words: Words) -> Running
    where
        Words: IntoIterator,
        Words::Item: Into<OsString>,
    {
        let words = words.into_iter().map(Into::into).collect::<Vec<OsString>>();
        let invoked = words
            .first()
            .and_then(|first| Path::new(first).file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name = self.name.clone().unwrap_or(invoked);
        if let Some(exit) = self.unsupported() {
            return Box::pin(async move { exit });
        }
        let command = self.command_named(name);
        let words = self.assigned(words);
        let exit = match command.clone().try_get_matches_from(words) {
            Ok(matches) => {
                return self
                    .dispatch_named(command, &matches)
                    .unwrap_or_else(|| unreachable!("clap requires a command"));
            }
            Err(error) => {
                let text = error.render().to_string();
                let status = u8::try_from(error.exit_code()).unwrap_or(USAGE_STATUS.get());
                if error.use_stderr() {
                    Exit { status, stdout: String::new(), stderr: text }
                } else {
                    Exit { status, stdout: text, stderr: String::new() }
                }
            }
        };
        Box::pin(async move { exit })
    }

    /// Refuses a program declaring command-line conventions, which `clap` reads by its own.
    ///
    /// A command that is `verbatim`, reads options first, or gives a numeric option its digits would
    /// otherwise run with a different meaning than it declares; `alux-cli-direct` reads all of them.
    fn unsupported(&self) -> Option<Exit> {
        let declared =
            self.descriptions().into_iter().find(|description| description.conventions != Conventions::default())?;
        let command = declared.path.join(" ");
        let stderr = format!(
            "error: `{command}` declares command-line conventions alux-cli-clap does not read; alux-cli-direct reads them\n"
        );
        Some(Exit { status: UNSUPPORTED_STATUS, stdout: String::new(), stderr })
    }

    /// Rewrites each operand `name=value` assigning an argument of the selected command into the
    /// option `clap` reads it as.
    ///
    /// The command is selected by its path's words as they come. Only an operand is rewritten: a
    /// word another command's arguments name and a value an option takes are left as they are. An
    /// operand after `--` assigns too, so its option is moved before the `--`.
    fn assigned(&self, words: Vec<OsString>) -> Vec<OsString> {
        let descriptions = self.descriptions();
        if !descriptions.iter().flat_map(|description| &description.arguments).any(is_assigned) {
            return words;
        }
        let mut path: Vec<&str> = Vec::new();
        let mut takes_value = false;
        let mut ended = None;
        let mut rewritten = Vec::with_capacity(words.len());
        for (index, word) in words.into_iter().enumerate() {
            let text = word.to_str();
            if index == 0 || std::mem::take(&mut takes_value) {
                rewritten.push(word);
                continue;
            }
            let beneath = descriptions.iter().filter(|description| description.path.starts_with(&path));
            let segment = beneath.clone().find_map(|description| {
                description.path.get(path.len()).copied().filter(|segment| Some(*segment) == text)
            });
            let own = descriptions.iter().find(|description| description.path == path);
            // An argument the command line reaches here: the command's own, or a context's.
            let mut reachable = own
                .into_iter()
                .flat_map(|description| &description.arguments)
                .chain(beneath.flat_map(|description| &description.context));
            match text {
                Some("--") if ended.is_none() => ended = Some(rewritten.len()),
                Some(_) if ended.is_none() && segment.is_some() => path.extend(segment),
                Some(text) if ended.is_none() && text.starts_with('-') && !text.contains('=') => {
                    takes_value = reachable.any(|argument| takes_separate_value(argument, text));
                }
                Some(text) => {
                    let assigned = text.split_once('=').filter(|(name, _)| {
                        own.is_some_and(|own| own.arguments.iter().any(|argument| matches!(&argument.source, SourceInfo::Assigned { name: assigned } if assigned == name)))
                    });
                    if let Some((name, value)) = assigned {
                        let option = OsString::from(format!("--{}={value}", assigned_long(name)));
                        match ended {
                            Some(at) => {
                                rewritten.insert(at, option);
                                ended = Some(at + 1);
                            }
                            None => rewritten.push(option),
                        }
                        continue;
                    }
                }
                None => {}
            }
            rewritten.push(word);
        }
        rewritten
    }

    /// Runs `program` on a full command line and waits for its exit, on the calling thread.
    pub fn run_blocking<Words>(&self, words: Words) -> Exit
    where
        Words: IntoIterator,
        Words::Item: Into<OsString>,
    {
        block_on(self.run(words))
    }
}

/// Whether an argument is read from an operand `name=value`.
fn is_assigned(argument: &alux_cli::ArgumentInfo) -> bool {
    matches!(argument.source, SourceInfo::Assigned { .. })
}

/// Whether `word` spells `argument` as an option whose value is the next word.
fn takes_separate_value(argument: &alux_cli::ArgumentInfo, word: &str) -> bool {
    let SourceInfo::Named { long, short, short_aliases } = &argument.source else { return false };
    if matches!(argument.arity, Arity::Presence | Arity::OptionalValue) {
        return false;
    }
    match word.strip_prefix("--") {
        Some(spelled) => spelled == long,
        // A short option takes the next word when it ends its cluster.
        None => {
            word.chars().last().is_some_and(|last| short.iter().chain(short_aliases.iter()).any(|short| *short == last))
        }
    }
}

/// Renders an argument the words do not make as `clap` renders a usage error for the command.
fn usage(mut command: Command, path: &[String], error: &CliArgumentError<String>) -> Exit {
    // Building names each subcommand by its whole path, as its usage shows it.
    command.build();
    let mut current = &mut command;
    for segment in path {
        current = current.find_subcommand_mut(segment).expect("the command was matched by its path");
    }
    // An argument is named as the command line spells its value.
    let spelled = |name: &str| name.replace('_', "-");
    let (kind, message) = match error {
        CliArgumentError::Missing(name) => (ErrorKind::MissingRequiredArgument, format!("missing <{}>", spelled(name))),
        CliArgumentError::Repeated(name) => {
            (ErrorKind::ArgumentConflict, format!("<{}> given more than once", spelled(name)))
        }
        CliArgumentError::Word(name, reason) => {
            (ErrorKind::ValueValidation, format!("invalid <{}>: {reason}", spelled(name)))
        }
        CliArgumentError::Conflict(first, second) => (
            ErrorKind::ArgumentConflict,
            format!("<{}> and <{}> cannot be given together", spelled(first), spelled(second)),
        ),
    };
    let error = current.error(kind, message);
    Exit { status: USAGE_STATUS.get(), stdout: String::new(), stderr: error.render().to_string() }
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

/// Wakes the thread waiting on a future.
struct Unpark(Thread);

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}
