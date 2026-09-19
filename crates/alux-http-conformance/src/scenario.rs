//! What answering the declared surface must produce, whoever answers it.
//!
//! The scenario names nothing of the domain and nothing of any framework. It states requests and
//! what they must be answered with, so anything exposing the surface can be held to it.

use alux_http::{HttpMethod, HttpStatus};
use alux_http_parts::{DirectRequest, DirectResponse};
use core::future::Future;

/// Answers a stated request, however the interpretation under test answers one.
///
/// An executing interpretation answers with its framework's response; the adapter that states this
/// is the only place a framework is named.
pub trait AnswerAlg {
    /// Answers one request.
    fn answer(&self, request: DirectRequest) -> impl Future<Output = DirectResponse> + Send;
}

/// One exchange: a request, and what answering it must produce.
#[derive(Debug, Clone)]
pub struct Exchange {
    /// What this exchange is checking.
    pub what: &'static str,
    /// The request a caller makes.
    pub request: DirectRequest,
    /// The status the answer must carry.
    pub status: HttpStatus,
    /// The body the answer must carry, where the exchange states one.
    pub body: Option<&'static str>,
    /// A header the answer must carry, where the exchange states one.
    pub header: Option<(&'static str, &'static str)>,
}

impl Exchange {
    /// States an exchange asking for something, with no body sent.
    fn asked(what: &'static str, method: HttpMethod, path: &str, status: HttpStatus) -> Self {
        Self { what, request: DirectRequest::new(method, path), status, body: None, header: None }
    }

    /// States one header the caller sends.
    #[must_use]
    fn sending(mut self, name: &str, value: &str) -> Self {
        self.request = self.request.with_header(name, value);
        self
    }

    /// States an exchange sending something of a stated media type.
    fn sent(
        what: &'static str,
        method: HttpMethod,
        path: &str,
        content_type: &str,
        body: &'static str,
        status: HttpStatus,
    ) -> Self {
        let request = DirectRequest::new(method, path).with_header("content-type", content_type).with_body(body);

        Self { what, request, status, body: None, header: None }
    }

    /// States the body this exchange must be answered with.
    #[must_use]
    fn answering(mut self, body: &'static str) -> Self {
        self.body = Some(body);
        self
    }

    /// States a header this exchange must be answered with.
    #[must_use]
    fn carrying(mut self, name: &'static str, value: &'static str) -> Self {
        self.header = Some((name, value));
        self
    }

    /// Returns where an answer disagreed with what this exchange states.
    fn disagreements(&self, answer: &DirectResponse) -> Vec<String> {
        let mut found = Vec::new();
        let what = self.what;
        if answer.status() != self.status {
            found.push(format!("{what}: answered {} where {} was stated", answer.status().code(), self.status.code()));
        }
        if let Some(body) = self.body
            && answer.text() != body
        {
            found.push(format!("{what}: answered `{}` where `{body}` was stated", answer.text()));
        }
        if let Some((name, value)) = self.header
            && answer.header(name) != Some(value)
        {
            found.push(format!(
                "{what}: answered `{name}: {}` where `{name}: {value}` was stated",
                answer.header(name).unwrap_or("nothing")
            ));
        }

        found
    }
}

/// Every exchange the declared surface must satisfy.
///
/// Each names one thing the specification states: a method, a path binding, an input role, an
/// output kind, a declared status, or what a failure means.
pub fn exchanges() -> Vec<Exchange> {
    vec![
        Exchange::asked("a reading as it stands", HttpMethod::Get, "/items", HttpStatus::OK).answering("[7]"),
        Exchange::asked("a path binding one segment", HttpMethod::Get, "/item/1", HttpStatus::OK).answering("7"),
        Exchange::asked("what a failure means", HttpMethod::Get, "/item/9", HttpStatus::NOT_FOUND)
            .answering("no such reading"),
        Exchange::sent(
            "a body read as a document, under a declared status",
            HttpMethod::Post,
            "/items",
            "application/json",
            "7",
            HttpStatus::new(201),
        )
        .answering("7"),
        Exchange::sent(
            "a body read as a form",
            HttpMethod::Put,
            "/items",
            "application/x-www-form-urlencoded",
            "value=7",
            HttpStatus::OK,
        )
        .answering("7"),
        Exchange::sent(
            "a body taken as it arrived",
            HttpMethod::Patch,
            "/items",
            "text/plain",
            "seven",
            HttpStatus::OK,
        )
        .answering("noted seven"),
        Exchange::asked("an answer with no body", HttpMethod::Delete, "/items", HttpStatus::NO_CONTENT).answering(""),
        Exchange::asked("a redirect", HttpMethod::Get, "/home", HttpStatus::SEE_OTHER).carrying("location", "/items"),
        Exchange::asked("a page", HttpMethod::Get, "/page", HttpStatus::OK)
            .answering("<p>7</p>")
            .carrying("content-type", "text/html; charset=utf-8"),
        Exchange::asked("the readings as stored", HttpMethod::Get, "/stored", HttpStatus::OK).answering("seven"),
        Exchange::asked("what the cookies a caller sent state", HttpMethod::Get, "/session", HttpStatus::OK)
            .sending("cookie", "session=abc; theme=dark")
            .answering("known as abc"),
        // An answer carries what the handler stated beside its body.
        Exchange::asked("a header an answer carries", HttpMethod::Get, "/cached", HttpStatus::OK)
            .answering("[7]")
            .carrying("cache-control", "max-age=60"),
        // A header name is words the wire spells with `-` and an argument spells with `_`.
        Exchange::asked("what the headers a caller sent state", HttpMethod::Get, "/agent", HttpStatus::OK)
            .sending("user-agent", "probe")
            .answering("sent by probe"),
    ]
}

/// Holds one interpretation to every exchange the surface states.
///
/// Every disagreement is collected rather than the first one raised, because what is worth knowing
/// is how two interpretations differ, not that they do.
///
/// # Errors
///
/// Answers with every disagreement found, in the order the exchanges state them.
pub async fn expect<Answers>(answers: &Answers) -> Result<(), Vec<String>>
where
    Answers: AnswerAlg,
{
    let mut found = Vec::new();
    for exchange in exchanges() {
        // A body produced over time is read to its end, because what is compared is what a caller
        // would have received.
        let answer = answers.answer(exchange.request.clone()).await.collected().await;
        found.extend(exchange.disagreements(&answer));
    }

    if found.is_empty() { Ok(()) } else { Err(found) }
}

/// Every exchange the streamed surface must satisfy.
pub fn stream_exchanges() -> Vec<Exchange> {
    vec![
        Exchange::asked("a body produced a piece at a time", HttpMethod::Get, "/ticks", HttpStatus::OK)
            .answering("onetwothree"),
    ]
}

/// Holds one interpretation to every exchange the streamed surface states.
///
/// # Errors
///
/// Answers with every disagreement found, in the order the exchanges state them.
pub async fn expect_streaming<Answers>(answers: &Answers) -> Result<(), Vec<String>>
where
    Answers: AnswerAlg,
{
    let mut found = Vec::new();
    for exchange in stream_exchanges() {
        let answer = answers.answer(exchange.request.clone()).await.collected().await;
        found.extend(exchange.disagreements(&answer));
    }

    if found.is_empty() { Ok(()) } else { Err(found) }
}

/// The body a caller sends as parts, and the boundary it states between them.
const BOUNDARY: &str = "alux";
const PARTS: &str = "--alux\r\nContent-Disposition: form-data; name=\"one\"\r\n\r\n1\r\n\
--alux\r\nContent-Disposition: form-data; name=\"two\"\r\n\r\n2\r\n--alux--\r\n";

/// Every exchange the parts surface must satisfy.
pub fn multipart_exchanges() -> Vec<Exchange> {
    vec![
        Exchange::sent(
            "a body arriving as parts",
            HttpMethod::Post,
            "/upload",
            &format!("multipart/form-data; boundary={BOUNDARY}"),
            PARTS,
            HttpStatus::OK,
        )
        .answering("one=1,two=2"),
    ]
}

/// Holds one interpretation to every exchange the parts surface states.
///
/// # Errors
///
/// Answers with every disagreement found, in the order the exchanges state them.
pub async fn expect_multipart<Answers>(answers: &Answers) -> Result<(), Vec<String>>
where
    Answers: AnswerAlg,
{
    let mut found = Vec::new();
    for exchange in multipart_exchanges() {
        let answer = answers.answer(exchange.request.clone()).await.collected().await;
        found.extend(exchange.disagreements(&answer));
    }

    if found.is_empty() { Ok(()) } else { Err(found) }
}
