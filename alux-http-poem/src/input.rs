use core::future::Future;
use core::marker::PhantomData;
use poem::Request;
use poem::web::headers::Header;
use poem::web::{FromRequest, Json, Path, Query, RequestBody, TypedHeader};
use serde::de::DeserializeOwned;

/// Marks a value extracted from the request path by Poem.
pub struct PoemPathInput<Input>(PhantomData<Input>);
/// Marks a value extracted from the query string by Poem.
pub struct PoemQueryInput<Input>(PhantomData<Input>);
/// Marks a JSON value extracted from the request body by Poem.
pub struct PoemBodyInput<Input>(PhantomData<Input>);
/// Marks a typed value extracted from a request header by Poem.
pub struct PoemHeaderInput<Input>(PhantomData<Input>);
/// Marks a value extracted through Poem's general request extractor.
pub struct PoemRequestInput<Input>(PhantomData<Input>);

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

impl<Input> PoemInputAlg<Input> for PoemHeaderInput<Input>
where
    Input: Header + Send,
{
    async fn extract(request: &Request, body: &mut RequestBody) -> poem::Result<Input> {
        Ok(TypedHeader::<Input>::from_request(request, body).await?.0)
    }
}

impl<Input> PoemInputAlg<Input> for PoemRequestInput<Input>
where
    Input: for<'a> FromRequest<'a> + Send,
{
    fn extract(request: &Request, body: &mut RequestBody) -> impl Future<Output = poem::Result<Input>> + Send {
        Input::from_request(request, body)
    }
}

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
