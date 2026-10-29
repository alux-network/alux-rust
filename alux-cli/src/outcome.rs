use core::{convert::Infallible, num::NonZeroU8};

/// Names the exit status of a command whose words did not make it: missing, repeated, or
/// undecodable arguments. It follows the convention of POSIX utilities.
pub const USAGE_STATUS: NonZeroU8 = NonZeroU8::new(2).unwrap();

/// States what a command answers: its value, or a failure that ends with a nonzero status.
///
/// A successful command exits with status zero. How a value or failure is rendered, and on which
/// stream, is the interpreter's choice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliOutcome<Value, Error> {
    /// The command succeeded with this value.
    Success(Value),
    /// The command failed with this error.
    Failure(Error),
}

/// States what a domain failure denotes on the command line: the status the command exits with.
///
/// A domain says this once for its own error type. The status is nonzero by its type, because zero
/// is success.
pub trait CliFailureAlg {
    /// The status a command failing with this error exits with.
    fn cli_status(&self) -> NonZeroU8;

    /// Whether the failure was already reported as it happened, so the exit carries only its
    /// status: `cat` reports each missing file and still reads the rest, and `false` reports
    /// nothing.
    fn cli_reported(&self) -> bool {
        false
    }
}

/// An error that cannot be constructed has no status, so a command carrying one only succeeds.
impl CliFailureAlg for Infallible {
    fn cli_status(&self) -> NonZeroU8 {
        match *self {}
    }
}

impl<Value, Error> CliOutcome<Value, Error>
where
    Error: CliFailureAlg,
{
    /// Returns the status the command exits with: zero on success, the failure's own otherwise.
    pub fn cli_status(&self) -> u8 {
        match self {
            Self::Success(_) => 0,
            Self::Failure(error) => error.cli_status().get(),
        }
    }
}

/// Reads every output as a success: the command's default.
#[derive(Debug, Default)]
pub struct Total;

/// Reads a `Result` output as success or failure: what `.fallible()` declares.
#[derive(Debug, Default)]
pub struct Fallible;

/// Reads an operation's output as a command's outcome under a declared mode.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot read the output `{Output}` as a command outcome",
    note = "a `.fallible()` command applies an operation whose output is a `Result`"
)]
pub trait CliOutcomeOf<Output> {
    /// The value a successful command answers with.
    type Value;
    /// The failure a command can end with.
    type Error;

    /// Reads `output` as the command's outcome.
    fn cli_outcome(&self, output: Output) -> CliOutcome<Self::Value, Self::Error>;
}

impl<Output> CliOutcomeOf<Output> for Total {
    type Value = Output;
    type Error = Infallible;

    fn cli_outcome(&self, output: Output) -> CliOutcome<Output, Infallible> {
        CliOutcome::Success(output)
    }
}

impl<Value, Error> CliOutcomeOf<Result<Value, Error>> for Fallible {
    type Value = Value;
    type Error = Error;

    fn cli_outcome(&self, output: Result<Value, Error>) -> CliOutcome<Value, Error> {
        match output {
            Ok(value) => CliOutcome::Success(value),
            Err(error) => CliOutcome::Failure(error),
        }
    }
}
