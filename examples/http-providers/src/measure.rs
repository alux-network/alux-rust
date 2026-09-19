//! Runs one measured round on a runtime of its own, with nothing else writing to the output.

use core::future::Future;
use log::{LevelFilter, Log, Metadata, Record};
use std::sync::OnceLock;
use tokio::runtime::Builder;

/// Takes the one global `log` logger, so nothing in the graph writes to this run's output.
///
/// Rocket installs a logger when a server is built and sets the maximum level from its config.
/// `actix-codec` and `actix-router` state `tracing` with its `log` feature, so what actix, Poem and
/// Salvo say through tracing becomes a `log` record as well, and Rocket's logger is what prints it.
/// Taking the logger first leaves Rocket's own `set_boxed_logger` failing, which is also what keeps
/// it from setting a level.
pub fn say_nothing() {
    /// A logger that is asked nothing and says nothing.
    struct Quiet;

    impl Log for Quiet {
        fn enabled(&self, _: &Metadata<'_>) -> bool {
            false
        }

        fn log(&self, _: &Record<'_>) {}

        fn flush(&self) {}
    }

    static TAKEN: OnceLock<()> = OnceLock::new();
    TAKEN.get_or_init(|| {
        let _ = log::set_boxed_logger(Box::new(Quiet));
        log::set_max_level(LevelFilter::Off);
    });
}

/// Runs `measuring` to completion on a current-thread runtime.
///
/// Built per call, off the clock, since a round times itself. What runs on it decides whether it
/// needs a local task set: several interpretations spawn tasks that are not `Send`.
pub fn measured<Measuring>(measuring: Measuring) -> Measuring::Output
where
    Measuring: Future,
{
    say_nothing();
    let runtime = Builder::new_current_thread().enable_all().build().expect("a runtime to measure on");

    runtime.block_on(measuring)
}
