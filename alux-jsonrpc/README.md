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
    /// Declares the status surface: a reading and an adjustment.
    fn status_rpc<Alg>(&self)
    where
        Alg: StatusAlg,
    {
        self.methods()
            // The reading as it stands, taking no parameters.
            .method("status_current", self.op(Alg::status_current))
            // An adjustment, decoded from a JSON object using the authored argument names.
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
    // The same method, declared without the convenience macro.
    .method("status_current", builder.op(StatusCurrentOperation::<App>::default()))
    .into_program();
let _merged = builder.methods().merge(builder.program(program)).into_program();

// Named decoding means these argument names, carried from the authored method.
assert_eq!(<StatusAdjustedOperation<App> as OperationAlg>::ARG_NAMES, ["temp"]);
```

## Composing surfaces

The reason to keep a surface first-order is that programs compose before anything interprets them. Two
crates that know nothing about each other can each declare part of a service, and a third can state the
whole of it — no shared method table, no registry, no framework in the picture yet.

```rust
use alux_ext::ext;
use alux_jsonrpc::{JsonRpcApiAlg, jsonrpc};
use core::future::Future;

trait StatusAlg {
    type Status;

    fn status(&self) -> impl Future<Output = Self::Status> + Send;
    fn status_set_temp(&self, temp: f32) -> impl Future<Output = Self::Status> + Send;
}

trait ItemsAlg {
    type Items;

    fn items(&self) -> impl Future<Output = Self::Items> + Send;
}

#[ext(name = StatusOperationExt, defunc)]
impl<This> This
where
    This: StatusAlg,
{
    /// Returns the status as it stands.
    async fn status_current(&self) -> This::Status {
        self.status().await
    }

    /// Returns the status after adjusting the temperature.
    async fn status_adjusted(&self, temp: f32) -> This::Status {
        self.status_set_temp(temp).await
    }
}

#[ext(name = ItemsOperationExt, defunc)]
impl<This> This
where
    This: ItemsAlg,
{
    /// Returns every item the domain holds.
    async fn items_current(&self) -> This::Items {
        self.items().await
    }
}

/// One surface fragment, offering one operation under both parameter modes.
#[ext(name = StatusRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares the status methods.
    fn status_rpc<Alg>(&self)
    where
        Alg: StatusAlg,
    {
        self.methods()
            // The reading as it stands, taking no parameters.
            .method("status_current", self.op(Alg::status_current))
            // An adjustment, decoded from a JSON array.
            .method("status_set_temp", self.op(Alg::status_adjusted).positional())
            // The same operation, decoded from a JSON object using the authored argument names.
            .method("status_set_temp_named", self.op(Alg::status_adjusted).named())
    }
}

/// Another fragment, declared independently — plausibly in another crate.
#[ext(name = ItemsRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares the item method.
    fn items_rpc<Alg>(&self)
    where
        Alg: ItemsAlg,
    {
        // Every item the domain holds.
        self.methods().method("items_current", self.op(Alg::items_current))
    }
}

/// The whole service, as the union of both fragments.
#[ext(name = ServiceRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares every method both fragments contribute.
    fn service_rpc<Alg>(&self)
    where
        Alg: StatusAlg + ItemsAlg,
    {
        self.methods().merge(self.status_rpc::<Alg>()).merge(self.items_rpc::<Alg>())
    }
}
```

Two things there are worth pausing on, because the authored text is not what runs:

- **The declarations look like they return nothing.** They do return something. The macro replaces the
  written signature with `-> ServiceRpcProgram<Alg>` and the written body with
  `ServiceRpcProgram::default()`, so calling `service_rpc` hands back a zero-sized program value.
  Writing no return type is the convention: the type is generated, and naming it by hand would only
  repeat the macro.
- **`merge` is not receiving a method collection.** The authored body is read as a description rather
  than executed, and a call to a sibling declaration is lifted into a nested program — the emitted body
  reads `builder.merge(builder.program(builder.status_rpc::<Alg>()))`. That is what type-checks, and it
  is why one fragment composes with another without either knowing how the other was declared.

What that buys, and why it is not just tidiness:

- **Fragments state their own dependencies.** Each names the capability it uses and nothing more, and
  `service_rpc` inherits exactly the union.
- **Merge is the whole composition.** A JSON-RPC surface is a set of named methods, so it composes as a
  monoid — there is no prefix to nest under, because a method name has no parts. HTTP carries selectors
  and therefore nesting; the two program algebras differ because their surfaces genuinely differ.
- **One operation can appear more than once.** `status_adjusted` is registered twice above, positionally
  and by name, which is a decision about the wire rather than about the domain.
- **Composition happens before interpretation.** `service_rpc` is a value; the interpreter that
  registers it never chooses a method name.

Positional parameters are the default. `.named()` decodes a JSON object using the argument names
retained by `alux-ext`. An argument the request leaves out reads as absent, which only an optional
argument accepts, so a positional array may stop short of the product and a parameter object may omit
a name.

`.fallible()` states that an operation can fail, so its `Result` answers with a value or with a
JSON-RPC error rather than with a success carrying an error-shaped value. The domain keeps stating
failure in its own vocabulary; only its statement of what that failure denotes at the boundary is
transport-facing.

[`alux-jsonrpc-jsonrpsee`](https://docs.rs/alux-jsonrpc-jsonrpsee) compiles the same program into
jsonrpsee `Methods`.
