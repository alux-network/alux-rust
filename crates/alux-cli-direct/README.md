# alux-cli-direct

Runs an [`alux-cli`](https://docs.rs/alux-cli) program as a command line, with no parser framework.

`CliDirect` folds a program into its commands, then for each command line it:
- selects the command by its path, the longest one the words name, skipping global options written before or within the path;
- parses the rest as the command's arguments, as POSIX `getopt` does: `--long value`, `--long=value`, `-s value`, `-svalue`, clustered switches `-abc` where the first letter taking a value takes the rest of the word (`-sd:`), `-NUM` for a command's numeric option, `name=value` operands for assigned arguments, and `--`, after which every word is positional;
- decodes each argument through its source, from the command line, then its environment variable, then its default, with the value's `DirectWord`;
- applies the operation to the handle it was given, or, beneath a `within`, to the context the context operation opens from it, and renders the outcome.

A success writes its value to standard output and exits with zero. A `.fallible()` command's failure writes its error to standard error and exits with the error's status. Words that do not make the command write the problem and the command's usage, and exit with `USAGE_STATUS`. `--help` or `-h` prints help built from the same program: command lists from the first line of each operation's documentation, and per-command usage, arguments, options, global options, environment variables, and defaults.

```rust
use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_cli_direct::{CliDirect, Exit};
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

let run = |line: &str| CliDirect::new(Arc::new(App)).run_blocking(CliProgramBuilder.greeting_cli::<App>(), line.split(' '));

assert_eq!(run("hi greet ana").stdout, "hello, ana\n");
assert_eq!(run("hi greet -l").stdout, "HELLO, WORLD\n");
assert_eq!(run("hi greet --nope").status, 2);

// In a binary, the process's own arguments: `CliDirect::new(app).run_blocking(program, std::env::args()).exit()`.
let _: fn(Exit) -> ! = Exit::exit;
```

Words are operating system strings, `std::env::args_os()` as given, and a value reads its word through `DirectWord`: `OsString` and `PathBuf` take it unchanged, whatever its bytes. Each command reads its words under the conventions it declares, such as `.options_first()` or `.verbatim()`. `failures_after(FailurePrefix::Command)` writes a failure as `cat: reason` instead of `error: reason`, a failure stating `cli_reported` exits with its status alone, and `unknown_command(127, ..)` ends a command line naming no command with that status and message. `usage_status(1)` ends words that do not make a command with that status instead of `USAGE_STATUS`, and `selecting(Some("cat"))` compiles only the commands beneath that first-layer name. In a debug build, a program declaring one command path more than once runs nothing and names the duplicates, exiting with `DUPLICATE_STATUS`; a release build skips the check, as `clap` does. [`alux-cli-multicall`](https://docs.rs/alux-cli-multicall) reads a command line for a multicall executable.

`run_once` takes the handle and applies the selected command to the handle itself, for a handle that owns what it reads and cannot be cloned; `run_once_blocking` waits for it. `run` answers a future instead, `Send` and `'static`, for an application that awaits it on its own runtime or runs several command lines at once on many threads; `run_blocking` waits on the calling thread. A successful value renders through `DirectRender`, which the standard scalars, `String`, paths, `Option`, and `Vec` (one value per line) implement and a domain type implements for itself.
