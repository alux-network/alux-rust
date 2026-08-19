# alux-http-poem

`alux-http-poem` interprets an [`alux-http`](https://docs.rs/alux-http) program as executable
[Poem](https://docs.rs/poem) routes. The interpreter chooses Poem extractors for input roles, Poem
responses for output kinds, and `Arc` for the runtime handle of a semantic context.

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_poem::PoemHandlerImpl;

let api = PoemHandlerImpl::new(App::new());
let route = api.compile_http(api.status_api::<App>()).into_poem();

poem::Server::new(poem::listener::TcpListener::bind("0.0.0.0:3000")).run(route).await?;
```

Poem bodies, headers, errors, and endpoint erasure stay inside this crate. Compiling the same program
with [`alux-http-text`](https://docs.rs/alux-http-text) observes the identical ordered surface.
