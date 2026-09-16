# ALUX Rust architecture

## Semantic shape

ALUX Rust turns derived operations and transport surfaces into typed first-order programs that can
be interpreted without changing their meaning:

```text
downstream semantic capabilities
    -> derived extension operation
    -> first-order operation value
    -> neutral HTTP / JSON-RPC program
    -> generic fold
    -> concrete framework, text, metadata, or test interpretation
```

The central object is not a framework callback or generated token stream. It is the typed operation
or program preserved between authoring and interpretation. A framework witnesses that meaning by
extracting inputs, applying the operation, converting output, and registering the result.

## Layers

| Layer | Crate/module | Responsibility |
| --- | --- | --- |
| Operation meaning | `alux-ext` | Context, argument product, argument names, output, and application |
| Extension lowering | `alux-ext-macros::ext` | Preserves ordinary extension methods and optionally reifies their application |
| Authored syntax | `alux-ext-macros::syntax` | Recognizes the shapes an author writes: visibility, naming, `op(...)`, nested programs, `self` |
| Program lowering | `alux-ext-macros::lower` | Turns any declaration method into a program type, its body, and its obligations |
| HTTP lowering | `alux-ext-macros::http_program` | States what a route declaration means and compiles it through `HttpProgramAlg` |
| JSON-RPC lowering | `alux-ext-macros::jsonrpc_program` | States what a method declaration means and compiles it through `JsonRpcProgramAlg` |
| HTTP interpreter algebra | `alux-http::algebra` and `output` | Selectors, routes, input roles, context handles, endpoints, and output conversion |
| HTTP first-order syntax | `alux-http::program` | Empty, merge, nest, endpoint, input, and output nodes plus the generic fold |
| Neutral HTTP witness | `alux-http-text` | Interprets one HTTP program as readable type/route documentation |
| Poem HTTP witness | `alux-http-poem` | Interprets the same program as executable Poem endpoints |
| axum HTTP witness | `alux-http-axum` | Interprets the same program as an executable axum router |
| actix-web HTTP witness | `alux-http-actix` | Interprets the same program as executable actix-web routes |
| Salvo HTTP witness | `alux-http-salvo` | Interprets the same program as an executable Salvo router |
| warp HTTP witness | `alux-http-warp` | Interprets the same program as an executable warp filter |
| Rocket HTTP witness | `alux-http-rocket` | Interprets the same program as executable Rocket routes |
| Direct HTTP witness | `alux-http-direct` | Answers the same program itself, with no framework |
| OpenAPI HTTP witness | `alux-http-openapi` | Reads the same program as the document that describes it |
| Client HTTP witness | `alux-http-typescript` | Reads the same program as the TypeScript client that calls it |
| Transport | `alux-http-hyper` | Serves the direct interpretation over hyper, with no framework |
| Shared reading | `alux-http-parts` | Reads a body arriving as parts, where a framework does not |
| Shared obligation | `alux-http-conformance` | One declared surface and the scenario every interpretation satisfies |
| JSON-RPC interpreter algebra | `alux-jsonrpc::algebra` | Empty/merge semantics and positional/named method registration |
| JSON-RPC first-order syntax | `alux-jsonrpc::program` | Empty, merge, named program, method, and parameter-mode nodes plus the generic fold |
| jsonrpsee witness | `alux-jsonrpc-jsonrpsee` | Interprets one JSON-RPC program as a jsonrpsee method collection |

The dependency direction is:

```text
downstream specification
    |----------------> alux-ext
    |----------------> alux-http -----> alux-ext
    `----------------> alux-jsonrpc --> alux-ext

application
    |----------------> alux-http-text --------> alux-http
    |----------------> alux-http-poem --------> alux-http + poem
    |----------------> alux-http-axum --------> alux-http + axum
    |----------------> alux-http-actix -------> alux-http + actix-web
    |----------------> alux-http-salvo -------> alux-http + salvo
    |----------------> alux-http-warp --------> alux-http + warp
    |----------------> alux-http-rocket ------> alux-http + rocket
    |----------------> alux-http-direct ------> alux-http
    |----------------> alux-http-parts -------> alux-http
    |----------------> alux-http-hyper -------> alux-http + alux-http-direct + hyper
    |----------------> alux-http-openapi -----> alux-http + alux-shape
    |----------------> alux-http-typescript --> alux-http + alux-shape
    |----------------> alux-http-conformance -> alux-http + alux-http-direct
    `----------------> alux-jsonrpc-jsonrpsee -> alux-jsonrpc + jsonrpsee

alux-ext -----------> alux-ext-macros
```

A specification crate depends only on `alux-ext`. Interpreter crates depend on the specification
they witness plus their framework; nothing depends on an interpreter in order to declare a program.

The proc-macro crate does not depend on product crates. It emits references to their public
surfaces, and generated extension code names `alux_ext::extend`, so a crate using `#[ext]` needs no
separate `extend` dependency. Program crates re-export their applicable backend attributes from
`alux_ext::macros`, so authored code imports meaningful entry points such as `alux_http::http`
rather than the implementation crate.

## Boundary with downstream specifications

This workspace does not own `StatusAlg`, `ConsensusAlg`, `StorageAlg`, or any other application/domain
capability. A downstream specification defines its primitive meaning and derives operations:

```rust
use alux_ext::ext;
use core::future::Future;

trait StatusAlg {
    type Status;

    fn status(&self) -> impl Future<Output = Self::Status> + Send;
}

#[ext(name = StatusOperationExt, defunc)]
impl<This> This
where
    This: StatusAlg,
{
    async fn status_current(&self) -> This::Status {
        self.status().await
    }
}
```

The downstream capability is authoritative for status meaning. `StatusCurrentOperation` preserves
application of the derived operation. HTTP and JSON-RPC programs can refer to that value without
learning what a status contains or how it is computed.

Examples inside this workspace are test fixtures. They must not become public domain vocabulary.

## First-order operation meaning

`OperationAlg` describes a generated operation:

```text
Context    semantic interpreter expected by the operation
Args       ordered product of authored arguments
ARG_NAMES  authored argument names in declaration order
```

`ApplyAlg<Handle, Args>` supplies:

```text
Output     inferred operation result
apply      interpretation of the operation against an owned/borrowed handle
```

The operation type is zero-sized. It stores no closure environment because the authored method
depends only on its context and explicit arguments. If a behavior needs captured policy, that policy
must be part of the semantic context or an explicit argument rather than hidden in generated state.

`HandlerContextAlg<Context>` separates semantic context from runtime ownership. A concrete server can
choose `Arc<Context>` while a neutral interpreter can choose another handle. `Arc`, `Send`, framework
data extractors, and server lifetimes therefore remain interpreter constraints rather than domain
handler parameters.

## Extension defunctionalization

`#[ext(name = ..., defunc)]` has two simultaneous results:

1. The normal extension trait preserves ergonomic Rust method calls.
2. A generated `*Operation<Context>` makes each eligible borrowed method first-order.

The operation implementation invokes the same extension method. It is not a copied second
implementation. Context, argument types, argument names, and output are inferred from authored Rust.

Direct first-order program blocks may also use plain `defunc`. Backend forms
`defunc(via = http)` and `defunc(via = jsonrpc)` additionally translate convenient method references
into generated operation values and derive interpreter evidence.

The macro layer is a lowering pass:

```text
authored extension syntax
    -> validated syntax tree
    -> public first-order operation/program types
    -> ordinary generic trait implementations
```

All semantic machinery required after expansion must remain expressible and testable without a
procedural macro.

## HTTP program algebra

### Inputs and outputs

`HttpInputAlg` lets each interpreter select extractor representations for:

- path values
- query values
- request bodies read as a document
- request bodies read as a form
- request bodies taken as they arrived
- request bodies arriving as parts
- headers
- cookies
- authentication values
- endpoint context values

A query string, a header collection, and a cookie collection are names and values, so an argument
read from one of them states `NamedValuesAlg`: it is a product, and a document keys one parameter per
member rather than one parameter carrying the product. Nothing in a type says whether it is a
product, so the type says it, which makes a value stating no names a compile error rather than one a
caller finds on the first request. A header parameter is keyed the way the wire spells a header name,
which is `write_header_name` rather than the member's own spelling.

Headers and cookies are names and values, so an argument read from either is an ordinary product and
an author states a type of their own rather than a framework's. `read_header_name` states the one
rule that makes it portable: a header name is words, spelled with `-` and lowercased on the wire and
with `_` in an argument. A framework's own extractor is what the endpoint-context role states, which
is the escape hatch rather than the default.

Several roles may still reach one extractor. An authentication value is a header, so it is read as
one, and that is a correct interpretation rather than a missing distinction: the roles differ in what
they mean, which is what a description or an OpenAPI document reads, not in how one framework happens
to extract them.

Neutral markers such as `Path<T>`, `Query<T>`, and `Body<T>` preserve those roles until the fold.
They accumulate in the same order as the handler's argument product. The program does not parse or
deserialize anything itself.

Output roles are similarly delayed. `JsonOut`, `FileOut`, `TextOut`, `HtmlOut`, `BytesOut`,
`EmptyOut`, `RedirectOut`, and `StreamOut` each select a family of conversions through
`OutputKindAlg`; the handler result remains inferred from `ApplyAlg`. An API declaration therefore
does not restate its return type merely to choose what it answers with.

A body arriving as parts is the reading counterpart of one answered over time, and it is the same
algebra: a sequence of parts is `ChunksAlg` whose chunk is a `PartAlg`, and a part's content is a
sequence once more. What an author names states `FromPartsAlg`, so a domain says what it makes of
parts without naming whichever reader produced them. Poem, axum, and Salvo read parts natively;
`alux-http-parts` states the reading once for the interpretations whose framework does not, rather
than copying a parser into each.

A body answered over time is stated by `ChunksAlg` rather than by a stream type: a domain says what
a chunk is and how the next one is taken, and each interpretation adapts that to whatever moves its
bytes. That is what keeps `.stream()` from committing a domain to one framework's stream, and it is
why the direct interpretation can carry a produced body at all, having no framework to hand one to.

A kind is not required to convert every result. `OutputKindAlg` names the converter, and that the
converter reads the handler's result is required where the endpoint is compiled, so an endpoint
stating no body converts a handler returning nothing and rejects one returning data. Declaring a
kind a handler cannot answer is a compile error rather than a value silently discarded.

`HeaderOut<Kind, Name>` answers with a header beside the body `Kind` states, reading the handler's
result as the header's value and the body. A header is a name and nothing more to a program, so
`HeaderNameAlg` is the whole of what one states, and a header the specification does not name is a
marker a domain writes for itself: every interpretation already witnesses `HeaderOutAlg` once and
answers with whichever name reaches it.

Two kinds read the kind beneath them rather than the handler. `StatusOut<Kind, CODE>` answers with a
declared status around the body `Kind` states, because the status of a created resource is a
property of the endpoint rather than a decision inside a handler. `ResultOut<Kind>` reads a handler
that returns `Result`: the success answers with `Kind`, and the failure answers with what
`HttpErrorAlg` says it means, as a portable `HttpStatus` and a message. A domain states its
failures, and `HttpErrorAlg` states how they are answered, which keeps status vocabulary out of the
domain and domain vocabulary out of the interpreters.

### Selectors and routes

The route algebra exposes categorical structure:

```text
SelectorAlg
    identity
    compose

RouteAlg
    initial
    coproduct
    precompose selector
    lift endpoint
```

`HttpSelectorAlg` adds a request-method selector, an exact-path selector, and a prefix selector. A
method is a value of `HttpMethod` rather than a function per method, so an interpreter witnesses
every method by interpreting one value, and naming a further method adds no obligation to any
interpreter. `HttpMethod::ALL` lists the standard methods a surface can answer on. Each has a
marker type carrying its `HttpMethodAlg::METHOD`, which is what an endpoint declaration is typed by.

Fluent names are conventional aliases over the neutral structure:

- `.get`, `.post`, `.put`, `.patch`, `.delete`, `.head`, `.options`, `.trace`, and `.connect`
  combine a method and path selector around a typed endpoint; `.method` states the same for a method
  held as a type parameter.
- `.merge` forms a route coproduct.
- `.nest` precomposes a route subtree with a prefix.
- `.at` lifts an already interpreted endpoint at an exact path.

A path is read into `RoutePath` segments when it is declared, not passed along as a string. Routers
disagree about how a parameter is written, so a path held as a string is a path written for one
framework: `:id` is Poem's spelling, `{id}` is what axum and actix read, and each states the same
segment. `RoutePath::parse` accepts either, along with `*rest` and `{*rest}` for a segment that binds
the remainder, and matches anything else literally.

Each interpreter then spells those segments for its own router through `PathSyntaxAlg`, while
`describe_path` states a composed path in the one spelling every interpretation shares. That
separation is what keeps two interpretations of a program comparable: `TextRoute::labels` and
`PoemRoute::labels` describe the same surface in the same words, and Poem's router is handed
`PoemSelector::poem_path` instead.

`Empty`, `Merge`, `Nest`, `Named`, and `Endpoint` retain the complete route tree as a Rust type and
value. `CompileRouteProgram` folds that tree through a chosen interpreter. Named subprograms compile
through the same interpreter as their parent, which makes independently published surface fragments
composable.

### HTTP interpreters

`TextHandlerImpl` is the neutral reference witness. It records route selectors and endpoint type
information without executing handlers. It proves the program can denote documentation or metadata,
not only runtime routing.

`PoemHandlerImpl`, published as `alux-http-poem`, interprets input roles with Poem extractors,
applies operations against shared context handles, converts outputs, and materializes a Poem route.
Poem-specific bodies, headers, errors, and endpoint erasure remain inside this interpreter.

`AxumHandlerImpl`, published as `alux-http-axum`, folds the same program into an axum `Router`. It
is the evidence that the program describes a surface rather than one framework's callbacks: axum
reads paths in a different spelling, filters methods rather than naming them, and routes a `tower`
service rather than an endpoint, and none of that reached the program. Shared state stays on the
semantic context handle rather than becoming axum state, because a framework composing domain code
is the inversion this design exists to avoid.

`ActixHandlerImpl`, `SalvoHandlerImpl`, `WarpHandlerImpl`, and `RocketHandlerImpl` fold the same
program into the four remaining frameworks, and each one differs from the others somewhere the
program never learns about. actix-web configures its services again per worker and its `Route` is
neither cloneable nor reusable, so that interpretation states an endpoint as what makes one. Salvo
writes into a response it is given rather than returning one. warp holds no route table at all: a
coproduct is `or` and a selector is the filters a request must pass. Rocket states a method on every
route and builds a response borrowing the request. None of that reached the declaration.

`DirectHandlerImpl`, published as `alux-http-direct`, hands routing to nobody. It matches a request
against the segments a program states, binds what they capture, reads each argument, and renders a
`DirectResponse`, including the 404, 405, and 400 that routing and reading produce. It is the only
interpretation that can be held to what routing means, and because it answers with values rather
than framework types it is the reference the others are compared against. It also shows that a
declared path needs no spelling at all when whoever routes it reads segments directly.

`OpenApiHandlerImpl`, published as `alux-http-openapi`, applies nothing. It reads each endpoint for
what a caller states and what they are answered with, asking for `alux-shape` shapes where an
executing interpretation asks for extractors. It is the strictest reader the program has, and four
things exist in the specification because it could not be written without them: `OperationAlg::NAME`,
since a type name says where an operation lives rather than what it was called; `OperationAlg::DOC`,
since what an operation is for is stated in the doc comment the author already wrote and nowhere
else; the `OperationAlg` bound on `finish_handler`, since an interpretation that names operations to
a reader has nowhere else to read them from; and `HttpErrorAlg::HTTP_STATUSES`, since an interpretation
that folds a program never holds a failure to ask what it means.

A doc comment already reads as a summary and then a description, so that is how both describing
interpretations state it: OpenAPI writes the first line as `summary` and the rest as `description`,
and the TypeScript client writes the whole of it as the comment above each call. A successful
response is described by what its operation was documented as; what a failure answers with the
program never says in words, so no document invents any.

`TsHttpClient`, published as `alux-http-typescript`, folds the same program from the caller's side:
what a caller states, where each argument goes, and what comes back. A path needs no second spelling
there either, because a described path is already the template a call fills in. It is the evidence
that a program is direction-neutral rather than a description of a server.

`alux-http-conformance` states one declared surface and the exchanges every interpretation of it
must satisfy. Two interpretations agreeing is evidence only when both were held to the same thing,
and that is what this holds them to: the executing ones answer the exchanges, and the describing ones
are held to the same composed surface. The exchanges name methods, paths, media types and statuses,
and nothing of the domain or of any framework; an interpretation answering with a framework's own
response states an adapter for `AnswerAlg`, which is the only place a framework is named.

A future interpreter should fold the same first-order program. It must not maintain a parallel route
list.

## JSON-RPC program algebra

JSON-RPC preserves a smaller composition:

```text
empty method collection
    + named method
    + program merge
    + named subprogram
    + positional or named parameter decoding
```

`Empty`, `Merge`, `Named`, and `Method` are neutral syntax nodes. `JsonRpcProgram` provides fluent
composition; `CompileJsonRpcProgram` folds the result through `JsonRpcAlg` and `JsonRpcMethodAlg`.

Positional parameters are the default. `.named()` selects object decoding using `ARG_NAMES` from the
operation. Heterogeneous Rust argument products remain typed even though JSON-RPC positional input is
a JSON array and named input is a JSON object. An argument a request leaves out reads as absent, which
only an optional argument accepts.

`.fallible()` is the other distinction a method carries: it converts the operation's error into a
JSON-RPC protocol error. It selects `JsonRpcFallibleAlg` instead of `JsonRpcMethodAlg`, so an
operation's `Result` answers in the response's `error` member rather than as a success carrying an
error-shaped value. The conversion belongs to the program because it is part of what the surface
promises, not an accident of a Rust type. A program whose methods all convert states it once as
`fallible` in the attribute, and every silent declaration in it is read that way.

`OutcomeAlg` names the two halves of a fallible output and `RpcErrorAlg` says what the failing half
denotes: a code and a message. Both live in the specification, so a domain states what its failures
mean without naming an interpreter, and an interpretation reads that statement to build whatever error
its transport library uses.

`JsonrpseeImpl`, published as `alux-jsonrpc-jsonrpsee`, owns deserialization, method registration,
context sharing, application, serialization, and boundary error conversion. These are jsonrpsee
mechanics, not JSON-RPC program meaning.

## Program composition

Programs compose before interpretation:

```text
status HTTP program -----\
download HTTP program ---- merge -> service HTTP program -> text / Poem / metadata

status RPC program -------\
items RPC program --------- merge -> service RPC program  -> jsonrpsee / test interpreter
```

A specification exposing both transports states one set of operations and two surfaces:

```text
                          /- HTTP program   (selectors, input roles, output kinds)
one capability -> operations
                          \- JSON-RPC program (method names, parameter modes)
```

The operations are shared because they carry no transport. The surfaces are separate because they
preserve different distinctions, and no interpreter may invent the missing ones: a route selector and
a path role cannot be derived from a method name, and choosing them would be domain policy hidden in
a framework. Two interpreters can then share one semantic context while each owns its own runtime
handle, so the same operation answers over Poem and over jsonrpsee with the same value.

Downstream specification crates can publish small named programs. An application selects which
program values to merge or nest and which concrete interpreter to use. This avoids a central API
registry while preserving static compatibility between handlers, arguments, extraction roles, and
outputs.

## Crate and framework boundaries

A specification crate contains only neutral meaning:

- `alux-http` and `alux-jsonrpc` contain their algebras, first-order syntax, and folds, and declare
  `alux-ext` as their single dependency.
- `alux-http-text` adds a description interpreter and needs no framework.
- `alux-http-poem` adds Poem and Serde.
- `alux-http-axum` adds axum, Tower, and Serde.
- `alux-http-actix` adds actix-web and Serde.
- `alux-http-salvo` adds Salvo and Serde.
- `alux-http-warp` adds warp, Bytes, and Serde.
- `alux-http-rocket` adds Rocket and Serde.
- `alux-http-direct` adds Serde and two encodings, and no framework.
- `alux-http-hyper` adds hyper and the body utilities, and no runtime.
- `alux-http-openapi` adds `alux-shape` and its JSON Schema interpretation, and no framework.
- `alux-http-typescript` adds `alux-shape` and its TypeScript interpretation, and no framework.
- `alux-jsonrpc-jsonrpsee` adds jsonrpsee, Serde, and boundary helper types.

The dependency list of a specification crate is the architectural test: `alux-http` and `alux-jsonrpc`
declare `alux-ext` and nothing else. Compiling proves less here than it appears to, because a crate can
only `use` what it declares; the added dependency is what a review has to catch. A framework must not be reachable from a specification crate even
behind a feature, because an optional dependency still makes framework meaning expressible there.

## Laws and test interpretation

Keep reusable laws distinct from concrete scenarios.

Useful laws include:

- selector identity and associativity
- route coproduct identity and associativity
- selector precomposition compatibility
- prefix nesting order
- named-program inclusion preserving the child program
- operation application agreeing with direct extension invocation
- positional and named decoding preserving the same argument product
- independent interpreters exposing the same ordered surface

Current tests provide finite witnesses:

- `alux-ext` compares direct extension invocation with generated `ApplyAlg`.
- `alux-ext-macros` checks generated public syntax and rejected forms.
- `alux-http-text` compiles direct and lowered programs and checks the route laws.
- `alux-http-poem` executes handlers and compares its ordered surface with the text interpretation.
- `alux-http-axum` executes the same declaration on a second framework and compares the same surface.
- `alux-http-direct` answers the same declaration itself, and is where routing's own failures are checked.
- `alux-http-openapi` reads the same declaration as a document, and is where the surface's description is checked.
- `alux-http-conformance` states the surface and scenario all of them are held to, which is what makes their agreement evidence.
- `alux-jsonrpc-jsonrpsee` runs one shared expectation against specification-first and native
  jsonrpsee APIs.

When a property applies to every interpreter, encode it as a generic law or expectation over public
capabilities. Do not duplicate framework-specific assertions and call the copies a specification.

## Publication architecture

The package graph determines publication order:

```text
alux-ext-macros
    -> alux-ext
        -> alux-http
            -> alux-http-parts
            -> alux-http-text
            -> alux-http-poem
            -> alux-http-axum
            -> alux-http-actix
            -> alux-http-salvo
            -> alux-http-warp
            -> alux-http-rocket
            -> alux-http-direct
            -> alux-http-hyper
            -> alux-http-openapi
            -> alux-http-typescript
            -> alux-http-conformance
        -> alux-jsonrpc
            -> alux-jsonrpc-jsonrpsee
```

Publish `alux-ext-macros`, wait for the registry index, then publish `alux-ext`. HTTP and JSON-RPC can
follow independently, and each interpreter follows the specification it witnesses.

Packaging strips `path` and leaves the version requirement behind, so a workspace dependency carries a
version exactly when it appears in someone's published manifest: `alux-ext-macros`, `alux-ext`,
`alux-http`, and `alux-jsonrpc` do. The interpreter crates do not, because nothing published depends on
them — `alux-http-text` is only a dev-dependency, and dev-dependencies are dropped when packaging.

Public compatibility includes:

- crate names
- public algebra signatures and associated types
- first-order syntax and fold behavior
- generated operation/program names
- argument-name preservation
- the authored forms the macros reject, and the wording of those rejections
- framework conversion behavior promised by interpreter documentation

## Adding a program distinction

1. State the meaning an interpreter must distinguish.
2. Confirm existing syntax cannot express it exactly.
3. Add the smallest public algebra method or first-order node.
4. Extend the generic fold.
5. Add a neutral interpretation before or alongside a framework interpretation.
6. Add laws or cross-interpreter scenarios.
7. Add macro lowering only if familiar authored syntax should construct the new node.
8. Document compatibility and migration effects.

Do not begin by pattern-matching more source syntax in the procedural macro. Begin with the public
first-order meaning the source should denote.

## Public surface

Crate roots are product surfaces. Internal parsing visitors, tuple accumulation helpers, Poem
extraction traits, and jsonrpsee selector details remain private. Re-export a module or item only
when its name belongs to vocabulary downstream specification authors or interpreter implementers
must learn.

The public surface should remain smaller and more stable than any one generated expansion or
framework implementation.
