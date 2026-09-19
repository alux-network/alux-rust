//! Reads each handler argument from where its role says it comes from.
//!
//! Nothing here awaits anything. A request has already arrived in full, so reading an argument out
//! of it is a function, and only applying the operation is asynchronous.

use crate::{DirectError, DirectRequest};
use alux_http::{FromPartsAlg, read_cookies, read_header_name};
use alux_http_parts::ReadParts;
use core::fmt::Display;
use core::future::Future;
use core::marker::PhantomData;
use core::str::FromStr;
use serde::de::DeserializeOwned;

/// Marks a value read from the segments a path bound.
pub struct DirectPathInput<Input>(PhantomData<Input>);
/// Marks a value read from the query string.
pub struct DirectQueryInput<Input>(PhantomData<Input>);
/// Marks a JSON value read from the request body.
pub struct DirectBodyInput<Input>(PhantomData<Input>);
/// Marks a form-encoded value read from the request body.
pub struct DirectFormInput<Input>(PhantomData<Input>);
/// Marks the request body taken as it arrived.
pub struct DirectRawBodyInput<Input>(PhantomData<Input>);
/// Marks a value read from the request head.
pub struct DirectHeadInput<Input>(PhantomData<Input>);

/// Reads the handler argument a path's captured segments state.
pub trait FromCapturesAlg: Sized {
    /// Reads the argument from the segments the path bound, in declaration order.
    fn from_captures(captures: &[String]) -> Option<Self>;
}

macro_rules! captured {
    ($($input:ty),+ $(,)?) => {
        $(
            impl FromCapturesAlg for $input {
                fn from_captures(captures: &[String]) -> Option<Self> {
                    captures.first()?.parse().ok()
                }
            }
        )+
    };
}

captured!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, bool, char, String);

macro_rules! captured_product {
    ($($input:ident => $index:tt),+ $(,)?) => {
        impl<$($input),+> FromCapturesAlg for ($($input,)+)
        where
            $($input: FromStr,)+
        {
            fn from_captures(captures: &[String]) -> Option<Self> {
                Some(($(captures.get($index)?.parse::<$input>().ok()?,)+))
            }
        }
    };
}

captured_product!(A => 0, B => 1);
captured_product!(A => 0, B => 1, C => 2);
captured_product!(A => 0, B => 1, C => 2, D => 3);
captured_product!(A => 0, B => 1, C => 2, D => 3, E => 4);
captured_product!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5);
captured_product!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6);
captured_product!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7);
captured_product!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8);
captured_product!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9);
captured_product!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10);
captured_product!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11);
captured_product!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12);
captured_product!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13);
captured_product!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14);
captured_product!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7, I => 8, J => 9, K => 10, L => 11, M => 12, N => 13, O => 14, P => 15);

/// Reads the handler argument a request body states, taken as it arrived.
pub trait FromBodyAlg: Sized {
    /// Reads the argument from the bytes the caller sent.
    fn from_body(body: &[u8]) -> Option<Self>;
}

impl FromBodyAlg for Vec<u8> {
    fn from_body(body: &[u8]) -> Option<Self> {
        Some(body.to_vec())
    }
}

impl FromBodyAlg for String {
    fn from_body(body: &[u8]) -> Option<Self> {
        Self::from_utf8(body.to_vec()).ok()
    }
}

/// Reads the handler argument a request head states.
pub trait FromHeadAlg: Sized {
    /// Reads the argument from everything that arrived before the body.
    fn from_head(request: &DirectRequest) -> Option<Self>;
}

impl FromHeadAlg for DirectRequest {
    fn from_head(request: &DirectRequest) -> Option<Self> {
        Some(request.clone())
    }
}

/// Reads one handler argument out of a request that has already arrived.
pub(crate) trait DirectInputAlg<Output> {
    fn extract(
        request: &DirectRequest,
        captures: &[String],
    ) -> impl Future<Output = Result<Output, DirectError>> + Send;
}

impl<Input> DirectInputAlg<Input> for DirectPathInput<Input>
where
    Input: FromCapturesAlg,
{
    async fn extract(_request: &DirectRequest, captures: &[String]) -> Result<Input, DirectError> {
        Input::from_captures(captures).ok_or_else(|| DirectError::unreadable("path", "it states something else"))
    }
}

impl<Input> DirectInputAlg<Input> for DirectQueryInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &DirectRequest, _captures: &[String]) -> Result<Input, DirectError> {
        serde_urlencoded::from_str(request.query())
            .map_err(|error| DirectError::unreadable("query", &error.to_string()))
    }
}

impl<Input> DirectInputAlg<Input> for DirectBodyInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &DirectRequest, _captures: &[String]) -> Result<Input, DirectError> {
        serde_json::from_slice(request.body()).map_err(|error| DirectError::unreadable("body", &error.to_string()))
    }
}

impl<Input> DirectInputAlg<Input> for DirectFormInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &DirectRequest, _captures: &[String]) -> Result<Input, DirectError> {
        serde_urlencoded::from_bytes(request.body())
            .map_err(|error| DirectError::unreadable("form", &error.to_string()))
    }
}

impl<Input> DirectInputAlg<Input> for DirectRawBodyInput<Input>
where
    Input: FromBodyAlg,
{
    async fn extract(request: &DirectRequest, _captures: &[String]) -> Result<Input, DirectError> {
        Input::from_body(request.body()).ok_or_else(|| DirectError::unreadable("body", "it states something else"))
    }
}

impl<Input> DirectInputAlg<Input> for DirectHeadInput<Input>
where
    Input: FromHeadAlg,
{
    async fn extract(request: &DirectRequest, _captures: &[String]) -> Result<Input, DirectError> {
        Input::from_head(request).ok_or_else(|| DirectError::unreadable("head", "it states something else"))
    }
}

/// Marks a value read from the cookies a caller sent.
pub struct DirectCookieInput<Input>(PhantomData<Input>);

/// Marks a value read from the headers a caller sent.
pub struct DirectHeaderInput<Input>(PhantomData<Input>);

/// Reads the headers a caller sent into the argument an author asked for.
///
/// Headers are names and values, so what reads them is what reads any other name-and-value product.
/// A framework's own extractor is what the endpoint-context role states instead.
fn headers_of<Input>(stated: impl Iterator<Item = (String, String)>) -> Result<Input, String>
where
    Input: DeserializeOwned,
{
    let named = stated.map(|(name, value)| (read_header_name(&name), value)).collect::<Vec<_>>();
    let encoded = serde_urlencoded::to_string(&named).map_err(|error| error.to_string())?;

    serde_urlencoded::from_str(&encoded).map_err(|error| error.to_string())
}

/// Reads the cookies a header states into the argument an author asked for.
///
/// Cookies are names and values, so what reads them is what reads any other name-and-value product.
fn cookies_of<Input>(header: Option<&str>) -> Result<Input, String>
where
    Input: DeserializeOwned,
{
    let stated = read_cookies(header.unwrap_or_default());
    let encoded = serde_urlencoded::to_string(&stated).map_err(|error| error.to_string())?;

    serde_urlencoded::from_str(&encoded).map_err(|error| error.to_string())
}

impl<Input> DirectInputAlg<Input> for DirectCookieInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &DirectRequest, _captures: &[String]) -> Result<Input, DirectError> {
        cookies_of(request.header("cookie")).map_err(|error| DirectError::unreadable("cookies", &error))
    }
}

impl<Input> DirectInputAlg<Input> for DirectHeaderInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &DirectRequest, _captures: &[String]) -> Result<Input, DirectError> {
        let stated = request.headers().map(|(name, value)| (name.to_owned(), value.to_owned()));

        headers_of(stated).map_err(|error| DirectError::unreadable("headers", &error))
    }
}

/// Marks an argument read from a body arriving as parts.
pub struct DirectMultipartInput<Input>(PhantomData<Input>);

impl<Input> DirectInputAlg<Input> for DirectMultipartInput<Input>
where
    Input: FromPartsAlg<ReadParts> + Send,
    Input::Error: Display,
{
    async fn extract(request: &DirectRequest, _captures: &[String]) -> Result<Input, DirectError> {
        let media_type = request
            .header("content-type")
            .ok_or_else(|| DirectError::unreadable("parts", "the body states no media type"))?;
        let parts = ReadParts::new(media_type, request.body().to_vec())
            .map_err(|error| DirectError::unreadable("parts", &error.to_string()))?;

        Input::from_parts(parts).await.map_err(|error| DirectError::unreadable("parts", &error.to_string()))
    }
}

/// Reads the whole argument product one endpoint states, in declaration order.
pub(crate) trait DirectInputsAlg<Outputs> {
    fn extract(
        request: &DirectRequest,
        captures: &[String],
    ) -> impl Future<Output = Result<Outputs, DirectError>> + Send;
}

impl DirectInputsAlg<()> for () {
    async fn extract(_request: &DirectRequest, _captures: &[String]) -> Result<(), DirectError> {
        Ok(())
    }
}

macro_rules! direct_inputs {
    ($($input:ident => $output:ident),+ $(,)?) => {
        impl<$($input, $output),+> DirectInputsAlg<($($output,)+)> for ($($input,)+)
        where
            $($input: DirectInputAlg<$output>, $output: Send,)+
        {
            async fn extract(request: &DirectRequest, captures: &[String]) -> Result<($($output,)+), DirectError> {
                Ok(($($input::extract(request, captures).await?,)+))
            }
        }
    };
}

direct_inputs!(I1 => O1);
direct_inputs!(I1 => O1, I2 => O2);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15);
direct_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15, I16 => O16);
