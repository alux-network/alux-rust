# alux-ext-macros

Procedural-macro implementation for ALUX first-order extension, HTTP, and JSON-RPC programs.

Downstream users should import `alux_ext::ext`, `alux_http::http`, or `alux_jsonrpc::jsonrpc` rather than
depending on this implementation crate directly. Generated code targets the public product crates.

HTTP lowering reifies operation references and states one `HttpOperationAlg` bound per endpoint from
its input roles and output kind, plus an `HttpProgramAlg` bound per nested program. Built-in
`.json()` and downstream `.out::<Kind>()` lower to the same public capability contract. See the workspace [migration notes](../../MIGRATION.md).
