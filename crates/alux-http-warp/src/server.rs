use crate::WarpRoute;
use alux_http::{HttpServerAlg, HttpServerSetup};
use alux_http_hyper::{DRAIN, HyperConnections};
use hyper_util::service::TowerToHyperService;
use tokio::net::TcpListener;

type WarpServerError = Box<dyn std::error::Error + Send + Sync>;

/// Serves a warp route at the address its setup names.
#[derive(Debug, Default)]
pub struct WarpServer;

impl HttpServerAlg for WarpServer {
    type Program = WarpRoute;
    type Open = HyperConnections;
    type Error = WarpServerError;

    async fn open(&mut self, setup: HttpServerSetup<Self::Program>) -> Result<Self::Open, Self::Error> {
        let (bind, route) = setup.into_parts();
        let listener = TcpListener::bind(bind.address()).await?;
        // `warp::service` turns a filter into a tower service, so this crate accepts.
        let service = TowerToHyperService::new(warp::service(route.into_warp()));

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
