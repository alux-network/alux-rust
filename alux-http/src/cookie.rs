//! States what a `Cookie` header carries, independently of how a framework reads one.
//!
//! Cookies are names and values, which is the same thing a query string carries and the same thing
//! an author reads them into. Every interpretation has the header; what it needs stating once is
//! what the header says.

/// Reads the cookies one `Cookie` header states, as the names and values it carries.
///
/// A header states pairs separated by `;`, each a name and a value separated by the first `=`. A
/// pair stating no value states nothing, and surrounding space is spelling rather than content.
pub fn read_cookies(header: &str) -> Vec<(&str, &str)> {
    header
        .split(';')
        .filter_map(|pair| {
            let (name, value) = pair.split_once('=')?;
            let (name, value) = (name.trim(), value.trim());

            (!name.is_empty()).then_some((name, value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::read_cookies;

    #[test]
    fn reads_the_names_and_values_a_header_states() {
        assert_eq!(read_cookies("session=abc; theme=dark"), [("session", "abc"), ("theme", "dark")]);
        // Space around a pair is spelling, and a pair stating no value states nothing.
        assert_eq!(read_cookies("  session = abc ;broken; theme=dark"), [("session", "abc"), ("theme", "dark")]);
        assert!(read_cookies("").is_empty());
    }
}
