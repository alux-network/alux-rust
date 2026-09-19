use alux_ext::ext;
use alux_http::{HttpApiAlg, RedirectOutAlg, TextOutAlg, http};
use alux_shape::Shape;
use core::future::Future;
use serde::{Deserialize, Serialize};

/// States the capability the shared HTTP specification consumes.
pub trait ExampleAppAlg {
    /// Returns the provider serving this application.
    fn provider(&self) -> impl Future<Output = String> + Send;

    /// Selects the provider named by the caller.
    fn switch(&self, provider: Provider) -> impl Future<Output = String> + Send;

    /// Returns the OpenAPI document for this API.
    fn openapi(&self) -> impl Future<Output = String> + Send;

    /// Returns the TypeScript client for this API.
    fn typescript(&self) -> impl Future<Output = String> + Send;
}

/// Names one HTTP interpreter serving the example.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Shape)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// Selects the dynamic front door.
    Dynamic,
    /// Selects axum.
    Axum,
    /// Selects Actix Web.
    Actix,
    /// Selects Poem.
    Poem,
    /// Selects Rocket.
    Rocket,
    /// Selects Salvo.
    Salvo,
    /// Selects warp.
    Warp,
    /// Selects hyper.
    Hyper,
}

#[ext(name = ExampleOperationExt, defunc)]
pub impl<This> This
where
    This: ExampleAppAlg,
{
    /// Returns the provider serving this application.
    async fn example_provider(&self) -> String {
        self.provider().await
    }

    /// Selects the API path named by the request body.
    async fn example_switch(&self, provider: Provider) -> String {
        self.switch(provider).await
    }

    /// Returns the OpenAPI document for this API.
    async fn example_openapi(&self) -> String {
        self.openapi().await
    }

    /// Returns the TypeScript client for this API.
    async fn example_typescript(&self) -> String {
        self.typescript().await
    }
}

#[ext(name = ExampleApiExt, defunc(via = http))]
pub impl<This> This
where
    This: HttpApiAlg + RedirectOutAlg + TextOutAlg,
{
    /// Declares the shared API endpoints.
    fn example_api<Alg>(&self)
    where
        Alg: ExampleAppAlg,
    {
        self.routes()
            .get("/api", self.op(Alg::example_provider).text())
            .post("/fw", self.op(Alg::example_switch).body::<Provider>().json())
            .get("/openapi-spec", self.op(Alg::example_openapi).text())
            .get("/ts-spec", self.op(Alg::example_typescript).text())
    }
}
