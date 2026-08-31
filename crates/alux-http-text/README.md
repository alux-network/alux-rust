# alux-http-text

`alux-http-text` interprets an [`alux-http`](https://docs.rs/alux-http) program as a readable
description of its route surface. It executes no handler, so it witnesses that a typed HTTP program
denotes selectors, extractor roles, arguments, results, and output conversions rather than framework
callbacks.

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_text::TextHandlerImpl;

let api = TextHandlerImpl;
let routes = api.compile_http(api.status_api::<App>());

assert_eq!(routes.labels(), ["GET /status"]);
println!("{}", routes.lines().join("\n"));
```

The same program value compiles through any other interpreter, such as
[`alux-http-poem`](https://docs.rs/alux-http-poem), without restating its routes.
