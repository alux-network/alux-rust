# alux-laws

`alux-laws` states laws as a first-order program: a suite is a typed list of named operations, and interpreters read it.

A suite is an extension over the trait whose meaning it states, written `#[ext(name = XLaws, defunc(via = laws))]`. Each method is a law. The [`laws`] backend lowers it to an operation `Op_<law>` that states its name and documentation through `OperationAlg` and how it is checked through [`LawOp`], and to the suite's law program `Program_x_laws`, returned by `x_laws()`. The program is tagless-final, as an HTTP or JSON-RPC program is: it folds each law, in block order, through the interpreter's [`LawAlg::finish_law`], and its laws through [`LawsAlg`].

A suite states only laws. A law that takes an input holds for every input; which subject it runs on, which inputs it gets, pinned or sampled, and how many belong to the run. Every reader is an interpreter of the same program: the checker in `alux-laws-check` runs the laws, [`Names`] lists them, and [`Definitions`] reads what each one is, with nothing run.

```text
x_laws()                         the suite's law program, Program_x_laws
a.join(b)                        primitive suites over one subject, checked together
suite.over(|root: &R| -> S)      the suite over a root R, its subject projected from it
```

```rust
use alux_ext::ext;
use alux_laws::{LawDefinitionsExt, LawNamesExt, laws};

pub trait CounterAlg {
    fn bump(&mut self);
    fn count(&self) -> u32;
}

#[ext(name = CounterLaws, defunc(via = laws))]
pub impl<This> This
where
    This: CounterAlg,
{
    /// Bumping adds one.
    fn bump_adds_one(&mut self) {
        let before = self.count();
        self.bump();
        assert_eq!(self.count(), before + 1);
    }

    /// Bumping a counter `steps` times adds `steps`.
    fn bumps_add_up(&mut self, steps: &u32) {
        let before = self.count();
        (0..*steps).for_each(|_| self.bump());
        assert_eq!(self.count(), before + steps);
    }
}

struct Counter(u32);

impl CounterAlg for Counter {
    fn bump(&mut self) {
        self.0 += 1;
    }

    fn count(&self) -> u32 {
        self.0
    }
}

let names = counter_laws::<Counter>().names();
assert_eq!(names, [("CounterLaws", "bump_adds_one"), ("CounterLaws", "bumps_add_up")]);

let definitions = counter_laws::<Counter>().definitions();
assert_eq!(definitions[1].doc, "Bumping a counter `steps` times adds `steps`.");
assert_eq!(definitions[1].input, "u32");
```
