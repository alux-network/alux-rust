# Coming from [`clap`](https://docs.rs/clap), or how to stop writing your CLI for the parser

With `clap` you write for the machine. You tell the parser, very politely, which words to accept, how to validate them, and what to hand back, and the parser is delighted. With `alux-cli` you write for the humans: what the words mean and the operation they apply. The machine still gets its instructions, from an interpreter, and is no less delighted. Each `clap` concept is either part of that meaning or part of one interpreter. Each item below shows the `clap` way first and the `alux-cli` way after it.

## Side by side

### 🎯 A command runs your code

The command is your method: naming it is the whole dispatch.

```rs
// clap
Command::new("lookup")
// then in main: Some(("lookup", m)) => lookup(m)

// alux-cli
.command("lookup", self.op(Machine::lookup))
```

### 🧩 Commands grouped and reused

A group of commands is written once and nested under a name, as routes nest under a prefix.

```rs
// clap
.subcommand(Command::new("lookup"))
.subcommand(Command::new("admin").subcommand(Command::new("lookup")))

// alux-cli
.merge(self.reading_cli::<Machine, Name>())
.nest("admin", self.reading_cli::<Machine, Name>())
```

### 📦 An argument arrives typed

The value arrives as your method's parameter, typed and ready to use.

```rs
// clap
.arg(Arg::new("name").required(true))
// then in the handler: m.get_one::<String>("name").unwrap()

// alux-cli
.arg(self.positional("Requested name"))
```

Both write `.arg(..)`, but they mean different things. In `clap` it registers an argument under an id that the handler looks up again, so a mistyped id fails when the program runs. In `alux-cli` it gives the method's next parameter its source, in order, and the parameter's type is checked when the program compiles. `.arg_for(..)` also names the parameter, and a name that is not the parameter at that position fails to compile:

```rs
// alux-cli: `name` is the method's first parameter
.arg_for("name", self.positional("Requested name"))
```

### 🔤 Spellings

The parameter `limit` is spelled `--limit` on its own, and `.short('n')` adds `-n`. `.long(..)` picks a different long spelling when you want one.

```rs
// clap
.arg(Arg::new("limit").long("limit").short('n'))

// alux-cli
.arg(self.named("Result limit").short('n'))
```

More spellings for one argument, an operand that assigns one, and a bare number:

```rs
// clap: `cp -r` and `cp -R` alike
.arg(Arg::new("recursive").short('r').visible_short_aliases(['R']).action(ArgAction::SetTrue))

// alux-cli
.arg(self.named("Copy directories recursively").switch().short('r').short_aliases(&['R']))

// clap: no spelling for `dd if=FILE`; it takes the operands and splits them itself

// alux-cli: `if=FILE` assigns the argument, and takes no positional place
.arg(self.named("Input file").assigned("if").optional())

// clap: no spelling for `head -5`

// alux-cli: on the command, `-5` reads as `--lines 5`
self.op(Machine::head).arg(self.named("Lines").optional().short('n')).numeric_option("lines")
```

### 🔢 How often it occurs, and how it parses

The source states the shape, `.optional()`, `.many()`, or `.switch()`, and the parameter's type must agree with it or the program does not compile. The value inside is any type the interpreter can read from a word, a domain type included.

```rs
// clap
.required(true)
.action(ArgAction::Append)
.value_parser(value_parser!(usize))

// alux-cli: the shape on the source, checked against the parameter's type
.arg(self.named("Tags").many())
```

Values that one word carries several of, and a value that may be left out:

```rs
// clap: `--peers a,b` is two values
.arg(Arg::new("peers").long("peers").value_delimiter(',').action(ArgAction::Append))

// alux-cli: the parameter is a `Vec`
.arg(self.named("Peers to dial").many().delimiter(','))

// clap: `-i` or `-i=.bak`; `-i.bak` errors, since clap needs the `=`
// so `sed -i.bak`, the usual way to write a backup suffix, cannot be parsed
.arg(Arg::new("in-place").short('i').num_args(0..=1).require_equals(true).default_missing_value(""))

// alux-cli: the parameter is `Option<Option<String>>`
// no `-i` is `None`, `-i` is `Some(None)`, and `-i.bak` is `Some(Some(".bak"))`
.arg(self.named("Edit in place, keeping a backup with this suffix").optional_value().short('i'))
```

### 🌐 Global arguments

`--store` opens what every command beneath runs against. A failure to open exits with its own status.

```rs
// clap
.arg(Arg::new("store").long("store").global(true))
// then in main: open the store, report a failure, then dispatch

// alux-cli
.within(self.op(App::open).arg(self.named("The store")), self.notes_cli::<Store>())
```

### 🤝 Arguments given together

The fields arrive together as one struct, and `.optional()` makes them all or none.

```rs
// clap
.group(ArgGroup::new("auth").args(["user", "password"]).multiple(true).requires_all(...))

// alux-cli
.arg(self.product().field(self.named("User")).field(self.named("Password")).optional())
```

### 🔀 Arguments that exclude each other

The choice arrives as one enum carrying exactly the alternative given.

```rs
// clap
.arg(Arg::new("file").conflicts_with("url"))

// alux-cli
.arg(self.sum().or(self.named("Write to a file")).or(self.named("Send to a URL")))
```

### 🪂 Fallbacks

The command line first, then the variable, then the default.

```rs
// clap
.env("NOTES_STORE").default_value("main")

// alux-cli
.env("NOTES_STORE").default("main")
```

### 🚦 Exit status

`Err` exits with the status your error type states, right where the method is declared.

```rs
// clap, in the handler
eprintln!("{e}"); std::process::exit(3)

// alux-cli
.fallible()  // on a method returning Result
```

### 📖 Help

Help lives in the doc comments right beside the code it describes.

```rs
// clap
.about("Lists notes.").arg(Arg::new("limit").help("Maximum number of notes to show"))

// alux-cli
/// Lists notes.  (on the method)
.arg(self.named("Maximum number of notes to show"))
```

### 🧬 Derives

The extension is the declaration: one program, parsed, documented, tested, or served over HTTP by whichever interpreter you pick.

```rs
// clap
#[derive(Parser)]
struct Cli { #[command(subcommand)] command: Commands }

// alux-cli
#[ext(name = NotesCliExt, defunc(via = cli))]
impl<This> This where This: CliCommandsAlg { /* fn notes_cli<Store>(&self) { .. } */ }
```

### 🏃 Running

Each command is a `Send` future, ready for any executor, and several run at once.

```rs
// clap
let m = Cli::parse(); match m.command { ... }  // on the main thread

// alux-cli
CliDirect::new(app).run_blocking(program, std::env::args()).exit()
```

Where `clap` has `required` and `multiple`, the source states `.optional()` or `.many()`, and the compiler checks it against the parameter's type, so the two cannot drift apart. `requires` and `conflicts_with` have no counterpart: a product's fields are given together and a sum's alternatives exclude each other, because the parameter's type says so. Two commands with the same name are an error, as in `clap`.

## Global arguments

A global argument is written the same way in both: before the command path, between its segments, or after it, alone or clustered with switches, and it is listed in the program's and each command's help. What differs is what it means once it is read. `clap` hands back a string in `ArgMatches`, and the application turns it into what the commands run against: in `main`, it reads `--store`, opens the store, reports a failure with whatever status it chooses, and then dispatches to the subcommand's handler with the store. `within` states that code as a declaration: `--store` is the argument of the context operation, every command beneath is applied to what that operation answers, and nothing else reads the string.

| | `clap` | `alux-cli` |
|---|---|---|
| Declaring it | `Arg::global(true)` on the parent | `.arg(self.named(..))` on the context operation |
| Using it | code in `main` | the context operation's body |
| Its failure | code in `main` | `.fallible()` and `CliFailureAlg`, with the error's own status |
| The commands' handle | one, whatever `main` passes | `CliContextAlg::Inner`, which can be another type for the subtree |
| Required | rejected: "Global arguments cannot be required" | allowed; `.default(..)` or `.env(..)` can supply it |
| Positional | accepted | rejected at compile time |

Every argument is decoded before the context is opened, so a usage error is reported first and a context is never opened for a command line that cannot run. With a context that fails on an unknown store, `notes add lost -s home` exits with that failure's status, while `notes add -s home` exits with `USAGE_STATUS` for the missing text and opens nothing.

Positional global arguments are rejected because their position depends on how many positionals each command has and how many a command line gives. In `clap` 4.6.7, with an optional positional `limit` on `list` and a positional global `store`, `notes list work` reads `work` as the limit and `notes list 5 work` reads it as the store, and nothing reports the difference. A named global argument has one spelling under every command. A `clap` application with a positional global argument makes it named when it moves here.

## Running on many threads

`clap` ends where parsing ends: `get_matches` answers `ArgMatches`, and what runs is whatever `main` does with them, usually a direct call on the main thread. Running commands concurrently, from a shell loop, a batch file, or a server, is code the application writes around `clap`.

Here a command is an operation value, and applying it answers a `Send` future over an owned handle, which is what `alux-ext` states of every operation: the generated application is `Send` wherever its arguments are, with nothing to write in the spec. The same command runs unchanged wherever a future does:

- `run_blocking` waits on the calling thread, as a command line usually does.
- `run` answers the future, so an application awaits it on its own runtime, or spawns several command lines at once on a multi-threaded one, each against a clone of the handle.
- The operation a command applies is the one an HTTP route or a JSON-RPC method applies, so a command line and a server share one concurrent implementation instead of the CLI being a separate, single-threaded entry point.

## Moving over

A `clap` application moves one command at a time, with [`alux-cli-clap`](https://docs.rs/alux-cli-clap) running the declared commands through `clap` meanwhile:

1. Extract the command's meaning into capability-based extensions with `defunc`, and write its laws.
2. Declare its name and argument sources in a `defunc(via = cli)` program, and drop its `clap` declaration.
3. Add `CliClap::command()`'s subcommands to the application's `clap` command, and run what `CliClap::dispatch` answers for; the commands still declared in `clap` are dispatched as before, so compatibility tests on output and exit status keep passing.
4. When no `clap` declaration is left, `CliClap::run_blocking` is the whole `main`, or `alux-cli-direct` runs the same program with no `clap` at all.
