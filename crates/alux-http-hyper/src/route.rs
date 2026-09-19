//! The compiled surface, as a route hyper can serve.

use crate::message::{BODY_LIMIT, HyperAnswer, answered, asked};
use alux_http_direct::DirectRoute;
use bytes::Bytes;
use core::convert::Infallible;
use core::fmt::Display;
use core::future::Future;
use core::pin::Pin;
use derive_new::new as New;
use hyper::Request;
use hyper::body::Body;
use hyper::service::Service;

/// Serves one compiled surface over hyper.
///
/// The surface does the routing and the reading; this states only how a hyper request becomes one a
/// surface answers, and how that answer becomes a hyper response.
#[derive(Clone, New)]
pub struct HyperRoute {
    surface: DirectRoute,
    #[new(value = "BODY_LIMIT")]
    reading: usize,
}

impl HyperRoute {
    /// Reads a request body of at most `bytes`, answering `413` for one larger than that.
    ///
    /// What a service accepts is the service's to state. The default is a sane bound rather than a
    /// policy, so anything serving callers it does not control states its own.
    #[must_use]
    pub const fn reading(mut self, bytes: usize) -> Self {
        self.reading = bytes;

        self
    }

    /// Answers one request, whatever carried it here.
    ///
    /// A request that cannot be read is answered rather than dropped, because a caller that sent
    /// something unreadable is still owed an answer.
    pub async fn answer<Sent>(&self, request: Request<Sent>) -> HyperAnswer
    where
        Sent: Body<Data = Bytes>,
        Sent::Error: Display,
    {
        match asked(request, self.reading).await {
            Ok(asked) => answered(self.surface.answer(asked).await),
            Err(answer) => answered(answer),
        }
    }
}

impl<Sent> Service<Request<Sent>> for HyperRoute
where
    Sent: Body<Data = Bytes> + Send + 'static,
    Sent::Error: Display,
{
    type Response = HyperAnswer;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, request: Request<Sent>) -> Self::Future {
        let served = self.clone();

        Box::pin(async move { Ok(served.answer(request).await) })
    }
}
