use alux_cli::{ArgumentInfo, Arity, CliWordAlg, CliWordsAlg, SourceInfo};
use clap::ArgMatches;
use std::ffi::{OsStr, OsString};

/// The words `clap` matched for one command, read back as each argument's words.
///
/// It holds the matches of the command itself, which `clap` gives its global arguments too.
#[derive(Clone, Debug)]
pub struct ClapWords(pub(crate) ArgMatches);

/// Keys the `clap` argument an argument is declared as by its source's kind and its spelling or
/// position, so an option `--path` and an operand `path` are two arguments.
pub(crate) fn id(argument: &ArgumentInfo) -> String {
    match &argument.source {
        SourceInfo::Named { long, .. } => format!("named:{long}"),
        SourceInfo::Assigned { name } => format!("assigned:{name}"),
        SourceInfo::Positional { index } => format!("positional:{index}"),
    }
}

impl CliWordsAlg for ClapWords {
    type Word = OsString;

    fn cli_words(&self, argument: &ArgumentInfo) -> Vec<OsString> {
        let id = id(argument);
        if argument.arity == Arity::Presence {
            // A switch is counted; each occurrence is the word `clap` reads as on.
            let count = self.0.try_get_one::<u8>(&id).ok().flatten().copied().unwrap_or(0);
            return vec![OsString::from("true"); usize::from(count)];
        }
        self.0.try_get_raw(&id).ok().flatten().map(|words| words.map(OsStr::to_os_string).collect()).unwrap_or_default()
    }

    fn cli_env(&self, name: &str) -> Option<OsString> {
        std::env::var_os(name)
    }

    fn cli_default(&self, default: &'static str) -> OsString {
        OsString::from(default)
    }

    fn cli_split(&self, word: OsString, delimiter: char) -> Vec<OsString> {
        // `clap` splits the command line's words itself; this splits the environment's and the default.
        split(&word, delimiter)
    }
}

/// Splits a word on a delimiter, keeping each part's bytes.
#[cfg(unix)]
fn split(word: &OsStr, delimiter: char) -> Vec<OsString> {
    use std::os::unix::ffi::OsStrExt;
    let mut encoded = [0; 4];
    let delimiter = delimiter.encode_utf8(&mut encoded).as_bytes();
    let bytes = word.as_bytes();
    let mut parts = Vec::new();
    let (mut start, mut at) = (0, 0);
    while at + delimiter.len() <= bytes.len() {
        if &bytes[at..at + delimiter.len()] == delimiter {
            parts.push(OsStr::from_bytes(&bytes[start..at]).to_os_string());
            at += delimiter.len();
            start = at;
        } else {
            at += 1;
        }
    }
    parts.push(OsStr::from_bytes(&bytes[start..]).to_os_string());
    parts
}

/// Splits a word on a delimiter; off Unix a word that is not text is kept whole.
#[cfg(not(unix))]
fn split(word: &OsStr, delimiter: char) -> Vec<OsString> {
    match word.to_str() {
        Some(text) => text.split(delimiter).map(OsString::from).collect(),
        None => vec![word.to_os_string()],
    }
}

/// States how one word denotes a value, read with the parser `clap` chooses for its type.
///
/// `clap_words!` states it for a type through `clap::value_parser!`, so a value reads exactly as
/// `clap` reads it: through `FromStr`, a `ValueEnum`, or the type's own `ValueParserFactory`.
pub trait ClapWord: Sized {
    /// Reads the value `word` denotes.
    ///
    /// # Errors
    ///
    /// Answers why `word` denotes no value.
    fn clap_word(word: &OsStr) -> Result<Self, String>;
}

/// States `ClapWord` for each type, reading it with `clap::value_parser!`.
///
/// A type is read as `clap` reads it wherever this names it, the defining crate included.
#[macro_export]
macro_rules! clap_words {
    ($($value:ty),* $(,)?) => {
        $(
            impl $crate::ClapWord for $value {
                fn clap_word(word: &::std::ffi::OsStr) -> ::core::result::Result<Self, ::std::string::String> {
                    // `clap` reads a value only within a command, so the word is one argument's value.
                    let value = $crate::__clap::Arg::new("value")
                        .value_parser($crate::__clap::value_parser!($value))
                        .allow_hyphen_values(true);
                    $crate::__clap::Command::new("")
                        .arg(value)
                        .try_get_matches_from([::std::ffi::OsStr::new(""), word])
                        .map_err(|error| {
                            // The parser's own reason ends the first line: `invalid value '..' for '..': reason`.
                            let rendered = error.to_string();
                            let first = rendered.lines().next().unwrap_or_default();
                            first.rsplit_once(": ").map_or(first, |(_, reason)| reason).to_owned()
                        })?
                        .remove_one::<$value>("value")
                        .ok_or_else(|| ::std::string::String::from("no value"))
                }
            }
        )*
    };
}

clap_words!(
    String,
    OsString,
    std::path::PathBuf,
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
    usize,
    std::num::NonZeroI8,
    std::num::NonZeroI16,
    std::num::NonZeroI32,
    std::num::NonZeroI64,
    std::num::NonZeroI128,
    std::num::NonZeroIsize,
    std::num::NonZeroU8,
    std::num::NonZeroU16,
    std::num::NonZeroU32,
    std::num::NonZeroU64,
    std::num::NonZeroU128,
    std::num::NonZeroUsize,
    std::net::IpAddr,
    std::net::Ipv4Addr,
    std::net::Ipv6Addr,
    std::net::SocketAddr,
);

/// A truth, as an environment variable states one: `true`/`false`, `1`/`0`, `yes`/`no`, `on`/`off`.
impl ClapWord for bool {
    fn clap_word(word: &OsStr) -> Result<Self, String> {
        let value = clap::Arg::new("value").value_parser(clap::builder::BoolishValueParser::new());
        clap::Command::new("")
            .arg(value)
            .try_get_matches_from([OsStr::new(""), word])
            .map_err(|_| format!("`{}` is not true or false", word.to_string_lossy()))?
            .remove_one::<bool>("value")
            .ok_or_else(|| String::from("no value"))
    }
}

/// Decodes a word with the value's `ClapWord`.
impl<Value> CliWordAlg<Value, OsString> for ClapWords
where
    Value: ClapWord,
{
    type Error = String;

    fn cli_word(&self, word: &OsString) -> Result<Value, String> {
        Value::clap_word(word)
    }
}
