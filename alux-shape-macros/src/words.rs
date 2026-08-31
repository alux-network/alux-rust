//! Reading the words an identifier or an already-spelled name states.
//!
//! A name is words, so a declaration that recovers one from Rust source has to segment it. That
//! happens here, at expansion time, because a term carries the words themselves and not the source
//! they came from.

/// Names the words an identifier states, splitting where it marks a boundary.
///
/// Only `_` separates, because it is the only boundary a snake-case identifier states explicitly.
pub(crate) fn of_snake(identifier: &str) -> Vec<String> {
    identifier.split('_').filter(|word| !word.is_empty()).map(str::to_lowercase).collect()
}

/// Names the words a camel or pascal identifier states, splitting where its case changes.
///
/// A run of capitals stays one word: `UserName` states `[user, name]` and `HTTPServer` states
/// `[http, server]`. This is the one place a name is recovered rather than stated, so a declaration
/// whose words the recovery would get wrong states them itself.
pub(crate) fn of_camel(identifier: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut previous: Option<char> = None;
    let mut chars = identifier.chars().peekable();

    while let Some(current) = chars.next() {
        // A capital begins a word where the character before it was not one, and where it ends a run
        // of capitals that the next character continues in lower case.
        let starts_word = current.is_uppercase()
            && !word.is_empty()
            && match previous {
                Some(previous) if previous.is_uppercase() => chars.peek().is_some_and(|next| next.is_lowercase()),
                _ => true,
            };

        if starts_word {
            words.push(std::mem::take(&mut word));
        }

        word.extend(current.to_lowercase());
        previous = Some(current);
    }

    if !word.is_empty() {
        words.push(word);
    }

    words
}

/// Names the words an already-spelled name states, whichever way it was written.
pub(crate) fn of_spelled(name: &str) -> Vec<String> {
    name.split(['_', '-', '.', ' ']).filter(|part| !part.is_empty()).flat_map(of_camel).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_identifier_states_its_boundaries() {
        assert_eq!(of_snake("display_name"), ["display", "name"]);
        assert_eq!(of_snake("email"), ["email"]);
    }

    #[test]
    fn a_cased_identifier_marks_its_boundaries_by_case() {
        assert_eq!(of_camel("UserName"), ["user", "name"]);
        assert_eq!(of_camel("Admin"), ["admin"]);
        assert_eq!(of_camel("HTTPServer"), ["http", "server"]);
        assert_eq!(of_camel("displayName"), ["display", "name"]);
        assert_eq!(of_camel("sha3Sum"), ["sha3", "sum"]);
    }

    #[test]
    fn a_spelled_name_is_read_back_whichever_way_it_was_written() {
        assert_eq!(of_spelled("avatarBytes"), ["avatar", "bytes"]);
        assert_eq!(of_spelled("content-type"), ["content", "type"]);
        assert_eq!(of_spelled("display_name"), ["display", "name"]);
    }
}
