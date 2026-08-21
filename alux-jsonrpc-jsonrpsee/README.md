# alux-jsonrpc-jsonrpsee

`alux-jsonrpc-jsonrpsee` interprets an [`alux-jsonrpc`](https://docs.rs/alux-jsonrpc) program as a
[jsonrpsee](https://docs.rs/jsonrpsee) method collection. The interpreter owns parameter decoding,
method registration, context sharing, serialization, and boundary error conversion.

```rust ignore
use alux_jsonrpc::JsonRpcProgramExt;
use alux_jsonrpc_jsonrpsee::JsonrpseeImpl;

let rpc = JsonrpseeImpl::new(App::new());
let methods = rpc.compile_jsonrpc(rpc.status_rpc::<App>())?;
```

A method declared `.fallible()` answers with its value or with the error its `Result` carries, so the
interpreter needs `Into<ErrorObjectOwned>` for that error. Every other method answers with any value
`serde` can serialize.

`RpcCtx` carries a semantic context when a native `#[rpc(server)]` trait is implemented instead, and
`ResultToRpcExt` converts semantic errors at the boundary. A shared scenario in `tests/` runs one
expectation against both the specification-first program and a native jsonrpsee service.
