//! Reads a command line's words once, as POSIX utilities do, for every interpretation of what they
//! say.
//!
//! The reader decides what each word is: an option with or without its value, a switch, an
//! operand, or the `--` that ends the options. It reports each through [`CommandLineAlg`], so
//! assigning words to arguments, skipping global options before a command's path, and printing a
//! canonical command line all read words by the same rules.

use alux_cli::{ArgumentInfo, Arity, Conventions, SourceInfo};
use std::ffi::{OsStr, OsString};

/// Places something read in the words: the word, by index, and the byte within it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    /// The word's index among the words read.
    pub word: usize,
    /// The byte within the word.
    pub byte: usize,
}

/// States why words do not read as a command's arguments, and where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// Where the words stop reading as the command's arguments.
    pub at: Span,
    /// What is wrong there.
    pub kind: ParseErrorKind,
}

/// States what is wrong where words stop reading as a command's arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// An option no argument of the command is spelled as.
    Unknown(String),
    /// An option that takes a value was last, with no value after it.
    MissingValue(String),
    /// A switch was given a value.
    SwitchValue(String),
    /// A positional word no positional argument takes.
    Unexpected(String),
}

/// Interprets what a command line's words say, in order.
pub(crate) trait CommandLineAlg {
    /// A switch was given, or an option whose value is optional was given none.
    fn switch(&mut self, argument: &ArgumentInfo);
    /// An option was given `value`.
    fn value(&mut self, argument: &ArgumentInfo, value: OsString);
    /// The word at index `at` is an operand.
    fn operand(&mut self, word: &OsStr, at: usize);
    /// `--` ended the options.
    fn end_of_options(&mut self) {}
}

/// What one word was read as.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Read {
    Options,
    Operand,
    EndOfOptions,
}

/// The words being read, and the next to read.
pub(crate) struct Cursor<'w> {
    words: &'w [OsString],
    at: usize,
}

impl<'w> Cursor<'w> {
    pub(crate) fn new(words: &'w [OsString]) -> Self {
        Self { words, at: 0 }
    }

    /// The index of the next word.
    pub(crate) fn at(&self) -> usize {
        self.at
    }

    /// Consumes the next word.
    pub(crate) fn bump(&mut self) -> Option<&'w OsString> {
        let word = self.words.get(self.at)?;
        self.at += 1;
        Some(word)
    }
}

/// Reads every word under `conventions` into `syntax`.
///
/// Accepts `--long value`, `--long=value`, `-s value`, `-svalue`, clustered switches `-abc`, and
/// `--`, after which every word is an operand, each as the conventions allow.
pub(crate) fn read(
    arguments: &[ArgumentInfo],
    words: &[OsString],
    conventions: Conventions,
    syntax: &mut impl CommandLineAlg,
) -> Result<(), ParseError> {
    if conventions.verbatim {
        words.iter().enumerate().for_each(|(at, word)| syntax.operand(word, at));
        return Ok(());
    }
    let mut cursor = Cursor::new(words);
    let mut options = true;
    while options {
        match read_word(arguments, conventions, &mut cursor, syntax)? {
            None => return Ok(()),
            Some(Read::EndOfOptions) => options = false,
            Some(Read::Operand) => options = !conventions.options_first,
            Some(Read::Options) => {}
        }
    }
    loop {
        let at = cursor.at();
        let Some(word) = cursor.bump() else { break };
        // An operand assigning an argument still assigns it after `--`: `dd -- if=x`.
        if !assign(arguments, word, syntax) {
            syntax.operand(word, at);
        }
    }

    Ok(())
}

/// Reads `word` as the operand `name=value` of an argument assigned `name`, answering whether it is
/// one. The value is everything after the first `=`, and may be empty.
fn assign(arguments: &[ArgumentInfo], word: &OsStr, syntax: &mut impl CommandLineAlg) -> bool {
    let bytes = word.as_encoded_bytes();
    let Some(at) = bytes.iter().position(|byte| *byte == b'=').filter(|at| *at > 0) else { return false };
    let assigned = arguments
        .iter()
        .find(|argument| matches!(&argument.source, SourceInfo::Assigned { name } if name.as_bytes() == &bytes[..at]));
    match assigned {
        Some(argument) => {
            syntax.value(argument, os_string(&bytes[at + 1..]));
            true
        }
        None => false,
    }
}

/// Reads the next word, with the value word an option takes after it, answering what it was, or
/// nothing at the end of the words.
pub(crate) fn read_word(
    arguments: &[ArgumentInfo],
    conventions: Conventions,
    cursor: &mut Cursor<'_>,
    syntax: &mut impl CommandLineAlg,
) -> Result<Option<Read>, ParseError> {
    let word_at = cursor.at();
    let Some(word) = cursor.bump() else { return Ok(None) };
    let bytes = word.as_encoded_bytes();
    if bytes == b"--" && conventions.double_dash_ends_options {
        syntax.end_of_options();
        return Ok(Some(Read::EndOfOptions));
    }
    if bytes.len() < 2 || bytes[0] != b'-' || bytes == b"--" {
        // An assigning operand takes no positional place, and so ends no options.
        if assign(arguments, word, syntax) {
            return Ok(Some(Read::Options));
        }
        syntax.operand(word, word_at);
        return Ok(Some(Read::Operand));
    }
    let mut options = Vec::new();
    let read = if bytes.starts_with(b"--") {
        long(arguments, bytes, &mut options)
    } else if let Some(numeric) = conventions.numeric_option.filter(|_| bytes[1..].iter().all(u8::is_ascii_digit)) {
        // `-5` is the numeric option given `5`, as `head -5` is `head -n 5`.
        let argument = arguments
            .iter()
            .find(|argument| matches!(&argument.source, SourceInfo::Named { long, .. } if long == numeric));
        match argument {
            Some(argument) => {
                options.push((argument, Given::Attached(os_string(&bytes[1..]))));
                Ok(())
            }
            None => Err(Unread { byte: 0, kind: ParseErrorKind::Unknown(word.to_string_lossy().into_owned()) }),
        }
    } else {
        short(arguments, bytes, &mut options)
    };
    match read {
        Err(Unread { kind: ParseErrorKind::Unknown(_), .. }) if conventions.unknown_options_are_operands => {
            // A word spelling no option of the command is its operand.
            syntax.operand(word, word_at);
            return Ok(Some(Read::Operand));
        }
        Err(Unread { byte, kind }) => return Err(ParseError { at: Span { word: word_at, byte }, kind }),
        Ok(()) => {}
    }
    for (argument, value) in options {
        match value {
            Given::Switch => syntax.switch(argument),
            Given::Attached(value) => syntax.value(argument, value),
            Given::Next(spelling) => {
                let at = Span { word: cursor.at(), byte: 0 };
                let value = cursor.bump().ok_or(ParseError { at, kind: ParseErrorKind::MissingValue(spelling) })?;
                syntax.value(argument, value.clone());
            }
        }
    }

    Ok(Some(Read::Options))
}

/// How an option in a word is given.
enum Given {
    /// By its spelling alone: a switch, or an option whose value is optional, given none.
    Switch,
    Attached(OsString),
    /// Its value is the next word; the option is named by its spelling, should there be none.
    Next(String),
}

/// What is wrong at a byte of the word being read.
struct Unread {
    byte: usize,
    kind: ParseErrorKind,
}

/// Reads `--long` or `--long=value`.
fn long<'a>(
    arguments: &'a [ArgumentInfo],
    bytes: &[u8],
    options: &mut Vec<(&'a ArgumentInfo, Given)>,
) -> Result<(), Unread> {
    let (spelling, value) = match bytes.iter().position(|byte| *byte == b'=') {
        Some(at) => (&bytes[2..at], Some(os_string(&bytes[at + 1..]))),
        None => (&bytes[2..], None),
    };
    // Spellings are ASCII, so only the value can be any operating system string.
    let spelled = format!("--{}", String::from_utf8_lossy(spelling));
    let argument = arguments
        .iter()
        .find(|argument| matches!(&argument.source, SourceInfo::Named { long, .. } if long.as_bytes() == spelling))
        .ok_or_else(|| Unread { byte: 0, kind: ParseErrorKind::Unknown(spelled.clone()) })?;
    let given = match (argument.arity, value) {
        (Arity::Presence | Arity::OptionalValue, None) => Given::Switch,
        (Arity::Presence, Some(_)) => return Err(Unread { byte: 0, kind: ParseErrorKind::SwitchValue(spelled) }),
        (_, Some(value)) => Given::Attached(value),
        (_, None) => Given::Next(spelled),
    };
    options.push((argument, given));

    Ok(())
}

/// Reads a cluster of short options as POSIX `getopt` does: `-s`, switches clustered as `-abc`,
/// and a letter that takes a value, which takes the rest of the word (`-d:`, `-n5`, `-sd:`) or,
/// when nothing follows it, the next word. `=` is no separator here, so `-d=:` gives `=:`.
fn short<'a>(
    arguments: &'a [ArgumentInfo],
    bytes: &[u8],
    options: &mut Vec<(&'a ArgumentInfo, Given)>,
) -> Result<(), Unread> {
    for (byte, letter) in bytes.iter().enumerate().skip(1) {
        // Spellings are ASCII, so a letter is one byte; the value after it can be any bytes.
        let letter = char::from(*letter);
        let argument = arguments
            .iter()
            .find(|argument| matches!(&argument.source, SourceInfo::Named { short, short_aliases, .. } if *short == Some(letter) || short_aliases.contains(&letter)))
            .ok_or_else(|| Unread { byte, kind: ParseErrorKind::Unknown(format!("-{letter}")) })?;
        if argument.arity == Arity::Presence {
            options.push((argument, Given::Switch));
            continue;
        }
        let rest = &bytes[byte + 1..];
        let given = match (rest.is_empty(), argument.arity) {
            // An optional value is only ever written in the option's own word: `-i` or `-i.bak`.
            (true, Arity::OptionalValue) => Given::Switch,
            (true, _) => Given::Next(format!("-{letter}")),
            (false, _) => Given::Attached(os_string(rest)),
        };
        options.push((argument, given));
        break;
    }

    Ok(())
}

/// Prints what words say as a canonical command line: options by their long spelling, each value
/// its own word, and the operands after `--` where it ends the options.
///
/// It witnesses the reading law in tests; no command line needs printing at run time.
#[cfg(test)]
pub(crate) struct Printed {
    conventions: Conventions,
    options: Vec<OsString>,
    operands: Vec<OsString>,
}

#[cfg(test)]
impl Printed {
    pub(crate) fn new(conventions: Conventions) -> Self {
        Self { conventions, options: Vec::new(), operands: Vec::new() }
    }

    /// The canonical words.
    pub(crate) fn words(mut self) -> Vec<OsString> {
        let protect =
            !self.operands.is_empty() && self.conventions.double_dash_ends_options && !self.conventions.verbatim;
        if protect {
            self.options.push("--".into());
        }
        self.options.extend(self.operands);
        self.options
    }

    fn spelling(argument: &ArgumentInfo) -> OsString {
        match &argument.source {
            SourceInfo::Named { long, .. } => format!("--{long}").into(),
            SourceInfo::Positional { .. } | SourceInfo::Assigned { .. } => OsString::new(),
        }
    }
}

#[cfg(test)]
impl CommandLineAlg for Printed {
    fn switch(&mut self, argument: &ArgumentInfo) {
        self.options.push(Self::spelling(argument));
    }

    fn value(&mut self, argument: &ArgumentInfo, value: OsString) {
        if let SourceInfo::Assigned { name } = &argument.source {
            let mut assigned = OsString::from(format!("{name}="));
            assigned.push(value);
            self.options.push(assigned);
            return;
        }
        let mut spelling = Self::spelling(argument);
        // An optional value is only read from the option's own word.
        if argument.arity == Arity::OptionalValue {
            spelling.push("=");
            spelling.push(value);
            self.options.push(spelling);
            return;
        }
        self.options.push(spelling);
        self.options.push(value);
    }

    fn operand(&mut self, word: &OsStr, _: usize) {
        self.operands.push(word.to_os_string());
    }
}

/// Ignores what words say, for a reader that needs only how many words an option takes.
pub(crate) struct Skipped;

impl CommandLineAlg for Skipped {
    fn switch(&mut self, _: &ArgumentInfo) {}

    fn value(&mut self, _: &ArgumentInfo, _: OsString) {}

    fn operand(&mut self, _: &OsStr, _: usize) {}
}

/// Rebuilds the operating system string an option value's bytes came from.
#[cfg(unix)]
pub(crate) fn os_string(bytes: &[u8]) -> OsString {
    use std::os::unix::ffi::OsStrExt;
    OsStr::from_bytes(bytes).to_os_string()
}

/// Rebuilds an option value; off Unix only its text is kept.
#[cfg(not(unix))]
pub(crate) fn os_string(bytes: &[u8]) -> OsString {
    OsString::from(String::from_utf8_lossy(bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parsed;
    use alux_cli::Fallback;

    fn argument(source: SourceInfo, arity: Arity) -> ArgumentInfo {
        ArgumentInfo {
            name: "argument",
            source,
            fallback: Fallback::default(),
            arity,
            doc: "",
            part_of: None,
            delimiter: None,
        }
    }

    fn named(long: &str, short: char, arity: Arity) -> ArgumentInfo {
        argument(SourceInfo::Named { long: long.into(), short: Some(short), short_aliases: &[] }, arity)
    }

    fn words(line: &str) -> Vec<OsString> {
        line.split_whitespace().map(OsString::from).collect()
    }

    /// How many words the first option takes, or nothing when the first word is no option.
    fn taken(arguments: &[ArgumentInfo], line: &str) -> Option<usize> {
        let words = words(line);
        let mut cursor = Cursor::new(&words);
        let read = read_word(arguments, Conventions::default(), &mut cursor, &mut Skipped).ok()??;
        (read == Read::Options).then(|| cursor.at())
    }

    #[test]
    fn takes_the_next_word_only_for_an_option_with_no_value_in_its_word() {
        let context = [named("verbose", 'v', Arity::Presence), named("store", 's', Arity::One)];
        assert_eq!(taken(&context, "--verbose x"), Some(1));
        assert_eq!(taken(&context, "--store work"), Some(2));
        assert_eq!(taken(&context, "--store=work x"), Some(1));
        assert_eq!(taken(&context, "-s work"), Some(2));
        assert_eq!(taken(&context, "-vs work"), Some(2));
        assert_eq!(taken(&context, "-vv x"), Some(1));
        // The first letter that takes a value takes the rest of the word, as in `getopt`.
        assert_eq!(taken(&context, "-sv x"), Some(1));
        assert_eq!(taken(&context, "-vswork x"), Some(1));
        assert_eq!(taken(&context, "-vx"), None);
        assert_eq!(taken(&context, "--nope"), None);
        assert_eq!(taken(&context, "list"), None);
    }

    #[test]
    fn reads_an_optional_value_only_from_the_option_s_own_word() {
        let arguments =
            [named("in-place", 'i', Arity::OptionalValue), argument(SourceInfo::Positional { index: 0 }, Arity::Many)];
        let parsed = |line: &str| Parsed::new(&arguments, &words(line), Conventions::default()).unwrap();
        let expected = |given: &[&str], operands: &str| {
            let mut printed =
                given.iter().map(|given| OsString::from(format!("--in-place={given}"))).collect::<Vec<_>>();
            printed.push("--".into());
            printed.extend(words(operands));
            Parsed::new(&arguments, &printed, Conventions::default()).unwrap()
        };
        assert_eq!(parsed("-i a"), expected(&[""], "a"));
        assert_eq!(parsed("-i.bak a"), expected(&[".bak"], "a"));
        assert_eq!(parsed("--in-place a"), expected(&[""], "a"));
        assert_eq!(parsed("--in-place=.bak a"), expected(&[".bak"], "a"));
    }

    #[test]
    fn reads_a_dash_and_digits_as_the_numeric_option() {
        let arguments = [named("lines", 'n', Arity::One), named("verbose", 'v', Arity::Presence)];
        let numeric = Conventions { numeric_option: Some("lines"), ..Conventions::default() };
        let parsed = |line: &str, conventions| Parsed::new(&arguments, &words(line), conventions);
        assert_eq!(parsed("-5", numeric), parsed("-n 5", numeric));
        assert_eq!(parsed("-v -20", numeric), parsed("-v --lines=20", numeric));
        // Without the convention, `-5` names no option.
        assert_eq!(parsed("-5", Conventions::default()).unwrap_err().kind, ParseErrorKind::Unknown("-5".into()));
    }

    #[test]
    fn places_an_error_at_its_word_and_byte() {
        let context = [named("verbose", 'v', Arity::Presence), named("store", 's', Arity::One)];
        let error = |line: &str| Parsed::new(&context, &words(line), Conventions::default()).unwrap_err();
        assert_eq!(error("-v -vx").at, Span { word: 1, byte: 2 });
        assert_eq!(error("-v -s").at, Span { word: 2, byte: 0 });
        assert_eq!(error("-v a").at, Span { word: 1, byte: 0 });
    }

    /// Reading the canonical printing of what words say says the same.
    #[test]
    fn reads_its_own_printing_as_what_was_read() {
        let arguments = [
            named("verbose", 'v', Arity::Presence),
            named("delimiter", 'd', Arity::One),
            named("tag", 't', Arity::Many),
            named("in-place", 'i', Arity::OptionalValue),
            named("lines", 'n', Arity::One),
            argument(SourceInfo::Assigned { name: "if".into() }, Arity::Many),
            argument(SourceInfo::Positional { index: 0 }, Arity::Many),
        ];
        let lines = [
            "-d: a b",
            "-d : -- -a",
            "-d=: -vt x -ty a",
            "--delimiter=, --tag t -- --verbose",
            "-vd -x -- -- b",
            "a -v b --tag=-c",
            "-i a -i.bak -vi.x --in-place --in-place=y",
            "-5 a -n 3 -12",
            "if=x a k=v if= -v -- if=a=b -i",
        ];
        for conventions in [
            Conventions::default(),
            Conventions { options_first: true, ..Conventions::default() },
            Conventions { unknown_options_are_operands: true, ..Conventions::default() },
            Conventions { double_dash_ends_options: false, ..Conventions::default() },
            Conventions { numeric_option: Some("lines"), ..Conventions::default() },
        ] {
            for line in lines {
                let Ok(parsed) = Parsed::new(&arguments, &words(line), conventions) else { continue };
                let mut printed = Printed::new(conventions);
                read(&arguments, &words(line), conventions, &mut printed).unwrap();
                let printed = printed.words();
                assert_eq!(Parsed::new(&arguments, &printed, conventions), Ok(parsed), "{line} printed as {printed:?}");
            }
        }
    }
}
