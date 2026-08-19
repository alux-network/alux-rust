//! Checks that one HTTP program describes its own surface, and that route composition obeys its laws.
//!
//! The domain and its programs are declared here the way a downstream author would, then folded by the
//! text interpreter, which executes no handler and only records what each endpoint denotes.

#![allow(async_fn_in_trait)]

use alux_ext::{ApplyAlg, ext};
use alux_http::{
    CompileRouteProgram, FileOutAlg, HttpApiAlg, HttpProgramBuilder, HttpProgramExt, HttpSelectorAlg, JsonOutAlg,
    RouteAlg, RouteAlgExt, SelectorAlg, http,
};
use alux_http_text::TextHandlerImpl;
use core::convert::Infallible;
use core::future::Future;
use std::sync::Arc;

/// Describes the domain status capability interpreted by this test.
trait StatusAlg {
    type Status;

    fn status(&self) -> impl Future<Output = Self::Status> + Send;
    fn status_set_temp(&self, temp: f32) -> impl Future<Output = Self::Status> + Send;
}

/// Describes the domain download capability interpreted by this test.
trait DownloadAlg {
    type Path;
    type File;
    type Info;
    type Error;

    fn dl_path(&self) -> &Self::Path;
    fn dl_info(&self) -> Self::Info;
    fn dl_file_name<'a>(&self, info: &'a Self::Info) -> &'a str;
    fn dl_at(&self, path: &Self::Path) -> impl Future<Output = Result<Self::File, Self::Error>> + Send;
}

#[ext(name = DownloadExt)]
impl<This, File, Error> This
where
    This: DownloadAlg<File = File, Error = Error>,
{
    async fn dl_file(&self) -> Result<File, Error> {
        self.dl_at(self.dl_path()).await
    }
}

#[ext(name = StatusAlgExt, defunc)]
impl<This> This
where
    This: StatusAlg,
{
    async fn status_current(&self) -> This::Status {
        self.status().await
    }

    async fn status_for_path(&self, id: u32) -> This::Status {
        let _ = id;
        self.status().await
    }

    async fn status_for_query(&self, query: String) -> This::Status {
        let _ = query;
        self.status().await
    }

    async fn status_adjusted(&self, temp: f32) -> This::Status {
        self.status_set_temp(temp).await
    }
}

#[ext(name = DownloadAlgExt, defunc)]
impl<This> This
where
    This: DownloadAlg,
    This::Info: Send,
{
    async fn download_current(&self) -> (Result<This::File, This::Error>, String) {
        let info = self.dl_info();
        let file_name = self.dl_file_name(&info).to_owned();

        (self.dl_file().await, file_name)
    }
}

#[ext(name = StatusApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + JsonOutAlg,
{
    fn status_api<Alg>(&self)
    where
        Alg: StatusAlg,
    {
        self.routes()
            .get("/status", self.op(Alg::status_current).json())
            .get("/status/:id", self.op(Alg::status_for_path).path::<u32>().json())
            .get("/status_search", self.op(Alg::status_for_query).query::<String>().json())
            .post("/set_temp", self.op(Alg::status_adjusted).body::<f32>().json())
    }
}

#[ext(name = DownloadApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + FileOutAlg,
{
    fn download_api<Alg>(&self)
    where
        Alg: DownloadAlg,
    {
        self.routes().get("/download", self.op(Alg::download_current).file())
    }
}

#[ext(name = ExampleApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg,
{
    fn example_api<Alg>(&self)
    where
        Alg: StatusAlg + DownloadAlg,
    {
        self.routes().merge(self.status_api::<Alg>()).merge(self.download_api::<Alg>())
    }
}

#[derive(Debug, Default, PartialEq)]
struct TestStatus {
    temp: f32,
}

impl StatusAlg for TestStatus {
    type Status = f32;

    async fn status(&self) -> f32 {
        self.temp
    }

    async fn status_set_temp(&self, temp: f32) -> f32 {
        temp
    }
}

impl DownloadAlg for TestStatus {
    type Path = ();
    type File = &'static [u8];
    type Info = &'static str;
    type Error = Infallible;

    fn dl_path(&self) -> &() {
        &()
    }

    fn dl_info(&self) -> &'static str {
        "data.bin"
    }

    fn dl_file_name<'a>(&self, info: &'a &'static str) -> &'a str {
        info
    }

    async fn dl_at(&self, _path: &()) -> Result<&'static [u8], Infallible> {
        Ok(b"data")
    }
}

#[test]
fn describes_example_http_surface() {
    let api = TextHandlerImpl;
    let routes = api.compile_http(api.example_api::<TestStatus>());
    let api_lines = routes.lines();

    assert_eq!(api_lines.len(), 5);
    assert!(api_lines[0].starts_with("### GET /status"));
    assert!(api_lines[0].contains("- `output`: `f32`"));
    assert!(api_lines[0].contains("TextJsonOutput"));
    assert!(api_lines[1].starts_with("### GET /status/:id"));
    assert!(api_lines[1].contains("PathRole"));
    assert!(api_lines[1].contains("- `args`: `(u32,)`"));
    assert!(api_lines[2].starts_with("### GET /status_search"));
    assert!(api_lines[2].contains("QueryRole"));
    assert!(api_lines[2].contains("- `args`: `(alloc::string::String,)`"));
    assert!(api_lines[3].starts_with("### POST /set_temp"));
    assert!(api_lines[3].contains("BodyRole"));
    assert!(api_lines[3].contains("- `args`: `(f32,)`"));
    assert!(api_lines[4].starts_with("### GET /download"));
    assert!(api_lines[4].contains("TextFileOutput"));
}

#[test]
fn compiles_a_first_order_route_without_an_http_program_macro() {
    let compiler = TextHandlerImpl;
    let builder = HttpProgramBuilder;
    let status = builder.routes().get("/status", builder.op(StatusCurrentOperation::<TestStatus>::default()).json());
    let program = builder.routes().nest("/api", status).into_program();
    let routes = program.compile_route(&compiler);

    assert!(routes.lines()[0].starts_with("### GET /api/status"));
}

#[test]
fn composes_routes_categorically() {
    let api = TextHandlerImpl;
    let example = api.compile_http(api.example_api::<TestStatus>());
    let nested_routes = api.routes().nest("/api", api.route(example)).into_route();
    let nested_lines = nested_routes.lines();

    assert!(nested_lines[0].starts_with("### GET /api/status"));

    let routes = api.compile_http(api.example_api::<TestStatus>());
    assert_eq!(api.coproduct(api.initial(), routes.clone()), routes);
    assert_eq!(api.precompose(api.identity(), routes.clone()), routes);

    let composed = api.compose(api.http_prefix("/api"), api.http_prefix("/v1"));
    let together = api.precompose(composed, routes.clone());
    let nested = api.precompose(api.http_prefix("/api"), api.precompose(api.http_prefix("/v1"), routes));

    assert_eq!(together, nested);
}

#[tokio::test]
async fn invokes_an_async_algebra_operation_without_boxing() {
    let status = Arc::new(TestStatus { temp: 21.5 });
    let output = StatusCurrentOperation::<TestStatus>::default().apply(status, ()).await;

    assert!((output - 21.5).abs() < f32::EPSILON);
}
