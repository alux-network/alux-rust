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
- request bodies
- headers
- authentication values
- endpoint context values

Neutral markers such as `Path<T>`, `Query<T>`, and `Body<T>` preserve those roles until the fold.
They accumulate in the same order as the handler's argument product. The program does not parse or
deserialize anything itself.

Output roles are similarly delayed. `JsonOut` and `FileOut` select a family of conversions through
`OutputKindAlg`; the handler result remains inferred from `ApplyAlg`. An API declaration therefore
does not restate its return type merely to choose JSON or file response semantics.

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

`HttpSelectorAlg` adds GET, POST, exact-path, and prefix selectors. Fluent names are conventional
aliases over the neutral structure:

- `.get`/`.post` combine a method and path selector around a typed endpoint.
- `.merge` forms a route coproduct.
- `.nest` precomposes a route subtree with a prefix.
- `.at` lifts an already interpreted endpoint at an exact path.

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

A future OpenAPI, Axum, documentation, client-generation, or conformance interpreter should fold the
same first-order program. It must not maintain a parallel route list.

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

`.fallible()` is the other distinction a method can carry: whether it can fail. It selects
`JsonRpcFallibleAlg` instead of `JsonRpcMethodAlg`, so an operation's `Result` answers as a JSON-RPC
error rather than as a success carrying an error-shaped value. Failability belongs to the program
because it is part of what the surface promises, not an accident of a Rust type.

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
            -> alux-http-text
            -> alux-http-poem
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
