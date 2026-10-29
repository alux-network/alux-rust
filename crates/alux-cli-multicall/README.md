# alux-cli-multicall

Reads an [`alux-cli`](https://docs.rs/alux-cli) program as a multicall executable: one binary installed under the name of each of its first-layer commands, BusyBox-style.

The program is the ordinary one, with no multicall syntax: its first layer is what the executable answers to. `Multicall` compiles it to its command paths and answers two things:

- `names()`: the first-layer command names, which an installer links to the executable and a `--list` command prints;
- `invocation(argv)`: the command line to run. Invoked as a first-layer command's name, `cat -u f`, that name is also the first word, and help shows the command beneath no program name; invoked under any other name, `box cat -u f`, the words select as usual. Its `command` names the first-layer command the words select, so an interpreter compiles only that one.

Any interpreter runs the invocation, [`alux-cli-direct`](https://docs.rs/alux-cli-direct) included:

```rust
use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_cli_direct::CliDirect;
use alux_cli_multicall::Multicall;
use alux_ext::ext;
use std::sync::Arc;

trait Words {
    fn separator(&self) -> String;
}

#[ext(name = WordsExt, defunc)]
impl<This> This
where
    This: Words,
{
    /// Writes its words.
    fn echo(&self, words: Vec<String>) -> String {
        words.join(&self.separator())
    }
}

#[ext(name = BoxCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn applets<Machine>(&self) {
        self.commands().command("echo", self.op(Machine::echo).arg(self.positional("Words").many()))
    }
}

struct Space;

impl Words for Space {
    fn separator(&self) -> String {
        " ".to_owned()
    }
}

let multicall = Multicall::new(CliProgramBuilder.applets::<Space>());
assert_eq!(multicall.names(), ["echo"]);

let run = |line: &str| {
    let invocation = multicall.invocation(line.split(' '));
    CliDirect::new(Arc::new(Space))
        .named(invocation.name)
        .selecting(invocation.command)
        .run_blocking(CliProgramBuilder.applets::<Space>(), invocation.words)
};
assert_eq!(run("/bin/echo a b").stdout, "a b\n");
assert_eq!(run("box echo a b").stdout, "a b\n");
```
