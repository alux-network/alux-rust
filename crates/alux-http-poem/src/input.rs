use crate::PoemParts;
use alux_http::{FromPartsAlg, read_cookies, read_header_name};
use core::fmt::Display;
use core::future::Future;
use core::marker::PhantomData;
use poem::Request;
use poem::http::StatusCode;
use poem::web::{Form, FromRequest, Json, Multipart, Path, Query, RequestBody};
use serde::de::DeserializeOwned;

/// Marks a value extracted from the request path by Poem.
pub struct PoemPathInput<Input>(PhantomData<Input>);
/// Marks a value extracted from the query string by Poem.
pub struct PoemQueryInput<Input>(PhantomData<Input>);
/// Marks a JSON value extracted from the request body by Poem.
pub struct PoemBodyInput<Input>(PhantomData<Input>);
/// Marks a form-encoded value extracted from the request body by Poem.
pub struct PoemFormInput<Input>(PhantomData<Input>);
/// Marks a value read from the headers a caller sent.
pub struct PoemHeaderInput<Input>(PhantomData<Input>);
/// Marks a value extracted through Poem's general request extractor.
pub struct PoemRequestInput<Input>(PhantomData<Input>);

impl<Input> PoemInputAlg<Input> for PoemCookieInput<Input>
where
    Input: DeserializeOwned + Send,
{
    async fn extract(request: &Request, _body: &mut RequestBody) -> poem::Result<Input> {
        cookies_of(request.header("cookie")).map_err(|error| poem::Error::from_string(error, StatusCode::BAD_REQUEST))
    }
}

pub(crate) trait PoemInputAlg<Output> {
    fn extract(request: &Request, body: &mut RequestBody) -> impl Future<Output = poem::Result<Output>> + Send;
}

macro_rules! poem_input {
    ($marker:ident, $extractor:ident) => {
        impl<Input> PoemInputAlg<Input> for $marker<Input>
        where
            Input: DeserializeOwned + Send,
        {
            async fn extract(request: &Request, body: &mut RequestBody) -> poem::Result<Input> {
                Ok($extractor::<Input>::from_request(request, body).await?.0)
            }
        }
    };
}

poem_input!(PoemPathInput, Path);
poem_input!(PoemQueryInput, Query);
poem_input!(PoemBodyInput, Json);
poem_input!(PoemFormInput, Form);

impl<Input> PoemInputAlg<Input> for PoemRequestInput<Input>
where
    Input: for<'a> FromRequest<'a> + Send,
{
    fn extract(request: &Request, body: &mut RequestBody) -> impl Future<Output = poem::Result<Input>> + Send {
        Input::from_request(request, body)
    }
}

/// Marks a value read from the cookies a caller sent.
pub struct PoemCookieInput<Input>(PhantomData<Input>);

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

impl<Input> PoemInputAlg<Input> for PoemHeaderInput<Input>
where
    Input: DeserializeOwned + Send,
{
    async fn extract(request: &Request, _body: &mut RequestBody) -> poem::Result<Input> {
        let stated = request
            .headers()
            .iter()
            .filter_map(|(name, value)| Some((name.to_string(), value.to_str().ok()?.to_owned())));

        headers_of(stated).map_err(|error| poem::Error::from_string(error, StatusCode::BAD_REQUEST))
    }
}

/// Marks an argument read from a body arriving as parts.
pub struct PoemMultipartInput<Input>(PhantomData<Input>);

impl<Input> PoemInputAlg<Input> for PoemMultipartInput<Input>
where
    Input: FromPartsAlg<PoemParts> + Send,
    Input::Error: Display,
{
    async fn extract(request: &Request, body: &mut RequestBody) -> poem::Result<Input> {
        let parts = Multipart::from_request(request, body).await?;

        Input::from_parts(PoemParts(parts))
            .await
            .map_err(|error| poem::Error::from_string(error.to_string(), StatusCode::BAD_REQUEST))
    }
}

/// Reads the whole argument product one endpoint states, in declaration order.
pub(crate) trait PoemInputsAlg<Outputs> {
    fn extract(request: &Request, body: &mut RequestBody) -> impl Future<Output = poem::Result<Outputs>> + Send;
}

impl PoemInputsAlg<()> for () {
    async fn extract(_request: &Request, _body: &mut RequestBody) -> poem::Result<()> {
        Ok(())
    }
}

macro_rules! poem_inputs {
    ($($input:ident => $output:ident),+ $(,)?) => {
        impl<$($input, $output),+> PoemInputsAlg<($($output,)+)> for ($($input,)+)
        where
            $($input: PoemInputAlg<$output>, $output: Send,)+
        {
            async fn extract(request: &Request, body: &mut RequestBody) -> poem::Result<($($output,)+)> {
                Ok(($($input::extract(request, body).await?,)+))
            }
        }
    };
}

poem_inputs!(I1 => O1);
poem_inputs!(I1 => O1, I2 => O2);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15);
poem_inputs!(I1 => O1, I2 => O2, I3 => O3, I4 => O4, I5 => O5, I6 => O6, I7 => O7, I8 => O8, I9 => O9, I10 => O10, I11 => O11, I12 => O12, I13 => O13, I14 => O14, I15 => O15, I16 => O16);
