# alux-http-openapi

`alux-http-openapi` interprets an [`alux-http`](https://docs.rs/alux-http) program as an
[`OpenAPI 3.1`](https://spec.openapis.org/oas/v3.1.0) document that describes it.

This interpreter does not execute handlers. It reads each endpoint to determine what a caller must
send and what the endpoint returns, so it needs [`alux-shape`](https://docs.rs/alux-shape) shapes
where an executing interpreter needs request extractors.

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_openapi::OpenApiHandlerImpl;

let api = OpenApiHandlerImpl::<App>::new();
let route = api.compile_http(api.status_api::<App>());
let document = api.document("status", "1.0", &route);

println!("{}", serde_json::to_string_pretty(&document)?);
```

An `OpenAPI` document can be generated only when the program states an operation's name, each
argument's name and source, the shape of every value that crosses the wire, and every status an
endpoint can return. All of this information comes from the same declaration that
[`alux-http-poem`](https://docs.rs/alux-http-poem) and
[`alux-http-axum`](https://docs.rs/alux-http-axum) compile, so a document and a running service
cannot describe different surfaces.

The source declaration may write `/readings/:id`. The `OpenAPI` interpreter converts that portable
path segment to the `/readings/{id}` spelling used in the generated document.


## Downstream output kinds and response alternatives

Implement `OutputKindAlg<OpenApiHandlerImpl<Context>, Result>` for a downstream kind. Its selected
transform implements `OpenApiOutputAlg<Result>` and returns every supported `OpenApiAnswer`, with
status, header names, and optional body media type and schema. No runtime result or execution
converter is needed to describe those alternatives. A redirect can be described by its status and
`location` header, alongside `set-cookie` when the handler supplies it; another alternative may
carry an HTML body. The [custom-output scenario](../alux-http-poem/tests/custom_output.rs) verifies
that an unchanged declaration agrees with Poem execution and text metadata.
