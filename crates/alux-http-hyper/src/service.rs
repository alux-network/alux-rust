//! Serves one compiled surface over hyper.

use crate::message::{HyperAnswer, answered, asked};
use alux_http_direct::DirectRoute;
use bytes::Bytes;
use core::convert::Infallible;
use core::fmt::Display;
use core::future::Future;
use core::pin::Pin;
use hyper::Request;
use hyper::body::Body;
use hyper::service::Service;

/// Serves one compiled surface over hyper.
///
/// The surface does the routing and the reading; this states only how a hyper request becomes one a
/// surface answers, and how that answer becomes a hyper response.
#[derive(Clone)]
pub struct HyperService {
    surface: DirectRoute,
}

impl HyperService {
    /// Serves the surface a program compiled.
    pub fn new(surface: DirectRoute) -> Self {
        Self { surface }
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
        match asked(request).await {
            Ok(asked) => answered(self.surface.answer(asked).await),
            Err(answer) => answered(answer),
        }
    }
}

impl<Sent> Service<Request<Sent>> for HyperService
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
