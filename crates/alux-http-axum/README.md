# alux-http-axum

`alux-http-axum` interprets an [`alux-http`](https://docs.rs/alux-http) program as executable
[axum](https://docs.rs/axum) routes. The interpreter chooses axum extractors for input roles, axum
responses for output kinds, and `Arc` for the runtime handle of a semantic context.

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_axum::AxumHandlerImpl;

let api = AxumHandlerImpl::new(App::new());
let router = api.compile_http(api.status_api::<App>()).into_axum();

axum::serve(tokio::net::TcpListener::bind("0.0.0.0:3000").await?, router).await?;
```

Shared state reaches a handler as the semantic context from the program, not as axum state. This
keeps framework types out of the specification and its domain operations.

axum uses `/status/{id}` for a parameter path. The same program compiled with
[`alux-http-poem`](https://docs.rs/alux-http-poem) routes `/status/:id` and describes it identically.

