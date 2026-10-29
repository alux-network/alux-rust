use crate::Conventions;
use alux_ext::ext;

/// Describes construction and composition of command collections.
///
/// Merge keeps declaration order and repetitions; which repetition wins is the interpreter's policy.
pub trait CliCommandsAlg {
    /// The interpreter's homogeneous command collection.
    type Commands;

    /// Returns the empty command collection.
    fn cli_empty(&self) -> Self::Commands;

    /// Combines two command collections, `left` first.
    fn cli_merge(&self, left: Self::Commands, right: Self::Commands) -> Self::Commands;
}

/// Compiles one command: a first-order operation under a name, with a source for each argument.
///
/// `Operation` is the reified extension method, so an interpreter reads its documentation and
/// argument names from `alux_ext::OperationAlg` and applies it through `alux_ext::ApplyAlg`.
/// `Inputs` is the ordered product of argument sources the declaration states, one per argument.
/// `Mode` is `Total` or `Fallible`, and reads the operation's output as the command's outcome
/// through `CliOutcomeOf`.
pub trait CliOperationAlg<Operation, Inputs, Mode> {
    /// The command collection one command compiles to.
    type Commands;

    /// Registers `operation` under `name`, decoded through `inputs` from words read under
    /// `conventions`, its output read under `mode`.
    fn cli_operation(
        &self,
        name: &'static str,
        operation: Operation,
        inputs: Inputs,
        mode: Mode,
        conventions: Conventions,
    ) -> Self::Commands;
}

/// Compiles a command collection beneath one subcommand name.
pub trait CliNestAlg {
    /// The command collection being nested.
    type Commands;

    /// Places `commands` beneath `name`.
    fn cli_nest(&self, name: &'static str, commands: Self::Commands) -> Self::Commands;

    /// Compiles a subtree with `compile` and places it beneath `name`.
    ///
    /// An interpreter that needs only the commands beneath some names compiles no other subtree.
    fn cli_nest_with(&self, name: &'static str, compile: impl FnOnce(&Self) -> Self::Commands) -> Self::Commands
    where
        Self: Sized,
    {
        self.cli_nest(name, compile(self))
    }
}

/// Compiles a subtree whose commands are applied to the context an operation answers.
///
/// `Operation` answers the context under `Mode`, from the arguments `Inputs` reads; those are the
/// subtree's global arguments. The subtree compiles with `Inner`, the interpreter for commands
/// applied to that context, so a context can be another type than the one around it.
pub trait CliContextAlg<Operation, Inputs, Mode> {
    /// The interpreter compiling the subtree.
    type Inner;
    /// The command collection the subtree compiles to.
    type InnerCommands;
    /// The command collection this compiles the subtree into.
    type Commands;

    /// Returns the interpreter compiling the subtree.
    fn cli_inner(&self) -> Self::Inner;

    /// Applies the subtree's `commands` to the context `operation` answers from `inputs`.
    fn cli_within(
        &self,
        operation: Operation,
        inputs: Inputs,
        mode: Mode,
        commands: Self::InnerCommands,
    ) -> Self::Commands;
}

/// Interprets a named, defunctionalized CLI program with `Compiler`.
pub trait CliProgramAlg<Compiler> {
    /// The command collection produced by `Compiler`.
    type Commands;

    /// Compiles the program through the supplied interpreter.
    fn compile_cli(self, compiler: &Compiler) -> Self::Commands;
}

/// Compiles defunctionalized CLI programs with an interpreter.
#[ext(name = CliProgramExt)]
pub impl<This> This {
    /// Compiles a named CLI program with this interpreter.
    fn compile_cli<Program>(&self, program: Program) -> Program::Commands
    where
        Program: CliProgramAlg<This>,
    {
        program.compile_cli(self)
    }
}
