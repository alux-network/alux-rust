//! The same surface as `spec_first_http`, composed by merging separately declared programs.
//!
//! Each program states one part of the surface and needs only the capabilities that part uses, so
//! independently published fragments compose into one route coproduct.

mod common;
mod expect;

use alux_ext::ext;
use alux_http::{FileOutAlg, HttpApiAlg, HttpProgramExt, JsonOutAlg, http};
use alux_http_poem::PoemHandlerImpl;
use alux_http_text::TextHandlerImpl;
use common::{
    App, DownloadAlg, DownloadCurrentOperation, StatusAdjustedOperation, StatusAlg, StatusCurrentOperation,
    StatusForIdOperation,
};
use expect::expect_example_api;

#[ext(name = StatusApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + JsonOutAlg,
{
    /// Declares the status surface: the current reading, one identified reading, and an adjustment.
    fn status_api<Alg>(&self)
    where
        Alg: StatusAlg,
    {
        self.routes()
            // The reading as it stands.
            .get("/status", self.op(Alg::status_current).json())
            // One identified reading, its id taken from the path.
            .get("/status/:id", self.op(Alg::status_for_id).path::<u32>().json())
            // An adjustment, its temperature taken from the request body.
            .post("/set_temp", self.op(Alg::status_adjusted).body::<f32>().json())
    }
}

#[ext(name = DownloadApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + FileOutAlg,
{
    /// Declares the download surface, whose output kind only a framework can convert.
    fn download_api<Alg>(&self)
    where
        Alg: DownloadAlg,
    {
        // The file and the name to offer it under.
        self.routes().get("/download", self.op(Alg::download_current).file())
    }
}

#[ext(name = ExampleApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg,
{
    /// Composes the whole surface from the two independently declared programs.
    fn example_api<Alg>(&self)
    where
        Alg: StatusAlg + DownloadAlg,
    {
        self.routes().merge(self.status_api::<Alg>()).merge(self.download_api::<Alg>())
    }
}

#[tokio::test]
async fn merges_separately_declared_programs_into_poem_routes() {
    let api = PoemHandlerImpl::new(App(40));
    let route = api.compile_http(api.example_api::<App>()).into_poem();

    expect_example_api(&route).await.unwrap();
}

#[test]
fn agrees_with_the_text_interpretation_on_the_same_program() {
    let poem = PoemHandlerImpl::new(App(40));
    let text = TextHandlerImpl;

    let described = text.compile_http(text.example_api::<App>());
    let compiled = poem.compile_http(poem.example_api::<App>());

    assert_eq!(compiled.labels(), described.labels());
    assert_eq!(compiled.labels(), ["GET /status", "GET /status/:id", "POST /set_temp", "GET /download"]);
}
