//! User code, then the protocol observations a JSON-RPC surface must satisfy.
//!
//! The domain and its declared surface are what a downstream author writes. Everything after them
//! observes the specification itself: request documents in, response documents out.

#![allow(async_fn_in_trait)]

use alux_ext::ext;
use alux_jsonrpc::{JsonRpcApiAlg, JsonRpcProgramExt, RpcErrorAlg, jsonrpc};
use alux_jsonrpc_direct::{DirectImpl, MethodTable};
use core::fmt::{self, Display};
use core::future::Future;

/// What every fallible reading in this domain answers with.
pub type StatusResult<Value> = Result<Value, NoHistory>;

/// The one reason this domain fails: it keeps no history.
#[derive(Debug)]
pub struct NoHistory;

impl Display for NoHistory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the domain keeps no history")
    }
}

/// The domain's one statement about its failure on an RPC surface.
impl RpcErrorAlg for NoHistory {
    fn rpc_code(&self) -> i32 {
        -32000
    }

    fn rpc_message(&self) -> String {
        self.to_string()
    }
}

/// Reads and adjusts the status of whatever the domain observes.
pub trait StatusAlg {
    /// The value a status reading denotes.
    type Status;

    /// Returns the current status.
    fn status(&self) -> impl Future<Output = Self::Status> + Send;
    /// Applies `temp` and returns the resulting status.
    fn status_set_temp(&self, temp: f32) -> impl Future<Output = Self::Status> + Send;
}

/// Lists what the domain holds.
pub trait ItemsAlg {
    /// The collection an item listing denotes.
    type Items;

    /// Returns every item.
    fn items(&self) -> Self::Items;
}

/// Derives the operations an API exposes.
#[ext(name = StatusOperationExt, defunc)]
pub impl<This> This
where
    This: StatusAlg + ItemsAlg,
{
    /// Returns the status as it stands.
    async fn jsonrpc_status_current(&self) -> This::Status {
        self.status().await
    }

    /// Returns the status after adjusting the temperature.
    async fn jsonrpc_status_adjusted(&self, temp: f32) -> This::Status {
        self.status_set_temp(temp).await
    }

    /// Returns the status after adjusting by an offset the caller may leave out.
    async fn jsonrpc_status_offset(&self, temp: f32, offset: Option<f32>) -> This::Status {
        self.status_set_temp(temp + offset.unwrap_or_default()).await
    }

    /// Returns the reading this domain does not keep, stating its own failure.
    async fn jsonrpc_status_history(&self) -> StatusResult<This::Status> {
        Err(NoHistory)
    }

    /// Returns the items as they stand.
    async fn jsonrpc_items_current(&self) -> This::Items {
        self.items()
    }
}

/// Declares the whole surface, mixing a converting method with methods that answer with a value.
#[ext(name = ExampleRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares every method this service answers to.
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
            // `.fallible()` converts the domain's error into a JSON-RPC protocol error.
            .method("status_history", self.op(Alg::jsonrpc_status_history).fallible())
            // Every item the domain holds.
            .method("items_current", self.op(Alg::jsonrpc_items_current))
    }
}

/// Supplies the domain interpretation the observations run against.
pub struct App(pub i16);

impl StatusAlg for App {
    type Status = u32;

    #[allow(clippy::cast_sign_loss)]
    async fn status(&self) -> Self::Status {
        self.0 as u32
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    async fn status_set_temp(&self, temp: f32) -> Self::Status {
        self.0 as u32 + temp as u32
    }
}

impl ItemsAlg for App {
    type Items = Vec<String>;

    fn items(&self) -> Self::Items {
        vec!["one".into(), "two".into()]
    }
}

/// Compiles the declared surface, which is the only setup every observation needs.
fn surface() -> MethodTable {
    let rpc = DirectImpl::new(App(40));

    rpc.compile_jsonrpc(rpc.example_rpc::<App>()).unwrap()
}

#[tokio::test]
async fn answers_every_declared_method() {
    let rpc = surface();

    assert_eq!(
        rpc.names(),
        [
            "items_current",
            "status_current",
            "status_history",
            "status_set_temp",
            "status_set_temp_named",
            "status_set_temp_offset"
        ]
    );
}

#[tokio::test]
async fn answers_a_method_with_the_value_the_domain_states() {
    let rpc = surface();

    assert_eq!(
        rpc.dispatch(r#"{"jsonrpc":"2.0","method":"status_current","id":1}"#).await.unwrap(),
        r#"{"jsonrpc":"2.0","result":40,"id":1}"#
    );
    assert_eq!(
        rpc.dispatch(r#"{"jsonrpc":"2.0","method":"items_current","params":[],"id":"a"}"#).await.unwrap(),
        r#"{"jsonrpc":"2.0","result":["one","two"],"id":"a"}"#
    );
}

#[tokio::test]
async fn decodes_positional_and_named_parameters() {
    let rpc = surface();

    assert_eq!(
        rpc.dispatch(r#"{"jsonrpc":"2.0","method":"status_set_temp","params":[2.0],"id":1}"#).await.unwrap(),
        r#"{"jsonrpc":"2.0","result":42,"id":1}"#
    );
    assert_eq!(
        rpc.dispatch(r#"{"jsonrpc":"2.0","method":"status_set_temp_named","params":{"temp":3.0},"id":1}"#)
            .await
            .unwrap(),
        r#"{"jsonrpc":"2.0","result":43,"id":1}"#
    );
}

#[tokio::test]
async fn reads_an_omitted_optional_parameter_as_absent() {
    let rpc = surface();

    assert_eq!(
        rpc.dispatch(r#"{"jsonrpc":"2.0","method":"status_set_temp_offset","params":[1.0,1.0],"id":1}"#).await.unwrap(),
        r#"{"jsonrpc":"2.0","result":42,"id":1}"#
    );
    assert_eq!(
        rpc.dispatch(r#"{"jsonrpc":"2.0","method":"status_set_temp_offset","params":[2.0],"id":1}"#).await.unwrap(),
        r#"{"jsonrpc":"2.0","result":42,"id":1}"#
    );
}

#[tokio::test]
async fn converts_the_domains_failure_into_a_protocol_error() {
    let rpc = surface();

    assert_eq!(
        rpc.dispatch(r#"{"jsonrpc":"2.0","method":"status_history","id":7}"#).await.unwrap(),
        r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"the domain keeps no history"},"id":7}"#
    );
}

#[tokio::test]
async fn states_the_protocols_own_errors() {
    let rpc = surface();

    // A document that is not JSON at all.
    assert_eq!(
        rpc.dispatch("{ not json").await.unwrap(),
        r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"invalid JSON"},"id":null}"#
    );
    // JSON that is not a request.
    assert_eq!(
        rpc.dispatch("[1]").await.unwrap(),
        r#"[{"jsonrpc":"2.0","error":{"code":-32600,"message":"invalid request"},"id":null}]"#
    );
    // A version this surface does not speak.
    assert_eq!(
        rpc.dispatch(r#"{"jsonrpc":"1.0","method":"status_current","id":1}"#).await.unwrap(),
        r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"invalid request"},"id":1}"#
    );
    // A name no method answers to.
    assert_eq!(
        rpc.dispatch(r#"{"jsonrpc":"2.0","method":"nope","id":1}"#).await.unwrap(),
        r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"method `nope` not found"},"id":1}"#
    );
    // A parameter the method cannot read.
    assert_eq!(
        rpc.dispatch(r#"{"jsonrpc":"2.0","method":"status_set_temp","params":[],"id":1}"#).await.unwrap(),
        r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"missing parameter at position 0"},"id":1}"#
    );
    // An empty batch asks for nothing coherent.
    assert_eq!(
        rpc.dispatch("[]").await.unwrap(),
        r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"invalid request"},"id":null}"#
    );
}

#[tokio::test]
async fn says_nothing_to_a_notification() {
    let rpc = surface();

    // No identifier, so nothing to answer to — including when the call itself is unanswerable.
    assert!(rpc.dispatch(r#"{"jsonrpc":"2.0","method":"status_current"}"#).await.is_none());
    assert!(rpc.dispatch(r#"{"jsonrpc":"2.0","method":"nope"}"#).await.is_none());
    // A null identifier is an identifier, so the call is answered.
    assert_eq!(
        rpc.dispatch(r#"{"jsonrpc":"2.0","method":"status_current","id":null}"#).await.unwrap(),
        r#"{"jsonrpc":"2.0","result":40,"id":null}"#
    );
}

#[tokio::test]
async fn answers_a_batch_with_one_response_per_answerable_call() {
    let rpc = surface();

    assert_eq!(
        rpc.dispatch(
            r#"[{"jsonrpc":"2.0","method":"status_current","id":1},
                {"jsonrpc":"2.0","method":"status_set_temp","params":[2.0]},
                {"jsonrpc":"2.0","method":"status_history","id":2}]"#
        )
        .await
        .unwrap(),
        concat!(
            r#"[{"jsonrpc":"2.0","result":40,"id":1},"#,
            r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"the domain keeps no history"},"id":2}]"#
        )
    );
    // A batch of notifications asks for nothing.
    assert!(
        rpc.dispatch(r#"[{"jsonrpc":"2.0","method":"status_current"},{"jsonrpc":"2.0","method":"status_current"}]"#)
            .await
            .is_none()
    );
}

#[tokio::test]
async fn refuses_a_surface_that_declares_one_name_twice() {
    let rpc = DirectImpl::new(App(40));
    let doubled = rpc.compile_jsonrpc(rpc.example_rpc::<App>()).unwrap();
    let error = doubled.merge(rpc.compile_jsonrpc(rpc.example_rpc::<App>()).unwrap()).unwrap_err();

    assert_eq!(error.to_string(), "method `items_current` is declared twice");
}
