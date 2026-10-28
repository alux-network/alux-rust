//! Runs a single-binary program configured from options and the environment through `clap`.

use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_cli_clap::CliClap;
use alux_cli_direct::Exit;
use alux_ext::ext;
use std::{net::IpAddr, sync::Arc};

trait Node {
    fn name(&self) -> String;
}

#[ext(name = NodeExt, defunc)]
impl<This> This
where
    This: Node,
{
    /// Runs the node.
    fn run_node(&self, dev: bool, rpc_addr: Option<IpAddr>, peers: Vec<String>) -> String {
        format!("{} dev={dev} rpc={rpc_addr:?} peers={peers:?}", self.name())
    }

    /// Checks the configuration.
    fn check(&self, file: String) -> String {
        format!("checked {file}")
    }
}

#[ext(name = NodeCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn node_cli<Machine>(&self) {
        self.commands()
            .cmd(
                self.op(Machine::run_node)
                    .arg(self.named("Development mode").switch().env("CARGO_PKG_VERSION_MINOR"))
                    .arg(self.named("RPC address").optional())
                    .arg(self.named("Peers to dial").many().env("CARGO_PKG_VERSION").delimiter('.')),
            )
            .command("check", self.op(Machine::check).arg(self.positional("Configuration file")))
    }
}

struct Local;

impl Node for Local {
    fn name(&self) -> String {
        "local".to_owned()
    }
}

fn node(line: &str) -> Exit {
    CliClap::new(Arc::new(Local), CliProgramBuilder.node_cli::<Local>())
        .version("1.2.3")
        .run_blocking(line.split_whitespace())
}

#[test]
fn reads_a_switch_and_a_list_from_the_environment_unless_the_command_line_gives_them() {
    // Cargo sets `CARGO_PKG_VERSION_MINOR=1` and `CARGO_PKG_VERSION=0.1.0` for every test it runs.
    let bare = node("node");
    assert_eq!(bare.stdout, "local dev=true rpc=None peers=[\"0\", \"1\", \"0\"]\n", "{bare:?}");
    assert_eq!(
        node("node --rpc-addr 127.0.0.1 --peers a.b --peers c").stdout,
        "local dev=true rpc=Some(127.0.0.1) peers=[\"a\", \"b\", \"c\"]\n"
    );
    assert_eq!(node("node check node.toml").stdout, "checked node.toml\n");
}

#[test]
fn answers_version_as_clap_does() {
    assert_eq!(node("node --version"), Exit { status: 0, stdout: "node 1.2.3\n".into(), stderr: String::new() });
    assert_eq!(node("node -V").stdout, "node 1.2.3\n");
    let help = node("node --help").stdout;
    assert!(help.contains("Development mode [env: CARGO_PKG_VERSION_MINOR]"), "{help}");
}
