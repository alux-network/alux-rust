use jsonrpsee::types::error::INVALID_PARAMS_CODE;
use jsonrpsee::types::params::ParamsSequence;
use jsonrpsee::types::{ErrorObjectOwned, Params};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

/// Parses the parameter product consumed by a JSON-RPC operation.
///
/// An argument a request leaves out reads as `null`, which only an optional argument accepts. A
/// positional array may therefore stop short of the product and a parameter object may omit a name:
/// both are how a caller says "absent" for an optional argument.
pub trait JsonrpseeArgs: Sized {
    /// Parses a positional JSON-RPC parameter array.
    ///
    /// # Errors
    ///
    /// Returns an invalid-parameters error when the JSON value does not match this argument product.
    fn from_positional(params: Params<'static>) -> Result<Self, ErrorObjectOwned>;

    /// Parses a named JSON-RPC parameter object using operation argument names.
    ///
    /// # Errors
    ///
    /// Returns an invalid-parameters error when a required named argument is absent or has the wrong
    /// type.
    fn from_named(params: Params<'static>, arg_names: &'static [&'static str]) -> Result<Self, ErrorObjectOwned>;
}

impl JsonrpseeArgs for () {
    fn from_positional(params: Params<'static>) -> Result<Self, ErrorObjectOwned> {
        match params.as_str() {
            None | Some("[]" | "null") => Ok(()),
            Some(_) => params.parse(),
        }
    }

    fn from_named(params: Params<'static>, _arg_names: &'static [&'static str]) -> Result<Self, ErrorObjectOwned> {
        match params.as_str() {
            None | Some("{}" | "null") => Ok(()),
            Some(_) => params.parse(),
        }
    }
}

fn invalid_params(message: impl Into<String>) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(INVALID_PARAMS_CODE, message.into(), None::<()>)
}

/// Reads the argument an absent parameter denotes, which only an optional argument accepts.
fn absent<Arg>() -> Option<Arg>
where
    Arg: DeserializeOwned,
{
    serde_json::from_value(Value::Null).ok()
}

/// Reads one argument of a positional array, by its place in the argument product.
fn positional_arg<Arg>(sequence: &mut ParamsSequence<'_>, index: usize) -> Result<Arg, ErrorObjectOwned>
where
    Arg: DeserializeOwned,
{
    match sequence.optional_next::<Arg>()? {
        Some(argument) => Ok(argument),
        None => absent().ok_or_else(|| invalid_params(format!("missing parameter at position {index}"))),
    }
}

/// Reads one argument of a parameter object, by the name the operation was authored with.
fn named_arg<Arg>(object: &mut Map<String, Value>, arg_names: &[&str], index: usize) -> Result<Arg, ErrorObjectOwned>
where
    Arg: DeserializeOwned,
{
    let name = arg_names.get(index).ok_or_else(|| invalid_params("operation argument metadata is incomplete"))?;
    match object.remove(*name) {
        Some(value) => serde_json::from_value(value).map_err(|error| invalid_params(error.to_string())),
        None => absent().ok_or_else(|| invalid_params(format!("missing named parameter `{name}`"))),
    }
}

macro_rules! jsonrpsee_args {
    ($($index:literal => $arg:ident),+ $(,)?) => {
        impl<$($arg),+> JsonrpseeArgs for ($($arg,)+)
        where
            $($arg: DeserializeOwned,)+
        {
            fn from_positional(params: Params<'static>) -> Result<Self, ErrorObjectOwned> {
                let mut sequence = params.sequence();

                Ok(($(positional_arg::<$arg>(&mut sequence, $index)?,)+))
            }

            fn from_named(
                params: Params<'static>,
                arg_names: &'static [&'static str],
            ) -> Result<Self, ErrorObjectOwned> {
                let mut object = params.parse::<Map<String, Value>>()?;
                Ok(($(named_arg::<$arg>(&mut object, arg_names, $index)?,)+))
            }
        }
    };
}

jsonrpsee_args!(0 => A1);
jsonrpsee_args!(0 => A1, 1 => A2);
jsonrpsee_args!(0 => A1, 1 => A2, 2 => A3);
jsonrpsee_args!(0 => A1, 1 => A2, 2 => A3, 3 => A4);
jsonrpsee_args!(0 => A1, 1 => A2, 2 => A3, 3 => A4, 4 => A5);
jsonrpsee_args!(0 => A1, 1 => A2, 2 => A3, 3 => A4, 4 => A5, 5 => A6);
jsonrpsee_args!(0 => A1, 1 => A2, 2 => A3, 3 => A4, 4 => A5, 5 => A6, 6 => A7);
jsonrpsee_args!(0 => A1, 1 => A2, 2 => A3, 3 => A4, 4 => A5, 5 => A6, 6 => A7, 7 => A8);

#[cfg(test)]
mod tests {
    use super::JsonrpseeArgs;
    use jsonrpsee::types::Params;
    use jsonrpsee::types::error::INVALID_PARAMS_CODE;

    fn params(raw: &'static str) -> Params<'static> {
        Params::new(Some(raw))
    }

    #[test]
    fn reads_a_complete_argument_product() {
        assert_eq!(<(u64, bool)>::from_positional(params("[7, true]")).unwrap(), (7, true));
        assert_eq!(<(u64, bool)>::from_named(params(r#"{"n": 7, "flag": true}"#), &["n", "flag"]).unwrap(), (7, true));
    }

    #[test]
    fn reads_an_omitted_optional_argument_as_absent() {
        assert_eq!(<(u64, Option<String>)>::from_positional(params("[7]")).unwrap(), (7, None));
        assert_eq!(<(u64, Option<String>)>::from_named(params(r#"{"n": 7}"#), &["n", "tag"]).unwrap(), (7, None));
    }

    #[test]
    fn reports_a_required_argument_the_request_omits() {
        let error = <(u64, bool)>::from_positional(params("[7]")).unwrap_err();
        assert_eq!(error.code(), INVALID_PARAMS_CODE);
        assert_eq!(error.message(), "missing parameter at position 1");

        let error = <(u64, bool)>::from_named(params(r#"{"n": 7}"#), &["n", "flag"]).unwrap_err();
        assert_eq!(error.code(), INVALID_PARAMS_CODE);
        assert_eq!(error.message(), "missing named parameter `flag`");
    }
}
