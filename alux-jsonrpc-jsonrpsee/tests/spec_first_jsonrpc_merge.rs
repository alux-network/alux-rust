//! The same surface as `spec_first_jsonrpc`, composed by merging separately declared programs.
//!
//! Each program states one part of the surface and needs only the capabilities that part uses, so
//! independently published fragments compose into one method collection. `history_rpc` states its
//! failability once for the whole program rather than once per method.

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

#[ext(name = StatusRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares the status surface, including one operation offered under both parameter modes.
    fn status_rpc<Alg>(&self)
    where
        Alg: StatusAlg,
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
    }
}

/// Declares a surface whose every method can fail, stated once for the whole program.
#[ext(name = HistoryRpcExt, defunc(via = jsonrpc), fallible)]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares the history surface, whose readings answer as JSON-RPC errors.
    fn history_rpc<Alg>(&self)
    where
        Alg: StatusAlg,
    {
        // A reading this domain does not keep. The program already said it can fail.
        self.methods().method("status_history", self.op(Alg::jsonrpc_status_history))
    }
}

#[ext(name = ItemsRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares the item surface.
    fn items_rpc<Alg>(&self)
    where
        Alg: ItemsAlg,
    {
        // Every item the domain holds.
        self.methods().method("items_current", self.op(Alg::jsonrpc_items_current))
    }
}

#[ext(name = ExampleRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Composes the whole surface from the three independently declared programs.
    fn example_rpc<Alg>(&self)
    where
        Alg: StatusAlg + ItemsAlg,
    {
        self.methods().merge(self.status_rpc::<Alg>()).merge(self.history_rpc::<Alg>()).merge(self.items_rpc::<Alg>())
    }
}

#[tokio::test]
async fn merges_separately_declared_programs_into_jsonrpsee_methods() {
    let rpc = JsonrpseeImpl::new(App(40));
    let methods = rpc.compile_jsonrpc(rpc.example_rpc::<App>()).unwrap();

    expect_example_rpc(&methods).await.unwrap();
}
