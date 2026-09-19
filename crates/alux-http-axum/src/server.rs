use crate::AxumRoute;
use alux_http::{HttpServerAlg, HttpServerSetup};
use alux_http_hyper::{DRAIN, HyperConnections};
use hyper_util::service::TowerToHyperService;
use tokio::net::TcpListener;

type AxumServerError = Box<dyn std::error::Error + Send + Sync>;

/// Serves an axum route at the address its setup names.
#[derive(Debug, Default)]
pub struct AxumServer;

impl HttpServerAlg for AxumServer {
    type Program = AxumRoute;
    type Open = HyperConnections;
    type Error = AxumServerError;

    async fn open(&mut self, setup: HttpServerSetup<Self::Program>) -> Result<Self::Open, Self::Error> {
        let (bind, route) = setup.into_parts();
        let listener = TcpListener::bind(bind.address()).await?;
        // `Router` is a tower service, so this crate accepts and axum only answers requests.
        let service = TowerToHyperService::new(route.into_axum());

        Ok(HyperConnections::serve(listener, service, DRAIN))
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
