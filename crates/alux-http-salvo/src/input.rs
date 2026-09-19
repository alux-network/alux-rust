//! Reads each handler argument with what Salvo states for its role.

use alux_http::{FromPartsAlg, read_cookies, read_header_name};
use alux_http_parts::ReadParts;
use core::fmt::Display;
use core::future::Future;
use core::marker::PhantomData;
use salvo::Request;
use salvo::http::header;
use salvo::http::{HeaderMap, StatusError};
use serde::de::DeserializeOwned;

/// Marks a value read from the segments a path bound.
pub struct SalvoPathInput<Input>(PhantomData<Input>);
/// Marks a value read from the query string.
pub struct SalvoQueryInput<Input>(PhantomData<Input>);
/// Marks a JSON value read from the request body.
pub struct SalvoBodyInput<Input>(PhantomData<Input>);
/// Marks a form-encoded value read from the request body.
pub struct SalvoFormInput<Input>(PhantomData<Input>);
/// Marks the request body taken as it arrived.
pub struct SalvoRawBodyInput<Input>(PhantomData<Input>);
/// Marks a value read from the request head.
pub struct SalvoHeadInput<Input>(PhantomData<Input>);

/// Reads the handler argument the request head states.
pub trait FromHeadersAlg: Sized {
    /// Reads the argument from the headers the caller sent.
    fn from_headers(headers: &HeaderMap) -> Option<Self>;
}

impl FromHeadersAlg for HeaderMap {
    fn from_headers(headers: &HeaderMap) -> Option<Self> {
        Some(headers.clone())
    }
}

/// Reads the handler argument a request body states, taken as it arrived.
pub trait FromPayloadAlg: Sized {
    /// Reads the argument from the bytes the caller sent.
    fn from_payload(payload: &[u8]) -> Option<Self>;
}

impl FromPayloadAlg for Vec<u8> {
    fn from_payload(payload: &[u8]) -> Option<Self> {
        Some(payload.to_vec())
    }
}

impl FromPayloadAlg for String {
    fn from_payload(payload: &[u8]) -> Option<Self> {
        Self::from_utf8(payload.to_vec()).ok()
    }
}

/// Reads one handler argument from a request Salvo is answering.
pub(crate) trait SalvoInputAlg<Output> {
    fn extract(request: &mut Request) -> impl Future<Output = Result<Output, StatusError>> + Send;
}

/// States that an argument could not be read from where its role says it comes from.
fn unreadable(role: &str, reason: &str) -> StatusError {
    StatusError::bad_request().brief(format!("the {role} could not be read: {reason}"))
}

macro_rules! salvo_parsed {
    ($marker:ident, $parse:ident, $role:literal) => {
        impl<Input> SalvoInputAlg<Input> for $marker<Input>
        where
            Input: DeserializeOwned + Send,
        {
            async fn extract(request: &mut Request) -> Result<Input, StatusError> {
                request.$parse().map_err(|error| unreadable($role, &error.to_string()))
            }
        }
    };
}

salvo_parsed!(SalvoQueryInput, parse_queries, "query");

impl<Input> SalvoInputAlg<Input> for SalvoPathInput<Input>
where
    Input: DeserializeOwned + Send,
{
    async fn extract(request: &mut Request) -> Result<Input, StatusError> {
        // Salvo reads what a path bound as a map of every name it bound. A path binding one segment
        // states that segment and not a map of one, so the single binding is read as itself, which
        // is what every other router's path extractor already does.
        let bound = request.params().keys().cloned().collect::<Vec<_>>();
        if let [only] = bound.as_slice()
            && let Some(value) = request.param::<Input>(only)
        {
            return Ok(value);
        }

        request.parse_params().map_err(|error| unreadable("path", &error.to_string()))
    }
}

macro_rules! salvo_awaited {
    ($marker:ident, $parse:ident, $role:literal) => {
        impl<Input> SalvoInputAlg<Input> for $marker<Input>
        where
            Input: DeserializeOwned + Send,
        {
            async fn extract(request: &mut Request) -> Result<Input, StatusError> {
                request.$parse().await.map_err(|error| unreadable($role, &error.to_string()))
            }
        }
    };
}

salvo_awaited!(SalvoBodyInput, parse_json, "body");
salvo_awaited!(SalvoFormInput, parse_form, "form");

impl<Input> SalvoInputAlg<Input> for SalvoRawBodyInput<Input>
where
    Input: FromPayloadAlg + Send,
{
    async fn extract(request: &mut Request) -> Result<Input, StatusError> {
        let payload = request.payload().await.map_err(|error| unreadable("body", &error.to_string()))?;

        Input::from_payload(payload).ok_or_else(|| unreadable("body", "it states something else"))
    }
}

/// Marks a value read from the cookies a caller sent.
pub struct SalvoCookieInput<Input>(PhantomData<Input>);

/// Marks a value read from the headers a caller sent.
pub struct SalvoHeaderInput<Input>(PhantomData<Input>);

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

impl<Input> SalvoInputAlg<Input> for SalvoCookieInput<Input>
where
    Input: DeserializeOwned + Send,
{
    async fn extract(request: &mut Request) -> Result<Input, StatusError> {
        let header = request.headers().get(header::COOKIE).and_then(|value| value.to_str().ok());

        cookies_of(header).map_err(|error| unreadable("cookies", &error))
    }
}

impl<Input> SalvoInputAlg<Input> for SalvoHeadInput<Input>
where
    Input: FromHeadersAlg + Send,
{
    async fn extract(request: &mut Request) -> Result<Input, StatusError> {
        Input::from_headers(request.headers()).ok_or_else(|| unreadable("headers", "they state something else"))
    }
}

impl<Input> SalvoInputAlg<Input> for SalvoHeaderInput<Input>
where
    Input: DeserializeOwned + Send,
{
    async fn extract(request: &mut Request) -> Result<Input, StatusError> {
        let stated = request
            .headers()
            .iter()
            .filter_map(|(name, value)| Some((name.to_string(), value.to_str().ok()?.to_owned())))
            .collect::<Vec<_>>();

        headers_of(stated.into_iter()).map_err(|error| unreadable("headers", &error))
    }
}

/// Marks an argument read from a body arriving as parts.
pub struct SalvoMultipartInput<Input>(PhantomData<Input>);

impl<Input> SalvoInputAlg<Input> for SalvoMultipartInput<Input>
where
    Input: FromPartsAlg<ReadParts> + Send,
    Input::Error: Display,
{
    async fn extract(request: &mut Request) -> Result<Input, StatusError> {
        let media_type = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| unreadable("parts", "the body states no media type"))?
            .to_owned();
        let body = request.payload().await.map_err(|error| unreadable("parts", &error.to_string()))?.to_vec();
        let parts = ReadParts::new(&media_type, body).map_err(|error| unreadable("parts", &error.to_string()))?;

        Input::from_parts(parts).await.map_err(|error| unreadable("parts", &error.to_string()))
    }
}

/// Reads the whole argument product one endpoint states, in declaration order.
pub(crate) trait SalvoInputsAlg<Outputs> {
    fn extract(request: &mut Request) -> impl Future<Output = Result<Outputs, StatusError>> + Send;
}

impl SalvoInputsAlg<()> for () {
    async fn extract(_request: &mut Request) -> Result<(), StatusError> {
        Ok(())
    }
}

macro_rules! salvo_inputs {
    ($($input:ident => $output:ident),+ $(,)?) => {
        impl<$($input, $output),+> SalvoInputsAlg<($($output,)+)> for ($($input,)+)
        where
            $($input: SalvoInputAlg<$output>, $output: Send,)+
        {
            async fn extract(request: &mut Request) -> Result<($($output,)+), StatusError> {
                Ok(($($input::extract(request).await?,)+))
            }
        }
    };
}

salvo_inputs!(I1 => O1);
salvo_inputs!(I1 => O1, I2 => O2);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15);
salvo_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15, I16 => O16);
