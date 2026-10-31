//! Runs commands whose operations fail with any error that displays, `anyhow::Error` among them.

use alux_cli::{CliCommandsAlg, CliProgramBuilder, cli};
use alux_cli_direct::{CliDirect, Exit};
use alux_ext::ext;
use anyhow::{Context, anyhow, ensure};
use std::sync::Arc;

trait Store {
    fn opened(&self) -> bool;
}

#[ext(name = StoreExt, defunc)]
impl<This> This
where
    This: Store,
{
    /// Opens the store.
    fn open(&self, name: String) -> anyhow::Result<String> {
        if self.opened() { Ok(format!("opened {name}")) } else { Err(anyhow!("refused")).context("opening the store") }
    }

    /// Checks a limit.
    fn check(&self, limit: u8) -> anyhow::Result<u8> {
        ensure!(limit <= 10, "limit {limit} is over 10");
        Ok(limit)
    }

    /// Locks the store.
    fn lock(&self) -> Result<(), String> {
        Err("locked elsewhere".to_owned())
    }
}

#[ext(name = StoreCliExt, defunc(via = cli))]
impl<This> This
where
    This: CliCommandsAlg,
{
    fn store_cli<Machine>(&self) {
        self.commands()
            .cmd("open", self.op(Machine::open).arg("name", self.positional("Store name")).fallible())
            .cmd("check", self.op(Machine::check).arg("limit", self.positional("Limit")).fallible())
            .cmd("lock", self.op(Machine::lock).fallible_as(122))
    }
}

struct Local(bool);

impl Store for Local {
    fn opened(&self) -> bool {
        self.0
    }
}

fn store(opened: bool, line: &str) -> Exit {
    CliDirect::new(Arc::new(Local(opened)))
        .run_blocking(CliProgramBuilder.store_cli::<Local>(), line.split_whitespace())
}

fn failed(status: u8, stderr: &str) -> Exit {
    Exit { status, stdout: String::new(), stderr: stderr.to_owned() }
}

#[test]
fn succeeds_with_the_value_of_an_anyhow_result() {
    assert_eq!(
        store(true, "store open main"),
        Exit { status: 0, stdout: "opened main\n".into(), stderr: String::new() }
    );
}

#[test]
fn fails_with_status_one_and_the_whole_chain_of_causes() {
    assert_eq!(store(false, "store open main"), failed(1, "error: opening the store: refused\n"));
}

#[test]
fn fails_with_status_one_where_an_operation_ensures() {
    assert_eq!(store(true, "store check 3").stdout, "3\n");
    assert_eq!(store(true, "store check 11"), failed(1, "error: limit 11 is over 10\n"));
}

#[test]
fn fails_with_the_status_the_declaration_states() {
    assert_eq!(store(true, "store lock"), failed(122, "error: locked elsewhere\n"));
}
