# alux-http-text

`alux-http-text` interprets an [`alux-http`](https://docs.rs/alux-http) program as a readable
description of its routes and types. It executes no handler; it lists the methods, paths, input
roles, arguments, results, and output conversions declared by the program.

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_text::TextHandlerImpl;

let api = TextHandlerImpl;
let routes = api.compile_http(api.status_api::<App>());

assert_eq!(routes.labels(), ["GET /status"]);
println!("{}", routes.lines().join("\n"));
```

The same program value can be compiled by any other interpreter, such as
[`alux-http-poem`](https://docs.rs/alux-http-poem), without restating its routes.

