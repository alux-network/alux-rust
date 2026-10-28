//! Runs a program that is only its own command, configured partly through a product of options.

use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_cli_direct::{CliDirect, Exit};
use alux_ext::ext;
use alux_shape::Shape;
use std::sync::Arc;

/// How the node reaches its peers.
#[derive(Shape)]
struct P2p {
    listen: Vec<String>,
    port: u16,
}

trait Playground {
    fn label(&self) -> String;
}

#[ext(name = PlaygroundExt, defunc)]
impl<This> This
where
    This: Playground,
{
    /// Runs the playground.
    fn play(&self, p2p: P2p) -> String {
        format!("{} listen={:?} port={}", self.label(), p2p.listen, p2p.port)
    }
}

#[ext(name = PlaygroundCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn playground_cli<Machine>(&self) {
        self.commands().cmd(
            self.op(Machine::play).arg(
                self.product()
                    .field(self.named("Addresses to listen on").many().delimiter(','))
                    .field(self.named("Port to listen on").default("30303")),
            ),
        )
    }
}

struct Local;

impl Playground for Local {
    fn label(&self) -> String {
        "local".to_owned()
    }
}

fn playground(line: &str) -> Exit {
    CliDirect::new(Arc::new(Local))
        .version("0.9.0")
        .run_blocking(CliProgramBuilder.playground_cli::<Local>(), line.split_whitespace())
}

#[test]
fn splits_a_delimited_field_of_a_product() {
    assert_eq!(
        playground("playground --listen a,b --listen c").stdout,
        "local listen=[\"a\", \"b\", \"c\"] port=30303\n"
    );
    assert_eq!(playground("playground --port 1").stdout, "local listen=[] port=1\n");
}

#[test]
fn lists_version_in_the_help_of_a_program_that_is_only_its_own_command() {
    let help = playground("playground --help").stdout;
    assert!(help.contains("--listen <listen>,..."), "{help}");
    assert!(help.contains("-V, --version"), "{help}");
    assert_eq!(playground("playground -V").stdout, "playground 0.9.0\n");
}
