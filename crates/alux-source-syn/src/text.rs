//! Syntax as text: what a fact keeps of a type or a path.

use quote::ToTokens;

/// `tokens` as the source would write them: a space only between two words.
pub(crate) fn text(tokens: &impl ToTokens) -> String {
    let spaced = tokens.to_token_stream().to_string();
    let mut out = String::with_capacity(spaced.len());
    let chars: Vec<char> = spaced.chars().collect();
    for (at, &character) in chars.iter().enumerate() {
        if character == ' ' {
            let word = |c: Option<&char>| c.is_some_and(|c| c.is_alphanumeric() || *c == '_' || *c == '\'');
            if !(word(out.chars().last().as_ref()) && word(chars.get(at + 1))) {
                continue;
            }
        }
        out.push(character);
    }
    out
}
