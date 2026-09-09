# alux-monad

Composes computations whose next step depends on the previous value. Like
`alux-traversable`, this crate gives concrete Rust instances of a common operation:

```text
bind : M<A> × (A -> M<B>) -> M<B>
join : M<M<A>> -> M<A>
join = bind(identity)
```

| Instance | Unit | `bind` meaning | `join` meaning |
| --- | --- | --- | --- |
| `Option` | `Some` | Continues only when present | Collapses nested optional values |
| `Result`, with fixed error type | `Ok` | Continues only on success | Collapses nested results |
| Iterator (ordered sequence) | `core::iter::once` | Lazily concatenates dependent sequences | Lazily concatenates nested sequences |

The unit constructors already exist in Rust. `bind` uses `and_then` for `Option`
and `Result`, and `flat_map` for iterators. Optional and fallible steps accept
`FnOnce`, including closures that consume captured values. Iterator steps accept
`FnMut` and run only as the returned iterator is consumed. Inner iterables may be
arrays, vectors, or iterators; no allocation or collection is imposed.

```rust
use alux_monad::*;

assert_eq!(Some(21).bind(|x| Some(x * 2)), Some(42));
assert_eq!(Some(Some(42)).join(), Some(42));

let parsed = Ok::<_, std::num::ParseIntError>("21")
    .bind(str::parse::<u32>)
    .bind(|x| Ok(x * 2));
assert_eq!(parsed, Ok(42));
assert_eq!(Ok::<_, ()>(Ok(42)).join(), Ok(42));

let expanded: Vec<_> = [1, 2].into_iter().bind(|x| [x, x * 10]).collect();
assert_eq!(expanded, [1, 10, 2, 20]);
let flattened: Vec<_> = [vec![1, 2], vec![], vec![3]].into_iter().join().collect();
assert_eq!(flattened, [1, 2, 3]);
```

`bind` composes within one effect; traversal moves an effect through another
structure. For example, optional bind returns `Option<B>`, while traversing an
option with a fallible function returns `Result<Option<B>, E>`.

The laws are left identity (`unit(a).bind(f) = f(a)`), right identity
(`m.bind(unit) = m`), and associativity
(`m.bind(f).bind(g) = m.bind(|a| f(a).bind(g))`). `join` agrees with binding
identity. Equality for iterators observes the yielded sequence, not adapter types.
These algebraic laws concern pure transformations; arbitrary closure side effects
are not an additional equivalence promise. Infinite inner sequences can prevent
later outer values from being visited. Integration tests give finite evidence for
the laws and separately check short-circuiting, ownership, laziness, and order.

The public surface consists of extensions generated with `alux_ext::ext` over
standard types and explicit iterator bounds. It introduces no universal context,
runtime, or first-order program: ordinary method composition preserves the meaning.
