//! Measures what opening, closing and ending one bound address cost.
//!
//! The scenario states what a server must do; this states how long it takes to do it. What a round
//! needs but does not measure, such as opening a server or putting a connection on it, happens off
//! the clock.

use crate::lifecycle::{LifecycleError, answered, free_address, hold};
use alux_ext::ext;
use alux_http::{HttpServerAlg, HttpServerSetup};
use core::time::Duration;
use std::time::Instant;
use tokio::task::LocalSet;

/// What is on the connection when a close begins, and how far the round waits for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Closing {
    asking_for: &'static str,
    ending: bool,
}

impl Closing {
    /// States a round holding `asking_for`, which ends the server after closing it when `ending`.
    pub const fn new(asking_for: &'static str, ending: bool) -> Self {
        Self { asking_for, ending }
    }

    /// Returns the path the held request asks for.
    pub const fn asking_for(self) -> &'static str {
        self.asking_for
    }

    /// Returns whether the round ends the server after closing it.
    pub const fn ending(self) -> bool {
        self.ending
    }
}

/// What is on the connection while one address is handed from a server to the next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Handover {
    /// Nothing.
    NoRequest,
    /// A request asked and answered, timed as part of the round.
    RequestOk,
    /// A request the server is still producing an answer for, held off the clock.
    RequestHeld(&'static str),
}

/// Measures the lifecycle of a concrete HTTP server.
#[ext(name = MeasureLifecycleExt)]
pub impl<This> This
where
    This: HttpServerAlg,
    This::Error: Into<LifecycleError>,
{
    /// Closes `rounds` servers, each holding one request, and answers how long the closing took.
    ///
    /// Opening a server and holding a request on it is setup, and so is dropping what a round left
    /// behind. Only `close`, and `end` where [`Closing::ending`] states one, is timed.
    ///
    /// A round drops its server rather than ending it, since `close` has already taken the address
    /// back and whatever is still draining has nothing left to answer to. Ending it instead would
    /// cost every round a drain it is not measuring.
    async fn time_closing<Compile>(mut self, mut compile: Compile, closing: Closing, rounds: u64) -> Duration
    where
        Compile: FnMut() -> Self::Program,
    {
        LocalSet::new()
            .run_until(async move {
                let bind = free_address().expect("an address nothing holds");
                let mut timed = Duration::ZERO;
                for _ in 0..rounds {
                    let Ok(mut open) = self.open(HttpServerSetup::new(bind, compile())).await else {
                        panic!("the server opens");
                    };
                    let held = hold(bind, closing.asking_for()).await.expect("a request the server is serving");

                    let round = Instant::now();
                    assert!(self.close(&mut open).await.is_ok(), "the open server closes");
                    if closing.ending() {
                        assert!(self.end(&mut open).await.is_ok(), "the closed server ends");
                    }
                    timed += round.elapsed();

                    drop(held);
                    drop(open);
                }

                timed
            })
            .await
    }

    /// Hands one address from a server to the next `rounds` times, and answers how long it took.
    ///
    /// A handover is `close` then `open` on the same address: what a caller replacing a server
    /// needs, and nothing more. [`Handover::RequestHeld`] holds its connection off the clock.
    async fn time_handover<Compile>(mut self, mut compile: Compile, load: Handover, rounds: u64) -> Duration
    where
        Compile: FnMut() -> Self::Program,
    {
        LocalSet::new()
            .run_until(async move {
                let bind = free_address().expect("an address nothing holds");
                let Ok(mut open) = self.open(HttpServerSetup::new(bind, compile())).await else {
                    panic!("the first server opens");
                };

                let mut holding = Vec::new();
                let mut timed = Duration::ZERO;
                for _ in 0..rounds {
                    if let Handover::RequestHeld(asking_for) = load {
                        holding.push(hold(bind, asking_for).await.expect("a request the server is serving"));
                    }

                    let round = Instant::now();
                    if load == Handover::RequestOk {
                        answered(bind).await.expect("the open server answers");
                    }
                    assert!(self.close(&mut open).await.is_ok(), "the open server closes");
                    let Ok(next) = self.open(HttpServerSetup::new(bind, compile())).await else {
                        panic!("the next server opens");
                    };
                    open = next;
                    timed += round.elapsed();
                }

                assert!(self.end(&mut open).await.is_ok(), "the last server ends");

                timed
            })
            .await
    }
}
