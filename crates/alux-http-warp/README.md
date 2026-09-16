# alux-http-warp

`alux-http-warp` interprets an [`alux-http`](https://docs.rs/alux-http) program as an executable
[warp](https://docs.rs/warp) filter.

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_warp::WarpHandlerImpl;

let api = WarpHandlerImpl::new(App::new());
let filter = api.compile_http(api.status_api::<App>()).into_warp();

warp::serve(filter).run(([0, 0, 0, 0], 3000)).await;
```

warp holds no route table: it composes filters, so a coproduct of routes is `or` and a selector is
the filters a request has to pass. A path is built from the segments the program states, with
`warp::path` for a literal, `warp::path::param` for a binding, and `warp::path::tail` for the rest,
so warp does the matching and the binding. What reaches an endpoint is what those filters gathered,
which is why the roles here read a gathered request rather than a framework extractor.

## A body produced over time

`.stream()` does not compile against this interpretation, and it compiles against every other one.
The reason is warp's, not the specification's, and there is no way around it from outside warp.

A warp reply is `warp::reply::Response`, an `http::Response` over a body type warp keeps in a
private module. Every public way to make one takes bytes that are already in hand: `From<Bytes>`,
`From<String>`, `From<Vec<u8>>`, `From<&'static str>`, `From<&'static [u8]>`, and
`From<Option<Bytes>>`. The two constructors that would take a stream, `wrap` and `wrap_stream`, are
crate-private. So a reply whose body arrives a piece at a time cannot be built by anyone but warp.

The HTTP program still declares streaming through `ChunksAlg`. This interpreter does not implement
the corresponding `StreamOutAlg`, so a declaration using `.stream()` fails to compile for warp while
it compiles for the other interpreters. This makes the limitation visible at compile time instead of
silently changing the response body.

Two things look like a way around it and are not:

- `warp::sse::reply` really does build a streaming body, but frames it as server-sent events. That
  is a different thing on the wire from the bytes `.stream()` states, so answering with it would make
  this interpretation disagree with the others about what the same program means.
- Implementing `warp::Reply` for a type of one's own does not help either. `Reply::into_response`
  answers with `warp::reply::Response`, so the same unconstructible body is still what has to be
  made.

This is a consequence of warp 0.4 moving to `hyper` 1 and `http-body` 1. Where warp 0.3 re-exported
hyper's own body type, which could be built from a stream, warp 0.4 defines its own over
`http_body_util::combinators::BoxBody` and keeps it private.

- [`warp::reply::Response`](https://docs.rs/warp/0.4.3/warp/reply/type.Response.html)
- [`warp::Reply`](https://docs.rs/warp/0.4.3/warp/reply/trait.Reply.html)
- [warp source](https://github.com/seanmonstar/warp), where `mod bodyt` is private
- [`BoxBody`](https://docs.rs/http-body-util/0.1/http_body_util/combinators/struct.BoxBody.html)

