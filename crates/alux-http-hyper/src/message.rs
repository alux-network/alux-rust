//! Reads a hyper request as the request a surface answers, and writes the answer back.

use alux_http::{HttpMethod, HttpStatus};
use alux_http_direct::{DirectBody, DirectError, DirectRequest, DirectResponse};
use bytes::Bytes;
use core::fmt::Display;
use core::pin::pin;
use futures::TryStreamExt;
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::{Body, Frame};
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Request, Response, StatusCode};
use std::io::Error as IoError;

/// The body a served answer carries, whether its bytes were in hand or are still to come.
///
/// A body still to come is whatever produces it, which one task reads in order, so what carries it
/// need not be shared across threads.
pub type HyperBody = UnsyncBoxBody<Bytes, IoError>;

/// The answer this service produces, which is a hyper response like any other.
pub type HyperAnswer = Response<HyperBody>;

/// How many bytes of a request body are read before the request is refused.
///
/// A transport carrying no framework inherits no framework's policy, so it states one here rather
/// than reading a body of any size into memory. A service that accepts more, or less, states what
/// it accepts with [`HyperRoute::reading`](crate::HyperRoute::reading), which production is
/// expected to do: this is a default, not a decision about a caller's uploads.
pub(crate) const BODY_LIMIT: usize = 8 * 1024 * 1024;

/// The status a body larger than what is read is answered with.
const TOO_LARGE: HttpStatus = HttpStatus::new(413);

/// Reads a hyper request as the request a surface answers.
pub(crate) async fn asked<Sent>(request: Request<Sent>, reading: usize) -> Result<DirectRequest, DirectResponse>
where
    Sent: Body<Data = Bytes>,
    Sent::Error: Display,
{
    let (head, sent) = request.into_parts();
    let Some(method) = named(head.method.as_str()) else {
        return Err(DirectError::method_not_allowed(head.uri.path()).into());
    };
    let mut asked = DirectRequest::new(method, head.uri.path());
    if let Some(query) = head.uri.query() {
        asked = asked.with_query(query);
    }
    for (name, value) in &head.headers {
        if let Ok(value) = value.to_str() {
            asked = asked.with_header(name.as_str(), value);
        }
    }

    Ok(asked.with_body(read(sent, reading).await?))
}

/// Reads the body a caller sent, up to `reading` bytes, and refuses one larger than that.
///
/// Frames are read one at a time and counted as they arrive, so a body that never ends is refused
/// at the byte the limit names rather than after it has been held whole.
async fn read<Sent>(sent: Sent, reading: usize) -> Result<Vec<u8>, DirectResponse>
where
    Sent: Body<Data = Bytes>,
    Sent::Error: Display,
{
    let mut body = Vec::new();
    let mut sent = pin!(sent);
    while let Some(frame) = sent.frame().await {
        let frame = frame.map_err(|error| DirectResponse::from(DirectError::unreadable("body", &error.to_string())))?;
        let Ok(data) = frame.into_data() else {
            continue;
        };
        if body.len() + data.len() > reading {
            let refused = format!("the body is larger than the {reading} bytes this service reads");

            return Err(DirectError::new(TOO_LARGE, refused).into());
        }
        body.extend_from_slice(&data);
    }

    Ok(body)
}

/// Reads a method name as the method a program states, where it states one.
fn named(method: &str) -> Option<HttpMethod> {
    HttpMethod::ALL.iter().copied().find(|stated| stated.label() == method)
}

/// Writes the answer a surface produced as the response hyper sends.
pub(crate) fn answered(answer: DirectResponse) -> HyperAnswer {
    let status = StatusCode::from_u16(answer.status().code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let stated = answer.headers().map(|(name, value)| (name.to_owned(), value.to_owned())).collect::<Vec<_>>();
    let mut response = Response::new(carried(answer.into_body()));
    *response.status_mut() = status;
    for (name, value) in stated {
        if let (Ok(name), Ok(value)) = (HeaderName::from_bytes(name.as_bytes()), HeaderValue::from_str(&value)) {
            response.headers_mut().append(name, value);
        }
    }

    response
}

/// Writes what an answer carries as the body hyper sends, however the answer carries it.
fn carried(body: DirectBody) -> HyperBody {
    match body {
        DirectBody::Stated(body) => Full::new(Bytes::from(body)).map_err(|never| match never {}).boxed_unsync(),
        DirectBody::Produced(chunks) => {
            StreamBody::new(chunks.map_ok(|chunk| Frame::data(Bytes::from(chunk)))).boxed_unsync()
        }
    }
}
