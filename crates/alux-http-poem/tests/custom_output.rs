//! Holds custom output meaning across tagless-final, reified, and direct programs.

use alux_ext::{OperationAlg, ext};
use alux_http::{
    CompileRouteProgram, HttpApiAlg, HttpOperationExt, HttpProgramBuilder, HttpProgramExt, HttpStatus, OutputAlg,
    OutputKindAlg, RouteAlgExt, http,
};
use alux_http_openapi::{OpenApiAnswer, OpenApiHandlerImpl, OpenApiOutputAlg};
use alux_http_poem::PoemHandlerImpl;
use alux_http_text::TextHandlerImpl;
use alux_shape::Shape;
use alux_shape_jsonschema::JsonSchemaShape;
use poem::http::{Method, StatusCode, header};
use poem::{Endpoint, Request, Response};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Shape)]
struct LoginParams {
    accepted: bool,
}

enum LoginAnswer {
    Accepted { location: String, cookie: String },
    Rejected { html: String },
}

trait LoginAlg {
    fn login(&self, params: LoginParams) -> LoginAnswer;
}

#[ext(name = LoginOperationExt, defunc)]
impl<This> This
where
    This: LoginAlg,
{
    async fn login_submit(&self, params: LoginParams) -> LoginAnswer {
        self.login(params)
    }
}

struct LoginOut;

// The spec names its capabilities; the macro supplies endpoint compatibility evidence.
#[ext(name = LoginApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg,
{
    fn login_api<Alg>(&self)
    where
        Alg: LoginAlg,
    {
        self.routes().post("/login", self.op(Alg::login_submit).form::<LoginParams>().out::<LoginOut>())
    }
}

// The two direct witnesses use the public fluent algebra and the first-order syntax respectively.
macro_rules! fluent {
    ($api:expr) => {
        $api.routes()
            .post("/login", $api.op(LoginSubmitOperation::<App>::default()).form::<LoginParams>().out::<LoginOut>())
            .into_route()
    };
}

macro_rules! direct {
    ($api:expr) => {{
        let syntax = HttpProgramBuilder;
        syntax
            .routes()
            .post("/login", syntax.op(LoginSubmitOperation::<App>::default()).form::<LoginParams>().out::<LoginOut>())
            .into_program()
            .compile_route($api)
    }};
}

struct App;

impl LoginAlg for App {
    fn login(&self, params: LoginParams) -> LoginAnswer {
        if params.accepted {
            LoginAnswer::Accepted { location: "/home".into(), cookie: "session=example; HttpOnly; Path=/".into() }
        } else {
            LoginAnswer::Rejected { html: "<p>Try again</p>".into() }
        }
    }
}

struct PoemLoginOutput;

impl<Context> OutputKindAlg<PoemHandlerImpl<Context>, LoginAnswer> for LoginOut {
    type Transform = PoemLoginOutput;
}

impl OutputAlg<LoginAnswer> for PoemLoginOutput {
    type Output = Response;

    fn output(answer: LoginAnswer) -> Response {
        match answer {
            LoginAnswer::Accepted { location, cookie } => Response::builder()
                .status(StatusCode::FOUND)
                .header(header::LOCATION, location)
                .header(header::SET_COOKIE, cookie)
                .finish(),
            LoginAnswer::Rejected { html } => Response::builder().content_type("text/html; charset=utf-8").body(html),
        }
    }
}

// Metadata names the custom conversion without applying a domain operation.
struct TextLoginOutput;

impl OutputKindAlg<TextHandlerImpl, LoginAnswer> for LoginOut {
    type Transform = TextLoginOutput;
}

impl OutputAlg<LoginAnswer> for TextLoginOutput {
    type Output = LoginAnswer;

    fn output(answer: LoginAnswer) -> LoginAnswer {
        answer
    }
}

struct OpenApiLoginOutput;

impl<Context> OutputKindAlg<OpenApiHandlerImpl<Context>, LoginAnswer> for LoginOut {
    type Transform = OpenApiLoginOutput;
}

impl OpenApiOutputAlg<LoginAnswer> for OpenApiLoginOutput {
    fn answers(_schema: &JsonSchemaShape) -> Vec<OpenApiAnswer> {
        let mut accepted = OpenApiAnswer::bodiless(HttpStatus::new(302));
        accepted.headers = vec!["location".to_owned(), "set-cookie".to_owned()];
        vec![accepted, OpenApiAnswer::content(HttpStatus::OK, "text/html", json!({"type": "string"}))]
    }
}

#[tokio::test]
async fn all_three_forms_execute_both_custom_answers() {
    let api = PoemHandlerImpl::new(App);
    let routes = [fluent!(api), api.compile_http(api.login_api::<App>()), direct!(&api)];
    for route in routes {
        let endpoint = route.into_poem();
        for accepted in [true, false] {
            let request = Request::builder()
                .method(Method::POST)
                .uri_str("/login")
                .content_type("application/x-www-form-urlencoded")
                .body(format!("accepted={accepted}"));
            let mut response = endpoint.call(request).await.unwrap();
            if accepted {
                assert_eq!(response.status(), StatusCode::FOUND);
                assert_eq!(response.headers()[header::LOCATION], "/home");
                assert_eq!(response.headers()[header::SET_COOKIE], "session=example; HttpOnly; Path=/");
                assert!(response.take_body().into_string().await.unwrap().is_empty());
            } else {
                assert_eq!(response.status(), StatusCode::OK);
                assert_eq!(response.headers()[header::CONTENT_TYPE], "text/html; charset=utf-8");
                assert!(!response.headers().contains_key(header::SET_COOKIE));
                assert_eq!(response.take_body().into_string().await.unwrap(), "<p>Try again</p>");
            }
        }
    }
}

#[test]
fn all_three_forms_describe_the_same_custom_answers() {
    assert_eq!(LoginSubmitOperation::<App>::ARG_NAMES, ["params"]);
    let text = TextHandlerImpl;
    let expected = fluent!(text);
    assert_eq!(expected.labels(), ["POST /login"]);
    assert_eq!(text.compile_http(text.login_api::<App>()), expected);
    assert_eq!(direct!(&text), expected);
    assert!(expected.lines()[0].contains("TextLoginOutput"));

    let api = OpenApiHandlerImpl::<App>::new();
    let routes = [fluent!(api), api.compile_http(api.login_api::<App>()), direct!(&api)];
    let documents = routes.iter().map(|route| api.document("Login", "1", route)).collect::<Vec<_>>();
    assert_eq!(documents[0], documents[1]);
    assert_eq!(documents[0], documents[2]);
    let responses = &documents[0]["paths"]["/login"]["post"]["responses"];
    assert!(responses["302"]["headers"]["location"].is_object());
    assert!(responses["302"]["headers"]["set-cookie"].is_object());
    assert!(responses["302"].get("content").is_none());
    assert_eq!(responses["200"]["content"]["text/html"]["schema"]["type"], "string");
}
