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


## Downstream output kinds

Implement `OutputKindAlg<TextHandlerImpl, Result>` for a downstream kind and select an
`OutputAlg<Result>` converter. The text interpreter records the converter and output types without
executing the operation or conversion. An identity converter is sufficient when its type names the
output meaning. The same explicit `HttpOperationAlg` declaration is used for execution and metadata;
see [the shared custom-output scenario](../alux-http-poem/tests/custom_output.rs).
