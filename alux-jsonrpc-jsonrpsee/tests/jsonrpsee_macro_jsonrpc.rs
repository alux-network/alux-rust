//! User code, written the framework's way: a native `#[rpc(server)]` trait and a forwarding impl.
//!
//! The trait and its attributes are the framework's, and the forwarding impl delegates to the derived
//! operations in `common`. This is the surface a jsonrpsee author writes without any specification
//! layer, and it must satisfy the same scenario as the program compiled in `spec_first_jsonrpc`.

mod common;
mod expect;

use alux_jsonrpc_jsonrpsee::{RpcCtx, RpcErrorExt};
use common::{App, ItemsAlg, ItemsOperationExt, StatusAlg, StatusOperationExt};
use expect::expect_example_rpc;
use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee::proc_macros::rpc;

/// The RPC surface, declared with the framework's own attributes.
#[rpc(server)]
trait ExampleRpc {
    /// Answers `status_current` with the status as it stands.
    #[method(name = "status_current")]
    async fn current(&self) -> RpcResult<u32>;

    /// Answers `status_set_temp` with a positional temperature.
    #[method(name = "status_set_temp")]
    async fn set_temp(&self, temp: f32) -> RpcResult<u32>;

    /// Answers the same call with a named temperature, decoded from a JSON object.
    #[method(name = "status_set_temp_named", param_kind = map)]
    async fn set_temp_named(&self, temp: f32) -> RpcResult<u32>;

    /// Answers `status_set_temp_offset` with an offset the caller may leave out.
    #[method(name = "status_set_temp_offset")]
    async fn set_temp_offset(&self, temp: f32, offset: Option<f32>) -> RpcResult<u32>;

    /// Answers `items_current` with every item the domain holds.
    #[method(name = "items_current")]
    async fn items(&self) -> RpcResult<Vec<String>>;

    /// Answers `status_history` with the failure the domain states.
    #[method(name = "status_history")]
    async fn history(&self) -> RpcResult<u32>;
}

#[async_trait]
impl<Context> ExampleRpcServer for RpcCtx<Context>
where
    Context: ItemsAlg<Items = Vec<String>> + StatusAlg<Status = u32> + Send + Sync + 'static,
{
    async fn current(&self) -> RpcResult<u32> {
        Ok(self.jsonrpc_status_current().await)
    }

    async fn set_temp(&self, temp: f32) -> RpcResult<u32> {
        Ok(self.jsonrpc_status_adjusted(temp).await)
    }

    async fn set_temp_named(&self, temp: f32) -> RpcResult<u32> {
        Ok(self.jsonrpc_status_adjusted(temp).await)
    }

    async fn set_temp_offset(&self, temp: f32, offset: Option<f32>) -> RpcResult<u32> {
        Ok(self.jsonrpc_status_offset(temp, offset).await)
    }

    async fn items(&self) -> RpcResult<Vec<String>> {
        Ok(self.jsonrpc_items_current().await)
    }

    async fn history(&self) -> RpcResult<u32> {
        // A native author converts the domain's failure at their own boundary.
        self.jsonrpc_status_history().await.map_err(|failure| failure.to_rpc_error())
    }
}

#[tokio::test]
async fn generates_jsonrpc_from_a_jsonrpsee_service_trait_and_forwarding_impl() {
    let methods = ExampleRpcServer::into_rpc(RpcCtx::new(App(40))).into();

    expect_example_rpc(&methods).await.unwrap();
}
