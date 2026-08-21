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
use alux_jsonrpc::{JsonRpcApiAlg, RpcErrorAlg, jsonrpc};
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

/// The one reason this domain fails: it keeps no history.
struct NoHistory;

/// What that failure denotes on an RPC surface. The domain says it once, naming no interpreter.
impl RpcErrorAlg for NoHistory {
    fn rpc_code(&self) -> i32 {
        -32000
    }

    fn rpc_message(&self) -> String {
        "the domain keeps no history".to_owned()
    }
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

    /// Returns the reading this domain does not keep, stating its own failure.
    async fn status_history(&self) -> Result<This::Status, NoHistory> {
        Err(NoHistory)
    }

    /// Returns the reading from a moment the domain no longer holds.
    async fn status_at(&self, moment: u32) -> Result<This::Status, NoHistory> {
        let _ = moment;

        Err(NoHistory)
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

/// One surface fragment, mixing methods that answer with a value and one that answers with a
/// protocol error.
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
            // `.fallible()` converts the operation's error into a JSON-RPC protocol error. This is
            // the method-level marker; `history_rpc` below states the same once on the ext.
            .method("status_at", self.op(Alg::status_at).fallible())
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

/// A fragment whose errors all answer as protocol errors, marked once on the ext instead of per
/// method.
#[ext(name = HistoryRpcExt, defunc(via = jsonrpc), fallible)]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares the history method, whose readings answer as JSON-RPC errors.
    fn history_rpc<Alg>(&self)
    where
        Alg: StatusAlg,
    {
        // The ext-level `fallible` marker states it for every method here, so this one stays silent.
        self.methods().method("status_history", self.op(Alg::status_history))
    }
}

/// The whole service, as the union of every fragment.
#[ext(name = ServiceRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares every method the fragments contribute.
    fn service_rpc<Alg>(&self)
    where
        Alg: StatusAlg + ItemsAlg,
    {
        self.methods()
            .merge(self.status_rpc::<Alg>())
            .merge(self.history_rpc::<Alg>())
            .merge(self.items_rpc::<Alg>())
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

`fallible` is the other distinction a method carries: it converts the operation's error into a JSON-RPC
protocol error, so the failure answers in the response's `error` member instead of inside a successful
result. It can be marked in either place. A program that mixes both kinds marks the declarations that
convert, as `status_rpc` marks `status_at` above; a program whose errors all convert says so once as
`fallible` on the ext, as `history_rpc` does, and each of its declarations stays silent. Marking it both
ways means what marking it once means, and the two distinctions are independent — `.named().fallible()`
states both, in either order.

What a failure denotes is `RpcErrorAlg`, implemented above for `NoHistory`: the code the JSON-RPC
specification carries and the message the failure states. A domain says that once for its own error
type, and nothing about it names an interpreter — which is what lets a specification state the meaning
of its failures without depending on whichever library answers the call.

Marking it is also the migration path away from a failure that never reaches the protocol. An
operation returning `Result` on the value path is serialized whole, so its error travels inside a
successful answer — the shape JSON-RPC reserves a member for:

```text
{"jsonrpc":"2.0","id":1,"result":{"Err":{"code":-32000,"message":"..."}}}   a failure as a success
{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"..."}}           a failure as a failure
```

Moving from the first line to the second is two edits: mark `fallible` on the method or its ext, and
implement `RpcErrorAlg` for the error type. Neither is guesswork — a fallible declaration does not
compile until the error says what it denotes, and an error that is not serializable does not compile on
the value path, which is what surfaces the mistake in the first place. An error type that happens to be
serializable compiles either way, so those are the declarations worth reading twice.

[`alux-jsonrpc-jsonrpsee`](https://docs.rs/alux-jsonrpc-jsonrpsee) compiles the same program into
jsonrpsee `Methods`.
