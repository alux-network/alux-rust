//! Test code, not user code: the scenario both surfaces in this crate must satisfy.
//!
//! A downstream author writes nothing like this file. It exists so that two independently built
//! surfaces can be held to one set of observations, which is what makes their agreement evidence
//! rather than coincidence.
//!
//! Nothing here mentions the domain either. The scenario knows only method and path selectors, wire
//! shapes, and expected values, so any service exposing that surface can be checked against it.

use anyhow::{Context, Result, ensure};
use core::fmt::Debug;
use poem::http::{Method, header};
use poem::{Endpoint, Request, Response};
use serde::de::DeserializeOwned;

/// Runs the shared behavioral scenario against a compiled HTTP surface.
///
/// # Errors
///
/// Returns an error when a request fails or an observed response differs from the specification.
pub async fn expect_example_api<E>(app: &E) -> Result<()>
where
    E: Endpoint<Output = Response>,
{
    expect_endpoint(app, Request::builder().uri_str("/status").finish(), 40_u32).await?;
    expect_endpoint(app, Request::builder().uri_str("/status/2").finish(), 42_u32).await?;
    expect_endpoint(app, post_json("/set_temp", "2.0"), 42_u32).await?;

    expect_download(app, "/download", "data.bin", "data").await
}

async fn expect_endpoint<E, Output>(app: &E, request: Request, expected: Output) -> Result<()>
where
    E: Endpoint<Output = Response>,
    Output: DeserializeOwned + PartialEq + Debug,
{
    let selector = format!("{} {}", request.method(), request.uri().path());
    let mut response = call(app, request).await?;

    ensure!(response.status().is_success(), "`{selector}` returned status {}", response.status());

    let body = response.take_body().into_bytes().await?;
    let actual: Output =
        serde_json::from_slice(&body).with_context(|| format!("decoding the response of `{selector}`"))?;
    ensure!(actual == expected, "`{selector}` returned {actual:?}, expected {expected:?}");

    Ok(())
}

async fn expect_download<E>(app: &E, path: &str, name: &str, expected: &str) -> Result<()>
where
    E: Endpoint<Output = Response>,
{
    let mut response = call(app, Request::builder().uri_str(path).finish()).await?;

    ensure!(response.status().is_success(), "`GET {path}` returned status {}", response.status());

    let disposition = response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .map(|value| value.to_str().unwrap_or_default().to_owned())
        .unwrap_or_default();
    ensure!(
        disposition == format!("attachment; filename=\"{name}\""),
        "`GET {path}` offered `{disposition}`, expected the download filename `{name}`"
    );

    let actual = response.take_body().into_string().await?;
    ensure!(actual == expected, "`GET {path}` returned `{actual}`, expected `{expected}`");

    Ok(())
}

fn post_json(path: &str, body: &'static str) -> Request {
    Request::builder().method(Method::POST).uri_str(path).content_type("application/json").body(body)
}

async fn call<E>(app: &E, request: Request) -> Result<Response>
where
    E: Endpoint<Output = Response>,
{
    app.call(request).await.map_err(|error| anyhow::anyhow!(error.to_string()))
}
