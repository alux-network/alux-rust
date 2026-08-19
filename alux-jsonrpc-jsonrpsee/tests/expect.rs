//! Test code, not user code: the scenario both surfaces in this crate must satisfy.
//!
//! A downstream author writes nothing like this file. It exists so that two independently built
//! surfaces can be held to one set of observations, which is what makes their agreement evidence
//! rather than coincidence.
//!
//! Nothing here mentions the domain either. The scenario knows only method names, parameter shapes,
//! and expected results, so any service exposing that surface can be checked against it.

use anyhow::{Context, Result, ensure};
use core::fmt::Debug;
use jsonrpsee::Methods;
use jsonrpsee::core::params::{ArrayParams, ObjectParams};
use jsonrpsee::core::traits::ToRpcParams;
use serde::de::DeserializeOwned;

/// Runs the shared behavioral scenario against a compiled JSON-RPC method set.
///
/// # Errors
///
/// Returns an error when a call fails or an observed result differs from the specification.
pub async fn expect_example_rpc(methods: &Methods) -> Result<()> {
    expect_method(methods, "status_current", ArrayParams::default(), 40_u32).await?;
    expect_method(methods, "status_set_temp", (2.0_f32,), 42_u32).await?;

    let mut named_params = ObjectParams::new();
    named_params.insert("temp", 3.0_f32).context("encoding named `temp` parameter")?;
    expect_method(methods, "status_set_temp_named", named_params, 43_u32).await?;

    expect_method(methods, "items_current", ArrayParams::default(), vec!["one".to_owned(), "two".to_owned()]).await
}

async fn expect_method<Params, Output>(methods: &Methods, method: &str, params: Params, expected: Output) -> Result<()>
where
    Params: ToRpcParams,
    Output: DeserializeOwned + Clone + PartialEq + Debug,
{
    let actual: Output = methods.call(method, params).await.with_context(|| format!("calling `{method}`"))?;
    ensure!(actual == expected, "`{method}` returned {actual:?}, expected {expected:?}");
    Ok(())
}
