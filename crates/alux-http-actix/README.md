# alux-http-actix

`alux-http-actix` interprets an [`alux-http`](https://docs.rs/alux-http) program as executable
[actix-web](https://docs.rs/actix-web) routes.

```rust ignore
use actix_web::{App, HttpServer};
use alux_http::HttpProgramExt;
use alux_http_actix::ActixHandlerImpl;

let api = ActixHandlerImpl::new(App::new());
let configure = api.compile_http(api.status_api::<App>()).into_actix();

HttpServer::new(move || App::new().configure(configure.clone())).bind(("0.0.0.0", 3000))?.run().await?;
```

actix-web creates its services again for every worker, and an `actix_web::Route` cannot be cloned or
reused. The interpreter therefore keeps the logic for constructing each endpoint and rebuilds the
routes for every worker. Shared state reaches a handler as the semantic context from the program,
not as `actix_web::Data`, so framework types stay out of the specification.

## Accepting

actix-web accepts on a socket this interpreter binds and hands over through `HttpServer::listen`, but it serves that socket from runtimes of its own: an `actix-rt` System with one current-thread Tokio runtime per worker thread. It exposes no entry point that takes a single connection, so the accept loop belongs to actix and cannot be driven from outside.

Stopping therefore goes through `ServerHandle::stop`. Dropping the task that awaits the server reaches neither the workers nor the socket they accept on, so the address would stay bound.

## Signals

This interpreter calls `HttpServer::disable_signals`, because whoever opened the server is the one who decides when it closes.

Left on, actix-web installs process-wide handlers and answers Ctrl-C and SIGTERM itself: its accept loop turns `SIGTERM` into a graceful stop and `SIGINT` into a forced one. An application built on `alux-http` never sees either, so its own shutdown never runs and `kill` or `pkill` appears to do nothing at all. Ending a server is what `HttpServerAlg::close` and `HttpServerAlg::end` are for, and a signal is the application's to interpret.
