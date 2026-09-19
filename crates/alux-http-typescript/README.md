# alux-http-typescript

`alux-http-typescript` interprets an [`alux-http`](https://docs.rs/alux-http) program as the
TypeScript client that calls it.

The same program an executing interpretation answers is read here from the other side: what a caller
states, where each argument goes, and what comes back. Nothing about the surface is written a second
time in another language, which is what makes a client and a service unable to disagree about it.

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_typescript::TsHttpClient;
use alux_shape::Spelling;

let client = TsHttpClient::new(Spelling::LowerCamel);
let module = client.compile_http(client.status_api::<App>());

println!("{}", module.render());
```

A generated module states each call as the program states it:

```ts
export const program = {
  readingAt: endpoint<[id: number], Reading>("GET", "/readings/{id}", ["path"]),
} as const
```

## One call per operation

Each call is named after the operation it calls. Declare the same operation on two endpoints and the
module has one call, the last one declared. Give each endpoint its own operation to get two calls.

A server still answers both paths, and [`alux-http-text`](https://docs.rs/alux-http-text) and
[`alux-http-openapi`](https://docs.rs/alux-http-openapi) still describe both.

The method, the path template, and where each argument goes are read from the declaration. A
described path is already a template, so nothing is spelled a second way for a caller. `endpoint`
comes from the runtime package, so a surface and what makes a surface's requests are upgraded
separately.

