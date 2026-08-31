//! How an interpretation spells the words a name is made of.
//!
//! A spelling is not part of a term — the same name reads `displayName` under one interpretation
//! and `display_name` under another. What lives here is only the vocabulary of spellings and the fold that
//! applies one, because both are functions of [`Words`], which this crate owns. Nothing in the
//! algebra consults them.

use crate::Words;

/// The spellings a name can be written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spelling {
    /// `display_name`
    Snake,
    /// `displayName`
    LowerCamel,
    /// `DisplayName`
    UpperCamel,
    /// `display-name`
    Kebab,
    /// `DISPLAY_NAME`
    Screaming,
}

impl Spelling {
    /// Writes a name in this spelling.
    ///
    /// Words are joined, never re-segmented, so a name survives any number of spellings unchanged —
    /// which a conversion between cased strings cannot promise wherever an acronym or a digit meets
    /// a boundary.
    pub fn spell(self, words: Words<'_>) -> String {
        match self {
            Self::Snake => words.join("_"),
            Self::Kebab => words.join("-"),
            Self::Screaming => words.join("_").to_uppercase(),
            Self::LowerCamel => words.iter().enumerate().map(|(i, w)| capitalize(w, i > 0)).collect(),
            Self::UpperCamel => words.iter().map(|w| capitalize(w, true)).collect(),
        }
    }
}

/// Names the words an identifier states, splitting where it marks a boundary.
///
/// An identifier reaches an interpretation wherever one is carried as source rather than as words —
/// an operation's argument names, for instance, which a JSON-RPC program states as they were
/// authored. Only `_` separates, because it is the only boundary a snake-case identifier states
/// explicitly.
pub fn words_of(identifier: &str) -> Vec<&str> {
    identifier.split('_').filter(|word| !word.is_empty()).collect()
}

/// Writes one word, capitalized or not.
fn capitalize(word: &str, upper: bool) -> String {
    let mut chars = word.chars();

    match chars.next() {
        Some(first) if upper => first.to_uppercase().collect::<String>() + chars.as_str(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DISPLAY_NAME: Words<'static> = &["display", "name"];
    const SHA3_SUM: Words<'static> = &["sha3", "sum"];

    #[test]
    fn one_name_spells_every_way() {
        assert_eq!(Spelling::Snake.spell(DISPLAY_NAME), "display_name");
        assert_eq!(Spelling::LowerCamel.spell(DISPLAY_NAME), "displayName");
        assert_eq!(Spelling::UpperCamel.spell(DISPLAY_NAME), "DisplayName");
        assert_eq!(Spelling::Kebab.spell(DISPLAY_NAME), "display-name");
        assert_eq!(Spelling::Screaming.spell(DISPLAY_NAME), "DISPLAY_NAME");
    }

    #[test]
    fn an_identifier_states_the_words_it_is_made_of() {
        assert_eq!(words_of("display_name"), ["display", "name"]);
        assert_eq!(Spelling::LowerCamel.spell(&words_of("display_name")), "displayName");
    }

    #[test]
    fn a_word_with_a_digit_keeps_its_boundary() {
        // Nothing re-segments, so the boundary the name states is the boundary every spelling keeps.
        assert_eq!(Spelling::LowerCamel.spell(SHA3_SUM), "sha3Sum");
        assert_eq!(Spelling::Snake.spell(SHA3_SUM), "sha3_sum");
    }
}
