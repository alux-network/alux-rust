//! One method reads sixteen parameters, positionally and by name.
//!
//! A method's parameters are read out of the document a caller sent, so what this guards is that
//! both readings stay in step with the argument product all the way up: the same sixteen values
//! reach the same sixteen arguments whether the caller wrote an array or an object.

#![allow(async_fn_in_trait)]

use alux_ext::ext;
use alux_jsonrpc::{JsonRpcApiAlg, JsonRpcProgramExt, jsonrpc};
use alux_jsonrpc_direct::{DirectImpl, MethodTable};
use core::future::Future;

/// Adds up whatever a caller states.
trait SumAlg {
    /// Returns the total the domain makes of a reading.
    fn total(&self, of: u32) -> impl Future<Output = u32> + Send;
}

#[ext(name = SumOperationExt, defunc)]
impl<This> This
where
    This: SumAlg,
{
    /// Returns the total of sixteen stated readings.
    #[allow(clippy::too_many_arguments)]
    async fn jsonrpc_sum_all(
        &self,
        first: u32,
        second: u32,
        third: u32,
        fourth: u32,
        fifth: u32,
        sixth: u32,
        seventh: u32,
        eighth: u32,
        ninth: u32,
        tenth: u32,
        eleventh: u32,
        twelfth: u32,
        thirteenth: u32,
        fourteenth: u32,
        fifteenth: u32,
        sixteenth: u32,
    ) -> u32 {
        let stated = first
            + second
            + third
            + fourth
            + fifth
            + sixth
            + seventh
            + eighth
            + ninth
            + tenth
            + eleventh
            + twelfth
            + thirteenth
            + fourteenth
            + fifteenth
            + sixteenth;

        self.total(stated).await
    }
}

#[ext(name = SumRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares the same sixteen-parameter method under both readings of a parameter document.
    fn sum_rpc<Alg>(&self)
    where
        Alg: SumAlg,
    {
        self.methods()
            // Sixteen values, decoded from a JSON array.
            .method("sum_all", self.op(Alg::jsonrpc_sum_all).positional())
            // The same sixteen, decoded from a JSON object using the authored argument names.
            .method("sum_all_named", self.op(Alg::jsonrpc_sum_all).named())
    }
}

/// Supplies the domain the observations run against.
struct App;

impl SumAlg for App {
    async fn total(&self, of: u32) -> u32 {
        of
    }
}

fn surface() -> MethodTable {
    let rpc = DirectImpl::new(App);

    rpc.compile_jsonrpc(rpc.sum_rpc::<App>()).unwrap()
}

#[tokio::test]
async fn reads_sixteen_positional_parameters() {
    let values = (1..=16).map(|value| value.to_string()).collect::<Vec<_>>().join(",");
    let request = format!(r#"{{"jsonrpc":"2.0","method":"sum_all","params":[{values}],"id":1}}"#);

    assert_eq!(surface().dispatch(&request).await.unwrap(), r#"{"jsonrpc":"2.0","result":136,"id":1}"#);
}

#[tokio::test]
async fn reads_sixteen_named_parameters() {
    let names = [
        "first",
        "second",
        "third",
        "fourth",
        "fifth",
        "sixth",
        "seventh",
        "eighth",
        "ninth",
        "tenth",
        "eleventh",
        "twelfth",
        "thirteenth",
        "fourteenth",
        "fifteenth",
        "sixteenth",
    ];
    let members = names
        .iter()
        .enumerate()
        .map(|(index, name)| format!(r#""{name}":{}"#, index + 1))
        .collect::<Vec<_>>()
        .join(",");
    let request = format!(r#"{{"jsonrpc":"2.0","method":"sum_all_named","params":{{{members}}},"id":1}}"#);

    // The same sixteen values reach the same sixteen arguments, however the caller wrote them.
    assert_eq!(surface().dispatch(&request).await.unwrap(), r#"{"jsonrpc":"2.0","result":136,"id":1}"#);
}
