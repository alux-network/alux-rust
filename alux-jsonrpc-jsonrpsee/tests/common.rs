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
use core::future::Future;

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
