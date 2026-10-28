use crate::command_line::{CommandLineAlg, ParseError, ParseErrorKind, Span, os_string, read};
use alux_cli::{ArgumentInfo, Arity, CliWordAlg, CliWordsAlg, Conventions, SourceInfo};
use std::{
    ffi::{OsStr, OsString},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    num::{
        NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8, NonZeroU16, NonZeroU32,
        NonZeroU64, NonZeroU128, NonZeroUsize,
    },
    path::PathBuf,
};

/// The words after a command's path, assigned to the command's arguments.
///
/// A word is an operating system string, as the command line gives it, so a word that is not
/// UTF-8 reaches an argument that takes it unchanged.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Parsed {
    positional: Vec<OsString>,
    /// How many operands the command's positional arguments take, when not without limit.
    takes: Option<usize>,
    /// The first operand beyond those, and its word's index.
    unexpected: Option<(usize, OsString)>,
    /// Each option occurrence, as its long spelling and its word, in command-line order.
    named: Vec<(String, OsString)>,
}

impl Parsed {
    /// Assigns `words` to `arguments` under `conventions`: options by spelling, everything else
    /// by position.
    pub(crate) fn new(
        arguments: &[ArgumentInfo],
        words: &[OsString],
        conventions: Conventions,
    ) -> Result<Self, ParseError> {
        let positions = arguments.iter().filter(|argument| matches!(argument.source, SourceInfo::Positional { .. }));
        let takes_many = positions.clone().any(|argument| argument.arity == Arity::Many);
        // A verbatim command ignores operands beyond its own.
        let takes = (!takes_many && !conventions.verbatim).then(|| positions.count());
        let mut parsed = Self { takes, ..Self::default() };
        read(arguments, words, conventions, &mut parsed)?;
        if let Some((word, extra)) = parsed.unexpected {
            let kind = ParseErrorKind::Unexpected(extra.to_string_lossy().into_owned());
            return Err(ParseError { at: Span { word, byte: 0 }, kind });
        }

        Ok(parsed)
    }

    fn record(&mut self, argument: &ArgumentInfo, given: OsString) {
        if let Some(key) = key(argument) {
            self.named.push((key, given));
        }
    }
}

/// Keys a spelled argument's words by its spelling: a product's field can share its name with another
/// argument, not its spelling. An assigned name keeps its `=`, so `if=` is never the option `--if`.
fn key(argument: &ArgumentInfo) -> Option<String> {
    match &argument.source {
        SourceInfo::Named { long, .. } => Some(long.clone()),
        SourceInfo::Assigned { name } => Some(format!("{name}=")),
        SourceInfo::Positional { .. } => None,
    }
}

/// A switch is recorded as the empty word, its own spelling.
impl CommandLineAlg for Parsed {
    fn switch(&mut self, argument: &ArgumentInfo) {
        self.record(argument, OsString::new());
    }

    fn value(&mut self, argument: &ArgumentInfo, value: OsString) {
        self.record(argument, value);
    }

    fn operand(&mut self, word: &OsStr, at: usize) {
        if self.unexpected.is_none() && self.takes == Some(self.positional.len()) {
            self.unexpected = Some((at, word.to_os_string()));
        }
        self.positional.push(word.to_os_string());
    }
}

impl CliWordsAlg for Parsed {
    type Word = OsString;

    fn cli_words(&self, argument: &ArgumentInfo) -> Vec<OsString> {
        match &argument.source {
            SourceInfo::Positional { index } if argument.arity == Arity::Many => {
                self.positional.get(*index..).unwrap_or_default().to_vec()
            }
            SourceInfo::Positional { index } => self.positional.get(*index).cloned().into_iter().collect(),
            SourceInfo::Named { .. } | SourceInfo::Assigned { .. } => {
                let key = key(argument);
                self.named.iter().filter(|(own, _)| Some(own) == key.as_ref()).map(|(_, word)| word.clone()).collect()
            }
        }
    }

    fn cli_env(&self, name: &str) -> Option<OsString> {
        std::env::var_os(name)
    }

    fn cli_default(&self, default: &'static str) -> OsString {
        OsString::from(default)
    }

    fn cli_split(&self, word: OsString, delimiter: char) -> Vec<OsString> {
        split(&word, delimiter)
    }
}

/// Splits a word on an ASCII-compatible delimiter, keeping each part's bytes.
pub(crate) fn split(word: &OsStr, delimiter: char) -> Vec<OsString> {
    let mut encoded = [0; 4];
    let delimiter = delimiter.encode_utf8(&mut encoded).as_bytes();
    let bytes = word.as_encoded_bytes();
    let mut parts = Vec::new();
    let mut start = 0;
    let mut at = 0;
    while at + delimiter.len() <= bytes.len() {
        if &bytes[at..at + delimiter.len()] == delimiter {
            parts.push(os_string(&bytes[start..at]));
            at += delimiter.len();
            start = at;
        } else {
            at += 1;
        }
    }
    parts.push(os_string(&bytes[start..]));
    parts
}

/// States how one word on the command line denotes a value.
///
/// Text and numbers are read with `FromStr` from a word that is UTF-8; `OsString` and `PathBuf` are
/// the word itself, whatever its bytes. A domain value implements it for itself.
pub trait DirectWord: Sized {
    /// Reads the value `word` denotes.
    ///
    /// # Errors
    ///
    /// Answers why `word` denotes no value.
    fn direct_word(word: &OsStr) -> Result<Self, String>;
}

macro_rules! text_words {
    ($($value:ty),* $(,)?) => {
        $(
            impl DirectWord for $value {
                fn direct_word(word: &OsStr) -> Result<Self, String> {
                    let text = word.to_str().ok_or_else(|| format!("`{}` is not UTF-8", word.to_string_lossy()))?;
                    text.parse().map_err(|error: <$value as std::str::FromStr>::Err| error.to_string())
                }
            }
        )*
    };
}

text_words!(
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroIsize,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroUsize,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddr,
    SocketAddrV4,
    SocketAddrV6,
    char,
    f32,
    f64,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize
);

/// A switch's own spelling, an empty word, is on; a word from the environment is a truth:
/// `true`, `false`, `1`, or `0`.
impl DirectWord for bool {
    fn direct_word(word: &OsStr) -> Result<Self, String> {
        match word.to_str() {
            Some("" | "true" | "1") => Ok(true),
            Some("false" | "0") => Ok(false),
            _ => Err(format!("`{}` is not true or false", word.to_string_lossy())),
        }
    }
}

/// Any UTF-8 word is its own text.
impl DirectWord for String {
    fn direct_word(word: &OsStr) -> Result<Self, String> {
        word.to_str().map(str::to_owned).ok_or_else(|| format!("`{}` is not UTF-8", word.to_string_lossy()))
    }
}

impl DirectWord for OsString {
    fn direct_word(word: &OsStr) -> Result<Self, String> {
        Ok(word.to_os_string())
    }
}

impl DirectWord for PathBuf {
    fn direct_word(word: &OsStr) -> Result<Self, String> {
        Ok(Self::from(word))
    }
}

/// Decodes a word with the value's `DirectWord`.
impl<Value> CliWordAlg<Value, OsString> for Parsed
where
    Value: DirectWord,
{
    type Error = String;

    fn cli_word(&self, word: &OsString) -> Result<Value, String> {
        Value::direct_word(word)
    }
}
