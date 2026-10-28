use crate::{
    CliCommandsAlg, CliContextAlg, CliNamedSources, CliNestAlg, CliOperationAlg, CliProgramAlg, CliSources,
    Conventions, Fallible, NamedSource, PositionalSource, ProductSource, SumSource, Total,
};
use alux_ext::OperationAlg;

/// Compiles a first-order CLI program with a concrete interpreter.
pub trait CompileCliProgram<Compiler> {
    /// The command collection produced by `Compiler`.
    type Commands;

    /// Folds the complete first-order program through `compiler`.
    fn compile_cli_program(self, compiler: &Compiler) -> Self::Commands;
}

/// Represents the empty CLI program.
#[derive(Debug, Default)]
pub struct Empty;

/// Represents the composition of two CLI programs, `left` first.
#[derive(Debug)]
pub struct Merge<Left, Right> {
    left: Left,
    right: Right,
}

/// Includes a separately named CLI program.
#[derive(Debug)]
pub struct Named<Program>(Program);

/// Places a CLI program beneath one subcommand name.
#[derive(Debug)]
pub struct Nest<Program> {
    name: &'static str,
    program: Program,
}

/// Applies every command of a program to the context an operation answers.
///
/// The operation's arguments are the subtree's global arguments: every command of `program` takes
/// them besides its own.
#[derive(Debug)]
pub struct Within<Handler, Inputs, Mode, Program> {
    handler: Handler,
    inputs: Inputs,
    mode: Mode,
    program: Program,
}

/// Represents one named command without choosing an interpreter.
#[derive(Debug)]
pub struct Command<Handler, Inputs, Mode = Total> {
    name: &'static str,
    handler: Handler,
    inputs: Inputs,
    mode: Mode,
    conventions: Conventions,
}

/// Carries a typed operation, the sources of its arguments (one per argument, in order), and how
/// its output is read as the command's outcome.
#[derive(Debug)]
pub struct Operation<Handler, Inputs = (), Mode = Total> {
    handler: Handler,
    inputs: Inputs,
    mode: Mode,
    conventions: Conventions,
}

/// Carries a typed CLI program during fluent composition.
#[derive(Debug)]
pub struct CliProgram<Program>(Program);

/// Constructs neutral CLI programs.
#[derive(Debug, Default)]
pub struct CliProgramBuilder;

impl CliProgramBuilder {
    /// Starts an empty, uninterpreted CLI program.
    pub fn commands(&self) -> CliProgram<Empty> {
        CliProgram(Empty)
    }

    /// Wraps a first-order handler operation in a neutral declaration with no arguments.
    pub fn op<Handler>(&self, handler: Handler) -> Operation<Handler> {
        Operation { handler, inputs: (), mode: Total, conventions: Conventions::default() }
    }

    /// Includes a named CLI program in another program.
    pub fn program<Program>(&self, program: Program) -> CliProgram<Named<Program>> {
        CliProgram(Named(program))
    }

    /// Declares a source taking an argument's words by position.
    pub fn positional(&self, doc: &'static str) -> PositionalSource {
        PositionalSource::new(doc)
    }

    /// Declares a product: an argument made of fields given together, one named source each.
    pub fn product(&self) -> ProductSource<()> {
        ProductSource::new()
    }

    /// Declares a sum: an argument given as exactly one of its alternatives, one named source each.
    pub fn sum(&self) -> SumSource<()> {
        SumSource::new()
    }

    /// Declares a source taking an argument's words after its spelling, `--argument-name`.
    pub fn named(&self, doc: &'static str) -> NamedSource {
        NamedSource::new(doc)
    }
}

impl<Handler, Inputs, Mode> Operation<Handler, Inputs, Mode> {
    /// Declares the source of the operation's next argument, the one named `name`.
    ///
    /// `defunc(via = cli)` checks the name against the operation's own when the program compiles,
    /// reporting a source written for another argument where it is written.
    pub fn arg_for<Source>(self, name: &'static str, source: Source) -> Operation<Handler, (Inputs, Source), Mode> {
        let _: &'static str = name;
        self.arg(source)
    }

    /// Declares the source of the operation's next argument, after those already declared.
    pub fn arg<Source>(self, source: Source) -> Operation<Handler, (Inputs, Source), Mode> {
        Operation {
            handler: self.handler,
            inputs: (self.inputs, source),
            mode: self.mode,
            conventions: self.conventions,
        }
    }

    /// Declares this operation under one command name, with no program around it.
    ///
    /// A program that states many commands composes their registrations rather than their types,
    /// so it needs each command on its own. `CliProgram::command` states the same thing inside a
    /// composition, and is what an author writes.
    pub fn declare(self, name: &'static str) -> Command<Handler, Inputs, Mode> {
        Command { name, handler: self.handler, inputs: self.inputs, mode: self.mode, conventions: self.conventions }
    }
}

impl<Handler, Inputs, Mode> Operation<Handler, Inputs, Mode> {
    /// Ends the command's options at its first operand.
    #[must_use]
    pub fn options_first(mut self) -> Self {
        self.conventions.options_first = true;
        self
    }

    /// Reads a word naming no option of the command as an operand.
    #[must_use]
    pub fn unknown_options_as_operands(mut self) -> Self {
        self.conventions.unknown_options_are_operands = true;
        self
    }

    /// Reads `--` as an operand rather than the end of the options.
    #[must_use]
    pub fn no_end_of_options(mut self) -> Self {
        self.conventions.double_dash_ends_options = false;
        self
    }

    /// Reads a word of a dash and digits as the option spelled `--long` given those digits, as
    /// `head -5` is `head -n 5`.
    #[must_use]
    pub fn numeric_option(mut self, long: &'static str) -> Self {
        self.conventions.numeric_option = Some(long);
        self
    }

    /// Reads every word as an operand, ignores operands beyond the command's own, and answers no
    /// help.
    #[must_use]
    pub fn verbatim(mut self) -> Self {
        self.conventions.verbatim = true;
        self
    }

    /// Applies every command of `program` to the context this operation answers, with no program
    /// around it.
    pub fn within<Program>(self, program: CliProgram<Program>) -> Within<Handler, Inputs, Mode, Program> {
        Within { handler: self.handler, inputs: self.inputs, mode: self.mode, program: program.0 }
    }
}

impl<Handler, Inputs> Operation<Handler, Inputs, Total> {
    /// Reads the operation's `Result` output as success or a failure with the error's own status.
    pub fn fallible(self) -> Operation<Handler, Inputs, Fallible> {
        Operation { handler: self.handler, inputs: self.inputs, mode: Fallible, conventions: self.conventions }
    }
}

impl<Program> CliProgram<Program> {
    /// Records the composition of two typed CLI programs.
    pub fn merge<Other>(self, other: CliProgram<Other>) -> CliProgram<Merge<Program, Other>> {
        CliProgram(Merge { left: self.0, right: other.0 })
    }

    /// Registers a typed operation under a command name.
    pub fn command<Handler, Inputs, Mode>(
        self,
        name: &'static str,
        operation: Operation<Handler, Inputs, Mode>,
    ) -> CliProgram<Merge<Program, Command<Handler, Inputs, Mode>>> {
        self.merge(CliProgram(operation.declare(name)))
    }

    /// Declares the program's own command: the one a command line naming no command runs.
    ///
    /// It sits beside named commands, which a command line naming one selects first.
    pub fn cmd<Handler, Inputs, Mode>(
        self,
        operation: Operation<Handler, Inputs, Mode>,
    ) -> CliProgram<Merge<Program, Command<Handler, Inputs, Mode>>> {
        self.merge(CliProgram(operation.declare("")))
    }

    /// Places `other` beneath the subcommand `name`.
    pub fn nest<Other>(self, name: &'static str, other: CliProgram<Other>) -> CliProgram<Merge<Program, Nest<Other>>> {
        self.merge(CliProgram(other.under(name)))
    }

    /// Applies every command of `other` to the context `operation` answers.
    pub fn within<Handler, Inputs, Mode, Other>(
        self,
        operation: Operation<Handler, Inputs, Mode>,
        other: CliProgram<Other>,
    ) -> CliProgram<Merge<Program, Within<Handler, Inputs, Mode, Other>>> {
        self.merge(CliProgram(operation.within(other)))
    }

    /// Places this program beneath the subcommand `name`, with no program around it.
    pub fn under(self, name: &'static str) -> Nest<Program> {
        Nest { name, program: self.0 }
    }

    /// Removes the fluent wrapper and returns the first-order syntax tree.
    pub fn into_program(self) -> Program {
        self.0
    }
}

impl<Compiler> CompileCliProgram<Compiler> for Empty
where
    Compiler: CliCommandsAlg,
{
    type Commands = Compiler::Commands;

    fn compile_cli_program(self, compiler: &Compiler) -> Self::Commands {
        compiler.cli_empty()
    }
}

impl<Compiler, Left, Right, Commands> CompileCliProgram<Compiler> for Merge<Left, Right>
where
    Compiler: CliCommandsAlg<Commands = Commands>,
    Left: CompileCliProgram<Compiler, Commands = Commands>,
    Right: CompileCliProgram<Compiler, Commands = Commands>,
{
    type Commands = Commands;

    fn compile_cli_program(self, compiler: &Compiler) -> Self::Commands {
        compiler.cli_merge(self.left.compile_cli_program(compiler), self.right.compile_cli_program(compiler))
    }
}

impl<Compiler, Program> CompileCliProgram<Compiler> for Named<Program>
where
    Program: CliProgramAlg<Compiler>,
{
    type Commands = Program::Commands;

    fn compile_cli_program(self, compiler: &Compiler) -> Self::Commands {
        self.0.compile_cli(compiler)
    }
}

impl<Compiler, Program, Commands> CompileCliProgram<Compiler> for Nest<Program>
where
    Compiler: CliNestAlg<Commands = Commands>,
    Program: CompileCliProgram<Compiler, Commands = Commands>,
{
    type Commands = Commands;

    fn compile_cli_program(self, compiler: &Compiler) -> Self::Commands {
        let program = self.program;
        compiler.cli_nest_with(self.name, |compiler| program.compile_cli_program(compiler))
    }
}

impl<Compiler, Handler, Inputs, Mode> CompileCliProgram<Compiler> for Command<Handler, Inputs, Mode>
where
    Compiler: CliOperationAlg<Handler, Inputs, Mode>,
    Handler: OperationAlg,
    Inputs: CliSources<Handler::Args>,
{
    type Commands = Compiler::Commands;

    fn compile_cli_program(self, compiler: &Compiler) -> Self::Commands {
        compiler.cli_operation(self.name, self.handler, self.inputs, self.mode, self.conventions)
    }
}

impl<Compiler, Handler, Inputs, Mode, Program, Inner, InnerCommands> CompileCliProgram<Compiler>
    for Within<Handler, Inputs, Mode, Program>
where
    Compiler: CliContextAlg<Handler, Inputs, Mode, Inner = Inner, InnerCommands = InnerCommands>,
    Handler: OperationAlg,
    Inputs: CliSources<Handler::Args> + CliNamedSources,
    Program: CompileCliProgram<Inner, Commands = InnerCommands>,
{
    type Commands = Compiler::Commands;

    fn compile_cli_program(self, compiler: &Compiler) -> Self::Commands {
        let inner = self.program.compile_cli_program(&compiler.cli_inner());
        compiler.cli_within(self.handler, self.inputs, self.mode, inner)
    }
}
