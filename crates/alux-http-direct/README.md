# alux-http-direct

`alux-http-direct` interprets an [`alux-http`](https://docs.rs/alux-http) program as a surface that
answers requests itself, with no framework.

This interpreter performs the specification directly: it matches a request against the paths in the
program, binds captured values, reads each argument from its declared role, applies the operation,
and renders the answer, including routing failures. It carries no transport or runtime, so the
caller decides how requests arrive and responses leave.

```rust ignore
use alux_http::{HttpMethod, HttpProgramExt};
use alux_http_direct::{DirectHandlerImpl, DirectRequest};

let api = DirectHandlerImpl::new(App::new());
let surface = api.compile_http(api.status_api::<App>());

let answer = surface.answer(DirectRequest::new(HttpMethod::Get, "/status")).await;
assert_eq!(answer.text(), "1");
```

Because it answers with values rather than with a framework's types, it is the reference another
interpretation can be held to: [`alux-http-poem`](https://docs.rs/alux-http-poem) and
[`alux-http-axum`](https://docs.rs/alux-http-axum) fold the same program and must reach the same
endpoint for the same request.

Routing here needs no path spelling at all. The program already states a path as segments, so a
literal matches itself, a parameter binds one segment, and a tail binds the rest, and nothing is ever
rendered for a router to parse back.

