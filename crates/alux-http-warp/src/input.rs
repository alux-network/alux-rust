//! Reads each handler argument from what warp's filters gathered.

use crate::WarpRequest;
use alux_http::{FromPartsAlg, read_cookies, read_header_name};
use alux_http::{HttpErrorAlg, HttpStatus};
use alux_http_parts::ReadParts;
use core::error::Error;
use core::fmt::{self, Display};
use core::future::Future;
use core::marker::PhantomData;
use core::str::FromStr;
use serde::de::DeserializeOwned;
use warp::http::HeaderMap;
use warp::http::header;

/// Marks a value read from the segments a path bound.
pub struct WarpPathInput<Input>(PhantomData<Input>);
/// Marks a value read from the query string.
pub struct WarpQueryInput<Input>(PhantomData<Input>);
/// Marks a JSON value read from the request body.
pub struct WarpBodyInput<Input>(PhantomData<Input>);
/// Marks a form-encoded value read from the request body.
pub struct WarpFormInput<Input>(PhantomData<Input>);
/// Marks the request body taken as it arrived.
pub struct WarpRawBodyInput<Input>(PhantomData<Input>);
/// Marks a value read from the headers the caller sent.
pub struct WarpHeadInput<Input>(PhantomData<Input>);

/// States that an argument could not be read from where its role says it comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarpError {
    message: String,
}

impl WarpError {
    fn unreadable(role: &str, reason: &str) -> Self {
        Self { message: format!("the {role} could not be read: {reason}") }
    }
}

impl HttpErrorAlg for WarpError {
    const HTTP_STATUSES: &'static [HttpStatus] = &[HttpStatus::BAD_REQUEST];

    fn http_status(&self) -> HttpStatus {
        HttpStatus::BAD_REQUEST
    }

    fn http_message(&self) -> String {
        self.message.clone()
    }
}

impl Display for WarpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for WarpError {}

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
    fn from_headers(headers: &HeaderMap) -> Option<Self>;
}

impl FromHeadersAlg for HeaderMap {
    fn from_headers(headers: &HeaderMap) -> Option<Self> {
        Some(headers.clone())
    }
}

/// Reads one handler argument out of what warp's filters gathered.
pub(crate) trait WarpInputAlg<Output> {
    fn extract(request: &WarpRequest) -> impl Future<Output = Result<Output, WarpError>> + Send;
}

impl<Input> WarpInputAlg<Input> for WarpPathInput<Input>
where
    Input: FromCapturedAlg,
{
    async fn extract(request: &WarpRequest) -> Result<Input, WarpError> {
        Input::from_captured(request.captures())
            .ok_or_else(|| WarpError::unreadable("path", "it states something else"))
    }
}

impl<Input> WarpInputAlg<Input> for WarpQueryInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &WarpRequest) -> Result<Input, WarpError> {
        serde_urlencoded::from_str(request.query()).map_err(|error| WarpError::unreadable("query", &error.to_string()))
    }
}

impl<Input> WarpInputAlg<Input> for WarpBodyInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &WarpRequest) -> Result<Input, WarpError> {
        serde_json::from_slice(request.body()).map_err(|error| WarpError::unreadable("body", &error.to_string()))
    }
}

impl<Input> WarpInputAlg<Input> for WarpFormInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &WarpRequest) -> Result<Input, WarpError> {
        serde_urlencoded::from_bytes(request.body()).map_err(|error| WarpError::unreadable("form", &error.to_string()))
    }
}

impl<Input> WarpInputAlg<Input> for WarpRawBodyInput<Input>
where
    Input: FromRawAlg,
{
    async fn extract(request: &WarpRequest) -> Result<Input, WarpError> {
        Input::from_raw(request.body()).ok_or_else(|| WarpError::unreadable("body", "it states something else"))
    }
}

impl<Input> WarpInputAlg<Input> for WarpHeadInput<Input>
where
    Input: FromHeadersAlg,
{
    async fn extract(request: &WarpRequest) -> Result<Input, WarpError> {
        Input::from_headers(request.headers())
            .ok_or_else(|| WarpError::unreadable("headers", "they state something else"))
    }
}

/// Marks a value read from the cookies a caller sent.
pub struct WarpCookieInput<Input>(PhantomData<Input>);

/// Marks a value read from the headers a caller sent.
pub struct WarpHeaderInput<Input>(PhantomData<Input>);

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

impl<Input> WarpInputAlg<Input> for WarpCookieInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &WarpRequest) -> Result<Input, WarpError> {
        let header = request.headers().get(header::COOKIE).and_then(|value| value.to_str().ok());

        cookies_of(header).map_err(|error| WarpError::unreadable("cookies", &error))
    }
}

impl<Input> WarpInputAlg<Input> for WarpHeaderInput<Input>
where
    Input: DeserializeOwned,
{
    async fn extract(request: &WarpRequest) -> Result<Input, WarpError> {
        let stated = request
            .headers()
            .iter()
            .filter_map(|(name, value)| Some((name.to_string(), value.to_str().ok()?.to_owned())));

        headers_of(stated).map_err(|error| WarpError::unreadable("headers", &error))
    }
}

/// Marks an argument read from a body arriving as parts.
pub struct WarpMultipartInput<Input>(PhantomData<Input>);

impl<Input> WarpInputAlg<Input> for WarpMultipartInput<Input>
where
    Input: FromPartsAlg<ReadParts> + Send,
    Input::Error: Display,
{
    async fn extract(request: &WarpRequest) -> Result<Input, WarpError> {
        let media_type = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| WarpError::unreadable("parts", "the body states no media type"))?;
        let parts = ReadParts::new(media_type, request.body().to_vec())
            .map_err(|error| WarpError::unreadable("parts", &error.to_string()))?;

        Input::from_parts(parts).await.map_err(|error| WarpError::unreadable("parts", &error.to_string()))
    }
}

/// Reads the whole argument product one endpoint states, in declaration order.
pub(crate) trait WarpInputsAlg<Outputs> {
    fn extract(request: &WarpRequest) -> impl Future<Output = Result<Outputs, WarpError>> + Send;
}

impl WarpInputsAlg<()> for () {
    async fn extract(_request: &WarpRequest) -> Result<(), WarpError> {
        Ok(())
    }
}

macro_rules! warp_inputs {
    ($($input:ident => $output:ident),+ $(,)?) => {
        impl<$($input, $output),+> WarpInputsAlg<($($output,)+)> for ($($input,)+)
        where
            $($input: WarpInputAlg<$output>, $output: Send,)+
        {
            async fn extract(request: &WarpRequest) -> Result<($($output,)+), WarpError> {
                Ok(($($input::extract(request).await?,)+))
            }
        }
    };
}

warp_inputs!(I1 => O1);
warp_inputs!(I1 => O1, I2 => O2);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15);
warp_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15, I16 => O16);
