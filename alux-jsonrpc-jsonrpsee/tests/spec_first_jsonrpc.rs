//! Declares the surface the specification-first way: one method program over the derived operations,
//! compiled by the jsonrpsee interpreter.
//!
//! `spec_first_jsonrpc_merge` states the same surface as separately declared programs instead.

mod common;
mod expect;

use alux_ext::ext;
use alux_jsonrpc::{JsonRpcApiAlg, JsonRpcProgramExt, jsonrpc};
use alux_jsonrpc_jsonrpsee::JsonrpseeImpl;
use common::{
    App, ItemsAlg, JsonrpcItemsCurrentOperation, JsonrpcStatusAdjustedOperation, JsonrpcStatusCurrentOperation,
    JsonrpcStatusHistoryOperation, JsonrpcStatusOffsetOperation, StatusAlg,
};
use expect::expect_example_rpc;

#[ext(name = ExampleRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares the whole surface: three status methods and one item listing.
    fn example_rpc<Alg>(&self)
    where
        Alg: StatusAlg + ItemsAlg,
    {
        self.methods()
            // The reading as it stands, taking no parameters.
            .method("status_current", self.op(Alg::jsonrpc_status_current))
            // An adjustment, decoded from a JSON array.
            .method("status_set_temp", self.op(Alg::jsonrpc_status_adjusted).positional())
            // The same operation, decoded from a JSON object using the authored argument names.
            .method("status_set_temp_named", self.op(Alg::jsonrpc_status_adjusted).named())
            // An adjustment whose offset the caller may leave out.
            .method("status_set_temp_offset", self.op(Alg::jsonrpc_status_offset))
            // `.fallible()` converts the domain's error into a JSON-RPC protocol error, marked
            // here per method. The same can be stated once on the ext, as the merge example does.
            .method("status_history", self.op(Alg::jsonrpc_status_history).fallible())
            // Every item the domain holds.
            .method("items_current", self.op(Alg::jsonrpc_items_current))
    }
}

#[tokio::test]
async fn compiles_a_spec_first_program_into_jsonrpsee_methods() {
    let rpc = JsonrpseeImpl::new(App(40));
    let methods = rpc.compile_jsonrpc(rpc.example_rpc::<App>()).unwrap();

    expect_example_rpc(&methods).await.unwrap();
}
