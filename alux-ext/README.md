# alux-ext

`alux-ext` gives you one attribute, `#[ext]`, that does two things: it writes the boilerplate needed
to add methods to types you do not own, and — when you ask for it — it turns each of those methods
into a *value* that stands for calling the method later.

The second part is the unusual one, so this README goes through it slowly: the problem being solved,
the code you write, the code the macro adds, and how the two fit together.

## The problem the `ext` attribute solves

Rust does not let you add a method to a type defined in another crate. The standard workaround is an
*extension trait*: declare a trait with the method, then implement it for the types you want. That
means writing every signature twice — once in the trait, once in the implementation — and keeping
them in sync forever.

[`extend::ext`](https://github.com/davidpdrsn/extend) removes that duplication. You write one
`impl` block that looks inherent, and the macro produces the trait and its blanket implementation:

```rust ignore
// You write this. It looks like an inherent impl, but `This` is a type parameter, so it applies to
// every type satisfying the bound rather than to one concrete type.
#[ext(name = ValueExt)]
impl<This> This
where
    This: ValueAlg,
{
    fn describe(&self) -> String {
        self.value().to_string()
    }
}

// The macro writes this for you: a trait carrying the signature, plus one blanket impl carrying the
// body. Any type implementing `ValueAlg` now has `.describe()` wherever `ValueExt` is in scope.
trait ValueExt<This> { fn describe(&self) -> String; }
impl<This> ValueExt<This> for This where This: ValueAlg { /* the body you wrote */ }
```

`alux_ext::ext` takes the same arguments as `extend::ext`: `name`, `supertraits`, and a visibility.
It generates the trait and the impl itself, except for a block stating no `name`, which it passes to
`extend::ext`. The `extend` crate is re-exported as `alux_ext::extend`, which is the path that
generated code names — that is why crates using `#[ext]` do not list `extend` as a dependency
themselves.

## What the attribute generates

`#[ext]` writes the trait and the impl itself, from one block, so each half says what it is for:

- **The trait** declares the future a method answers, which is what a caller can bound. `Send` is
  declared only where the block wrote it, since only a body satisfies it. Arguments are declared by
  name, without `mut`, and a pattern is declared as `_`, since how a body binds an argument is the
  body's business.
- **The impl** states the same method as the `async fn` it is, with the bindings the body asks for.

Either spelling in the block reaches both halves:

```rust ignore
// Written as `async fn`, and declared as the future it answers.
async fn gathered(mut self) -> Result<Vec<Self::Chunk>, Self::Error> { … }
fn gathered(self) -> impl core::future::Future<Output = Result<Vec<This::Chunk>, This::Error>>;

// Written as a future, which is how a method says its future is `Send`, and carried as the body
// the `async` block is made of.
fn gathered(mut self) -> impl Future<Output = …> + Send { async move { … } }
fn gathered(self) -> impl Future<Output = …> + Send;              // declared
async fn gathered(mut self) -> Result<Vec<This::Chunk>, This::Error> { … }   // carried
```

A future the block did not build here, such as one forwarding another call, is carried as written.

Both halves name the carrier where the block wrote `Self`. The trait is generic over the carrier, so
`Self::Chunk` would need a bound on `Self`, while `This::Chunk` is resolved by the bounds the block
already states. A nested item states its own `Self` and is left alone.

`supertraits` is what the trait extends, and nothing is added to it.

With `defunc`, one operation type per method is generated beside the two halves. A block handed to a
backend with `defunc(via = path)` is left to the backend, which states its own trait, and a block
stating no `name` is passed to `extend::ext`, which reads a name off the carrier.

## What `alux-ext` adds on top

Adding `defunc` to the attribute keeps everything above and generates one extra type per method:

```text
#[ext(name = ValueExt)]           ->  extension trait + blanket impl
#[ext(name = ValueExt, defunc)]   ->  extension trait + blanket impl + operation
```

An *operation* is a value that means "apply this method." It records the context the method needs,
the method's arguments as a tuple, the argument names as written in the source, and the result type.
Nothing is executed when you build one.

Why bother, when Rust already has function pointers and closures? Because these methods are generic
over an abstract interpreter and return abstract associated types, and such a method cannot be stored
uniformly in a table, list, or program tree. A zero-sized type per method can. That is what lets
`alux-http` and `alux-jsonrpc` hold your operations inside a route table or a method list, then hand
the whole thing to a framework later. The technique has a name — defunctionalization — and is
explained in the guidelines linked at the end.

## Step 1: the code you write

```rust ignore
use alux_ext::ext;

// You write this: a capability trait stating what the outside world must provide. It is the only
// place the actual value comes from.
trait ValueAlg {
    fn value(&self) -> u32;
}

// You write this too: a derived method, defined purely in terms of the capability above. `defunc`
// asks for the operation type in addition to the normal extension trait.
#[ext(name = ValueExt, defunc)]
impl<This> This
where
    This: ValueAlg,
{
    async fn value_plus(&self, increment: u32) -> u32 {
        self.value() + increment
    }
}

// You write this as well, wherever the real value lives: a concrete type answering the capability.
// It knows nothing about extensions, operations, or transports.
struct Value(u32);

impl ValueAlg for Value {
    fn value(&self) -> u32 {
        self.0
    }
}
```

That is the whole authored surface. Note what is absent: no struct holding state, no trait object, no
registry, no framework, and no repetition of the argument or return types.

## Step 2: the code the macro adds

The expansion below is real output with only the `#[allow(...)]` noise removed. The first half is the
ordinary extension trait described earlier; the second half is the operation.

```rust ignore
// --- First half: the ordinary extension, as a trait declaring the method and an impl carrying its
// body. ---

// The signature, lifted into a trait so it can apply to types this crate does not own. `async fn` is
// how a body is written, not something a declaration can state, so the declaration states the future
// that body answers with. A caller can name and bound that future, and there is nothing here for
// `async_fn_in_trait` to warn about.
trait ValueExt<This>
where
    This: ValueAlg,
{
    fn value_plus(&self, increment: u32) -> impl core::future::Future<Output = u32>;
}

// The body you wrote, kept as the `async fn` it was written as, implemented for every type
// satisfying the bound. This is what makes `value.value_plus(2).await` compile as an ordinary method
// call. Declaring the future and stating the body are the two halves of one written method.
impl<This> ValueExt<This> for This
where
    This: ValueAlg,
{
    async fn value_plus(&self, increment: u32) -> u32 {
        self.value() + increment
    }
}

// --- Second half: the operation. This is what `defunc` adds. ---

// The value that means "apply `value_plus`". It is zero-sized: `PhantomData` only remembers which
// context type the operation expects, so there is no hidden state to configure or get wrong.
#[doc(hidden)]
struct ValuePlusOperation<Context>(core::marker::PhantomData<fn() -> Context>);

// Because it holds nothing, `Default` is enough to build one. Program declarations rely on this.
#[doc(hidden)]
impl<Context> core::default::Default for ValuePlusOperation<Context> {
    fn default() -> Self {
        Self(core::marker::PhantomData)
    }
}

// The method's signature, now readable as data at compile time: which context it needs, its
// arguments as a tuple in declaration order, and the names you gave those arguments. JSON-RPC named
// parameters read `ARG_NAMES` instead of asking you to list the names a second time.
#[doc(hidden)]
impl<Context> ::alux_ext::OperationAlg for ValuePlusOperation<Context> {
    type Context = Context;
    type Args = (u32,);
    const ARG_NAMES: &'static [&'static str] = &["increment"];
}

// Application, kept separate from the signature. `Handle` is whatever carrier the caller chose —
// `Arc<Context>` for a server, something cheaper for a test — so `Arc`, `Send`, and executor
// concerns live here at the boundary instead of in the method you wrote.
#[doc(hidden)]
impl<Context, Handle> ::alux_ext::ApplyAlg<Handle, (u32,)> for ValuePlusOperation<Context>
where
    Handle: AsRef<Context> + Send,
    Context: Sync,
    Context: ValueAlg,
{
    // Inferred from your return type, so no declaration ever restates it.
    type Output = u32;

    fn apply(
        &self,
        __context: Handle,
        __args: (u32,),
    ) -> impl core::future::Future<Output = Self::Output> + Send {
        async move {
            // The tuple is unpacked back into named arguments...
            let (increment,) = __args;
            // ...and the method you wrote is *called*. The body is never copied, so the operation
            // cannot drift away from the method it denotes.
            __context.as_ref().value_plus(increment).await
        }
    }
}
```

`#[doc(hidden)]` hides the machinery, not the name: downstream code refers to `ValuePlusOperation`,
so that name is public compatibility surface and renaming it is a breaking change.

## Step 3: using both halves

```rust ignore
use alux_ext::ApplyAlg;
use std::sync::Arc;

// You write this: the ordinary call. With the generated `ValueExt` in scope, the method is simply
// there. Most code stays at this level and never mentions operations at all.
let value = Arc::new(Value(40));
assert_eq!(value.value_plus(2).await, 42);

// You write this only when application must become data: the same computation, reached through the
// operation value. `(2,)` is the argument tuple, `value` is the chosen handle.
let operation = ValuePlusOperation::<Value>::default();
assert_eq!(operation.apply(value, (2,)).await, 42);
```

Both lines produce `42`, and that is not a coincidence to be tested per method: the generated `apply`
calls `value_plus`, so agreement is structural. `alux-ext`'s own tests check it once.

## The public surface

Three traits, and the attribute:

| Item | Meaning |
| --- | --- |
| `OperationAlg` | The signature of a reified method: context, argument tuple, argument names |
| `ApplyAlg<Handle, Args>` | Application of that method against a handle, producing `Output` |
| `HandlerContextAlg<Context>` | An interpreter's choice of owned handle (for example `Arc<Context>`) for a context |
| `#[ext]` | The attribute above, with optional `defunc`, `defunc(via = http)`, `defunc(via = jsonrpc)` |

`HandlerContextAlg` exists so that a domain method never mentions ownership. The method takes
`&self`; the interpreter decides that sharing it across requests means `Arc<Context>`.

## When to reify a method

Use `defunc` only when the application must become data — stored in a route or method program,
inspected by another interpreter, or composed before anything runs. An ordinary method call that
composes fine needs no operation type, and adding one buys nothing.

For the reasoning behind all of this, read
[First-order programs](https://alux-network.github.io/alux-programming/rust-dd/first-order-programs.html)
and [Defunctionalization](https://alux-network.github.io/alux-programming/concepts/defunctionalization.html)
in the ALUX programming guidelines.
