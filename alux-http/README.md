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
    fn status_api<Alg>(&self)
    where
        Alg: StatusAlg,
    {
        self.routes()
            .get("/status", self.op(Alg::status_current).json())
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
    .get("/status", builder.op(StatusCurrentOperation::<App>::default()).json())
    .into_program();
let _nested = builder.routes().nest("/api", builder.program(program)).into_program();

// Argument names and order survive from the authored method into the program.
assert_eq!(<StatusForIdOperation<App> as OperationAlg>::ARG_NAMES, ["id"]);
```

A program declared this way compiles through any crate that witnesses its algebras:

- [`alux-http-text`](https://docs.rs/alux-http-text) describes the surface as documentation or metadata.
- [`alux-http-poem`](https://docs.rs/alux-http-poem) compiles the same program into executable Poem routes.
