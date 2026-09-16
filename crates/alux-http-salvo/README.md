# alux-http-salvo

`alux-http-salvo` interprets an [`alux-http`](https://docs.rs/alux-http) program as executable
[Salvo](https://docs.rs/salvo) routes.

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_salvo::SalvoHandlerImpl;

let api = SalvoHandlerImpl::new(App::new());
let router = api.compile_http(api.status_api::<App>()).into_salvo();

salvo::Server::new(salvo::conn::TcpListener::new("0.0.0.0:3000").bind().await).serve(router).await;
```

A Salvo handler writes into the response it receives rather than returning one, so every output kind
here produces a `salvo::Response` for the endpoint to install. Salvo represents path captures as a
map, even when the path has only one parameter. This interpreter extracts that single value directly,
so it agrees with the path semantics of the other routers.
