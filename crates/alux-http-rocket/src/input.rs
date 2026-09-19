//! Reads each handler argument from what Rocket routed here.

use crate::RocketRequest;
use alux_http::{FromPartsAlg, read_cookies, read_header_name};
use alux_http::{HttpErrorAlg, HttpStatus};
use alux_http_parts::ReadParts;
use core::error::Error;
use core::fmt::{self, Display};
use core::future::Future;
use core::marker::PhantomData;
use core::str::FromStr;
use rocket::http::HeaderMap;
use serde::de::DeserializeOwned;

/// Marks a value read from the segments a path bound.
pub struct RocketPathInput<Input>(PhantomData<Input>);
/// Marks a value read from the query string.
pub struct RocketQueryInput<Input>(PhantomData<Input>);
/// Marks a JSON value read from the request body.
pub struct RocketBodyInput<Input>(PhantomData<Input>);
/// Marks a form-encoded value read from the request body.
pub struct RocketFormInput<Input>(PhantomData<Input>);
/// Marks the request body taken as it arrived.
pub struct RocketRawBodyInput<Input>(PhantomData<Input>);
/// Marks a value read from the headers the caller sent.
pub struct RocketHeadInput<Input>(PhantomData<Input>);

/// States that an argument could not be read from where its role says it comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RocketError {
    message: String,
}

impl RocketError {
    fn unreadable(role: &str, reason: &str) -> Self {
        Self { message: format!("the {role} could not be read: {reason}") }
    }
}

impl HttpErrorAlg for RocketError {
    const HTTP_STATUSES: &'static [HttpStatus] = &[HttpStatus::BAD_REQUEST];

    fn http_status(&self) -> HttpStatus {
        HttpStatus::BAD_REQUEST
    }

    fn http_message(&self) -> String {
        self.message.clone()
    }
}

impl Display for RocketError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for RocketError {}

/// Reads the handler argument a path's captured segments state.
pub trait FromCapturedAlg: Sized {
    /// Reads the argument from the segments the path bound, in declaration order.
    fn from_captured(captures: &[String]) -> Option<Self>;
}

macro_rules! captured {
    ($($input:ty),+ $(,)?) => {
        $(
            impl FromCapturedAlg for $input {
                fn from_captured(captures: &[String]) -> Option<Self> {
                    captures.first()?.parse().ok()
                }
            }
        )+
    };
}

captured!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, bool, char, String);

macro_rules! captured_product {
    ($($input:ident => $index:tt),+ $(,)?) => {
        impl<$($input),+> FromCapturedAlg for ($($input,)+)
        where
            $($input: FromStr,)+
        {
            fn from_captured(captures: &[String]) -> Option<Self> {
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
pub trait FromRawAlg: Sized {
    /// Reads the argument from the bytes the caller sent.
    fn from_raw(body: &[u8]) -> Option<Self>;
}

impl FromRawAlg for Vec<u8> {
    fn from_raw(body: &[u8]) -> Option<Self> {
        Some(body.to_vec())
    }
}

impl FromRawAlg for String {
    fn from_raw(body: &[u8]) -> Option<Self> {
        Self::from_utf8(body.to_vec()).ok()
    }
}

/// Reads the handler argument the headers state.
pub trait FromHeadersAlg: Sized {
    /// Reads the argument from the headers the caller sent.
    fn from_headers(headers: &HeaderMap<'static>) -> Option<Self>;
}

impl FromHeadersAlg for HeaderMap<'static> {
    fn from_headers(headers: &HeaderMap<'static>) -> Option<Self> {
        Some(headers.clone())
    }
}

/// Reads one handler argument out of what Rocket routed here.
pub(crate) trait RocketInputAlg<Output> {
    fn extract(request: &RocketRequest) -> impl Future<Output = Result<Output, RocketError>> + Send;
}

impl<Input> RocketInputAlg<Input> for RocketPathInput<Input>
where
    Input: FromCapturedAlg,
{
    async fn extract(request: &RocketRequest) -> Result<Input, RocketError> {
        Input::from_captured(request.captures())
            .ok_or_else(|| RocketError::unreadable("path", "it states something else"))
    }
}

impl<Input> RocketInputAlg<Input> for RocketQueryInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &RocketRequest) -> Result<Input, RocketError> {
        serde_urlencoded::from_str(request.query())
            .map_err(|error| RocketError::unreadable("query", &error.to_string()))
    }
}

impl<Input> RocketInputAlg<Input> for RocketBodyInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &RocketRequest) -> Result<Input, RocketError> {
        serde_json::from_slice(request.body()).map_err(|error| RocketError::unreadable("body", &error.to_string()))
    }
}

impl<Input> RocketInputAlg<Input> for RocketFormInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &RocketRequest) -> Result<Input, RocketError> {
        serde_urlencoded::from_bytes(request.body())
            .map_err(|error| RocketError::unreadable("form", &error.to_string()))
    }
}

impl<Input> RocketInputAlg<Input> for RocketRawBodyInput<Input>
where
    Input: FromRawAlg,
{
    async fn extract(request: &RocketRequest) -> Result<Input, RocketError> {
        Input::from_raw(request.body()).ok_or_else(|| RocketError::unreadable("body", "it states something else"))
    }
}

impl<Input> RocketInputAlg<Input> for RocketHeadInput<Input>
where
    Input: FromHeadersAlg,
{
    async fn extract(request: &RocketRequest) -> Result<Input, RocketError> {
        Input::from_headers(request.headers())
            .ok_or_else(|| RocketError::unreadable("headers", "they state something else"))
    }
}

/// Marks a value read from the cookies a caller sent.
pub struct RocketCookieInput<Input>(PhantomData<Input>);

/// Marks a value read from the headers a caller sent.
pub struct RocketHeaderInput<Input>(PhantomData<Input>);

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

impl<Input> RocketInputAlg<Input> for RocketCookieInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &RocketRequest) -> Result<Input, RocketError> {
        let header = request.headers().get_one("cookie");

        cookies_of(header).map_err(|error| RocketError::unreadable("cookies", &error))
    }
}

impl<Input> RocketInputAlg<Input> for RocketHeaderInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &RocketRequest) -> Result<Input, RocketError> {
        let stated = request.headers().iter().map(|header| (header.name().to_string(), header.value().to_owned()));

        headers_of(stated).map_err(|error| RocketError::unreadable("headers", &error))
    }
}

/// Marks an argument read from a body arriving as parts.
pub struct RocketMultipartInput<Input>(PhantomData<Input>);

impl<Input> RocketInputAlg<Input> for RocketMultipartInput<Input>
where
    Input: FromPartsAlg<ReadParts> + Send,
    Input::Error: Display,
{
    async fn extract(request: &RocketRequest) -> Result<Input, RocketError> {
        let media_type = request
            .headers()
            .get_one("content-type")
            .ok_or_else(|| RocketError::unreadable("parts", "the body states no media type"))?;
        let parts = ReadParts::new(media_type, request.body().to_vec())
            .map_err(|error| RocketError::unreadable("parts", &error.to_string()))?;

        Input::from_parts(parts).await.map_err(|error| RocketError::unreadable("parts", &error.to_string()))
    }
}

/// Reads the whole argument product one endpoint states, in declaration order.
pub(crate) trait RocketInputsAlg<Outputs> {
    fn extract(request: &RocketRequest) -> impl Future<Output = Result<Outputs, RocketError>> + Send;
}

impl RocketInputsAlg<()> for () {
    async fn extract(_request: &RocketRequest) -> Result<(), RocketError> {
        Ok(())
    }
}

macro_rules! rocket_inputs {
    ($($input:ident => $output:ident),+ $(,)?) => {
        impl<$($input, $output),+> RocketInputsAlg<($($output,)+)> for ($($input,)+)
        where
            $($input: RocketInputAlg<$output>, $output: Send,)+
        {
            async fn extract(request: &RocketRequest) -> Result<($($output,)+), RocketError> {
                Ok(($($input::extract(request).await?,)+))
            }
        }
    };
}

rocket_inputs!(I1 => O1);
rocket_inputs!(I1 => O1, I2 => O2);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15);
rocket_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15, I16 => O16);
