//! States what a header name says, independently of how a framework spells one.
//!
//! A header name is words. The wire spells them with `-` and lowercases them, because case is not
//! part of a header name; an argument spells the same words with `_`. Reading one as the other is
//! the whole of what an interpretation needs stating once, and it is why an author states a header
//! argument as an ordinary product rather than as a framework's type.

/// Reads the name a header states as the name an argument states.
pub fn read_header_name(name: &str) -> String {
    name.trim().to_lowercase().replace('-', "_")
}

/// Writes the name an argument states as the name a header states.
///
/// The reverse of [`read_header_name`], for anything stating a header name to a reader rather than
/// reading one: a document keying a parameter, or a client sending it.
pub fn write_header_name(name: &str) -> String {
    name.replace('_', "-")
}

#[cfg(test)]
mod tests {
    use super::{read_header_name, write_header_name};

    #[test]
    fn writes_the_words_an_argument_states_as_a_header_name() {
        assert_eq!(write_header_name("user_agent"), "user-agent");
        // A name survives the round trip, which is what makes one reading the other's reverse.
        assert_eq!(read_header_name(&write_header_name("user_agent")), "user_agent");
    }

    #[test]
    fn reads_a_header_name_as_the_words_it_states() {
        assert_eq!(read_header_name("User-Agent"), "user_agent");
        assert_eq!(read_header_name("  CONTENT-TYPE "), "content_type");
        // A name already spelled the way an argument spells it states the same words.
        assert_eq!(read_header_name("user_agent"), "user_agent");
    }
}
