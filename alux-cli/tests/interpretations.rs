//! Interprets one CLI program as a description and as execution, and checks the laws on both.

use alux_cli::{
    ArgumentInfo, Arity, CliArgumentError, CliCommandsAlg, CliCommandsObserved, CliCompositionLawsExt, CliContextAlg,
    CliContextLawsExt, CliDeclarationLawsExt, CliDecodeSources, CliFailureAlg, CliNestAlg, CliNestLawsExt,
    CliOperationAlg, CliOutcome, CliOutcomeLawsExt, CliOutcomeOf, CliProgramBuilder, CliProgramExt, CliSources,
    CliWordAlg, CliWordsAlg, CommandDescription, CompileCliProgram, Conventions, Fallback, Fallible, SourceInfo, Total,
    USAGE_STATUS, cli,
};
use alux_ext::{ApplyAlg, OperationAlg, ext};
use core::{
    fmt,
    future::Future,
    num::NonZeroU8,
    pin::pin,
    str::FromStr,
    task::{Context, Poll, Waker},
};
use std::sync::Arc;

trait Reading {
    type Name;
    type Value;

    fn read(&self, name: &Self::Name) -> Self::Value;
    fn find(&self, name: &Self::Name) -> Option<Self::Value>;
    fn total(&self) -> Self::Value;
}

/// A name that denotes no value.
#[derive(Debug)]
struct Unknown;

impl fmt::Display for Unknown {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("no such name")
    }
}

impl CliFailureAlg for Unknown {
    fn cli_status(&self) -> NonZeroU8 {
        NonZeroU8::new(3).unwrap()
    }
}

#[ext(name = ReadingExt, defunc)]
impl<This, Name, Value> This
where
    This: Reading<Name = Name, Value = Value>,
    Name: Send + ToString,
{
    /// Reads the value denoted by the requested name.
    fn lookup(&self, name: Name) -> Value {
        self.read(&name)
    }

    /// Finds the value a name denotes, failing for a name that denotes none.
    fn find_value(&self, name: Name) -> Result<Value, Unknown> {
        self.find(&name).ok_or(Unknown)
    }

    /// Sums every value.
    fn sum(&self) -> Value {
        self.total()
    }

    /// Lists names from one, at most a limit of them, with any tags, verbosely or not.
    fn page(&self, from: Name, limit: Option<u8>, tags: Vec<String>, verbose: bool) -> String {
        format!("{} {limit:?} {tags:?} {verbose}", from.to_string())
    }
}

#[ext(name = ReadingCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    /// Declares the reading commands.
    fn reading_cli<Machine, Name>(&self) {
        self.commands()
            .command("lookup", self.op(Machine::lookup).arg(self.positional("Requested name")))
            .command("find", self.op(Machine::find_value).arg(self.positional("Requested name")).fallible())
            .command("sum", self.op(Machine::sum))
            .command(
                "page",
                self.op(Machine::page)
                    .arg(self.positional("First name"))
                    .arg(self.named("Most names").optional().short('n'))
                    .arg(self.named("Tag to match").many().long("tag"))
                    .arg(self.named("Verbose output").switch().short('v')),
            )
    }

    /// Declares `page` with every argument falling back to the environment or a default.
    fn defaults_cli<Machine>(&self)
    where
        This: CliCommandsAlg,
    {
        self.commands().command(
            "page",
            self.op(Machine::page)
                .arg(self.positional("First name").default("1"))
                .arg(self.named("Most names").optional().env("READING_LIMIT").default("9"))
                .arg(self.named("Tag to match").many().long("tag").env("READING_TAG").default("all"))
                .arg(self.named("Verbose output").switch().short('v')),
        )
    }

    /// Declares the reading commands within a context its `--default-name` opens.
    fn context_cli<Machine, Name>(&self)
    where
        This: CliCommandsAlg,
    {
        self.commands().within(
            self.op(Machine::lookup).arg(self.named("Default name").short('d')),
            self.reading_cli::<Machine, Name>(),
        )
    }

    /// Declares the application: the reading commands, and the same beneath `admin`.
    fn app_cli<Machine, Name>(&self) {
        self.commands().merge(self.reading_cli::<Machine, Name>()).nest("admin", self.reading_cli::<Machine, Name>())
    }
}

struct Memory;

impl Reading for Memory {
    type Name = u8;
    type Value = u16;

    fn read(&self, name: &u8) -> u16 {
        u16::from(*name) * 10
    }

    fn find(&self, name: &u8) -> Option<u16> {
        (*name < 10).then(|| u16::from(*name) * 10)
    }

    fn total(&self) -> u16 {
        99
    }
}

/// Describes commands without applying any.
#[derive(Clone, Copy)]
struct Describe;

impl CliCommandsAlg for Describe {
    type Commands = Vec<CommandDescription>;

    fn cli_empty(&self) -> Self::Commands {
        Vec::new()
    }

    fn cli_merge(&self, mut left: Self::Commands, right: Self::Commands) -> Self::Commands {
        left.extend(right);
        left
    }
}

impl<Operation, Sources, Mode> CliOperationAlg<Operation, Sources, Mode> for Describe
where
    Operation: OperationAlg,
    Sources: CliSources<Operation::Args>,
{
    type Commands = Vec<CommandDescription>;

    fn cli_operation(
        &self,
        name: &'static str,
        _: Operation,
        sources: Sources,
        _: Mode,
        conventions: Conventions,
    ) -> Self::Commands {
        let mut description = CommandDescription::of::<Operation, Sources>(name, &sources);
        description.conventions = conventions;
        vec![description]
    }
}

impl<Operation, Sources, Mode> CliContextAlg<Operation, Sources, Mode> for Describe
where
    Operation: OperationAlg,
    Sources: CliSources<Operation::Args>,
{
    type Inner = Describe;
    type InnerCommands = Vec<CommandDescription>;
    type Commands = Vec<CommandDescription>;

    fn cli_inner(&self) -> Describe {
        Describe
    }

    fn cli_within(&self, _: Operation, sources: Sources, _: Mode, mut commands: Self::Commands) -> Self::Commands {
        let global = sources.cli_arguments(Operation::ARG_NAMES);
        for command in &mut commands {
            command.context.splice(0..0, global.iter().cloned());
        }
        commands
    }
}

impl CliNestAlg for Describe {
    type Commands = Vec<CommandDescription>;

    fn cli_nest(&self, name: &'static str, mut commands: Self::Commands) -> Self::Commands {
        for command in &mut commands {
            command.path.insert(0, name);
        }
        commands
    }
}

impl CliCommandsObserved for Describe {
    type Commands = Vec<CommandDescription>;

    fn commands_observed(&self, commands: &Self::Commands) -> Vec<CommandDescription> {
        commands.clone()
    }
}

/// Runs a command against `Memory` from the words after its path.
type Run = Arc<dyn Fn(&[&str]) -> Observed>;

/// What one run of a command shows: its exit status and what it wrote to each stream.
#[derive(Debug, PartialEq, Eq)]
struct Observed {
    status: u8,
    out: String,
    err: String,
}

/// Executes commands against one shared context.
struct Execute(Arc<Memory>);

impl CliCommandsAlg for Execute {
    type Commands = Vec<(Vec<&'static str>, Run)>;

    fn cli_empty(&self) -> Self::Commands {
        Vec::new()
    }

    fn cli_merge(&self, mut left: Self::Commands, right: Self::Commands) -> Self::Commands {
        left.extend(right);
        left
    }
}

impl CliNestAlg for Execute {
    type Commands = Vec<(Vec<&'static str>, Run)>;

    fn cli_nest(&self, name: &'static str, mut commands: Self::Commands) -> Self::Commands {
        for (path, _) in &mut commands {
            path.insert(0, name);
        }
        commands
    }
}

/// The environment every run sees.
const ENVIRONMENT: &[(&str, &str)] = &[("READING_LIMIT", "5")];

/// Words split into positionals and `--long[=value]` or `-s[=value]` occurrences.
struct Parsed<'a> {
    positional: Vec<&'a str>,
    named: Vec<(&'a str, &'a str)>,
}

impl<'a> Parsed<'a> {
    fn new(words: &[&'a str]) -> Self {
        let mut parsed = Parsed { positional: Vec::new(), named: Vec::new() };
        for word in words {
            match word.strip_prefix("--").or_else(|| word.strip_prefix('-')) {
                Some(spelled) => parsed.named.push(spelled.split_once('=').unwrap_or((spelled, ""))),
                None => parsed.positional.push(word),
            }
        }
        parsed
    }
}

impl CliWordsAlg for Parsed<'_> {
    type Word = String;

    fn cli_words(&self, argument: &ArgumentInfo) -> Vec<String> {
        let words: Vec<&str> = match &argument.source {
            SourceInfo::Positional { index } if argument.arity == Arity::Many => {
                self.positional.get(*index..).unwrap_or_default().to_vec()
            }
            SourceInfo::Positional { index } => self.positional.get(*index).copied().into_iter().collect(),
            SourceInfo::Named { long, short, .. } => self
                .named
                .iter()
                .filter(|(spelled, _)| *spelled == long || short.is_some_and(|short| spelled.chars().eq([short])))
                .map(|(_, value)| *value)
                .collect(),
            // This witness reads options and positions only.
            SourceInfo::Assigned { .. } => Vec::new(),
        };
        words.into_iter().map(str::to_owned).collect()
    }

    fn cli_env(&self, name: &str) -> Option<String> {
        ENVIRONMENT.iter().find(|(variable, _)| *variable == name).map(|(_, value)| (*value).to_owned())
    }

    fn cli_default(&self, default: &'static str) -> String {
        default.to_owned()
    }

    fn cli_split(&self, word: String, delimiter: char) -> Vec<String> {
        word.split(delimiter).map(str::to_owned).collect()
    }
}

impl<Value> CliWordAlg<Value, String> for Parsed<'_>
where
    Value: FromStr,
{
    type Error = String;

    fn cli_word(&self, word: &String) -> Result<Value, String> {
        // A switch's own spelling is an empty word, and it is on.
        let word = if word.is_empty() { "true" } else { word };
        word.parse().map_err(|_| format!("`{word}` is not a value"))
    }
}

impl<Operation, Sources, Mode> CliOperationAlg<Operation, Sources, Mode> for Execute
where
    Operation: OperationAlg + ApplyAlg<Arc<Memory>, Operation::Args> + 'static,
    Sources: CliSources<Operation::Args> + for<'a> CliDecodeSources<Operation::Args, Parsed<'a>, String> + 'static,
    Mode: CliOutcomeOf<Operation::Output> + 'static,
    Mode::Value: ToString,
    Mode::Error: CliFailureAlg + ToString,
{
    type Commands = Vec<(Vec<&'static str>, Run)>;

    fn cli_operation(
        &self,
        name: &'static str,
        operation: Operation,
        sources: Sources,
        mode: Mode,
        _: Conventions,
    ) -> Self::Commands {
        let context = Arc::clone(&self.0);
        // Sources decode the arguments they are described as.
        let arguments = sources.cli_arguments(Operation::ARG_NAMES);
        let run: Run = Arc::new(move |words| {
            // Words that do not make the command are a usage error, and apply nothing.
            let args = match sources.cli_decode(&arguments, &Parsed::new(words)) {
                Ok(args) => args,
                Err(error) => {
                    return Observed { status: USAGE_STATUS.get(), out: String::new(), err: format!("{error:?}") };
                }
            };
            let outcome = mode.cli_outcome(ready(operation.apply(Arc::clone(&context), args)));
            let status = outcome.cli_status();
            match outcome {
                CliOutcome::Success(value) => Observed { status, out: value.to_string(), err: String::new() },
                CliOutcome::Failure(error) => Observed { status, out: String::new(), err: error.to_string() },
            }
        });
        vec![(vec![name], run)]
    }
}

/// Answers a future that a synchronous extension method completes on its first poll.
fn ready<Output>(future: impl Future<Output = Output>) -> Output {
    let mut future = pin!(future);
    match future.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(output) => output,
        Poll::Pending => unreachable!("a synchronous operation answered later"),
    }
}

fn run(commands: &[(Vec<&'static str>, Run)], path: &[&str], words: &[&str]) -> Observed {
    let (_, run) = commands.iter().find(|(declared, _)| declared == path).expect("the command is declared");
    run(words)
}

#[test]
fn describes_the_program_with_paths_documentation_and_roles() {
    let described = Describe.compile_cli(CliProgramBuilder.app_cli::<Memory, u8>());
    let paths = described.iter().map(|command| command.path.clone()).collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            vec!["lookup"],
            vec!["find"],
            vec!["sum"],
            vec!["page"],
            vec!["admin", "lookup"],
            vec!["admin", "find"],
            vec!["admin", "sum"],
            vec!["admin", "page"]
        ]
    );
    assert_eq!(described[0].doc, "Reads the value denoted by the requested name.");
    assert_eq!(described[0].argument_names, ["name"]);
    assert!(described[2].arguments.is_empty());
    // Every argument is the operation's, in its order, and its arity is what its type says.
    let page = &described[3].arguments;
    assert_eq!(page.iter().map(|argument| argument.name).collect::<Vec<_>>(), ["from", "limit", "tags", "verbose"]);
    assert_eq!(
        page.iter().map(|argument| argument.arity).collect::<Vec<_>>(),
        [Arity::One, Arity::Optional, Arity::Many, Arity::Presence]
    );
    assert_eq!(page[0].source, SourceInfo::Positional { index: 0 });
    assert_eq!(page[1].source, SourceInfo::Named { long: "limit".into(), short: Some('n'), short_aliases: &[] });
    assert_eq!(page[2].source, SourceInfo::Named { long: "tag".into(), short: None, short_aliases: &[] });
}

#[test]
fn executes_the_same_program_by_applying_its_operations() {
    let commands = Execute(Arc::new(Memory)).compile_cli(CliProgramBuilder.app_cli::<Memory, u8>());
    let succeeded = |out: &str| Observed { status: 0, out: out.into(), err: String::new() };
    let usage = |err: CliArgumentError<String>| Observed { status: 2, out: String::new(), err: format!("{err:?}") };
    assert_eq!(run(&commands, &["lookup"], &["7"]), succeeded("70"));
    assert_eq!(run(&commands, &["admin", "sum"], &[]), succeeded("99"));
    assert_eq!(
        run(&commands, &["page"], &["3", "-n=2", "--tag=a", "--tag=b", "-v"]),
        succeeded("3 Some(2) [\"a\", \"b\"] true")
    );
    assert_eq!(run(&commands, &["page"], &["3"]), succeeded("3 None [] false"));
    // A fallible command exits with its failure's own status, and writes the failure, not a value.
    assert_eq!(run(&commands, &["find"], &["4"]), succeeded("40"));
    assert_eq!(
        run(&commands, &["find"], &["40"]),
        Observed { status: 3, out: String::new(), err: "no such name".into() }
    );
    // Words that do not make the command exit with the usage status before anything applies.
    assert_eq!(run(&commands, &["lookup"], &["x"]), usage(CliArgumentError::Word("name", "`x` is not a value".into())));
    assert_eq!(run(&commands, &["lookup"], &[]), usage(CliArgumentError::Missing("name")));
    assert_eq!(run(&commands, &["page"], &["3", "-n=1", "--limit=2"]), usage(CliArgumentError::Repeated("limit")));
}

#[test]
fn direct_construction_folds_like_the_lowered_declaration() {
    let builder = CliProgramBuilder;
    let direct = builder
        .commands()
        .command("lookup", builder.op(LookupOperation::<Memory>::default()).arg(builder.positional("Requested name")))
        .command(
            "find",
            builder.op(FindValueOperation::<Memory>::default()).arg(builder.positional("Requested name")).fallible(),
        )
        .command("sum", builder.op(SumOperation::<Memory>::default()))
        .command(
            "page",
            builder
                .op(PageOperation::<Memory>::default())
                .arg(builder.positional("First name"))
                .arg(builder.named("Most names").optional().short('n'))
                .arg(builder.named("Tag to match").many().long("tag"))
                .arg(builder.named("Verbose output").switch().short('v')),
        );
    let lowered = Describe.compile_cli(CliProgramBuilder.reading_cli::<Memory, u8>());
    assert_eq!(direct.into_program().compile_cli_program(&Describe), lowered);
}

#[test]
fn description_satisfies_the_laws() {
    let reading = Describe.compile_cli(CliProgramBuilder.reading_cli::<Memory, u8>());
    let app = Describe.compile_cli(CliProgramBuilder.app_cli::<Memory, u8>());
    Describe.law_cli_identity(&app).unwrap();
    Describe.law_cli_merge_conservation(&reading, &app).unwrap();
    Describe.law_cli_associativity(&reading, &app, &reading).unwrap();
    Describe.law_cli_nest("admin", &app).unwrap();
    Describe
        .law_cli_declaration(
            "lookup",
            LookupOperation::<Memory>::default(),
            ((), CliProgramBuilder.positional("Requested name")),
            Total,
            Conventions::default(),
        )
        .unwrap();
    Total.law_cli_outcome(7_u16).unwrap();
    Fallible.law_cli_outcome(Ok::<u16, Unknown>(7)).unwrap();
    Fallible.law_cli_outcome(Err::<u16, Unknown>(Unknown)).unwrap();
    // Two arguments read from one spelling cannot be told apart.
    assert_eq!(
        Describe.law_cli_declaration(
            "page",
            PageOperation::<Memory>::default(),
            (
                (
                    (((), CliProgramBuilder.positional("First")), CliProgramBuilder.named("Most").long("x")),
                    CliProgramBuilder.named("Tag").long("x"),
                ),
                CliProgramBuilder.named("Verbose").switch(),
            ),
            Total,
            Conventions::default(),
        ),
        Err("two CLI arguments share a source")
    );
}

#[test]
fn falls_back_to_the_environment_then_the_default() {
    let commands = Execute(Arc::new(Memory)).compile_cli(CliProgramBuilder.defaults_cli::<Memory>());
    let succeeded = |out: &str| Observed { status: 0, out: out.into(), err: String::new() };
    // `from` and `tags` take their defaults, and `limit` the environment over its default.
    assert_eq!(run(&commands, &["page"], &[]), succeeded("1 Some(5) [\"all\"] false"));
    // The command line wins over both.
    assert_eq!(run(&commands, &["page"], &["2", "--limit=3", "--tag=x"]), succeeded("2 Some(3) [\"x\"] false"));
    let described = Describe.compile_cli(CliProgramBuilder.defaults_cli::<Memory>());
    assert_eq!(described[0].arguments[1].fallback, Fallback { env: Some("READING_LIMIT"), default: Some("9") });
}

#[test]
fn a_context_adds_its_global_arguments_to_every_command() {
    let app = Describe.compile_cli(CliProgramBuilder.app_cli::<Memory, u8>());
    Describe.law_cli_within(SumOperation::<Memory>::default(), (), Total, &app).unwrap();
    Describe
        .law_cli_within(
            LookupOperation::<Memory>::default(),
            ((), CliProgramBuilder.named("Default name").short('d')),
            Total,
            &app,
        )
        .unwrap();
}

#[test]
fn a_lowered_context_describes_like_the_context_built_directly() {
    let reading = Describe.compile_cli(CliProgramBuilder.reading_cli::<Memory, u8>());
    let lowered = Describe.compile_cli(CliProgramBuilder.context_cli::<Memory, u8>());
    let sources = ((), CliProgramBuilder.named("Default name").short('d'));
    assert_eq!(lowered, Describe.cli_within(LookupOperation::<Memory>::default(), sources, Total, reading.clone()));
    assert!(lowered.iter().all(|command| command.context.len() == 1 && command.context[0].name == "name"));
    Describe.law_cli_within(LookupOperation::<Memory>::default(), sources, Total, &reading).unwrap();
}

#[test]
fn a_compound_argument_is_one_operation_argument() {
    #[derive(alux_shape::Shape)]
    struct Pair {
        first: u8,
        second: String,
    }
    trait Pairs {
        fn pair(&self, pair: Pair, verbose: bool) -> u8;
    }
    impl Pairs for Memory {
        fn pair(&self, _: Pair, _: bool) -> u8 {
            0
        }
    }
    #[ext(name = PairsExt, defunc)]
    impl<This> This
    where
        This: Pairs,
    {
        /// Takes a pair.
        fn take_pair(&self, pair: Pair, verbose: bool) -> u8 {
            self.pair(pair, verbose)
        }
    }
    let cli = CliProgramBuilder;
    Describe
        .law_cli_declaration(
            "pair",
            TakePairOperation::<Memory>::default(),
            (((), cli.product().field(cli.named("First")).field(cli.named("Second"))), cli.named("Verbose").switch()),
            Total,
            Conventions::default(),
        )
        .unwrap();
}
