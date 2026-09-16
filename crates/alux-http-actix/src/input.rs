//! Reads each handler argument with the extractor actix-web states for its role.

use actix_web::dev::Payload;
use actix_web::error::ErrorBadRequest;
use actix_web::http::header;
use actix_web::web::{Bytes, Form, Json, Path, Query};
use actix_web::{Error, FromRequest, HttpRequest};
use alux_http::{FromPartsAlg, read_cookies, read_header_name};
use alux_http_parts::ReadParts;
use core::fmt::Display;
use core::future::Future;
use core::marker::PhantomData;
use serde::de::DeserializeOwned;

/// Marks a value extracted from the request path by actix-web.
pub struct ActixPathInput<Input>(PhantomData<Input>);
/// Marks a value extracted from the query string by actix-web.
pub struct ActixQueryInput<Input>(PhantomData<Input>);
/// Marks a JSON value extracted from the request body by actix-web.
pub struct ActixBodyInput<Input>(PhantomData<Input>);
/// Marks a form-encoded value extracted from the request body by actix-web.
pub struct ActixFormInput<Input>(PhantomData<Input>);
/// Marks a value extracted through actix-web's own request extractor.
pub struct ActixRequestInput<Input>(PhantomData<Input>);

/// Reads one handler argument from a request actix-web is answering.
pub(crate) trait ActixInputAlg<Output> {
    fn extract(request: &HttpRequest, payload: &mut Payload) -> impl Future<Output = Result<Output, Error>>;
}

macro_rules! actix_input {
    ($marker:ident, $extractor:ident) => {
        impl<Input> ActixInputAlg<Input> for $marker<Input>
        where
            Input: DeserializeOwned + 'static,
        {
            async fn extract(request: &HttpRequest, payload: &mut Payload) -> Result<Input, Error> {
                Ok($extractor::<Input>::from_request(request, payload).await?.into_inner())
            }
        }
    };
}

actix_input!(ActixPathInput, Path);
actix_input!(ActixQueryInput, Query);
actix_input!(ActixBodyInput, Json);
actix_input!(ActixFormInput, Form);

impl<Input> ActixInputAlg<Input> for ActixRequestInput<Input>
where
    Input: FromRequest + 'static,
{
    async fn extract(request: &HttpRequest, payload: &mut Payload) -> Result<Input, Error> {
        Input::from_request(request, payload).await.map_err(Into::into)
    }
}

/// Marks a value read from the cookies a caller sent.
pub struct ActixCookieInput<Input>(PhantomData<Input>);

/// Marks a value read from the headers a caller sent.
pub struct ActixHeaderInput<Input>(PhantomData<Input>);

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

impl<Input> ActixInputAlg<Input> for ActixCookieInput<Input>
where
    Input: DeserializeOwned + 'static,
{
    async fn extract(request: &HttpRequest, _payload: &mut Payload) -> Result<Input, Error> {
        let header = request.headers().get(header::COOKIE).and_then(|value| value.to_str().ok());

        cookies_of(header).map_err(ErrorBadRequest)
    }
}

impl<Input> ActixInputAlg<Input> for ActixHeaderInput<Input>
where
    Input: DeserializeOwned + 'static,
{
    async fn extract(request: &HttpRequest, _payload: &mut Payload) -> Result<Input, Error> {
        let stated = request
            .headers()
            .iter()
            .filter_map(|(name, value)| Some((name.to_string(), value.to_str().ok()?.to_owned())));

        headers_of(stated).map_err(ErrorBadRequest)
    }
}

/// Marks an argument read from a body arriving as parts.
pub struct ActixMultipartInput<Input>(PhantomData<Input>);

impl<Input> ActixInputAlg<Input> for ActixMultipartInput<Input>
where
    Input: FromPartsAlg<ReadParts> + 'static,
    Input::Error: Display,
{
    async fn extract(request: &HttpRequest, payload: &mut Payload) -> Result<Input, Error> {
        let media_type = request.headers().get(header::CONTENT_TYPE).and_then(|value| value.to_str().ok());
        let media_type = media_type.ok_or_else(|| ErrorBadRequest("the body states no media type"))?.to_owned();
        let body = Bytes::from_request(request, payload).await?;
        let parts = ReadParts::new(&media_type, body.to_vec()).map_err(ErrorBadRequest)?;

        Input::from_parts(parts).await.map_err(|error| ErrorBadRequest(error.to_string()))
    }
}

/// Reads the whole argument product one endpoint states, in declaration order.
pub(crate) trait ActixInputsAlg<Outputs> {
    fn extract(request: &HttpRequest, payload: &mut Payload) -> impl Future<Output = Result<Outputs, Error>>;
}

impl ActixInputsAlg<()> for () {
    async fn extract(_request: &HttpRequest, _payload: &mut Payload) -> Result<(), Error> {
        Ok(())
    }
}

macro_rules! actix_inputs {
    ($($input:ident => $output:ident),+ $(,)?) => {
        impl<$($input, $output),+> ActixInputsAlg<($($output,)+)> for ($($input,)+)
        where
            $($input: ActixInputAlg<$output>,)+
        {
            async fn extract(request: &HttpRequest, payload: &mut Payload) -> Result<($($output,)+), Error> {
                Ok(($($input::extract(request, payload).await?,)+))
            }
        }
    };
}

actix_inputs!(I1 => O1);
actix_inputs!(I1 => O1, I2 => O2);
actix_inputs!(I1 => O1, I2 => O2, I3 => O3);
actix_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4);
actix_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5);
actix_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6);
actix_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7);
actix_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8);
actix_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9);
actix_inputs!(
    I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10
);
actix_inputs!(
    I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10,
    I11 => O11
);
actix_inputs!(
    I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10,
    I11 => O11, I12 => O12
);
actix_inputs!(
    I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10,
    I11 => O11, I12 => O12, I13 => O13
);
actix_inputs!(
    I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10,
    I11 => O11, I12 => O12, I13 => O13, I14 => O14
);
actix_inputs!(
    I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10,
    I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15
);
actix_inputs!(
    I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10,
    I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15, I16 => O16
);
