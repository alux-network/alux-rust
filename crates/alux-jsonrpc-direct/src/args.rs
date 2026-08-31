use crate::error::{INVALID_PARAMS, RpcError};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::vec;

/// Parses the parameter product consumed by a JSON-RPC operation.
///
/// A parameter a request leaves out, or supplies as `null`, reads as absent, which only an optional
/// argument accepts. A positional array may therefore stop short of the product and a parameter
/// object may omit a name: both are how a caller says "absent" for an optional argument.
pub trait DirectArgs: Sized {
    /// Parses a positional JSON-RPC parameter array.
    ///
    /// # Errors
    ///
    /// Answers with an invalid-parameters error when the value does not match this argument product.
    fn from_positional(params: Option<Value>) -> Result<Self, RpcError>;

    /// Parses a named JSON-RPC parameter object using operation argument names.
    ///
    /// # Errors
    ///
    /// Answers with an invalid-parameters error when a required named argument is absent or has the
    /// wrong type.
    fn from_named(params: Option<Value>, arg_names: &'static [&'static str]) -> Result<Self, RpcError>;
}

/// States that a parameter the method cannot read was supplied.
fn invalid_params(message: impl Into<String>) -> RpcError {
    RpcError::new(INVALID_PARAMS, message)
}

/// Reads the argument an absent parameter denotes, which only an optional argument accepts.
fn absent<Arg>() -> Option<Arg>
where
    Arg: DeserializeOwned,
{
    serde_json::from_value(Value::Null).ok()
}

/// Reads the positional array a request supplies, treating an absent `params` member as empty.
fn positional(params: Option<Value>) -> Result<vec::IntoIter<Value>, RpcError> {
    match params {
        None | Some(Value::Null) => Ok(Vec::new().into_iter()),
        Some(Value::Array(values)) => Ok(values.into_iter()),
        Some(_) => Err(invalid_params("expected a parameter array")),
    }
}

/// Reads the parameter object a request supplies, treating an absent `params` member as empty.
fn named(params: Option<Value>) -> Result<Map<String, Value>, RpcError> {
    match params {
        None | Some(Value::Null) => Ok(Map::new()),
        Some(Value::Object(members)) => Ok(members),
        Some(_) => Err(invalid_params("expected a parameter object")),
    }
}

/// Reads one argument of a positional array, by its place in the argument product.
fn positional_arg<Arg>(values: &mut vec::IntoIter<Value>, index: usize) -> Result<Arg, RpcError>
where
    Arg: DeserializeOwned,
{
    match values.next() {
        Some(Value::Null) | None => {
            absent().ok_or_else(|| invalid_params(format!("missing parameter at position {index}")))
        }
        Some(value) => serde_json::from_value(value).map_err(|error| invalid_params(error.to_string())),
    }
}

/// Reads one argument of a parameter object, by the name the operation was authored with.
fn named_arg<Arg>(members: &mut Map<String, Value>, arg_names: &[&str], index: usize) -> Result<Arg, RpcError>
where
    Arg: DeserializeOwned,
{
    let name = arg_names.get(index).ok_or_else(|| invalid_params("operation argument metadata is incomplete"))?;
    match members.remove(*name) {
        Some(Value::Null) | None => absent().ok_or_else(|| invalid_params(format!("missing named parameter `{name}`"))),
        Some(value) => serde_json::from_value(value).map_err(|error| invalid_params(error.to_string())),
    }
}

impl DirectArgs for () {
    fn from_positional(params: Option<Value>) -> Result<Self, RpcError> {
        match positional(params)?.len() {
            0 => Ok(()),
            _ => Err(invalid_params("expected no parameters")),
        }
    }

    fn from_named(params: Option<Value>, _arg_names: &'static [&'static str]) -> Result<Self, RpcError> {
        if named(params)?.is_empty() { Ok(()) } else { Err(invalid_params("expected no parameters")) }
    }
}

macro_rules! direct_args {
    ($($index:literal => $arg:ident),+ $(,)?) => {
        impl<$($arg),+> DirectArgs for ($($arg,)+)
        where
            $($arg: DeserializeOwned,)+
        {
            fn from_positional(params: Option<Value>) -> Result<Self, RpcError> {
                let mut values = positional(params)?;

                Ok(($(positional_arg::<$arg>(&mut values, $index)?,)+))
            }

            fn from_named(params: Option<Value>, arg_names: &'static [&'static str]) -> Result<Self, RpcError> {
                let mut members = named(params)?;

                Ok(($(named_arg::<$arg>(&mut members, arg_names, $index)?,)+))
            }
        }
    };
}

direct_args!(0 => A1);
direct_args!(0 => A1, 1 => A2);
direct_args!(0 => A1, 1 => A2, 2 => A3);
direct_args!(0 => A1, 1 => A2, 2 => A3, 3 => A4);
direct_args!(0 => A1, 1 => A2, 2 => A3, 3 => A4, 4 => A5);
direct_args!(0 => A1, 1 => A2, 2 => A3, 3 => A4, 4 => A5, 5 => A6);
direct_args!(0 => A1, 1 => A2, 2 => A3, 3 => A4, 4 => A5, 5 => A6, 6 => A7);
direct_args!(0 => A1, 1 => A2, 2 => A3, 3 => A4, 4 => A5, 5 => A6, 6 => A7, 7 => A8);

#[cfg(test)]
mod tests {
    use super::DirectArgs;
    use crate::error::INVALID_PARAMS;
    use serde_json::json;

    #[test]
    fn reads_a_complete_argument_product() {
        assert_eq!(<(u64, bool)>::from_positional(Some(json!([7, true]))).unwrap(), (7, true));
        assert_eq!(<(u64, bool)>::from_named(Some(json!({"n": 7, "flag": true})), &["n", "flag"]).unwrap(), (7, true));
    }

    #[test]
    fn reads_an_omitted_optional_argument_as_absent() {
        assert_eq!(<(u64, Option<String>)>::from_positional(Some(json!([7]))).unwrap(), (7, None));
        assert_eq!(<(u64, Option<String>)>::from_named(Some(json!({"n": 7})), &["n", "tag"]).unwrap(), (7, None));
    }

    #[test]
    fn reports_a_required_argument_the_request_omits() {
        let error = <(u64, bool)>::from_positional(Some(json!([7]))).unwrap_err();
        assert_eq!(error.code, INVALID_PARAMS);
        assert_eq!(error.message, "missing parameter at position 1");

        let error = <(u64, bool)>::from_named(Some(json!({"n": 7})), &["n", "flag"]).unwrap_err();
        assert_eq!(error.code, INVALID_PARAMS);
        assert_eq!(error.message, "missing named parameter `flag`");
    }

    #[test]
    fn reports_a_parameter_shape_the_mode_cannot_read() {
        let error = <(u64,)>::from_positional(Some(json!({"n": 7}))).unwrap_err();
        assert_eq!(error.message, "expected a parameter array");

        let error = <(u64,)>::from_named(Some(json!([7])), &["n"]).unwrap_err();
        assert_eq!(error.message, "expected a parameter object");
    }

    #[test]
    fn reads_no_parameters_for_the_empty_product() {
        assert_eq!(<()>::from_positional(None).unwrap(), ());
        assert_eq!(<()>::from_positional(Some(json!([]))).unwrap(), ());
        assert_eq!(<()>::from_named(Some(json!({})), &[]).unwrap(), ());
        assert_eq!(<()>::from_positional(Some(json!([1]))).unwrap_err().message, "expected no parameters");
    }
}
