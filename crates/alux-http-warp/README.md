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

warp streams response bodies. It does not expose a way to build one.

`warp::reply::Response` is `http::Response<BodyT>`, and `BodyT` lives in warp's private `mod bodyt`.
Its two constructors that take a stream are `pub(crate)`:

```rust ignore
// warp 0.4.3, src/bodyt.rs
pub(crate) fn wrap<B>(body: B) -> Self
where B: http_body::Body + Send + Sync + 'static;

pub(crate) fn wrap_stream<S, B, E>(stream: S) -> Self
where S: Stream<Item = Result<B, E>> + Send + Sync + 'static, B: Into<Bytes>;
```

The public routes to a `Response` all take bytes already in hand: `From<Bytes>`, `From<String>`,
`From<Vec<u8>>`, `From<&'static str>`, `From<&'static [u8]>`, `From<Option<Bytes>>`. Implementing
`warp::Reply` does not help, because `into_response` must answer with the same `Response`.

So this crate does not implement `StreamOutAlg`, and `.stream()` fails to compile here while it
compiles for every other interpreter. The alternative is to collect the body first, which would make
this interpretation answer differently from the others for the same program.

`warp::sse::reply` does build a streaming body, but frames it as server-sent events: `data:` lines
and blank-line terminators. That is a different response body from the bytes `.stream()` states.

warp 0.3 re-exported hyper 0.14's `Body`, which was publicly constructible from a stream. warp 0.4
moved to hyper 1 and `http-body` 1, defined `BodyT` over
[`BoxBody`](https://docs.rs/http-body-util/0.1/http_body_util/combinators/struct.BoxBody.html), and
kept it private.

- [`wrap_stream`](https://github.com/seanmonstar/warp/blob/v0.4.3/src/bodyt.rs#L59), the constructor
- [`warp::reply::Response`](https://docs.rs/warp/0.4.3/warp/reply/type.Response.html)
- [`warp::Reply`](https://docs.rs/warp/0.4.3/warp/reply/trait.Reply.html)
