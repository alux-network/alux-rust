use alux_cli::{CliCommandsAlg, CliContextAlg, CliNestAlg, CliOperationAlg, CliProgramAlg, Conventions};
use alux_ext::OperationAlg;
use std::{ffi::OsString, path::Path};

/// Compiles a CLI program into the paths of its commands, which is all multicall reads.
#[derive(Clone, Copy, Debug, Default)]
pub struct MulticallCompiler;

/// The commands of one program, as their paths, read for the names the executable answers to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Multicall {
    paths: Vec<Vec<&'static str>>,
}

/// A command line as a multicall executable reads it: the program name its help shows, and the
/// words to run, the first of them the program's own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MulticallInvocation {
    /// The name help shows for the program: empty when the executable was invoked by a command's
    /// name, so help shows that command beneath no program name.
    pub name: String,
    /// The full command line to run.
    pub words: Vec<OsString>,
    /// The first-layer command the words name, invoked by its name or written after the
    /// executable's; only its commands need compiling to run them.
    pub command: Option<&'static str>,
}

impl Multicall {
    /// Reads the commands `program` declares.
    pub fn new<Program>(program: Program) -> Self
    where
        Program: CliProgramAlg<MulticallCompiler, Commands = Vec<Vec<&'static str>>>,
    {
        Self { paths: program.compile_cli(&MulticallCompiler) }
    }

    /// The names the executable answers to: each first-layer command's, once, in declaration order.
    ///
    /// An installer links each of them to the executable.
    pub fn names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        // The program's own command has no name to be invoked by.
        for name in self.paths.iter().filter_map(|path| path.first()).filter(|name| !name.is_empty()) {
            if !names.contains(name) {
                names.push(*name);
            }
        }
        names
    }

    /// Reads a command line: invoked by a first-layer command's name, that name is also its first
    /// word; invoked under any other name, the words that follow select as usual.
    pub fn invocation<Words>(&self, words: Words) -> MulticallInvocation
    where
        Words: IntoIterator,
        Words::Item: Into<OsString>,
    {
        let mut words = words.into_iter().map(Into::into).collect::<Vec<OsString>>();
        let names = self.names();
        let invoked_as = |word: Option<&OsString>| {
            let word = word.and_then(|word| Path::new(word).file_name()).and_then(|name| name.to_str())?;
            names.iter().copied().find(|name| *name == word)
        };
        if let Some(command) = invoked_as(words.first()) {
            words.insert(1, OsString::from(command));
            return MulticallInvocation { name: String::new(), words, command: Some(command) };
        }
        let invoked = words.first().and_then(|first| Path::new(first).file_name()).and_then(|name| name.to_str());
        let name = invoked.unwrap_or_default().to_owned();
        // Beneath the executable's own name, the next word names the command, unless it is an option.
        let command = words
            .get(1)
            .filter(|word| !word.as_encoded_bytes().starts_with(b"-"))
            .and_then(|word| names.iter().copied().find(|name| word.to_str() == Some(*name)));
        MulticallInvocation { name, words, command }
    }
}

impl CliCommandsAlg for MulticallCompiler {
    type Commands = Vec<Vec<&'static str>>;

    fn cli_empty(&self) -> Self::Commands {
        Vec::new()
    }

    fn cli_merge(&self, mut left: Self::Commands, right: Self::Commands) -> Self::Commands {
        left.extend(right);
        left
    }
}

impl CliNestAlg for MulticallCompiler {
    type Commands = Vec<Vec<&'static str>>;

    fn cli_nest(&self, name: &'static str, mut commands: Self::Commands) -> Self::Commands {
        for path in &mut commands {
            path.insert(0, name);
        }
        commands
    }
}

impl<Operation, Inputs, Mode> CliOperationAlg<Operation, Inputs, Mode> for MulticallCompiler
where
    Operation: OperationAlg,
{
    type Commands = Vec<Vec<&'static str>>;

    fn cli_operation(&self, name: &'static str, _: Operation, _: Inputs, _: Mode, _: Conventions) -> Self::Commands {
        vec![if name.is_empty() { Vec::new() } else { vec![name] }]
    }
}

/// A context adds global arguments and changes no path.
impl<Operation, Inputs, Mode> CliContextAlg<Operation, Inputs, Mode> for MulticallCompiler {
    type Inner = Self;
    type InnerCommands = Vec<Vec<&'static str>>;
    type Commands = Vec<Vec<&'static str>>;

    fn cli_inner(&self) -> Self {
        Self
    }

    fn cli_within(&self, _: Operation, _: Inputs, _: Mode, commands: Self::InnerCommands) -> Self::Commands {
        commands
    }
}
