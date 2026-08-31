# alux-jsonrpc-direct

`alux-jsonrpc-direct` interprets an [`alux-jsonrpc`](https://docs.rs/alux-jsonrpc) program as a
JSON-RPC 2.0 message handler. It implements the specification rather than delegating to a framework:
name dispatch, parameter decoding, response rendering, batches, notifications, and the protocol's own
error codes. Its only dependencies are `serde` and `serde_json`.

```rust ignore
use alux_jsonrpc::JsonRpcProgramExt;
use alux_jsonrpc_direct::DirectImpl;

let rpc = DirectImpl::new(App(40));
let methods = rpc.compile_jsonrpc(rpc.example_rpc::<App>())?;

let answer = methods.dispatch(r#"{"jsonrpc":"2.0","method":"status_current","id":1}"#).await;

assert_eq!(answer.unwrap(), r#"{"jsonrpc":"2.0","result":40,"id":1}"#);
```

`dispatch` answers with nothing when the document asks for nothing — a notification, or a batch of
them. Everything else answers with one response document.

A method marked fallible converts its error into a protocol error through `RpcErrorAlg`, exactly as
[`alux-jsonrpc-jsonrpsee`](https://docs.rs/alux-jsonrpc-jsonrpsee) does, so the same program and the
same domain compile through either interpretation unchanged.

## What it owns, and what it does not

It owns the message layer: a `MethodTable` maps a name to a decoded, applied, rendered answer, and
composes with another table by `merge`, which is defined exactly when the two name different methods.

It owns no transport and no runtime. A surface answers one request document and says nothing about how
that document arrived, so serving it over HTTP, a WebSocket, a pipe, or a test harness is a separate
decision — and an HTTP surface can be declared with [`alux-http`](https://docs.rs/alux-http) rather
than bundled in here.

## Differences from the jsonrpsee interpretation

- A method's output needs `Serialize`, not `Serialize + Clone`, because no answer is ever duplicated.
- Protocol errors are stated by this crate rather than by a framework, so their codes and messages are
  part of its observable behavior and are covered by its tests.
- `merge` reports a duplicated method name as `DuplicateMethod` rather than a framework error.
