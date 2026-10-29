use core::marker::PhantomData;
use std::ffi::OsStr;

use crate::{Arity, CliArgumentError, CliWordAlg};

/// Marks a source whose argument comes only from the command line, with no fallback.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoFallback;

/// Marks a source that falls back to an environment variable and nothing else.
///
/// A switch takes this fallback, since off is already what it is when nothing turns it on.
#[derive(Clone, Copy, Debug, Default)]
pub struct WithEnv;

/// Marks a source that falls back to a default word, after an environment variable if it has one.
#[derive(Clone, Copy, Debug, Default)]
pub struct WithFallback;

/// States the fallback mode a source takes once it also reads an environment variable.
pub trait EnvFallback {
    /// The mode with the environment variable.
    type WithEnv;
}

impl EnvFallback for NoFallback {
    type WithEnv = WithEnv;
}

impl EnvFallback for WithEnv {
    type WithEnv = WithEnv;
}

impl EnvFallback for WithFallback {
    type WithEnv = WithFallback;
}

/// Names where an argument's words come from when the command line gives none.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fallback {
    /// The environment variable read next.
    pub env: Option<&'static str>,
    /// The word used last.
    pub default: Option<&'static str>,
}

/// Marks a source supplying exactly one value, the operation's argument itself.
#[derive(Clone, Copy, Debug, Default)]
pub struct Whole;

/// Marks a source supplying `Option` of a value: at most one. A product takes all of its fields or
/// none of them, a sum one alternative or none.
#[derive(Clone, Copy, Debug, Default)]
pub struct Optional;

/// Marks a source supplying `Vec` of values: any number, in the order given.
#[derive(Clone, Copy, Debug, Default)]
pub struct Many;

/// Marks a named source supplying `bool`: whether its spelling is given. It takes no value.
#[derive(Clone, Copy, Debug, Default)]
pub struct Switch;

/// Marks a named source supplying `Option<Option<_>>`: whether its spelling is given, and the value
/// written in the same word, as `sed -i.bak` gives one and `sed -i` none.
#[derive(Clone, Copy, Debug, Default)]
pub struct OptionalValue;

/// States how many occurrences a source of one shape takes.
pub trait CliShape {
    /// The occurrences the shape takes.
    const ARITY: Arity;
}

impl CliShape for Whole {
    const ARITY: Arity = Arity::One;
}

impl CliShape for Optional {
    const ARITY: Arity = Arity::Optional;
}

impl CliShape for Many {
    const ARITY: Arity = Arity::Many;
}

impl CliShape for OptionalValue {
    const ARITY: Arity = Arity::OptionalValue;
}

impl CliShape for Switch {
    const ARITY: Arity = Arity::Presence;
}

/// Takes an argument's words by position, in the order the positional arguments are declared.
///
/// It supplies one value; `.optional()` and `.many()` state another shape.
#[derive(Clone, Copy, Debug)]
pub struct PositionalSource<Mode = NoFallback, Shape = Whole> {
    doc: &'static str,
    fallback: Fallback,
    mode: PhantomData<fn() -> (Mode, Shape)>,
}

/// Marks a named source spelled as options: `--long` or `-s`, with the value after it.
#[derive(Clone, Copy, Debug, Default)]
pub struct Options;

/// Marks a named source spelled as an operand assigning its value, `name=value`, as `dd if=FILE`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Assigned;

/// Takes an argument's words after its spelling, `--long` or `-s`, or from an operand `name=value`
/// once `.assigned(name)` spells it so.
///
/// The long spelling defaults to the operation's argument name with `_` written as `-`. It supplies
/// one value; `.optional()`, `.many()`, and `.switch()` state another shape.
#[derive(Clone, Copy, Debug)]
pub struct NamedSource<Mode = NoFallback, Shape = Whole, Spelling = Options> {
    /// The long spelling, or the assigned name once `.assigned(..)` spells the source.
    long: Option<&'static str>,
    short: Option<char>,
    short_aliases: &'static [char],
    doc: &'static str,
    fallback: Fallback,
    mode: Markers<Mode, Shape, Spelling>,
}

/// The type-level choices a named source carries: its fallback, shape, and spelling.
type Markers<Mode, Shape, Spelling> = PhantomData<fn() -> (Mode, Shape, Spelling)>;

impl PositionalSource {
    /// Declares a positional source documented by `doc`.
    pub fn new(doc: &'static str) -> Self {
        Self { doc, fallback: Fallback::default(), mode: PhantomData }
    }
}

impl<Mode> PositionalSource<Mode> {
    /// Supplies `Option` of the value: at most one word.
    pub fn optional(self) -> PositionalSource<Mode, Optional> {
        PositionalSource { doc: self.doc, fallback: self.fallback, mode: PhantomData }
    }

    /// Supplies `Vec` of values: every remaining positional word.
    pub fn many(self) -> PositionalSource<Mode, Many> {
        PositionalSource { doc: self.doc, fallback: self.fallback, mode: PhantomData }
    }
}

impl<Mode, Shape> PositionalSource<Mode, Shape> {
    /// Reads the environment variable `name` when the command line gives no words.
    pub fn env(self, name: &'static str) -> PositionalSource<Mode::WithEnv, Shape>
    where
        Mode: EnvFallback,
    {
        PositionalSource { doc: self.doc, fallback: Fallback { env: Some(name), ..self.fallback }, mode: PhantomData }
    }

    /// Uses `word` when neither the command line nor the environment gives one.
    pub fn default(self, word: &'static str) -> PositionalSource<WithFallback, Shape> {
        PositionalSource {
            doc: self.doc,
            fallback: Fallback { default: Some(word), ..self.fallback },
            mode: PhantomData,
        }
    }
}

impl NamedSource {
    /// Declares a named source documented by `doc`, spelled after the argument name.
    pub fn new(doc: &'static str) -> Self {
        Self { long: None, short: None, short_aliases: &[], doc, fallback: Fallback::default(), mode: PhantomData }
    }
}

impl<Mode> NamedSource<Mode> {
    /// Spells the source as the operand `name=value` instead of an option: the value is everything
    /// after the first `=`, and may be empty. It takes no positional place, and no other spelling.
    pub fn assigned(self, name: &'static str) -> NamedSource<Mode, Whole, Assigned> {
        NamedSource {
            long: Some(name),
            short: None,
            short_aliases: &[],
            doc: self.doc,
            fallback: self.fallback,
            mode: PhantomData,
        }
    }

    /// Supplies `bool`: whether the spelling is given, with no value after it.
    pub fn switch(self) -> NamedSource<Mode, Switch> {
        NamedSource {
            long: self.long,
            short: self.short,
            short_aliases: self.short_aliases,
            doc: self.doc,
            fallback: self.fallback,
            mode: PhantomData,
        }
    }

    /// Supplies `Option<Option<_>>`: whether the spelling is given, and the value written in its
    /// own word, `-i.bak` or `--in-place=.bak`. The next word is never its value.
    pub fn optional_value(self) -> NamedSource<Mode, OptionalValue> {
        NamedSource {
            long: self.long,
            short: self.short,
            short_aliases: self.short_aliases,
            doc: self.doc,
            fallback: self.fallback,
            mode: PhantomData,
        }
    }
}

impl<Mode, Spelling> NamedSource<Mode, Whole, Spelling> {
    /// Supplies `Option` of the value: at most one occurrence.
    pub fn optional(self) -> NamedSource<Mode, Optional, Spelling> {
        NamedSource {
            long: self.long,
            short: self.short,
            short_aliases: self.short_aliases,
            doc: self.doc,
            fallback: self.fallback,
            mode: PhantomData,
        }
    }

    /// Supplies `Vec` of values: one per occurrence, in order.
    pub fn many(self) -> NamedSource<Mode, Many, Spelling> {
        NamedSource {
            long: self.long,
            short: self.short,
            short_aliases: self.short_aliases,
            doc: self.doc,
            fallback: self.fallback,
            mode: PhantomData,
        }
    }
}

impl<Mode, Shape> NamedSource<Mode, Shape> {
    /// Spells the source `--long` instead of after the argument name.
    #[must_use]
    pub fn long(self, long: &'static str) -> Self {
        Self { long: Some(long), ..self }
    }

    /// Also spells the source `-short`.
    #[must_use]
    pub fn short(self, short: char) -> Self {
        Self { short: Some(short), ..self }
    }

    /// Also spells the source as each of `aliases`, as `cp -R` is `cp -r`. Help shows them after
    /// the short spelling.
    #[must_use]
    pub fn short_aliases(self, aliases: &'static [char]) -> Self {
        Self { short_aliases: aliases, ..self }
    }
}

impl<Mode, Shape, Spelling> NamedSource<Mode, Shape, Spelling> {
    /// Reads the environment variable `name` when the command line gives no words.
    pub fn env(self, name: &'static str) -> NamedSource<Mode::WithEnv, Shape, Spelling>
    where
        Mode: EnvFallback,
    {
        let fallback = Fallback { env: Some(name), ..self.fallback };
        NamedSource {
            long: self.long,
            short: self.short,
            short_aliases: self.short_aliases,
            doc: self.doc,
            fallback,
            mode: PhantomData,
        }
    }

    /// Uses `word` when neither the command line nor the environment gives one.
    pub fn default(self, word: &'static str) -> NamedSource<WithFallback, Shape, Spelling> {
        let fallback = Fallback { default: Some(word), ..self.fallback };
        NamedSource {
            long: self.long,
            short: self.short,
            short_aliases: self.short_aliases,
            doc: self.doc,
            fallback,
            mode: PhantomData,
        }
    }
}

/// Describes where an argument's words come from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceInfo {
    /// The `index`th positional argument of the command, counting from zero.
    Positional {
        /// The position among the command's positional arguments.
        index: usize,
    },
    /// The value after `name=` in an operand that assigns it, as `dd if=FILE` does.
    Assigned {
        /// The name before `=`.
        name: String,
    },
    /// Words following `--long` or `-short`.
    Named {
        /// The long spelling, without `--`.
        long: String,
        /// The short spelling, without `-`.
        short: Option<char>,
        /// Other short spellings that mean the same, without `-`.
        short_aliases: &'static [char],
    },
}

/// Names the compound argument a described argument is one part of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// A field of a product: given together with the product's other fields.
    Product(&'static str),
    /// An alternative of a sum: given instead of the sum's other alternatives.
    Sum(&'static str),
}

/// Describes one argument of a command: the operation's argument and the source it is read from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentInfo {
    /// The operation's argument name, or the field or alternative name within a compound argument.
    pub name: &'static str,
    /// Where its words come from on the command line.
    pub source: SourceInfo,
    /// Where its words come from when the command line gives none.
    pub fallback: Fallback,
    /// How many occurrences its type takes.
    pub arity: Arity,
    /// What the argument means on the command line.
    pub doc: &'static str,
    /// The compound argument this is a part of, if any.
    pub part_of: Option<Part>,
    /// The character every word is split on before decoding, for a list one word gives.
    pub delimiter: Option<char>,
}

/// States that a source can supply an argument of type `Argument`.
///
/// A named source supplies any argument. A positional source supplies a value, an optional
/// value, or many values, but not a switch, because a switch is only its spelling. For the same
/// reason a switch has no fallback. A product source supplies a `Product`, and a sum source a `Sum`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot supply an argument of type `{Argument}`",
    note = "sources bind to the operation's arguments in order, one `.arg(..)` each; check the order of the `.arg(..)` calls",
    note = "a plain source supplies the value itself, `.optional()` an `Option`, `.many()` a `Vec`, `.switch()` a `bool`, `.product()` and `.sum()` their type"
)]
pub trait CliSourceFor<Argument> {
    /// Describes this source for the argument `name` into `out`, counting positional sources in
    /// `positions`. A compound source describes one argument per part.
    fn cli_describe(&self, name: &'static str, positions: &mut usize, out: &mut Vec<ArgumentInfo>);
}

/// Decodes the argument a source supplies, with `Decoder`.
pub trait CliSourceDecode<Argument, Decoder, Error> {
    /// Decodes the argument from the words `decoder` gives for the arguments this source was
    /// described as, taken from the front of `arguments`.
    ///
    /// # Errors
    ///
    /// Answers with the part of the argument whose words do not make its value.
    fn cli_decode_source(
        &self,
        arguments: &mut &[ArgumentInfo],
        decoder: &Decoder,
    ) -> Result<Argument, CliArgumentError<Error>>;
}

/// Takes the next described argument, the one a source of one argument was described as.
///
/// A command is described from the same sources it decodes with, so there is always one.
pub(crate) fn next<'a>(arguments: &mut &'a [ArgumentInfo]) -> &'a ArgumentInfo {
    let (argument, rest) = arguments.split_first().expect("a source decodes the argument it was described as");
    *arguments = rest;
    argument
}

/// States how a named source is spelled on the command line.
pub trait CliSpelling {
    /// Describes the spelling of a source spelled `long`, `-short`, and `-aliases`.
    fn cli_spelling(long: String, short: Option<char>, short_aliases: &'static [char]) -> SourceInfo;
}

impl CliSpelling for Options {
    fn cli_spelling(long: String, short: Option<char>, short_aliases: &'static [char]) -> SourceInfo {
        SourceInfo::Named { long, short, short_aliases }
    }
}

impl CliSpelling for Assigned {
    fn cli_spelling(name: String, _: Option<char>, _: &'static [char]) -> SourceInfo {
        SourceInfo::Assigned { name }
    }
}

impl<Mode, Shape: CliShape, Spelling: CliSpelling> NamedSource<Mode, Shape, Spelling> {
    fn describe(&self, name: &'static str, _: &mut usize) -> ArgumentInfo {
        let long = self.long.map_or_else(|| name.replace('_', "-"), str::to_owned);
        ArgumentInfo {
            name,
            source: Spelling::cli_spelling(long, self.short, self.short_aliases),
            fallback: self.fallback,
            arity: Shape::ARITY,
            doc: self.doc,
            part_of: None,
            delimiter: None,
        }
    }
}

/// Takes a list's words after its spelling, each word split on a delimiter: `--peers a,b` and
/// `PEERS=a,b` both give two values.
///
/// It supplies only a `Vec`, the one arity a word that splits can fill.
#[derive(Clone, Copy, Debug)]
pub struct DelimitedSource<Mode = NoFallback, Spelling = Options> {
    source: NamedSource<Mode, Many, Spelling>,
    delimiter: char,
}

impl<Mode, Spelling> NamedSource<Mode, Many, Spelling> {
    /// Splits every word, from the command line, the environment, or the default, on `delimiter`.
    pub fn delimiter(self, delimiter: char) -> DelimitedSource<Mode, Spelling> {
        DelimitedSource { source: self, delimiter }
    }
}

impl<Mode, Spelling: CliSpelling> DelimitedSource<Mode, Spelling> {
    fn describe(&self, name: &'static str, positions: &mut usize) -> ArgumentInfo {
        ArgumentInfo { delimiter: Some(self.delimiter), ..self.source.describe(name, positions) }
    }
}

impl<Mode, Shape: CliShape> PositionalSource<Mode, Shape> {
    fn describe(&self, name: &'static str, positions: &mut usize) -> ArgumentInfo {
        let index = *positions;
        *positions += 1;
        ArgumentInfo {
            name,
            source: SourceInfo::Positional { index },
            fallback: self.fallback,
            arity: Shape::ARITY,
            doc: self.doc,
            part_of: None,
            delimiter: None,
        }
    }
}

/// Lets each shape of source supply its argument for any value, and decodes it by shape: the value
/// is the interpreter's to read from a word, so the source states only how many words it takes.
macro_rules! shaped_sources {
    ($([$($generic:ident),*] $source:ty, $shape:ident => $argument:ty;)*) => {
        $(
            impl<Mode, Value $(, $generic: CliSpelling)*> CliSourceFor<$argument> for $source {
                fn cli_describe(&self, name: &'static str, positions: &mut usize, out: &mut Vec<ArgumentInfo>) {
                    out.push(self.describe(name, positions));
                }
            }

            impl<Mode, Value, Decoder, Error $(, $generic)*> CliSourceDecode<$argument, Decoder, Error> for $source
            where
                Decoder: CliWordsAlg + CliWordAlg<Value, Decoder::Word, Error = Error>,
            {
                fn cli_decode_source(
                    &self,
                    arguments: &mut &[ArgumentInfo],
                    decoder: &Decoder,
                ) -> Result<$argument, CliArgumentError<Error>> {
                    let argument = next(arguments);
                    $shape(argument.name, &words(argument, decoder), decoder)
                }
            }
        )*
    };
}

shaped_sources! {
    [] PositionalSource<Mode, Whole>, one => Value;
    [] PositionalSource<Mode, Optional>, optional => Option<Value>;
    [] PositionalSource<Mode, Many>, many => Vec<Value>;
    [Spelling] NamedSource<Mode, Whole, Spelling>, one => Value;
    [Spelling] NamedSource<Mode, Optional, Spelling>, optional => Option<Value>;
    [Spelling] NamedSource<Mode, Many, Spelling>, many => Vec<Value>;
    [Spelling] DelimitedSource<Mode, Spelling>, many => Vec<Value>;
}

/// Reads exactly one value from `words`.
fn one<Value, Decoder, Word, Error>(
    name: &'static str,
    words: &[Word],
    decoder: &Decoder,
) -> Result<Value, CliArgumentError<Error>>
where
    Decoder: CliWordAlg<Value, Word, Error = Error>,
{
    match words {
        [] => Err(CliArgumentError::Missing(name)),
        [word] => decoder.cli_word(word).map_err(|error| CliArgumentError::Word(name, error)),
        _ => Err(CliArgumentError::Repeated(name)),
    }
}

/// Reads at most one value from `words`.
fn optional<Value, Decoder, Word, Error>(
    name: &'static str,
    words: &[Word],
    decoder: &Decoder,
) -> Result<Option<Value>, CliArgumentError<Error>>
where
    Decoder: CliWordAlg<Value, Word, Error = Error>,
{
    if words.is_empty() { Ok(None) } else { one(name, words, decoder).map(Some) }
}

/// Reads one value from each of `words`, in order.
fn many<Value, Decoder, Word, Error>(
    name: &'static str,
    words: &[Word],
    decoder: &Decoder,
) -> Result<Vec<Value>, CliArgumentError<Error>>
where
    Decoder: CliWordAlg<Value, Word, Error = Error>,
{
    words.iter().map(|word| decoder.cli_word(word).map_err(|error| CliArgumentError::Word(name, error))).collect()
}

/// A switch, and a switch read from the environment as well: on when given, else as the variable
/// says. Its own spelling, which an interpreter answers as an empty word, decodes on.
macro_rules! switches {
    ($($mode:ident),*) => {
        $(
            impl CliSourceFor<bool> for NamedSource<$mode, Switch> {
                fn cli_describe(&self, name: &'static str, positions: &mut usize, out: &mut Vec<ArgumentInfo>) {
                    out.push(self.describe(name, positions));
                }
            }

            impl<Decoder, Error> CliSourceDecode<bool, Decoder, Error> for NamedSource<$mode, Switch>
            where
                Decoder: CliWordsAlg + CliWordAlg<bool, Decoder::Word, Error = Error>,
            {
                fn cli_decode_source(
                    &self,
                    arguments: &mut &[ArgumentInfo],
                    decoder: &Decoder,
                ) -> Result<bool, CliArgumentError<Error>> {
                    let argument = next(arguments);
                    let words = words(argument, decoder);
                    if words.is_empty() { Ok(false) } else { one(argument.name, &words, decoder) }
                }
            }
        )*
    };
}

switches!(NoFallback, WithEnv);

/// An option with a value written only in its own word: absent, given bare, which an interpreter
/// answers as an empty word, or given a value.
impl<Value> CliSourceFor<Option<Option<Value>>> for NamedSource<NoFallback, OptionalValue> {
    fn cli_describe(&self, name: &'static str, positions: &mut usize, out: &mut Vec<ArgumentInfo>) {
        out.push(self.describe(name, positions));
    }
}

impl<Value, Decoder, Error> CliSourceDecode<Option<Option<Value>>, Decoder, Error>
    for NamedSource<NoFallback, OptionalValue>
where
    Decoder: CliWordsAlg + CliWordAlg<Value, Decoder::Word, Error = Error>,
    Decoder::Word: AsRef<OsStr>,
{
    fn cli_decode_source(
        &self,
        arguments: &mut &[ArgumentInfo],
        decoder: &Decoder,
    ) -> Result<Option<Option<Value>>, CliArgumentError<Error>> {
        let argument = next(arguments);
        match words(argument, decoder).as_slice() {
            [] => Ok(None),
            [word] if word.as_ref().is_empty() => Ok(Some(None)),
            words => one(argument.name, words, decoder).map(|value| Some(Some(value))),
        }
    }
}

/// An alternative of a sum carrying no value: given by its spelling alone.
impl CliSourceFor<()> for NamedSource<NoFallback, Switch> {
    fn cli_describe(&self, name: &'static str, positions: &mut usize, out: &mut Vec<ArgumentInfo>) {
        out.push(self.describe(name, positions));
    }
}

impl<Decoder, Error> CliSourceDecode<(), Decoder, Error> for NamedSource<NoFallback, Switch>
where
    Decoder: CliWordsAlg,
{
    fn cli_decode_source(
        &self,
        arguments: &mut &[ArgumentInfo],
        decoder: &Decoder,
    ) -> Result<(), CliArgumentError<Error>> {
        let argument = next(arguments);
        match words(argument, decoder).len() {
            0 => Err(CliArgumentError::Missing(argument.name)),
            1 => Ok(()),
            _ => Err(CliArgumentError::Repeated(argument.name)),
        }
    }
}

/// States that a declared product of sources supplies exactly the operation arguments `Args`.
///
/// The sources are written in the order of the operation's arguments, one each, so a missing,
/// extra, or mismatched source fails to compile.
#[diagnostic::on_unimplemented(
    message = "the declared sources `{Self}` do not supply the operation arguments `{Args}`",
    note = "declare one source per operation argument, in order, with `.arg(self.positional(..))` or `.arg(self.named(..))`"
)]
pub trait CliSources<Args> {
    /// Describes every argument, named by the operation's `names`, in order.
    fn cli_arguments(&self, names: &'static [&'static str]) -> Vec<ArgumentInfo>;
}

/// States that every source in a product is named.
///
/// A subtree's global arguments are named: a positional one would take positions every command of
/// the subtree already numbers for its own arguments. A product's fields and a sum's alternatives
/// are named for the same reason.
#[diagnostic::on_unimplemented(
    message = "the sources `{Self}` are not all named",
    note = "declare global arguments, product fields, and alternatives with `self.named(..)`"
)]
pub trait CliNamedSources {}

impl CliNamedSources for () {}

impl<Previous, Mode, Shape, Spelling> CliNamedSources for (Previous, NamedSource<Mode, Shape, Spelling>) where
    Previous: CliNamedSources
{
}

/// Marks a source spelled by name, which is what a product's field or a sum's alternative takes.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a named source",
    note = "declare a product's fields and a sum's alternatives with `self.named(..)`"
)]
pub trait CliNamedSource {}

impl<Mode, Shape, Spelling> CliNamedSource for NamedSource<Mode, Shape, Spelling> {}

impl<Mode, Spelling> CliNamedSource for DelimitedSource<Mode, Spelling> {}

impl<Previous, Mode, Spelling> CliNamedSources for (Previous, DelimitedSource<Mode, Spelling>) where
    Previous: CliNamedSources
{
}

/// Answers the words given for one argument and the environment it falls back to.
///
/// An interpreter reads both from what it was given. For a switch, it answers one word per
/// occurrence.
pub trait CliWordsAlg {
    /// What one word is: text, an operating system string, bytes. The interpreter chooses, so a
    /// word the command line gives reaches the value it denotes unchanged.
    type Word;

    /// Returns the words the command line gives for `argument`, in the order they were given.
    fn cli_words(&self, argument: &ArgumentInfo) -> Vec<Self::Word>;

    /// Returns the value of the environment variable `name`, if it is set.
    fn cli_env(&self, name: &str) -> Option<Self::Word>;

    /// Returns the word a declared default spells.
    fn cli_default(&self, default: &'static str) -> Self::Word;

    /// Splits `word` on `delimiter`, for a list one word gives.
    fn cli_split(&self, word: Self::Word, delimiter: char) -> Vec<Self::Word>;
}

/// Returns the words that make `argument`: the command line's, else the environment's, else the
/// default.
pub(crate) fn words<Decoder: CliWordsAlg>(argument: &ArgumentInfo, decoder: &Decoder) -> Vec<Decoder::Word> {
    let mut words = decoder.cli_words(argument);
    if words.is_empty() {
        let Fallback { env, default } = argument.fallback;
        words = env
            .and_then(|name| decoder.cli_env(name))
            .or_else(|| default.map(|default| decoder.cli_default(default)))
            .into_iter()
            .collect();
    }
    match argument.delimiter {
        Some(delimiter) => words.into_iter().flat_map(|word| decoder.cli_split(word, delimiter)).collect(),
        None => words,
    }
}

/// Decodes the operation arguments `Args` through declared sources, with `Decoder`.
pub trait CliDecodeSources<Args, Decoder, Error> {
    /// Decodes every argument from the words `decoder` gives for it, reading the arguments these
    /// sources were described as, in order.
    ///
    /// # Errors
    ///
    /// Answers with the first argument whose words do not make its value.
    fn cli_decode(&self, arguments: &[ArgumentInfo], decoder: &Decoder) -> Result<Args, CliArgumentError<Error>>;
}

impl CliSources<()> for () {
    fn cli_arguments(&self, _: &'static [&'static str]) -> Vec<ArgumentInfo> {
        Vec::new()
    }
}

impl<Decoder, Error> CliDecodeSources<(), Decoder, Error> for () {
    fn cli_decode(&self, _: &[ArgumentInfo], _: &Decoder) -> Result<(), CliArgumentError<Error>> {
        Ok(())
    }
}

/// Spells a product of sources as `.argument` builds it: `((((), S1), S2), S3)`.
macro_rules! nested {
    ($inner:ty;) => { $inner };
    ($inner:ty; $head:ident $($tail:ident)*) => { nested!(($inner, $head); $($tail)*) };
}

/// Destructures a product of sources the same way, binding each source to a name.
macro_rules! nested_pattern {
    ($inner:tt;) => { $inner };
    ($inner:tt; $head:ident $($tail:ident)*) => { nested_pattern!(($inner, $head); $($tail)*) };
}

pub(crate) use {nested, nested_pattern};

macro_rules! sources {
    ($(($source:ident, $argument:ident, $value:ident, $index:tt)),+) => {
        impl<$($source, $argument),+> CliSources<($($argument,)+)> for nested!((); $($source)+)
        where
            $($source: CliSourceFor<$argument>,)+
        {
            fn cli_arguments(&self, names: &'static [&'static str]) -> Vec<ArgumentInfo> {
                let nested_pattern!((); $($value)+) = self;
                let mut positions = 0;
                let mut out = Vec::new();
                $(CliSourceFor::<$argument>::cli_describe($value, names[$index], &mut positions, &mut out);)+
                out
            }
        }

        impl<Decoder, Error, $($source, $argument),+> CliDecodeSources<($($argument,)+), Decoder, Error>
            for nested!((); $($source)+)
        where
            $($source: CliSourceDecode<$argument, Decoder, Error>,)+
        {
            fn cli_decode(
                &self,
                mut arguments: &[ArgumentInfo],
                decoder: &Decoder,
            ) -> Result<($($argument,)+), CliArgumentError<Error>> {
                let nested_pattern!((); $($value)+) = self;
                Ok(($($value.cli_decode_source(&mut arguments, decoder)?,)+))
            }
        }
    };
}

sources!((S0, A0, s0, 0));
sources!((S0, A0, s0, 0), (S1, A1, s1, 1));
sources!((S0, A0, s0, 0), (S1, A1, s1, 1), (S2, A2, s2, 2));
sources!((S0, A0, s0, 0), (S1, A1, s1, 1), (S2, A2, s2, 2), (S3, A3, s3, 3));
sources!((S0, A0, s0, 0), (S1, A1, s1, 1), (S2, A2, s2, 2), (S3, A3, s3, 3), (S4, A4, s4, 4));
sources!((S0, A0, s0, 0), (S1, A1, s1, 1), (S2, A2, s2, 2), (S3, A3, s3, 3), (S4, A4, s4, 4), (S5, A5, s5, 5));
sources!(
    (S0, A0, s0, 0),
    (S1, A1, s1, 1),
    (S2, A2, s2, 2),
    (S3, A3, s3, 3),
    (S4, A4, s4, 4),
    (S5, A5, s5, 5),
    (S6, A6, s6, 6)
);
sources!(
    (S0, A0, s0, 0),
    (S1, A1, s1, 1),
    (S2, A2, s2, 2),
    (S3, A3, s3, 3),
    (S4, A4, s4, 4),
    (S5, A5, s5, 5),
    (S6, A6, s6, 6),
    (S7, A7, s7, 7)
);
sources!(
    (S0, A0, s0, 0),
    (S1, A1, s1, 1),
    (S2, A2, s2, 2),
    (S3, A3, s3, 3),
    (S4, A4, s4, 4),
    (S5, A5, s5, 5),
    (S6, A6, s6, 6),
    (S7, A7, s7, 7),
    (S8, A8, s8, 8)
);
sources!(
    (S0, A0, s0, 0),
    (S1, A1, s1, 1),
    (S2, A2, s2, 2),
    (S3, A3, s3, 3),
    (S4, A4, s4, 4),
    (S5, A5, s5, 5),
    (S6, A6, s6, 6),
    (S7, A7, s7, 7),
    (S8, A8, s8, 8),
    (S9, A9, s9, 9)
);
sources!(
    (S0, A0, s0, 0),
    (S1, A1, s1, 1),
    (S2, A2, s2, 2),
    (S3, A3, s3, 3),
    (S4, A4, s4, 4),
    (S5, A5, s5, 5),
    (S6, A6, s6, 6),
    (S7, A7, s7, 7),
    (S8, A8, s8, 8),
    (S9, A9, s9, 9),
    (S10, A10, s10, 10)
);
sources!(
    (S0, A0, s0, 0),
    (S1, A1, s1, 1),
    (S2, A2, s2, 2),
    (S3, A3, s3, 3),
    (S4, A4, s4, 4),
    (S5, A5, s5, 5),
    (S6, A6, s6, 6),
    (S7, A7, s7, 7),
    (S8, A8, s8, 8),
    (S9, A9, s9, 9),
    (S10, A10, s10, 10),
    (S11, A11, s11, 11)
);
sources!(
    (S0, A0, s0, 0),
    (S1, A1, s1, 1),
    (S2, A2, s2, 2),
    (S3, A3, s3, 3),
    (S4, A4, s4, 4),
    (S5, A5, s5, 5),
    (S6, A6, s6, 6),
    (S7, A7, s7, 7),
    (S8, A8, s8, 8),
    (S9, A9, s9, 9),
    (S10, A10, s10, 10),
    (S11, A11, s11, 11),
    (S12, A12, s12, 12)
);
sources!(
    (S0, A0, s0, 0),
    (S1, A1, s1, 1),
    (S2, A2, s2, 2),
    (S3, A3, s3, 3),
    (S4, A4, s4, 4),
    (S5, A5, s5, 5),
    (S6, A6, s6, 6),
    (S7, A7, s7, 7),
    (S8, A8, s8, 8),
    (S9, A9, s9, 9),
    (S10, A10, s10, 10),
    (S11, A11, s11, 11),
    (S12, A12, s12, 12),
    (S13, A13, s13, 13)
);
sources!(
    (S0, A0, s0, 0),
    (S1, A1, s1, 1),
    (S2, A2, s2, 2),
    (S3, A3, s3, 3),
    (S4, A4, s4, 4),
    (S5, A5, s5, 5),
    (S6, A6, s6, 6),
    (S7, A7, s7, 7),
    (S8, A8, s8, 8),
    (S9, A9, s9, 9),
    (S10, A10, s10, 10),
    (S11, A11, s11, 11),
    (S12, A12, s12, 12),
    (S13, A13, s13, 13),
    (S14, A14, s14, 14)
);
sources!(
    (S0, A0, s0, 0),
    (S1, A1, s1, 1),
    (S2, A2, s2, 2),
    (S3, A3, s3, 3),
    (S4, A4, s4, 4),
    (S5, A5, s5, 5),
    (S6, A6, s6, 6),
    (S7, A7, s7, 7),
    (S8, A8, s8, 8),
    (S9, A9, s9, 9),
    (S10, A10, s10, 10),
    (S11, A11, s11, 11),
    (S12, A12, s12, 12),
    (S13, A13, s13, 13),
    (S14, A14, s14, 14),
    (S15, A15, s15, 15)
);
