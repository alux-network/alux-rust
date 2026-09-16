use crate::AxumParts;
use alux_http::{FromPartsAlg, read_cookies, read_header_name};
use axum::body::Body;
use axum::extract::{FromRequest, FromRequestParts, Multipart, Path, Query, Request};
use axum::http::request::Parts;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Form, Json};
use core::fmt::Display;
use core::future::Future;
use core::marker::PhantomData;
use serde::de::DeserializeOwned;

/// Marks a value extracted from the request path by axum.
pub struct AxumPathInput<Input>(PhantomData<Input>);
/// Marks a value extracted from the query string by axum.
pub struct AxumQueryInput<Input>(PhantomData<Input>);
/// Marks a JSON value extracted from the request body by axum.
pub struct AxumBodyInput<Input>(PhantomData<Input>);
/// Marks a form-encoded value extracted from the request body by axum.
pub struct AxumFormInput<Input>(PhantomData<Input>);
/// Marks the request body taken by axum as it arrived.
pub struct AxumRawBodyInput<Input>(PhantomData<Input>);
/// Marks a value extracted from the request head by axum.
pub struct AxumPartsInput<Input>(PhantomData<Input>);

/// Reads one handler argument from a request axum has taken apart.
///
/// The head is read in place and the body is taken once, so an endpoint reading the body reads it
/// exactly as a framework handler would.
pub(crate) trait AxumInputAlg<Output> {
    fn extract(parts: &mut Parts, body: &mut Option<Body>) -> impl Future<Output = Result<Output, Response>> + Send;
}

impl<Input> AxumInputAlg<Input> for AxumPartsInput<Input>
where
    Input: FromRequestParts<()> + Send,
{
    async fn extract(parts: &mut Parts, _body: &mut Option<Body>) -> Result<Input, Response> {
        Input::from_request_parts(parts, &()).await.map_err(IntoResponse::into_response)
    }
}

macro_rules! axum_parts_input {
    ($marker:ident, $extractor:ident) => {
        impl<Input> AxumInputAlg<Input> for $marker<Input>
        where
            Input: DeserializeOwned + Send,
        {
            async fn extract(parts: &mut Parts, _body: &mut Option<Body>) -> Result<Input, Response> {
                Ok($extractor::<Input>::from_request_parts(parts, &()).await.map_err(IntoResponse::into_response)?.0)
            }
        }
    };
}

axum_parts_input!(AxumPathInput, Path);
axum_parts_input!(AxumQueryInput, Query);

/// Rebuilds the request one body extractor reads, taking the body it consumes.
fn taken(parts: &Parts, body: &mut Option<Body>) -> Request {
    Request::from_parts(parts.clone(), body.take().unwrap_or_default())
}

macro_rules! axum_body_input {
    ($marker:ident, $extractor:ident) => {
        impl<Input> AxumInputAlg<Input> for $marker<Input>
        where
            Input: DeserializeOwned + Send,
        {
            async fn extract(parts: &mut Parts, body: &mut Option<Body>) -> Result<Input, Response> {
                let request = taken(parts, body);

                Ok($extractor::<Input>::from_request(request, &()).await.map_err(IntoResponse::into_response)?.0)
            }
        }
    };
}

axum_body_input!(AxumBodyInput, Json);
axum_body_input!(AxumFormInput, Form);

impl<Input> AxumInputAlg<Input> for AxumRawBodyInput<Input>
where
    Input: FromRequest<()> + Send,
{
    async fn extract(parts: &mut Parts, body: &mut Option<Body>) -> Result<Input, Response> {
        let request = taken(parts, body);

        Input::from_request(request, &()).await.map_err(IntoResponse::into_response)
    }
}

/// Marks a value read from the cookies a caller sent.
pub struct AxumCookieInput<Input>(PhantomData<Input>);

/// Marks a value read from the headers a caller sent.
pub struct AxumHeaderInput<Input>(PhantomData<Input>);

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

impl<Input> AxumInputAlg<Input> for AxumCookieInput<Input>
where
    Input: DeserializeOwned + Send,
{
    async fn extract(parts: &mut Parts, _body: &mut Option<Body>) -> Result<Input, Response> {
        let header = parts.headers.get(header::COOKIE).and_then(|value| value.to_str().ok());

        cookies_of(header).map_err(|error| (StatusCode::BAD_REQUEST, error).into_response())
    }
}

impl<Input> AxumInputAlg<Input> for AxumHeaderInput<Input>
where
    Input: DeserializeOwned + Send,
{
    async fn extract(parts: &mut Parts, _body: &mut Option<Body>) -> Result<Input, Response> {
        let stated =
            parts.headers.iter().filter_map(|(name, value)| Some((name.to_string(), value.to_str().ok()?.to_owned())));

        headers_of(stated).map_err(|error| (StatusCode::BAD_REQUEST, error).into_response())
    }
}

/// Marks an argument read from a body arriving as parts.
pub struct AxumMultipartInput<Input>(PhantomData<Input>);

impl<Input> AxumInputAlg<Input> for AxumMultipartInput<Input>
where
    Input: FromPartsAlg<AxumParts> + Send,
    Input::Error: Display,
{
    async fn extract(parts: &mut Parts, body: &mut Option<Body>) -> Result<Input, Response> {
        let request = taken(parts, body);
        let read = Multipart::from_request(request, &()).await.map_err(IntoResponse::into_response)?;

        Input::from_parts(AxumParts(read))
            .await
            .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()).into_response())
    }
}

/// Reads the whole argument product one endpoint states, in declaration order.
pub(crate) trait AxumInputsAlg<Outputs> {
    fn extract(parts: &mut Parts, body: &mut Option<Body>) -> impl Future<Output = Result<Outputs, Response>> + Send;
}

impl AxumInputsAlg<()> for () {
    async fn extract(_parts: &mut Parts, _body: &mut Option<Body>) -> Result<(), Response> {
        Ok(())
    }
}

macro_rules! axum_inputs {
    ($($input:ident => $output:ident),+ $(,)?) => {
        impl<$($input, $output),+> AxumInputsAlg<($($output,)+)> for ($($input,)+)
        where
            $($input: AxumInputAlg<$output>, $output: Send,)+
        {
            async fn extract(parts: &mut Parts, body: &mut Option<Body>) -> Result<($($output,)+), Response> {
                Ok(($($input::extract(parts, body).await?,)+))
            }
        }
    };
}

axum_inputs!(I1 => O1);
axum_inputs!(I1 => O1, I2 => O2);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15);
axum_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15, I16 => O16);
