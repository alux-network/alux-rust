/// States how many occurrences an argument takes, as its source's shape says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arity {
    /// Exactly one value.
    One,
    /// At most one value: `Option` of it.
    Optional,
    /// Any number of values, in order: `Vec` of them.
    Many,
    /// No value; whether it occurs: `bool`.
    Presence,
    /// A value written only in the option's own word, or none: `-i`, `-i.bak`, `--in-place=.bak`.
    /// `Option<Option<_>>`: whether it occurs, and the value it was given.
    OptionalValue,
}

/// Decodes one word as a value; an interpreter answers it for the values it supports.
pub trait CliWordAlg<Value, Word: ?Sized> {
    /// Why a word does not denote a value.
    type Error;

    /// Decodes `word` as a value.
    ///
    /// # Errors
    ///
    /// Answers with the interpreter's error when `word` does not denote a value.
    fn cli_word(&self, word: &Word) -> Result<Value, Self::Error>;
}

/// States why the words given for an argument do not make its value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliArgumentError<Error> {
    /// An argument of arity one was not given.
    Missing(&'static str),
    /// An argument taking at most one value was given more than once.
    Repeated(&'static str),
    /// A word did not decode as the argument's value.
    Word(&'static str, Error),
    /// Two alternatives of a sum were given together.
    Conflict(&'static str, &'static str),
}
