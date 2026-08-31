//! User code, written the framework's way: a `poem-openapi` service over the same capabilities.
//!
//! The attributes and response payloads are the framework's, and the methods delegate to the derived
//! operations in `common`. This is the surface a Poem author writes without any specification layer,
//! and it must satisfy the same scenario as the program compiled in `spec_first_http`.

mod common;
mod expect;

use common::{App, DownloadAlg, DownloadOperationExt, StatusAlg, StatusOperationExt};
use core::convert::Infallible;
use expect::expect_example_api;
use poem::{Body, Route};
use poem_openapi::param::Path;
use poem_openapi::payload::{Attachment, Json};
use poem_openapi::{OpenApi, OpenApiService};

struct Api<T>(T);

#[OpenApi]
impl<T> Api<T>
where
    T: DownloadAlg<File = Body, Error = Infallible> + StatusAlg<Status = u32> + Send + Sync + 'static,
{
    /// Returns the reading as it stands.
    #[oai(path = "/status", method = "get")]
    async fn get_status(&self) -> Json<u32> {
        Json(self.0.status_current().await)
    }

    /// Returns one identified reading, its id taken from the path.
    #[oai(path = "/status/:id", method = "get")]
    async fn get_status_for_id(&self, id: Path<u32>) -> Json<u32> {
        Json(self.0.status_for_id(*id).await)
    }

    /// Applies an adjustment, its temperature taken from the request body.
    #[oai(path = "/set_temp", method = "post")]
    async fn post_adjusted_temp(&self, adj_temp: Json<f32>) -> Json<u32> {
        Json(self.0.status_adjusted(*adj_temp).await)
    }

    /// Sends the file under the name the domain offers it as.
    #[oai(path = "/download", method = "get")]
    async fn download_file(&self) -> Attachment<Body> {
        let (file, name) = self.0.download_current().await;
        let file = file.expect("this domain cannot fail to read");

        Attachment::new(file).filename(name)
    }
}

#[tokio::test]
async fn generates_http_from_a_native_poem_openapi_service() {
    let service = OpenApiService::new(Api(App(40)), "example", "1.0");
    let route = Route::new().nest("/", service);

    expect_example_api(&route).await.unwrap();
}
