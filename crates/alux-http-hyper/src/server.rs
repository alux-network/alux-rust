use crate::{DRAIN, HyperConnections, HyperRoute};
use alux_http::{HttpServerAlg, HttpServerSetup};
use tokio::net::TcpListener;

type HyperServerError = Box<dyn std::error::Error + Send + Sync>;

/// Serves a direct HTTP surface over hyper at the address its setup names.
#[derive(Debug, Default)]
pub struct HyperServer;

impl HttpServerAlg for HyperServer {
    type Program = HyperRoute;
    type Open = HyperConnections;
    type Error = HyperServerError;

    async fn open(&mut self, setup: HttpServerSetup<Self::Program>) -> Result<Self::Open, Self::Error> {
        let (bind, route) = setup.into_parts();
        let listener = TcpListener::bind(bind.address()).await?;

        Ok(HyperConnections::serve(listener, route, DRAIN))
    }

    async fn close(&mut self, open: &mut Self::Open) -> Result<(), Self::Error> {
        open.close().await;

        Ok(())
    }

    async fn end(&mut self, open: &mut Self::Open) -> Result<(), Self::Error> {
        open.end().await;

        Ok(())
    }
}
