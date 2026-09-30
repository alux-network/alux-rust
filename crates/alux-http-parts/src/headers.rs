//! Writes the headers a named product states, which is reading headers into a product in reverse.

use alux_http::write_header_name;
use serde::Serialize;
use serde_json::Value;

/// Writes each member of `headers` as the header its member name states, in member-name order.
///
/// A member stating nothing writes no header, a member stating many values writes one header for
/// each, and anything else is written as its text.
///
/// # Errors
///
/// Answers with why `headers` is not a product of named values.
pub fn write_headers<Headers>(headers: &Headers) -> Result<Vec<(String, String)>, String>
where
    Headers: Serialize,
{
    let Value::Object(members) = serde_json::to_value(headers).map_err(|error| error.to_string())? else {
        return Err("headers are written from a product of named values".to_owned());
    };

    // Sorted here rather than left to the map, whose order a feature elsewhere can change.
    let mut members = members.into_iter().collect::<Vec<_>>();
    members.sort_by(|(left, _), (right, _)| left.cmp(right));

    Ok(members
        .into_iter()
        .flat_map(|(name, value)| {
            let name = write_header_name(&name);
            let values = match value {
                Value::Null => Vec::new(),
                Value::Array(values) => values,
                value => vec![value],
            };

            values.into_iter().filter(|value| !value.is_null()).map(move |value| (name.clone(), written(value)))
        })
        .collect())
}

/// Writes one value as the text a header carries.
fn written(value: Value) -> String {
    match value {
        Value::String(text) => text,
        value => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::write_headers;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Stated {
        cache_control: String,
        etag: Option<String>,
        set_cookie: Vec<String>,
        age: u32,
    }

    #[test]
    fn writes_each_member_as_the_header_its_name_states() {
        let stated = Stated {
            cache_control: "max-age=60".to_owned(),
            etag: None,
            set_cookie: vec!["a=1".to_owned(), "b=2".to_owned()],
            age: 3,
        };

        assert_eq!(
            write_headers(&stated).unwrap(),
            [
                ("age".to_owned(), "3".to_owned()),
                ("cache-control".to_owned(), "max-age=60".to_owned()),
                ("set-cookie".to_owned(), "a=1".to_owned()),
                ("set-cookie".to_owned(), "b=2".to_owned()),
            ]
        );
    }

    #[test]
    fn writes_nothing_but_a_product() {
        assert!(write_headers(&7).is_err());
    }
}
