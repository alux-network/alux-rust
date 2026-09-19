use crate::PoemRoute;
use alux_http::{HttpServerAlg, HttpServerSetup};
use core::time::Duration;
use poem::listener::TcpAcceptor;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

type PoemServerError = Box<dyn std::error::Error + Send + Sync>;

/// Bounds how long Poem drains connections before it ends them.
///
/// The same bound the other interpretations state, so a request still being served this long after
/// a close is ended wherever it was made.
const DRAIN: Duration = Duration::from_secs(5);

/// Serves a Poem route at the address its setup names.
#[derive(Debug, Default)]
pub struct PoemServer;

/// Carries an open Poem server that is accepting requests.
pub struct PoemOpen {
    shutdown: Option<oneshot::Sender<()>>,
    task: JoinHandle<Result<(), PoemServerError>>,
}

impl Drop for PoemOpen {
    fn drop(&mut self) {
        drop(self.shutdown.take());
        self.task.abort();
    }
}

impl HttpServerAlg for PoemServer {
    type Program = PoemRoute;
    type Open = PoemOpen;
    type Error = PoemServerError;

    async fn open(&mut self, setup: HttpServerSetup<Self::Program>) -> Result<Self::Open, Self::Error> {
        let (bind, route) = setup.into_parts();
        let listener = TcpListener::bind(bind.address()).await?;
        let acceptor = TcpAcceptor::from_tokio(listener)?;
        let (shutdown, closing) = oneshot::channel();
        // Poem owns the accept loop but takes a shutdown signal, so closing asks rather than
        // aborts: it drops the acceptor at once and gives what it is serving the drain to finish.
        let task = tokio::task::spawn_local(async move {
            poem::Server::new_with_acceptor(acceptor)
                .run_with_graceful_shutdown(
                    route.into_poem(),
                    async move {
                        let _ = closing.await;
                    },
                    Some(DRAIN),
                )
                .await
                .map_err(Into::into)
        });

        Ok(PoemOpen { shutdown: Some(shutdown), task })
    }

    async fn close(&mut self, open: &mut Self::Open) -> Result<(), Self::Error> {
        // The address is free once the task holding the acceptor ends, so wait for that.
        drop(open.shutdown.take());
        if !open.task.is_finished() {
            let _ = (&mut open.task).await;
        }

        Ok(())
    }

    // Nothing here can tell the address being released from the drain being over, because both
    // happen inside the framework's own shutdown. So ending is closing, which satisfies both.
    async fn end(&mut self, open: &mut Self::Open) -> Result<(), Self::Error> {
        self.close(open).await
    }
}
