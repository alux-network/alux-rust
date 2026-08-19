use jsonrpsee::types::error::INVALID_PARAMS_CODE;
use jsonrpsee::types::{ErrorObjectOwned, Params};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

/// Parses the parameter product consumed by a JSON-RPC operation.
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
    /// Returns an invalid-parameters error when a named argument is absent or has the wrong type.
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

fn named_arg<Arg>(object: &mut Map<String, Value>, arg_names: &[&str], index: usize) -> Result<Arg, ErrorObjectOwned>
where
    Arg: DeserializeOwned,
{
    let name = arg_names.get(index).ok_or_else(|| invalid_params("operation argument metadata is incomplete"))?;
    let value = object.remove(*name).ok_or_else(|| invalid_params(format!("missing named parameter `{name}`")))?;
    serde_json::from_value(value).map_err(|error| invalid_params(error.to_string()))
}

macro_rules! jsonrpsee_args {
    ($($index:literal => $arg:ident),+ $(,)?) => {
        impl<$($arg),+> JsonrpseeArgs for ($($arg,)+)
        where
            $($arg: DeserializeOwned,)+
        {
            fn from_positional(params: Params<'static>) -> Result<Self, ErrorObjectOwned> {
                params.parse()
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
