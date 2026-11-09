//! Runs a check with its panic caught and located, so a broken law is a verdict, not a crash.

use alux_laws::Failure;
use std::any::Any;
use std::cell::Cell;
use std::panic::{self, AssertUnwindSafe, catch_unwind};
use std::sync::Once;

thread_local! {
    /// Whether this thread is inside [`caught`], whose panic is reported rather than printed.
    static QUIET: Cell<bool> = const { Cell::new(false) };
    /// Where the last panic inside [`caught`] on this thread happened.
    static PANICKED_AT: Cell<Option<String>> = const { Cell::new(None) };
}

/// Installs, once per process, a panic hook that keeps a panic inside [`caught`] quiet and records
/// where it happened. Every other panic goes to the hook that was there before.
fn quiet_caught_panics() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if QUIET.get() {
                PANICKED_AT.set(info.location().map(|at| format!("{}:{}", at.file(), at.line())));
            } else {
                previous(info);
            }
        }));
    });
}

/// Runs `f`, reporting a panic, and where it happened, as its failure.
pub(crate) fn caught<T>(f: impl FnOnce() -> Result<T, Failure>) -> Result<T, Failure> {
    quiet_caught_panics();
    let outer = QUIET.replace(true);
    PANICKED_AT.set(None);
    let outcome = catch_unwind(AssertUnwindSafe(f));
    QUIET.set(outer);
    outcome.unwrap_or_else(|panic| {
        let why = panic_text(&*panic).unwrap_or("a panic without a message");
        let failure = Failure::new(why);
        Err(match PANICKED_AT.take() {
            Some(at) => failure.at(at),
            None => failure,
        })
    })
}

/// The message of a panic payload, when it has one.
fn panic_text(payload: &(dyn Any + Send)) -> Option<&str> {
    payload.downcast_ref::<String>().map(String::as_str).or_else(|| payload.downcast_ref::<&str>().copied())
}
