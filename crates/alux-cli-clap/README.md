# alux-cli-clap

Runs an [`alux-cli`](https://docs.rs/alux-cli) program through [`clap`](https://docs.rs/clap): `clap` parses the command line and renders help and usage errors, and the program states what the words mean.

`CliClap` compiles a program once, then:
- builds the `clap::Command` the program declares: a subcommand per path segment, an argument per source with its spellings, documentation, environment variable and default in its help, a global argument declared `global` on the command its context begins at, and a sum's alternatives in an `ArgGroup` so `clap` rejects two of them;
- reads `clap`'s `ArgMatches` back as each argument's words, decodes them through their sources, applies the operation, and renders the outcome as [`alux-cli-direct`](https://docs.rs/alux-cli-direct) does.

An argument is declared once, in the program, and `clap` never sees a second declaration of it. A value is read as `clap` reads it, with `value_parser!`: the standard types are, and a domain type states that it is with `clap_words!(Level)`, whether it reads through `FromStr`, a `ValueEnum`, or its own `ValueParserFactory`.

```rust
use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_cli_clap::CliClap;
use alux_ext::ext;
use std::sync::Arc;

trait Greeting {
    fn greeting(&self) -> String;
}

#[ext(name = GreetingExt, defunc)]
impl<This> This
where
    This: Greeting,
{
    /// Greets someone, loudly or not.
    fn greet(&self, name: String, loud: bool) -> String {
        let greeting = format!("{}, {name}", self.greeting());
        if loud { greeting.to_uppercase() } else { greeting }
    }
}

#[ext(name = GreetingCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn greeting_cli<App>(&self) {
        self.commands().command(
            "greet",
            self.op(App::greet).arg(self.positional("Who to greet").default("world")).arg(self.named("Shout").switch().short('l')),
        )
    }
}

struct App;

impl Greeting for App {
    fn greeting(&self) -> String {
        "hello".to_owned()
    }
}

let cli = CliClap::new(Arc::new(App), CliProgramBuilder.greeting_cli::<App>());
let run = |line: &str| cli.run_blocking(line.split(' '));

assert_eq!(run("hi greet ana").stdout, "hello, ana\n");
assert_eq!(run("hi greet -l").stdout, "HELLO, WORLD\n");
assert_eq!(run("hi greet --nope").status, 2);
assert!(run("hi greet --help").stdout.contains("Shout"));
```

## Moving an application over

An application that already parses with `clap` moves one command at a time. `command()` is the `clap` command the program declares, so its subcommands join the application's own, and `dispatch` runs the command the matches name when it is one of the program's, answering nothing for the application's own:

```rust
# use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
# use alux_cli_clap::CliClap;
# use alux_ext::ext;
# use std::sync::Arc;
# trait Greeting { fn greeting(&self) -> String; }
# #[ext(name = GreetingExt, defunc)]
# impl<This> This where This: Greeting {
#     /// Greets someone.
#     fn greet(&self, name: String) -> String { format!("{}, {name}", self.greeting()) }
# }
# #[ext(name = GreetingCliExt, defunc(via = cli))]
# impl<This> This where This: CliCommandsAlg {
#     fn greeting_cli<App>(&self) { self.commands().command("greet", self.op(App::greet).arg(self.positional("Who to greet"))) }
# }
# struct App;
# impl Greeting for App { fn greeting(&self) -> String { "hello".to_owned() } }
use clap::Command;

let cli = CliClap::new(Arc::new(App), CliProgramBuilder.greeting_cli::<App>());

// The application's own commands, and the program's beside them.
let legacy = Command::new("hi").subcommand(Command::new("version"));
let app = legacy.subcommands(cli.command().get_subcommands().cloned());

let matches = app.get_matches_from(["hi", "greet", "ana"]);
match cli.dispatch_blocking(&matches) {
    Some(exit) => assert_eq!(exit.stdout, "hello, ana\n"),
    // `version` is still the application's to run.
    None => {}
}
```

When every command is declared, the application's `clap` declarations are gone and `run_blocking(std::env::args()).exit()` is its whole `main`, or `alux-cli-direct` runs the same program with no `clap` at all.
