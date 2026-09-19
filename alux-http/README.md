# alux-http

`alux-http` lets you declare an HTTP API once, as an ordinary value, and then run that same
declaration on any web framework. The same declaration also reads as an [`OpenAPI 3.1`](https://spec.openapis.org/oas/v3.1.0)
document or a typed TypeScript client, so the API is never written a second time.

This crate holds only the declaration: which routes exist, where each handler argument comes from,
and what each endpoint answers with. It contains no web framework and depends only on
[`alux-ext`](https://docs.rs/alux-ext). A separate crate then *interprets* the declaration: one turns
it into Poem routes, another into an axum router, another into an `OpenAPI` document. Adding one
never changes the declaration.

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

/// A derived method becomes a value an endpoint can be declared with, argument names included.
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

## Methods

Declare the method an endpoint answers on with `.get`, `.post`, `.put`, `.patch`, `.delete`,
`.head`, `.options`, `.trace`, or `.connect`. Use `.method` to take the method as a type parameter
instead.

## Inputs

Every handler argument says where it comes from: `.path()`, `.query()`, `.body()` for JSON,
`.form()`, `.raw_body()`, `.multipart()`, `.in_header()`, `.cookie()`, `.auth()`, and `.context()` when
you want the framework's own extractor. `.with()` takes a value the interpreter supplies directly.

Arguments are filled in the order you declare them. The declaration parses nothing itself: each
interpreter does that with its framework's extractors.

**Query strings, headers and cookies hold names and values**, so the type you read one into has to be
a struct rather than a single value. Say so by implementing [`NamedValuesAlg`](NamedValuesAlg):

```rust ignore
#[derive(Deserialize)]
struct Filters {
    since: u64,
    limit: Option<u32>,
}

impl NamedValuesAlg for Filters {}

// `?since=…&limit=…`, and `limit` is optional because the field is.
self.routes().get("/readings", self.op(Alg::search).query::<Filters>().json())
```

`.query::<String>()` does not compile, because a lone `String` has no name for a caller to send it
under. Header names are converted for you, so a `User-Agent` header arrives in a `user_agent` field.

**`.multipart::<T>()`** reads a body that arrives as parts. Your type implements
[`FromPartsAlg`](FromPartsAlg), which says how to build it from a reader of parts, and each
interpreter supplies whichever reader its framework has. A reader of parts is a
[`ChunksAlg`](ChunksAlg), which is a sequence you take one item at a time. Its items are
[`PartAlg`](PartAlg), and a part's own content is another such sequence.

## Outputs

Declare what an endpoint answers with: `.json()`, `.text()`, `.html()`, `.bytes()`, `.file()`,
`.empty()`, `.redirect()`, or `.stream()`. Your handler's return type is inferred, so you never
repeat it just to pick a format.

Not every kind accepts every result, and that is checked when you compile. `.empty()` takes a handler
returning `()` and rejects one returning data, so you cannot quietly throw a value away.

`.stream()` answers with a body produced over time. Your handler returns a type implementing
[`ChunksAlg`](ChunksAlg), which says what a chunk is and how to take the next one, so you are not
committed to any particular stream type.

Three kinds wrap the one before them:

- **`.status::<201>()`** sets the status code. Which code a created resource answers with belongs to
  the endpoint, not the handler.
- **`.result()`** handles a handler returning `Result`. Success answers with the kind you already
  chose; a failure answers with the status and message its `HttpErrorAlg` impl gives.
- **`.out_header::<CacheControl>()`** adds a response header. The handler returns `(value, body)`,
  because only the handler knows an `ETag` or a cache lifetime.

A header is just a name, so one this crate does not already ship is three lines of your own and no
interpreter changes:

```rust ignore
use alux_http::HeaderNameAlg;

/// States the `x-request-id` header an answer carries.
struct RequestId;

impl HeaderNameAlg for RequestId {
    const HEADER_NAME: &'static str = "x-request-id";
}
```

```rust ignore
self.routes()
    // A recording, which creates something and says so.
    .post("/record", self.op(Alg::record).body::<u32>().json().status::<201>())
    // One identified reading, or what its failure means.
    .get("/find/{id}", self.op(Alg::find).path::<u32>().json().result())
```

## Paths

Paths are parsed into segments when you declare them, so you do not write them for one particular
router. `:id` and `{id}` both mean one segment bound as `id`; `*rest` and `{*rest}` both mean
everything left over. Anything else matches literally.

Each interpreter then renders those segments the way its own router wants: Poem gets `:id`, axum and
actix-web get `{id}`, Rocket gets `<id>`. Every interpreter *describes* the path the same way though,
so two interpretations of one declaration can be compared.

## Composing surfaces

A declaration is a value, so it composes before anything runs it. Two crates that know nothing about
each other can each declare part of a service, and a third can declare the whole of it, with no
shared route table, no registry, and no framework in the picture yet.

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

/// Another fragment, declared independently, plausibly in another crate.
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

/// The whole service: both fragments, with one of them under a path prefix.
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
            // Both fragments, side by side.
            .merge(self.status_api::<Alg>())
            // The whole items fragment, under one prefix.
            .nest("/v1", self.items_api::<Alg>())
    }
}
```

Write no return type on a declaration: calling it hands back the declared API, and the type is
generated for you. Inside the body, `merge` puts two declarations beside each other and `nest` puts
one under a prefix, including a declaration from another crate.

That gives you:

- **Fragments that state their own needs.** `status_api` requires `JsonOutAlg`; a fragment answering
  with a file requires `FileOutAlg` instead. Neither imposes its needs on the other, and
  `service_api` requires exactly the union.
- **One surface everywhere.** `service_api` is a value, so the served API, the `OpenAPI` document and
  the generated client are the same merged surface and cannot drift apart.
- **No special cases.** A merged declaration is a declaration, so it can be merged or nested again.

## Servers and lifecycle

`HttpServerAlg` specifies what it means to open, close and end one framework's executable route
surface. Closing releases the address; ending waits for what was already being served as well, up to
the interpreter's own drain. Each crate that serves chooses the surface, open handle and error types it needs. `.lifecycle(commands)`
turns a sequence of open and close commands into the events they produce, so an application can run one
server or switch between several without the declaration knowing anything about a runtime.

These crates serve the same declaration, without changing it:

- [`alux-http-poem`](https://docs.rs/alux-http-poem) serves it with Poem.
- [`alux-http-axum`](https://docs.rs/alux-http-axum) serves it with axum.
- [`alux-http-actix`](https://docs.rs/alux-http-actix) serves it with actix-web.
- [`alux-http-salvo`](https://docs.rs/alux-http-salvo) serves it with Salvo.
- [`alux-http-warp`](https://docs.rs/alux-http-warp) serves it with warp.
- [`alux-http-rocket`](https://docs.rs/alux-http-rocket) serves it with Rocket.
- [`alux-http-direct`](https://docs.rs/alux-http-direct) serves it with no web framework at all: it
  matches the request against the routes and calls the handler itself.
  [`alux-http-hyper`](https://docs.rs/alux-http-hyper) carries the bytes for it over hyper.

These read the same declaration instead of serving it:

- [`alux-http-openapi`](https://docs.rs/alux-http-openapi) generates an `OpenAPI` document for it.
- [`alux-http-typescript`](https://docs.rs/alux-http-typescript) generates a typed TypeScript client
  for it.
- [`alux-http-text`](https://docs.rs/alux-http-text) prints its routes and types, which is useful in
  tests and documentation.

[`alux-http-conformance`](https://docs.rs/alux-http-conformance) is a shared test suite: one declared
API, plus the requests and expected responses every crate above is checked against. It is how the
project knows they all behave the same.
