use alux_http::OutputAlg;
use core::convert::Infallible;
use poem::http::{HeaderValue, StatusCode, header};
use poem::web::Json;
use poem::{IntoResponse, Response};
use std::io::{Error as IoError, ErrorKind};

/// Converts semantic results into Poem JSON responses.
pub struct PoemJsonOutput;

impl<From> OutputAlg<From> for PoemJsonOutput {
    type Output = Json<From>;

    fn output(from: From) -> Self::Output {
        Json(from)
    }
}

/// Converts semantic file results into downloadable Poem responses.
pub struct PoemFileOutput;

/// Carries a file result and filename until Poem creates the response.
pub struct PoemFileResponse<From>(From);

impl<From> OutputAlg<From> for PoemFileOutput {
    type Output = PoemFileResponse<From>;

    fn output(from: From) -> Self::Output {
        PoemFileResponse(from)
    }
}

/// Maps file-opening failures to their HTTP response status.
pub trait PoemFileErrorAlg {
    /// Returns the HTTP status represented by this failure.
    fn status(&self) -> StatusCode;
}

impl PoemFileErrorAlg for IoError {
    fn status(&self) -> StatusCode {
        if self.kind() == ErrorKind::NotFound { StatusCode::NOT_FOUND } else { StatusCode::INTERNAL_SERVER_ERROR }
    }
}

impl PoemFileErrorAlg for Infallible {
    fn status(&self) -> StatusCode {
        match *self {}
    }
}

impl<File, Error> IntoResponse for PoemFileResponse<(Result<File, Error>, String)>
where
    File: IntoResponse,
    Error: PoemFileErrorAlg + Send,
{
    fn into_response(self) -> Response {
        let (file, name) = self.0;
        match file {
            Ok(file) => {
                let mut response = file.into_response();
                let name = name.replace(['\r', '\n', '"'], "_");
                if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")) {
                    response.headers_mut().insert(header::CONTENT_DISPOSITION, value);
                }
                response
            }
            Err(error) => Response::builder().status(error.status()).finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PoemFileOutput;
    use alux_http::OutputAlg;
    use poem::http::{StatusCode, header};
    use poem::{Body, IntoResponse};
    use std::io::{Error as IoError, ErrorKind};

    fn download(file: Result<Body, IoError>) -> poem::Response {
        PoemFileOutput::output((file, "data.bin".to_owned())).into_response()
    }

    #[tokio::test]
    async fn names_the_file_in_a_download_response() {
        let mut response = download(Ok(Body::from_bytes((&b"data"[..]).into())));
        let disposition = response.headers().get(header::CONTENT_DISPOSITION).unwrap().to_str().unwrap().to_owned();

        assert!(response.status().is_success());
        assert_eq!(disposition, "attachment; filename=\"data.bin\"");
        assert_eq!(response.take_body().into_string().await.unwrap(), "data");
    }

    #[test]
    fn maps_a_missing_file_to_its_response_status() {
        let response = download(Err(IoError::new(ErrorKind::NotFound, "gone")));

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
