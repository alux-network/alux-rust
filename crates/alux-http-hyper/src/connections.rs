//! Accepting at one address, for interpreters whose framework states a service rather than a loop.

use core::cell::RefCell;
use core::error::Error;
use core::pin::pin;
use core::time::Duration;
use futures::{FutureExt, StreamExt};
use hyper::body::{Body, Incoming};
use hyper::service::Service;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use std::rc::Rc;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::TcpListenerStream;

/// Carries an error returned while serving connections.
pub type ConnectionsError = Box<dyn Error + Send + Sync>;

/// The drain to serve with where a caller states none of its own.
///
/// The same bound the framework-owning interpretations state, so a request outliving a close is
/// ended after the same wait wherever it was made.
pub const DRAIN: Duration = Duration::from_secs(5);

/// Accepts at one address and serves each connection with one service.
///
/// This is all a framework needs to be served when it exposes a service rather than an accept loop.
/// The listener stays here, so closing drops it and frees the address immediately, while
/// connections already accepted are given the drain to finish.
pub struct HyperConnections {
    accepting: JoinHandle<Result<(), ConnectionsError>>,
    /// Tells every connection to finish what it is answering and then stop.
    closing: watch::Sender<bool>,
    /// Every connection accepted and not yet finished, which closing drains and then ends.
    serving: Rc<RefCell<Vec<JoinHandle<()>>>>,
    /// How long closing waits for those before ending them.
    drain: Duration,
}

impl Drop for HyperConnections {
    fn drop(&mut self) {
        self.accepting.abort();
        for connection in self.serving.borrow().iter() {
            connection.abort();
        }
    }
}

impl HyperConnections {
    /// Serves every connection accepted at `listener` with `service`, draining for `drain`.
    ///
    /// The drain is stated here rather than fixed, because how long a request may take is what the
    /// surface being served decides, not what accepting it does. [`DRAIN`] is the usual answer.
    pub fn serve<Answering, ResponseBody>(listener: TcpListener, service: Answering, drain: Duration) -> Self
    where
        Answering: Service<Request<Incoming>, Response = Response<ResponseBody>> + Clone + 'static,
        Answering::Error: Into<ConnectionsError>,
        Answering::Future: Send,
        ResponseBody: Body + Send + 'static,
        ResponseBody::Data: Send,
        ResponseBody::Error: Into<ConnectionsError>,
    {
        let serving = Rc::new(RefCell::new(Vec::<JoinHandle<()>>::new()));
        let (closing, closed) = watch::channel(false);
        // An accept error affects that connection only, not the listener, so keep accepting.
        let connections = TcpListenerStream::new(listener).filter_map(|accepted| async move { accepted.ok() });
        let accepting = tokio::task::spawn_local(
            connections
                .for_each({
                    let serving = Rc::clone(&serving);
                    move |stream| {
                        let service = service.clone();
                        let serving = Rc::clone(&serving);
                        let mut closed = closed.clone();
                        async move {
                            // One task per connection, so releasing the address leaves these
                            // running, and holding the handle is what lets closing end them.
                            let connection = tokio::task::spawn_local(async move {
                                let connection = Builder::new(TokioExecutor::new());
                                let mut connection = pin!(connection.serve_connection(TokioIo::new(stream), service));
                                tokio::select! {
                                    _ = connection.as_mut() => {}
                                    _ = closed.changed() => {
                                        // Finish the answer being produced, then stop rather than
                                        // wait for a request that is never coming.
                                        connection.as_mut().graceful_shutdown();
                                        let _ = connection.await;
                                    }
                                }
                            });
                            let mut serving = serving.borrow_mut();
                            serving.retain(|connection| !connection.is_finished());
                            serving.push(connection);
                        }
                    }
                })
                .map(Ok),
        );

        Self { accepting, closing, serving, drain }
    }

    /// Releases the address, leaving what is already being served to finish on its own.
    ///
    /// Aborting the accept task drops the listener, which is what frees the address, and awaiting
    /// the handle is what makes it free on return rather than shortly after. Each connection is
    /// then told to shut down gracefully, which is what makes one finish: it answers what it is
    /// producing and then ends, rather than waiting for a request no caller will send. This does
    /// not wait for that, which is what [`Self::end`] is for.
    pub async fn close(&mut self) {
        self.accepting.abort();
        // Only where it is still running: closing twice, or closing and then ending, would
        // otherwise poll a handle that has already answered, which panics.
        if !self.accepting.is_finished() {
            let _ = (&mut self.accepting).await;
        }
        let _ = self.closing.send(true);
    }

    /// Releases the address, then gives what is already being served up to the drain to finish.
    ///
    /// Whatever is still serving when the drain runs out is ended, so this resolves within the
    /// drain of [`Self::close`] however slow a request is.
    pub async fn end(&mut self) {
        self.close().await;

        let mut serving = self.serving.take();
        let draining = async {
            for connection in &mut serving {
                let _ = connection.await;
            }
        };
        let _ = tokio::time::timeout(self.drain, draining).await;

        for connection in &serving {
            connection.abort();
        }
    }
}
