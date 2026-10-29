# alux-cli

`alux-cli` describes typed command-line programs independently of an argument parser.

The crate is a specification. It carries no interpreter and depends only on [`alux-ext`](https://docs.rs/alux-ext) and [`alux-shape`](https://docs.rs/alux-shape), whose products and sums its arguments share with HTTP and JSON-RPC: a command surface is described here as first-order syntax plus the capability algebras an interpreter must witness. It names no parser, process, or OS string type. Domain specifications provide their own capability traits and compose defunctionalized operations.

A CLI program is the same projection as an `alux-jsonrpc` method program. Commands form an ordered collection under merge, each command names a reified `alux_ext` operation, and named programs nest. The CLI adds two things of its own: a source for each argument of the operation, and nesting a program beneath a subcommand name.

```rust
use alux_cli::{CliCommandsAlg, cli};
use alux_ext::ext;

/// A downstream specification owns its primitive domain meaning.
trait Reading {
    type Name;
    type Value;

    fn read(&self, name: &Self::Name) -> Self::Value;
}

/// Derived operations become first-order values that preserve their argument names.
#[ext(name = ReadingExt, defunc)]
impl<This, Name, Value> This
where
    This: Reading<Name = Name, Value = Value>,
{
    /// Reads the value denoted by the requested name.
    fn lookup(&self, name: Name) -> Value {
        self.read(&name)
    }
}

/// The command program is declared before any parser is chosen.
#[ext(name = ReadingCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    /// Declares the reading commands.
    fn reading_cli<Machine, Name>(&self) {
        // An empty program, with one command added.
        self.commands().command(
            // `lookup` applies the operation `defunc` made from `lookup` above, and its
            // documentation is the command's help.
            "lookup",
            // `name: Name`, once: the first positional word.
            self.op(Machine::lookup).arg(self.positional("Requested name")),
        )
    }

    /// Declares the application: the reading commands, and the same beneath `admin`.
    fn app_cli<Machine, Name>(&self) {
        self.commands()
            // `lookup`, from the program above.
            .merge(self.reading_cli::<Machine, Name>())
            // `admin lookup`: the same program, each path prefixed with `admin`.
            .nest("admin", self.reading_cli::<Machine, Name>())
    }
}

```

A program declared in another extension block, or another crate, composes the same way. Whatever `merge`, `nest`, or `within` takes as its program is one, so `self.cat_commands::<Machine>()` there names the program `cat_commands` declares.

A declaration can bind its parts with `let` first and name them in the chain, `let dev = self.named("Dev mode").switch(); self.commands().cmd(self.op(Machine::run).arg(dev))`, which reads as if each were written where it is named and keeps every line short enough for `rustfmt`.

Each command path must be unique. Repeat one in the same declaration and the program does not compile; repeat one across programs you combine with `merge`, `nest`, or `within`, and running the program reports it:

```rust compile_fail
# use alux_cli::{CliCommandsAlg, cli};
# use alux_ext::ext;
# trait Lines { fn first(&self) -> usize; }
# #[ext(name = LinesExt, defunc)]
# impl<This> This where This: Lines { fn head(&self) -> usize { self.first() } fn tail(&self) -> usize { self.first() } }
#[ext(name = LinesCliExt, defunc(via = cli))]
impl<This> This where This: CliCommandsAlg {
    fn lines_cli<Machine>(&self) {
        // error: the command `head` is declared more than once
        self.commands().command("head", self.op(Machine::head)).command("head", self.op(Machine::tail))
    }
}
```

## Arguments

A command's arguments are its operation's arguments, and the declaration gives each one a source, in order. `positional(doc)` takes its words by position. `named(doc)` takes the words after `--argument-name`, respelled with `.long(..)` and `.short(..)`. A source supplies one value; `.optional()` makes it supply `Option` of one, `.many()` a `Vec` of any number, and `.switch()` a `bool` given by its spelling alone. The declared shape must be the argument's type, or the program does not compile, and the value inside is any type: reading it from a word is the interpreter's. When the command line gives an argument no words, `.env("VAR")` reads the environment variable next and `.default("word")` uses a word last. `.short_aliases(&['R'])` adds short spellings that mean the same, `.optional_value()` supplies `Option<Option<_>>` for a value written only in the option's own word (`-i` or `-i.bak`), `.many().delimiter(',')` splits each word into values, and `.assigned("if")` spells the argument as the operand `if=VALUE` instead of an option, taking no positional place. A command states how its words are read with `.options_first()`, `.unknown_options_as_operands()`, `.no_end_of_options()`, `.verbatim()`, and `.numeric_option("lines")`, which reads `-5` as `--lines 5`.

```rust
use alux_cli::{CliCommandsAlg, cli};
use alux_ext::ext;

trait Notes {
    type Text;
    type Tag;
    type Store;
}

#[ext(name = NotesExt, defunc)]
impl<This, Text, Tag, Store> This
where
    This: Notes<Text = Text, Tag = Tag, Store = Store>,
{
    /// Adds a note.
    fn add_note(&self, store: Store, text: Text, tags: Vec<Tag>, pinned: bool) {
        todo!()
    }

    /// Lists notes, newest first.
    fn list_notes(&self, store: Store, tag: Option<Tag>, limit: usize) -> Vec<Text> {
        todo!()
    }
}

#[ext(name = NotesCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    /// Declares the notes commands.
    fn notes_cli<Machine>(&self) {
        self.commands()
            .command(
                "add",
                self.op(Machine::add_note)
                    // `store: Store`, once: `--store <store>` or `-s <store>`, else `$NOTES_STORE`, else `notes`.
                    .arg(self.named("Store to write to").short('s').env("NOTES_STORE").default("notes"))
                    // `text: Text`, once: the first positional word.
                    .arg(self.positional("Text of the note"))
                    // `tags: Vec<Tag>`, any number of times: `--tag <tag>` or `-t <tag>`.
                    .arg(self.named("Tag the note carries").many().long("tag").short('t'))
                    // `pinned: bool`, a switch: true when `--pinned` or `-p` is given.
                    .arg(self.named("Pin the note").switch().short('p')),
            )
            .command(
                "list",
                self.op(Machine::list_notes)
                    // `store: Store`, the same source as in `add`.
                    .arg(self.named("Store to read from").short('s').env("NOTES_STORE").default("notes"))
                    // `tag: Option<Tag>`, at most once: the first positional word, if there is one.
                    .arg(self.positional("Only notes with this tag").optional())
                    // `limit: usize`, once: `--limit <limit>` or `-n <limit>`, else `10`.
                    .arg(self.named("How many notes").short('n').default("10")),
            )
    }
}
```

A source that cannot supply its argument does not compile once the program is: a missing or extra source, a source of the wrong shape for its argument (`.many()` for a `String`), and a switch with a `.default(..)`. A switch is turned on by its spelling and may fall back to `.env(..)` only.

`.arg_for("name", source)` declares the same source for the argument it names. The names are checked against the operation's own, in order, when the program compiles, so two arguments of one type cannot be swapped unnoticed:

```rust compile_fail,E0080
# use alux_cli::{CliCommandsAlg, CliProgramExt, cli};
# use alux_ext::ext;
# trait Remote { type Text; }
# #[ext(name = RemoteExt, defunc)]
# impl<This, Text: Default> This where This: Remote<Text = Text> {
#     /// Connects.
#     fn connect(&self, host: Text, user: Text) -> Text { Text::default() }
# }
#[ext(name = RemoteCliExt, defunc(via = cli))]
impl<This> This where This: CliCommandsAlg {
    fn remote_cli<Machine>(&self) {
        // error: argument 0 of `ConnectOperation<Machine>` is not `user`
        self.commands().command(
            "connect",
            self.op(Machine::connect).arg_for("user", self.named("User")).arg_for("host", self.named("Host")),
        )
    }
}
# struct Count;
# impl alux_cli::CliCommandsAlg for Count { type Commands = usize; fn cli_empty(&self) -> usize { 0 } fn cli_merge(&self, l: usize, r: usize) -> usize { l + r } }
# impl<O: alux_ext::OperationAlg, S, M> alux_cli::CliOperationAlg<O, S, M> for Count { type Commands = usize; fn cli_operation(&self, _: &'static str, _: O, _: S, _: M, _: alux_cli::Conventions) -> usize { O::ARG_NAMES.len() } }
# struct Box;
# impl Remote for Box { type Text = String; }
# Count.compile_cli(alux_cli::CliProgramBuilder.remote_cli::<Box>());
```

## Products and sums

An argument can be made of several named sources. A product supplies a value whose fields are given together: the type states its fields once as an [`alux-shape`](https://docs.rs/alux-shape) `Product`, which `#[derive(Shape)]` states with its shape, and the declaration gives each field a named source with `.field(..)`. With `.optional()` it supplies `Option` of the product instead: all of its fields or none of them, so a field given without the others is missing. A sum supplies a value given as exactly one of its alternatives: the type states them as an `alux-shape` `Sum` of two to sixteen alternatives, which `#[derive(Shape)]` states with its shape, the declaration gives each a named source with `.or(..)`, an alternative carrying no value is `()` and given by its spelling alone, and two alternatives given together conflict. With `.optional()` it supplies `Option` of the sum instead: one alternative or none.

```rust
use alux_cli::{CliCommandsAlg, cli};
use alux_ext::ext;
use alux_shape::Shape;

/// Where an export goes: to a location, or to the standard output.
#[derive(Shape)]
enum Target<Location> {
    At(Location),
    Out,
}

/// Who an export is made as.
#[derive(Shape)]
struct Credentials<User, Secret> {
    user: User,
    secret: Secret,
}

trait Exporting {
    type Location;
    type User;
    type Secret;

    fn export(&self, target: Target<Self::Location>, credentials: Option<Credentials<Self::User, Self::Secret>>);
}

#[ext(name = ExportingExt, defunc)]
impl<This, Location, User, Secret> This
where
    This: Exporting<Location = Location, User = User, Secret = Secret>,
{
    /// Exports everything to a target, as someone or anonymously.
    fn export_all(&self, target: Target<Location>, credentials: Option<Credentials<User, Secret>>) {
        self.export(target, credentials)
    }
}

#[ext(name = ExportingCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn exporting_cli<Machine, Location, User, Secret>(&self) {
        self.commands().command(
            "export",
            self.op(Machine::export_all)
                .arg(self.sum().or(self.named("Export to a location").short('a')).or(self.named("Export to the standard output").switch()))
                .arg(self.product().field(self.named("User to export as")).field(self.named("Secret of the user")).optional()),
        )
    }
}

```

Fields and alternatives are named, because positions belong to the command:

```rust compile_fail
# use alux_cli::{CliCommandsAlg, cli};
# use alux_ext::ext;
# #[derive(alux_shape::Shape)]
# struct Pair<A, B> { a: A, b: B }
# trait Pairing { type A; type B; fn pair(&self, pair: Pair<Self::A, Self::B>); }
# #[ext(name = PairingExt, defunc)]
# impl<This, A, B> This where This: Pairing<A = A, B = B> { fn paired(&self, pair: Pair<A, B>) { self.pair(pair) } }
#[ext(name = PairingCliExt, defunc(via = cli))]
impl<This> This where This: CliCommandsAlg {
    fn pairing_cli<Machine, A, B>(&self) {
        // error: a field takes a named source
        self.commands().command("pair", self.op(Machine::paired).arg(self.product().field(self.positional("A")).field(self.named("B"))))
    }
}
```

## Contexts

`within(operation, program)` applies every command of `program` to the value `operation` answers: a store it opens, a configuration it reads. The operation's arguments are the subtree's global arguments, which every command beneath it takes besides its own, and `.fallible()` lets opening the context fail with its error's status. The subtree compiles with the interpreter `CliContextAlg` hands over for that value, so a context can be another type than the one around it. Global arguments are named, because positions belong to each command:

```rust compile_fail
# use alux_cli::{CliCommandsAlg, cli};
# use alux_ext::ext;
# trait Shelf { type Store; type Opened; fn open(&self, store: Self::Store) -> Self::Opened; fn count(&self) -> usize; }
# #[ext(name = ShelfExt, defunc)]
# impl<This, Store, Opened> This where This: Shelf<Store = Store, Opened = Opened> {
#     fn opened(&self, store: Store) -> Opened { self.open(store) }
#     fn counted(&self) -> usize { self.count() }
# }
#[ext(name = ShelfCliExt, defunc(via = cli))]
impl<This> This where This: CliCommandsAlg {
    fn shelf_cli<Machine>(&self) {
        // error: a global argument is named
        self.commands().within(self.op(Machine::opened).arg(self.positional("Store")), self.inner_cli::<Machine>())
    }
    fn inner_cli<Machine>(&self) {
        self.commands().command("count", self.op(Machine::counted))
    }
}
```

The subtree is a program declared by a method of its own, because the context's interpreter compiles it and its bound names its type. An inline subtree is rejected where it is written:

```rust compile_fail
# use alux_cli::{CliCommandsAlg, cli};
# use alux_ext::ext;
# trait Open { type Opened; fn open(&self) -> Self::Opened; }
# #[ext(name = OpenExt, defunc)]
# impl<This, Opened> This where This: Open<Opened = Opened> { fn opened(&self) -> Opened { self.open() } }
#[ext(name = InlineCliExt, defunc(via = cli))]
impl<This> This where This: CliCommandsAlg {
    fn inline_cli<Shelf>(&self) {
        // error: `within` applies a named program
        self.commands().within(self.op(Shelf::opened), self.commands().command("x", self.op(Shelf::opened)))
    }
}
```

## Syntax

`CliProgramBuilder` constructs the same first-order program directly: `Empty`, `Merge`, `Named`, `Nest`, and `Command`, whose `Operation` carries its handler and the product of its argument sources. `defunc(via = cli)` lowers the fluent declaration to that syntax. A handler written as `op(Machine::method)` becomes the operation `ext(defunc)` generated for the method, and the command becomes one `CliOperationAlg` bound on the interpreter.

## Interpretation

`CliCommandsAlg` interprets the empty collection and ordered merge. `CliOperationAlg<Operation, Sources, Mode>` interprets one command. A describing interpreter reads `OperationAlg` and `CliSources::cli_arguments`. An executing interpreter answers two things: `CliWordsAlg`, the words the command line gives for an argument and the environment, and `CliWordAlg<Value>`, how one word decodes as a value. `CliDecodeSources` then takes each argument's words from the command line, its environment variable, or its default, and assembles the operation's arguments by their arities, and the interpreter applies the operation through `ApplyAlg`. `CliNestAlg` interprets nesting, and only a program that nests asks for it. An interpreter missing a binding cannot compile the program.

## Outcome

A command succeeds with its operation's output and exits with status zero. `.fallible()` reads an output that is a `Result` instead: `Ok` succeeds, and `Err` fails with the status its error states through `CliFailureAlg`, nonzero by its type. `CliOutcomeOf` reads an output under the declared mode, `Total` or `Fallible`, as a `CliOutcome`, so an interpreter that applies a `.fallible()` command whose operation does not answer a `Result` fails to compile. Words that do not make the command exit with `USAGE_STATUS` before anything applies.

How a value or a failure is rendered, and on which stream, belongs to the interpreter, as do splitting words, diagnostics, and help. [`alux-cli-direct`](https://docs.rs/alux-cli-direct) is the interpretation that runs a program as a command line with no parser framework.

## Laws

`CliCompositionLawsExt` states that merge has the empty collection as identity, is associative, and conserves declarations in order. `CliDeclarationLawsExt` states that a command keeps its operation's documentation, gives each of its arguments one source in order, and reads no two arguments from the same source. `CliNestLawsExt` states that nesting prefixes every path and changes nothing else. `CliContextLawsExt` states that a context adds its global arguments to every command beneath it and changes nothing else. `CliOutcomeLawsExt` states that a command exits with zero exactly when it succeeds. An interpreter states what it observes through `CliCommandsObserved`.

## Coming from `clap`

[Coming from `clap`](https://github.com/alux-network/alux-rust/blob/master/alux-cli/COMING_FROM_CLAP.md) shows each `clap` concept beside its `alux-cli` form, and how an application moves over one command at a time.
