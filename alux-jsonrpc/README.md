# alux-jsonrpc

`alux-jsonrpc` describes typed JSON-RPC method programs independently of an RPC framework.

The crate is a specification. It carries no interpreter and depends only on
[`alux-ext`](https://docs.rs/alux-ext): a JSON-RPC surface is described here as first-order syntax
plus the capability algebras an interpreter must witness. Domain specifications provide their own
capability traits and compose defunctionalized operations.

```rust
use alux_ext::{OperationAlg, ext};
use alux_jsonrpc::{JsonRpcApiAlg, JsonRpcProgramBuilder, jsonrpc};
use core::future::Future;

/// A downstream specification owns its primitive domain meaning.
trait StatusAlg {
    type Status;

    fn status(&self) -> impl Future<Output = Self::Status> + Send;
    fn status_set_temp(&self, temp: f32) -> impl Future<Output = Self::Status> + Send;
}

/// Derived operations become first-order values that preserve their argument names.
#[ext(name = StatusOperationExt, defunc)]
impl<This> This
where
    This: StatusAlg,
{
    async fn status_current(&self) -> This::Status {
        self.status().await
    }

    async fn status_adjusted(&self, temp: f32) -> This::Status {
        self.status_set_temp(temp).await
    }
}

/// The method program is declared before any framework is chosen.
#[ext(name = StatusRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    fn status_rpc<Alg>(&self)
    where
        Alg: StatusAlg,
    {
        self.methods()
            .method("status_current", self.op(Alg::status_current))
            .method("status_set_temp", self.op(Alg::status_adjusted).named())
    }
}

struct App;

impl StatusAlg for App {
    type Status = u32;

    async fn status(&self) -> u32 {
        1
    }

    async fn status_set_temp(&self, temp: f32) -> u32 {
        temp as u32
    }
}

// The same program is constructible directly, without the convenience macro and without an
// interpreter, because a method program is an ordinary value.
let builder = JsonRpcProgramBuilder;
let program = builder
    .methods()
    .method("status_current", builder.op(StatusCurrentOperation::<App>::default()))
    .into_program();
let _merged = builder.methods().merge(builder.program(program)).into_program();

// Named decoding means these argument names, carried from the authored method.
assert_eq!(<StatusAdjustedOperation<App> as OperationAlg>::ARG_NAMES, ["temp"]);
```

Positional parameters are the default. `.named()` decodes a JSON object using the argument names
retained by `alux-ext`.

[`alux-jsonrpc-jsonrpsee`](https://docs.rs/alux-jsonrpc-jsonrpsee) compiles the same program into
jsonrpsee `Methods`.
