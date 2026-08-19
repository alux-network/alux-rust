# alux-http

`alux-http` describes typed HTTP programs independently of a web framework.

The crate is a specification. It carries no interpreter and depends only on
[`alux-ext`](https://docs.rs/alux-ext): an HTTP surface is described here as first-order syntax plus
the capability algebras an interpreter must witness. Domain specifications provide their own algebras
and compose first-order operations into routes.

```rust
use alux_ext::{OperationAlg, ext};
use alux_http::{HttpApiAlg, HttpProgramBuilder, JsonOutAlg, http};
use core::future::Future;

/// A downstream specification owns its primitive domain meaning.
trait StatusAlg {
    type Status;

    fn status(&self) -> impl Future<Output = Self::Status> + Send;
    fn status_at(&self, id: u32) -> impl Future<Output = Self::Status> + Send;
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

    async fn status_for_id(&self, id: u32) -> This::Status {
        self.status_at(id).await
    }
}

/// The route program is declared before any framework is chosen.
#[ext(name = StatusApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + JsonOutAlg,
{
    /// Declares the status surface: the current reading and one identified reading.
    fn status_api<Alg>(&self)
    where
        Alg: StatusAlg,
    {
        self.routes()
            // The reading as it stands.
            .get("/status", self.op(Alg::status_current).json())
            // One identified reading, its id taken from the path.
            .get("/status/:id", self.op(Alg::status_for_id).path::<u32>().json())
    }
}

struct App;

impl StatusAlg for App {
    type Status = u32;

    async fn status(&self) -> u32 {
        1
    }

    async fn status_at(&self, id: u32) -> u32 {
        id
    }
}

// The same program is constructible directly, without the convenience macro and without an
// interpreter, because a route program is an ordinary value.
let builder = HttpProgramBuilder;
let program = builder
    .routes()
    // The same endpoint, declared without the convenience macro.
    .get("/status", builder.op(StatusCurrentOperation::<App>::default()).json())
    .into_program();
let _nested = builder.routes().nest("/api", builder.program(program)).into_program();

// Argument names and order survive from the authored method into the program.
assert_eq!(<StatusForIdOperation<App> as OperationAlg>::ARG_NAMES, ["id"]);
```

## Composing surfaces

The reason to keep a surface first-order is that programs compose before anything interprets them. Two
crates that know nothing about each other can each declare part of a service, and a third can state
the whole of it — no shared route table, no registry, no framework in the picture yet.

```rust
use alux_ext::ext;
use alux_http::{HttpApiAlg, JsonOutAlg, http};
use core::future::Future;

trait StatusAlg {
    type Status;

    fn status(&self) -> impl Future<Output = Self::Status> + Send;
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

/// One surface fragment. Its bounds name only what it uses: status, and JSON output.
#[ext(name = StatusApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + JsonOutAlg,
{
    /// Declares the status route.
    fn status_api<Alg>(&self)
    where
        Alg: StatusAlg,
    {
        // The reading as it stands.
        self.routes().get("/status", self.op(Alg::status_current).json())
    }
}

/// Another fragment, declared independently — plausibly in another crate.
#[ext(name = ItemsApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + JsonOutAlg,
{
    /// Declares the item route.
    fn items_api<Alg>(&self)
    where
        Alg: ItemsAlg,
    {
        // Every item the domain holds.
        self.routes().get("/items", self.op(Alg::items_current).json())
    }
}

/// The whole service: a coproduct of both fragments, with one of them under a path prefix.
#[ext(name = ServiceApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg,
{
    /// Declares `/status` beside `/v1/items`.
    fn service_api<Alg>(&self)
    where
        Alg: StatusAlg + ItemsAlg,
    {
        self.routes()
            // Merge forms the route coproduct.
            .merge(self.status_api::<Alg>())
            // Nesting precomposes a prefix over an entire subtree.
            .nest("/v1", self.items_api::<Alg>())
    }
}
```

Two things there are worth pausing on, because the authored text is not what runs:

- **The declarations look like they return nothing.** They do return something. The macro replaces the
  written signature with `-> ServiceApiProgram<Alg>` and the written body with
  `ServiceApiProgram::default()`, so calling `service_api` hands back a zero-sized program value.
  Writing no return type is the convention: the type is generated, and naming it by hand would only
  repeat the macro.
- **`merge` and `nest` are not receiving routes.** The authored body is read as a description rather
  than executed, and a call to a sibling declaration is lifted into a nested program — the emitted body
  reads `builder.merge(builder.program(builder.status_api::<Alg>()))`. That is what type-checks, and it
  is why one fragment composes with another without either knowing how the other was declared.

What that buys, and why it is not just tidiness:

- **Fragments state their own dependencies.** `status_api` requires `JsonOutAlg`; a fragment returning a
  file would require `FileOutAlg` instead. Neither imposes its needs on the other, and `service_api`
  inherits exactly the union.
- **Nesting is selector precomposition**, not a router feature — so `/v1/items` arises from composing
  `/v1` with a subtree that never mentions it.
- **Composition happens before interpretation.** `service_api` is a value; every interpreter sees the
  same merged surface, so documentation and execution cannot drift apart.
- **The surface is closed under composition.** A merged program is a program, so it can be merged or
  nested again without a special case.

A program declared this way compiles through any crate that witnesses its algebras:

- [`alux-http-text`](https://docs.rs/alux-http-text) describes the surface as documentation or metadata.
- [`alux-http-poem`](https://docs.rs/alux-http-poem) compiles the same program into executable Poem routes.
