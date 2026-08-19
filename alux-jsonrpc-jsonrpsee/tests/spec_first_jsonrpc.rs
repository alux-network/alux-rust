//! Declares the surface the specification-first way: method programs over the derived operations,
//! compiled by the jsonrpsee interpreter.

mod common;
mod expect;

use alux_ext::ext;
use alux_jsonrpc::{JsonRpcApiAlg, JsonRpcProgramExt, jsonrpc};
use alux_jsonrpc_jsonrpsee::JsonrpseeImpl;
use common::{
    App, ItemsAlg, JsonrpcItemsCurrentOperation, JsonrpcStatusAdjustedOperation, JsonrpcStatusCurrentOperation,
    StatusAlg,
};
use expect::expect_example_rpc;

#[ext(name = StatusRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    fn status_rpc<Alg>(&self)
    where
        Alg: StatusAlg,
    {
        self.methods()
            .method("status_current", self.op(Alg::jsonrpc_status_current))
            .method("status_set_temp", self.op(Alg::jsonrpc_status_adjusted).positional())
            .method("status_set_temp_named", self.op(Alg::jsonrpc_status_adjusted).named())
    }
}

#[ext(name = ItemsRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    fn items_rpc<Alg>(&self)
    where
        Alg: ItemsAlg,
    {
        self.methods().method("items_current", self.op(Alg::jsonrpc_items_current))
    }
}

#[ext(name = ExampleRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    fn example_rpc<Alg>(&self)
    where
        Alg: StatusAlg + ItemsAlg,
    {
        self.methods().merge(self.status_rpc::<Alg>()).merge(self.items_rpc::<Alg>())
    }
}

#[tokio::test]
async fn compiles_a_spec_first_program_into_jsonrpsee_methods() {
    let rpc = JsonrpseeImpl::new(App(40));
    let methods = rpc.compile_jsonrpc(rpc.example_rpc::<App>()).unwrap();

    expect_example_rpc(&methods).await.unwrap();
}
