use crate::{ArgumentInfo, CliSources};
use alux_ext::OperationAlg;

/// Describes one command as a describing interpretation observes it.
///
/// Its path comes from the program, its documentation and argument names from the reified
/// operation, and its arguments from the declared sources.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandDescription {
    /// The subcommand names leading to the command, ending with its own name.
    pub path: Vec<&'static str>,
    /// The documentation of the extension method the command applies.
    pub doc: &'static str,
    /// The argument names of the extension method, in order.
    pub argument_names: &'static [&'static str],
    /// Every argument with the source it is read from, in the operation's order.
    pub arguments: Vec<ArgumentInfo>,
    /// The global arguments of every context the command is applied within, outermost first.
    pub context: Vec<ArgumentInfo>,
    /// How the command's words are read as its options and operands.
    pub conventions: Conventions,
}

/// States how one command's words are read as its options and operands.
///
/// The default reads options anywhere among the operands, rejects an option no argument is
/// spelled as, ends options at `--`, and answers `--help` and `-h` with help. POSIX utilities
/// differ per command, and a declaration states where one does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Conventions {
    /// Options end at the first operand: `echo hello -n` has two operands.
    pub options_first: bool,
    /// A word naming an option no argument is spelled as is an operand, as is a cluster with one
    /// such letter: `echo -x` writes `-x`.
    pub unknown_options_are_operands: bool,
    /// `--` ends the options. Where it does not, it is an operand like any other word.
    pub double_dash_ends_options: bool,
    /// Every word is an operand, operands beyond the command's own are ignored, and help is not
    /// answered: `true --help` exits 0.
    pub verbatim: bool,
    /// The long spelling of the option a word of a dash and digits gives its digits to: `head -5`
    /// is `head --lines 5`.
    pub numeric_option: Option<&'static str>,
}

impl Default for Conventions {
    fn default() -> Self {
        Self {
            options_first: false,
            unknown_options_are_operands: false,
            double_dash_ends_options: true,
            verbatim: false,
            numeric_option: None,
        }
    }
}

impl CommandDescription {
    /// Describes `Operation` declared under `name`, its arguments read from `sources`.
    pub fn of<Operation, Sources>(name: &'static str, sources: &Sources) -> Self
    where
        Operation: OperationAlg,
        Sources: CliSources<Operation::Args>,
    {
        Self {
            // The program's own command, declared with no name, is reached by no word at all.
            path: if name.is_empty() { Vec::new() } else { vec![name] },
            doc: Operation::DOC,
            argument_names: Operation::ARG_NAMES,
            arguments: sources.cli_arguments(Operation::ARG_NAMES),
            context: Vec::new(),
            conventions: Conventions::default(),
        }
    }
}
