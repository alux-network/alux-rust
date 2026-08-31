//! User code: the specification, its derived operations, then its implementation.
//!
//! `StatusAlg` and `ItemsAlg` are the specification — what the domain promises, stated with no
//! transport, no framework, and no interpreter in sight. The `#[ext(defunc)]` blocks derive the
//! operations an API actually exposes, and `App` is the implementation: one concrete answer to those
//! promises.
//!
//! This is everything a downstream author writes before deciding how the outside world reaches the
//! domain, which is why both examples in this crate share the file unchanged: the specification-first
//! program and the native jsonrpsee service expose the same `App` two different ways.

#![allow(async_fn_in_trait)]

use alux_ext::ext;
use alux_jsonrpc::RpcErrorAlg;
use core::fmt::{self, Display};
use core::future::Future;

/// What every fallible reading in this domain answers with, named once like a real domain does.
pub type StatusResult<Value> = Result<Value, NoHistory>;

/// The one reason this domain fails: it keeps no history.
#[derive(Debug)]
pub struct NoHistory;

impl Display for NoHistory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the domain keeps no history")
    }
}

/// The domain's one statement about its failure on an RPC surface: the code it carries and the
/// message it states. No interpreter is named here, or anywhere else in this file.
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

/// Derives the status operations an API exposes.
#[ext(name = StatusOperationExt, defunc)]
pub impl<This> This
where
    This: StatusAlg,
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
}

/// Derives the item operations an API exposes.
#[ext(name = ItemsOperationExt, defunc)]
pub impl<This> This
where
    This: ItemsAlg,
{
    /// Returns the items as they stand.
    async fn jsonrpc_items_current(&self) -> This::Items {
        self.items()
    }
}

/// Supplies the shared domain interpretation used by both JSON-RPC examples.
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
