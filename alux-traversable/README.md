# alux-traversable

`traverse` and the `Traversable` class are [Conor McBride](http://strictlypositive.org/) and [Ross Paterson](https://www.staff.city.ac.uk/~ross/)'s, from [Applicative programming with effects](https://www.staff.city.ac.uk/~ross/papers/Applicative.pdf) (Journal of Functional Programming 18(1):1–13, 2008, [doi:10.1017/S0956796807006326](https://doi.org/10.1017/S0956796807006326)) — the paper that introduced applicative functors and, with them, the operator for running an effectful function over a structure:

```haskell
traverse  :: (Traversable t, Applicative f) => (a -> f b) -> t a -> f (t b)
sequenceA :: (Traversable t, Applicative f) => t (f a) -> f (t a)
sequenceA = traverse id
```

> We introduce Applicative functors — an abstract characterisation of an applicative style of effectful programming, weaker than Monads and hence more widespread.
>
> … hence we introduce the type class Traversable, capturing functorial data structures through which we can thread an applicative computation.
>
> — McBride and Paterson, §1 and §3

[Jeremy Gibbons](https://www.cs.ox.ac.uk/jeremy.gibbons/) and [Bruno C. d. S. Oliveira](https://i.cs.hku.hk/~bruno/) later showed in [The essence of the Iterator pattern](https://www.cs.ox.ac.uk/jeremy.gibbons/publications/iterator.pdf) (Journal of Functional Programming 19(3–4):377–402, 2009, [doi:10.1017/S0956796809007291](https://doi.org/10.1017/S0956796809007291)) that this one operator is what the iterator pattern is reaching for: walking a structure, doing something effectful at each element, and rebuilding the structure with the effects sequenced.

`t` is the traversable structure and `f` is the applicative effect. Rust cannot quantify over `t`, so this crate provides the two instances worth having — `Option` and iterators — with `f` fixed to `Result`, whose applicative is the short-circuiting one. So: a function returning `Result` becomes composable inside an `Option` or an iterator, sequencing the `Result` effect while preserving the shape and order of the outer value. The suffix identifies the wrapper produced inside `Result`:

| Method | Mapping result | Meaning |
| --- | --- | --- |
| `traverse` | `Result<R, E>` | Maps each input to exactly one output. |
| `traverse_opt` | `Result<Option<R>, E>` | Maps each input to zero or one output. |
| `traverse_iter` | `Result<I, E>` where `I: IntoIterator` | Maps each input to zero or many outputs. |

`sequence`, `sequence_opt`, and `sequence_iter` are the corresponding identity mappings for values that already contain `Result`:

- `Some(42).traverse(|x| Ok(x + 1)) == Ok(Some(43))`
- `Some(Ok(42)).sequence() == Ok(Some(42))`
- `[1, 2].traverse(|x| Ok(x + 1)) == Ok(vec![2, 3])`
- `[1, 2].traverse_opt(|x| Ok(Some(x))) == Ok(vec![1, 2])`
- `[1, 2].traverse_iter(|x| Ok([x])) == Ok(vec![1, 2])`
- `[Ok(None), Ok(Some(1))].sequence_opt() == Ok(vec![1])`

Which method applies is decided by what the mapping states inside `Result`, and what comes out is decided by what was traversed:

```text
Option .traverse/sequence T        ==> Option
Option .traverse/sequence Option   ==> Option
Option .traverse/sequence Iter     ==> Vec
Iter   .traverse/sequence T        ==> Vec
Iter   .traverse/sequence Option   ==> Vec
Iter   .traverse/sequence Iter     ==> Vec
```

Iterator traversal accepts stateful `FnMut` transformations, preserving source order and stopping at the first error.

The papers state the laws as naturality, identity, and composition: traversal commutes with applicative morphisms, traversal in the identity applicative is `fmap`, and two traversals composed are one traversal in the composed applicative. That last one is why the suffixes are worth having — `traverse_opt` and `traverse_iter` are each a traversal whose inner structure is already composed, so a caller writes one pass where two would nest.

Read in the `Option` and iterator shapes, the laws are:

- traversing `None` performs no effect and returns `Ok(None)`;
- iterator traversal preserves order and stops at the first error;
- optional iterator traversal omits `None` without reordering the remaining values;
- sequencing equals traversal by the identity function;
- `traverse_iter` concatenates each successful inner iterator in input order.

### What it is for

Hold an `Option<Key>`, and a function that reads one: `fn record(Key) -> Result<Record, Error>`. Map one over the other and you get `Option<Result<Record, Error>>` — which is the two layers in the wrong order. `?` works on the outside, and the failure is now on the inside, so it cannot be reached without taking the `Option` apart first. What the caller wants is `Result<Option<Record>, Error>`: fail if the read failed, state nothing if there was no key.

The paper arrives at it from the same direction, mapping a failure-prone function across a list and then collecting:

> As you can see, `flakyMap` traverses `ss` twice — once to apply `f`, and again to collect the results. More generally, it is preferable to define this applicative mapping operation directly, with a single traversal.
>
> — McBride and Paterson, §3

`traverse` is that single traversal: the swap done while mapping — one pass, and the result comes out with the failure outermost where `?` can reach it:

| You hold | Mapping gives | `traverse` gives |
| --- | --- | --- |
| `Option<A>` and `A -> Result<B, E>` | `Option<Result<B, E>>` | `Result<Option<B>, E>` |
| `Vec<A>` and `A -> Result<B, E>` | `Vec<Result<B, E>>` | `Result<Vec<B>, E>` |
| `Option<Result<A, E>>` — already mapped | | `Result<Option<A>, E>`, by `sequence` |

The reason this is worth a name is that the layers accumulate. A store read both fails and states nothing: `Result<Option<Raw>, Error>`. What it states must still be decoded, which fails on its own: `Raw -> Result<Record, Error>`. Put the two together without traversing and you are holding `Result<Option<Result<Record, Error>>, Error>` before doing anything with it — and the next step adds another layer.

### Why it reads better

Traversed, the path has one line per layer, and each line is an operation with a fixed meaning: `traverse` runs a fallible step inside a structure and brings the failure to the outside, `traverse_opt` does the same where the step itself may state nothing, `traverse_iter` where it may state several things. The shape of the code is the shape of the path, so a reader checks it by reading it, and a fourth level costs one more line rather than one more level of nesting.

By hand, every one of those layers becomes control flow invented on the spot: an early return here, an accumulator there, a `continue` that has to mean the right loop, a `match` whose two `None` arms mean different things. None of it is wrong Rust. What it lacks is that its correctness is re-established by reading it, at every site, every time it is touched — while the traversal's correctness was settled once, by the laws above, for every site at once. A loop states *how*; `traverse` states *what*, and that is the difference the combinator was named for.

The cost shows up in change, not in the first writing. Add a step to the traversed version and the types refuse anything that drops a layer. Add one to the hand-written version and the compiler is happy either way — a `continue` that should have been a `return`, or an accumulator cleared one loop too high, reads exactly like the code that was right.

### Reading the example

Read it as one layer coming off at a time. `raw(key)?` leaves an `Option<Raw>`; `traverse` runs the decode inside it and states `Result<Option<Record>, Error>`, which `?` opens again. `traverse_opt` on the parent key takes two layers at once — the key may not be there, and what it names may not be there either — and both come out as one `Option<Record>`. `traverse` over the parts turns many reads into one, stopping at the first key that had to be there. The hand-written twin meets the same three layers as a `match` returning early, a `match` inside a `match` whose two `None` arms mean different things, and a loop that has to remember which absence is fatal.

The example is a spec rather than an executable program: the store is stated over abstract types, nothing is implemented behind it, and nothing runs. That is the denotational style this workspace is written in, and it is also the shortest way to show a traversal without first inventing a database. That style is what the ALUX programming guidelines are about: [Denotations](https://alux-network.github.io/alux-programming/denotational-design/denotations.html) and [Laws and interpretations](https://alux-network.github.io/alux-programming/denotational-design/laws-and-interpretations.html).

```rust
use alux_ext::ext;
use alux_traversable::*;

trait Store {
    type Error;
    type Key: Copy;
    type Raw;
    type Record;
    type Part;
    type Assembly;

    /// Reads what a key stands for, which may not be there.
    fn raw(&self, key: Self::Key) -> Result<Option<Self::Raw>, Self::Error>;
    /// Decodes a record, which fails on its own terms.
    fn record(&self, raw: Self::Raw) -> Result<Self::Record, Self::Error>;
    /// Decodes a part, likewise.
    fn part(&self, raw: Self::Raw) -> Result<Self::Part, Self::Error>;
    /// The key a record may name.
    fn parent(&self, record: &Self::Record) -> Option<Self::Key>;
    /// The keys a record does name, each of which must be there.
    fn parts(&self, record: &Self::Record) -> Vec<Self::Key>;
    /// States what a record, its parent, and its parts come to.
    fn assemble(
        &self,
        record: Self::Record,
        parent: Option<Self::Record>,
        parts: Vec<Self::Part>,
    ) -> Self::Assembly;
    /// States a key that had to be there and was not.
    fn missing(&self, key: Self::Key) -> Self::Error;
}

/// Reads one whole assembly: a record, the record it may name, and the parts it does.
#[ext(name = StoreAssembly)]
pub impl<This> This
where
    This: Store,
{
    /// One line per layer, each a named operation.
    fn assembled(&self, key: This::Key) -> Result<Option<This::Assembly>, This::Error> {
        self.raw(key)? // may fail, and may state nothing
            .traverse(|raw| self.record(raw))? // decode inside the option
            .traverse_opt(|record| {
                // no record, no assembly
                let parent = self
                    .parent(&record)
                    .traverse_opt(|key| self.raw(key)?.traverse(|raw| self.record(raw)))?; // two absences, one option

                let parts = self.parts(&record).into_iter().traverse(|key| {
                    // many reads, one result, stopping at the first failure
                    let raw = self.raw(key)?.ok_or_else(|| self.missing(key))?; // this one must be there

                    self.part(raw)
                })?;

                Ok(Some(self.assemble(record, parent, parts)))
            })
    }

    /// The same, with every layer re-derived as control flow.
    fn assembled_by_hand(&self, key: This::Key) -> Result<Option<This::Assembly>, This::Error> {
        let record = match self.raw(key)? {
            Some(raw) => self.record(raw)?,
            None => return Ok(None), // this absence leaves the function
        };

        let parent = match self.parent(&record) {
            Some(key) => match self.raw(key)? {
                Some(raw) => Some(self.record(raw)?),
                None => None, // and these two mean different things
            },
            None => None,
        };

        let keys = self.parts(&record);
        let mut parts = Vec::with_capacity(keys.len());
        for key in keys {
            let raw = match self.raw(key)? {
                Some(raw) => raw,
                None => return Err(self.missing(key)), // this one is fatal, two loops in
            };

            parts.push(self.part(raw)?); // and the push must not outlive its `?`
        }

        Ok(Some(self.assemble(record, parent, parts)))
    }
}
```
