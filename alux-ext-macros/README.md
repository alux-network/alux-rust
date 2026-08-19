# alux-ext-macros

Procedural-macro implementation for ALUX first-order extension, HTTP, and JSON-RPC programs.

Downstream users should import `alux_ext::ext`, `alux_http::http`, or `alux_jsonrpc::jsonrpc` rather than
depending on this implementation crate directly. Generated code targets the public product crates.
