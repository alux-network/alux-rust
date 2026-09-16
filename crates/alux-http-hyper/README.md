# alux-http-hyper

`alux-http-hyper` serves an [`alux-http`](https://docs.rs/alux-http) surface over
[hyper](https://docs.rs/hyper), with no web framework.

[`alux-http-direct`](https://docs.rs/alux-http-direct) answers a request with values but carries no
transport. This crate connects it to hyper: it reads a hyper request into a `DirectRequest`, passes
it to the surface, and writes the answer back as a hyper response.

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_direct::DirectHandlerImpl;
use alux_http_hyper::HyperService;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;

let api = DirectHandlerImpl::new(App::new());
let served = HyperService::new(api.compile_http(api.status_api::<App>()));

let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
loop {
    let (stream, _) = listener.accept().await?;
    let served = served.clone();
    tokio::spawn(async move {
        let connection = Builder::new(TokioExecutor::new());
        let _ = connection.serve_connection(TokioIo::new(stream), served).await;
    });
}
```

The crate states a [`hyper::service::Service`] and stops there. Which runtime listens, and how
connections are driven, stays yours, so this names no runtime: `tokio` and `hyper-util` above are
the caller's choice, not this crate's dependencies.

A body the surface produces over time is served as one, so `.stream()` reaches a caller a chunk at a
time rather than being collected first.

Two kinds of request cannot reach a handler at all, and both get a response instead of killing the
connection. `HttpMethod` names nine methods, so a request using anything else, such as the
[`WebDAV`](https://www.rfc-editor.org/rfc/rfc4918) `PROPFIND`, is answered `405 Method Not Allowed`.
A request whose body cannot be read to the end is
answered `400 Bad Request`.

