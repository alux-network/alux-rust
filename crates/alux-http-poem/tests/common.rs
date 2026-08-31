//! User code: the specification, its derived operations, then its implementation.
//!
//! `StatusAlg` and `DownloadAlg` are the specification — what the domain promises, stated with no
//! transport, no framework, and no interpreter in sight. The `#[ext(defunc)]` blocks derive the
//! operations an API actually exposes, and `App` is the implementation: one concrete answer to those
//! promises.
//!
//! This is everything a downstream author writes before deciding how the outside world reaches the
//! domain, which is why both examples in this crate share the file unchanged: the specification-first
//! program and the hand-written Poem service expose the same `App` two different ways.

#![allow(async_fn_in_trait)]

use alux_ext::ext;
use core::convert::Infallible;
use core::future::Future;
use poem::Body;

/// Reads and adjusts the status of whatever the domain observes.
pub trait StatusAlg {
    /// The value a status reading denotes.
    type Status;

    /// Returns the current status.
    fn status(&self) -> impl Future<Output = Self::Status> + Send;
    /// Returns the status recorded for `id`.
    fn status_at(&self, id: u32) -> impl Future<Output = Self::Status> + Send;
    /// Applies `temp` and returns the resulting status.
    fn status_set_temp(&self, temp: f32) -> impl Future<Output = Self::Status> + Send;
}

/// Offers one file for download.
pub trait DownloadAlg {
    /// The file contents a download denotes.
    type File;
    /// The reason a file cannot be read.
    type Error;

    /// Returns the file name offered to the caller.
    fn dl_name(&self) -> &'static str;
    /// Reads the file.
    fn dl_file(&self) -> impl Future<Output = Result<Self::File, Self::Error>> + Send;
}

/// Derives the status operations an API exposes.
#[ext(name = StatusOperationExt, defunc)]
pub impl<This> This
where
    This: StatusAlg,
{
    /// Returns the status as it stands.
    async fn status_current(&self) -> This::Status {
        self.status().await
    }

    /// Returns the status of one identified reading.
    async fn status_for_id(&self, id: u32) -> This::Status {
        self.status_at(id).await
    }

    /// Returns the status after adjusting the temperature.
    async fn status_adjusted(&self, temp: f32) -> This::Status {
        self.status_set_temp(temp).await
    }
}

/// Derives the download operation an API exposes.
#[ext(name = DownloadOperationExt, defunc)]
pub impl<This> This
where
    This: DownloadAlg,
{
    /// Returns the file to send and the name to offer it under.
    async fn download_current(&self) -> (Result<This::File, This::Error>, String) {
        let name = self.dl_name().to_owned();

        (self.dl_file().await, name)
    }
}

/// Supplies the shared domain interpretation used by both HTTP examples.
pub struct App(pub i16);

impl StatusAlg for App {
    type Status = u32;

    #[allow(clippy::cast_sign_loss)]
    async fn status(&self) -> Self::Status {
        self.0 as u32
    }

    #[allow(clippy::cast_sign_loss)]
    async fn status_at(&self, id: u32) -> Self::Status {
        self.0 as u32 + id
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    async fn status_set_temp(&self, temp: f32) -> Self::Status {
        self.0 as u32 + temp as u32
    }
}

impl DownloadAlg for App {
    type File = Body;
    type Error = Infallible;

    fn dl_name(&self) -> &'static str {
        "data.bin"
    }

    async fn dl_file(&self) -> Result<Body, Infallible> {
        Ok(Body::from_bytes((&b"data"[..]).into()))
    }
}
