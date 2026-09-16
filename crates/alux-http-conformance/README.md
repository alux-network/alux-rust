# alux-http-conformance

`alux-http-conformance` states one [`alux-http`](https://docs.rs/alux-http) surface and the scenario
every interpretation of it must satisfy.

Two interpretations can be compared only when they compile the same declaration and answer the same
requests. This crate provides that shared evidence: one declaration, compiled by the interpretation
under test, and one set of exchanges describing the required methods, paths, media types, and
statuses. The exchanges know nothing about the domain or any framework.

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_conformance::{Shop, ShopApiExt, expect};
use alux_http_direct::DirectHandlerImpl;

let api = DirectHandlerImpl::new(Shop);
let surface = api.compile_http(api.shop_api::<Shop>());

expect(&surface).await.unwrap();
```

An interpretation that answers with a framework's own response states an adapter for
[`AnswerAlg`](AnswerAlg), which is the only place a framework is named.

