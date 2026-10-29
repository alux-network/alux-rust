//! Reads a domain value as `clap` reads it, once the domain states that it does.

use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_cli_clap::{CliClap, clap_words};
use alux_ext::ext;
use core::str::FromStr;
use std::sync::Arc;

/// A log level, read from its name.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Level(u8);

impl FromStr for Level {
    type Err = String;

    fn from_str(word: &str) -> Result<Self, String> {
        match word {
            "quiet" => Ok(Self(0)),
            "loud" => Ok(Self(2)),
            other => Err(format!("no level `{other}`")),
        }
    }
}

clap_words!(Level);

trait Logging {
    fn prefix(&self) -> String;
}

#[ext(name = LoggingExt, defunc)]
impl<This> This
where
    This: Logging,
{
    /// Sets the level.
    fn set_level(&self, level: Level) -> String {
        format!("{}{}", self.prefix(), level.0)
    }
}

#[ext(name = LoggingCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn logging_cli<Machine>(&self) {
        self.commands().command("level", self.op(Machine::set_level).arg(self.positional("The level")))
    }
}

struct Log;

impl Logging for Log {
    fn prefix(&self) -> String {
        "level ".to_owned()
    }
}

#[test]
fn reads_a_domain_value_through_its_from_str() {
    let cli = CliClap::new(Arc::new(Log), CliProgramBuilder.logging_cli::<Log>());
    assert_eq!(cli.run_blocking(["log", "level", "loud"]).stdout, "level 2\n");
    let rejected = cli.run_blocking(["log", "level", "shout"]);
    assert_eq!(
        (rejected.status, rejected.stderr.lines().next()),
        (2, Some("error: invalid <level>: no level `shout`"))
    );
}
