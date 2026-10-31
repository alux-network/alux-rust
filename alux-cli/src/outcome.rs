use core::{
    convert::Infallible,
    fmt::{self, Display, Formatter},
    num::NonZeroU8,
};

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

/// Reads a `Result` output as success or failure, failing at the status the declaration states:
/// `1` for `.fallible()`, any other for `.fallible_as(status)`. The error can be any error that
/// displays, `anyhow::Error` among them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fallible {
    status: NonZeroU8,
}

impl Fallible {
    /// Fails at `status`.
    ///
    /// # Panics
    ///
    /// Panics when `status` is zero, which is success.
    #[must_use]
    #[track_caller]
    pub const fn new(status: u8) -> Self {
        match NonZeroU8::new(status) {
            Some(status) => Self { status },
            None => panic!("a failure exits with a nonzero status"),
        }
    }
}

/// Fails at status `1`, as `.fallible()` declares.
impl Default for Fallible {
    fn default() -> Self {
        Self { status: NonZeroU8::MIN }
    }
}

/// Reads a `Result` output as success or failure, failing as its error states: what
/// `.fallible_by_error()` declares, for an error that implements `CliFailureAlg`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FallibleByError;

/// An error a command fails with, at the status its declaration states.
///
/// It displays as its error does, alternate form included, so an interpreter writing `{:#}`
/// writes an `anyhow` error with its chain of causes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliFailure<Error> {
    /// The error the operation answered with.
    pub error: Error,
    status: NonZeroU8,
}

impl<Error> CliFailureAlg for CliFailure<Error> {
    fn cli_status(&self) -> NonZeroU8 {
        self.status
    }
}

impl<Error> Display for CliFailure<Error>
where
    Error: Display,
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        self.error.fmt(formatter)
    }
}

/// Reads an operation's output as a command's outcome under a declared mode.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot read the output `{Output}` as a command outcome",
    note = "a `.fallible()`, `.fallible_as(..)`, or `.fallible_by_error()` command applies an operation whose output is a `Result`"
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
    type Error = CliFailure<Error>;

    fn cli_outcome(&self, output: Result<Value, Error>) -> CliOutcome<Value, CliFailure<Error>> {
        match output {
            Ok(value) => CliOutcome::Success(value),
            Err(error) => CliOutcome::Failure(CliFailure { error, status: self.status }),
        }
    }
}

impl<Value, Error> CliOutcomeOf<Result<Value, Error>> for FallibleByError {
    type Value = Value;
    type Error = Error;

    fn cli_outcome(&self, output: Result<Value, Error>) -> CliOutcome<Value, Error> {
        match output {
            Ok(value) => CliOutcome::Success(value),
            Err(error) => CliOutcome::Failure(error),
        }
    }
}
