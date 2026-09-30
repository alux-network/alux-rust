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


## Downstream output kinds

A downstream kind implements `OutputKindAlg<PoemHandlerImpl<Context>, Result>` and selects a converter
implementing `OutputAlg<Result>`. Its output must satisfy Poem's response requirements. The converter
may match a downstream enum and return different statuses, headers, and bodies; no endpoint bridge
or built-in output-family bound is needed. The API declares `.out::<Kind>()` and an explicit
`HttpOperationAlg` capability. See [the custom-output scenario](tests/custom_output.rs), which checks
redirect-with-cookie and HTML alternatives against text and `OpenAPI` interpretations.
