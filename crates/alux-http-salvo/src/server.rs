use crate::SalvoRoute;
use alux_http::{HttpServerAlg, HttpServerSetup};
use core::time::Duration;
use salvo::conn::tcp::TcpAcceptor;
use salvo::server::ServerHandle;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

type SalvoServerError = Box<dyn std::error::Error + Send + Sync>;

/// Bounds how long Salvo drains connections before it drops its acceptor.
///
/// Salvo waits for every connection to close and drops the acceptor only after that wait, so with
/// no timeout one client holding a connection open holds the address with it.
const DRAIN: Duration = Duration::from_secs(5);

/// Serves a Salvo route at the address its setup names.
#[derive(Debug, Default)]
pub struct SalvoServer;

/// Carries an open Salvo server that is accepting requests.
pub struct SalvoOpen {
    handle: ServerHandle,
    task: JoinHandle<Result<(), SalvoServerError>>,
}

impl Drop for SalvoOpen {
    fn drop(&mut self) {
        self.handle.stop_graceful(DRAIN);
        self.task.abort();
    }
}

impl HttpServerAlg for SalvoServer {
    type Program = SalvoRoute;
    type Open = SalvoOpen;
    type Error = SalvoServerError;

    async fn open(&mut self, setup: HttpServerSetup<Self::Program>) -> Result<Self::Open, Self::Error> {
        let (bind, route) = setup.into_parts();
        // Salvo owns its accept loop but accepts through a `TcpAcceptor`, which is what its own
        // bind builds. Binding here makes a taken address fail like it does in the other crates.
        let acceptor = TcpAcceptor::try_from(TcpListener::bind(bind.address()).await?)?;
        let server = salvo::Server::new(acceptor);
        let handle = server.handle();
        let task = tokio::task::spawn_local(async move {
            server.serve(route.into_salvo()).await;
            Ok(())
        });

        Ok(SalvoOpen { handle, task })
    }

    async fn close(&mut self, open: &mut Self::Open) -> Result<(), Self::Error> {
        // The address is free once the task holding the acceptor ends, so wait for that.
        open.handle.stop_graceful(DRAIN);
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
