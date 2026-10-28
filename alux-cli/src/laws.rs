use crate::{
    CliCommandsAlg, CliContextAlg, CliFailureAlg, CliNamedSources, CliNestAlg, CliOperationAlg, CliOutcome,
    CliOutcomeOf, CliSources, CommandDescription, Conventions, Part,
};
use alux_ext::{OperationAlg, ext};

/// Observes a command collection as the commands it describes, in order.
pub trait CliCommandsObserved {
    /// The collection observed.
    type Commands;

    /// Returns every command the collection describes, in order.
    fn commands_observed(&self, commands: &Self::Commands) -> Vec<CommandDescription>;
}

/// Reports a broken CLI law by the obligation it breaks.
pub type CliLawResult = Result<(), &'static str>;

fn require(condition: bool, message: &'static str) -> CliLawResult {
    if condition { Ok(()) } else { Err(message) }
}

/// States that command collections form an ordered monoid under merge.
#[ext(name = CliCompositionLawsExt)]
pub impl<This, Commands> This
where
    This: CliCommandsAlg<Commands = Commands> + CliCommandsObserved<Commands = Commands>,
    Commands: Clone,
{
    /// Checks that the empty collection describes nothing and is an identity of merge.
    fn law_cli_identity(&self, commands: &Commands) -> CliLawResult {
        let observed = self.commands_observed(commands);
        require(self.commands_observed(&self.cli_empty()).is_empty(), "empty CLI was not empty")?;
        require(
            self.commands_observed(&self.cli_merge(self.cli_empty(), commands.clone())) == observed,
            "left CLI identity failed",
        )?;
        require(
            self.commands_observed(&self.cli_merge(commands.clone(), self.cli_empty())) == observed,
            "right CLI identity failed",
        )
    }

    /// Checks that merge describes `left` then `right`, losing and reordering nothing.
    fn law_cli_merge_conservation(&self, left: &Commands, right: &Commands) -> CliLawResult {
        let mut expected = self.commands_observed(left);
        expected.extend(self.commands_observed(right));
        require(
            self.commands_observed(&self.cli_merge(left.clone(), right.clone())) == expected,
            "CLI merge lost order or declarations",
        )
    }

    /// Checks that merge is associative.
    fn law_cli_associativity(&self, a: &Commands, b: &Commands, c: &Commands) -> CliLawResult {
        let left = self.cli_merge(self.cli_merge(a.clone(), b.clone()), c.clone());
        let right = self.cli_merge(a.clone(), self.cli_merge(b.clone(), c.clone()));
        require(self.commands_observed(&left) == self.commands_observed(&right), "CLI merge was not associative")
    }
}

/// States that declaring a command preserves its operation and gives every argument one source.
#[ext(name = CliDeclarationLawsExt)]
pub impl<This> This {
    /// Checks that `operation` declared under `name` with `sources` describes exactly that, with
    /// one argument per operation argument, in order, and distinct spellings and positions.
    fn law_cli_declaration<Operation, Sources, Mode, Commands>(
        &self,
        name: &'static str,
        operation: Operation,
        sources: Sources,
        mode: Mode,
        conventions: Conventions,
    ) -> CliLawResult
    where
        This: CliOperationAlg<Operation, Sources, Mode, Commands = Commands> + CliCommandsObserved<Commands = Commands>,
        Operation: OperationAlg,
        Sources: CliSources<Operation::Args>,
    {
        let mut expected = CommandDescription::of::<Operation, Sources>(name, &sources);
        expected.conventions = conventions;
        // A compound argument describes one part each; read back, its parts are one argument.
        let mut names = expected
            .arguments
            .iter()
            .map(|argument| match argument.part_of {
                Some(Part::Product(name) | Part::Sum(name)) => name,
                None => argument.name,
            })
            .collect::<Vec<_>>();
        names.dedup();
        require(names == Operation::ARG_NAMES, "CLI arguments are not the operation's arguments in order")?;
        let sources_described = expected.arguments.iter().map(|argument| &argument.source).collect::<Vec<_>>();
        require(
            sources_described.iter().enumerate().all(|(index, source)| !sources_described[..index].contains(source)),
            "two CLI arguments share a source",
        )?;
        let declared = self.cli_operation(name, operation, sources, mode, conventions);
        require(
            self.commands_observed(&declared) == vec![expected],
            "CLI declaration changed its operation or arguments",
        )
    }
}

/// States that nesting prefixes every path and changes nothing else.
#[ext(name = CliNestLawsExt)]
pub impl<This, Commands> This
where
    This: CliNestAlg<Commands = Commands> + CliCommandsObserved<Commands = Commands>,
    Commands: Clone,
{
    /// Checks that `commands` nested under `name` describe the same commands beneath `name`.
    fn law_cli_nest(&self, name: &'static str, commands: &Commands) -> CliLawResult {
        let mut expected = self.commands_observed(commands);
        for description in &mut expected {
            description.path.insert(0, name);
        }
        require(
            self.commands_observed(&self.cli_nest(name, commands.clone())) == expected,
            "subcommand nesting changed a declaration",
        )
    }
}

/// States that a command's outcome and its exit status agree.
#[ext(name = CliOutcomeLawsExt)]
pub impl<This> This {
    /// Checks that `output` read under this mode exits with zero exactly when it succeeds.
    fn law_cli_outcome<Output>(&self, output: Output) -> CliLawResult
    where
        This: CliOutcomeOf<Output>,
        This::Error: CliFailureAlg,
    {
        let outcome = self.cli_outcome(output);
        let succeeded = matches!(outcome, CliOutcome::Success(_));
        require((outcome.cli_status() == 0) == succeeded, "a command's exit status disagreed with its outcome")
    }
}

/// States that a context adds its global arguments to every command beneath it and changes nothing
/// else.
#[ext(name = CliContextLawsExt)]
pub impl<This, Commands> This
where
    This: CliCommandsObserved<Commands = Commands>,
    Commands: Clone,
{
    /// Checks that `commands` applied within `operation` describe the same commands, each taking the
    /// operation's arguments, read from `sources`, ahead of any context it already had.
    fn law_cli_within<Operation, Sources, Mode>(
        &self,
        operation: Operation,
        sources: Sources,
        mode: Mode,
        commands: &Commands,
    ) -> CliLawResult
    where
        This: CliContextAlg<Operation, Sources, Mode, InnerCommands = Commands, Commands = Commands>,
        Operation: OperationAlg,
        Sources: CliSources<Operation::Args> + CliNamedSources,
    {
        let global = sources.cli_arguments(Operation::ARG_NAMES);
        let mut expected = self.commands_observed(commands);
        for description in &mut expected {
            description.context.splice(0..0, global.iter().cloned());
        }
        let within = self.cli_within(operation, sources, mode, commands.clone());
        require(self.commands_observed(&within) == expected, "a context changed the commands beneath it")
    }
}
