//! What opening and closing one bound surface must do, whoever serves it.
//!
//! The scenario names no framework. It opens a server at an address, puts a connection on it,
//! closes it, and opens another at the same address, which is the whole contract: closing resolves
//! only once the address is released, so the address is free for whatever comes next.

use alux_ext::ext;
use alux_http::{HttpBind, HttpServerAlg, HttpServerSetup};
use core::error::Error;
use core::fmt::{self, Display, Formatter};
use core::str;
use core::time::Duration;
use std::io;
use std::net::{SocketAddr, TcpListener};
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::LocalSet;

/// Carries why a lifecycle scenario was not satisfied.
pub type LifecycleError = Box<dyn Error + Send + Sync>;

/// Bounds how long closing may take before the scenario calls it a close that never returns.
///
/// Every interpretation bounds its own drain well inside this. It is here to fail a hang as a test
/// failure rather than as a suite that never finishes.
pub const LIMIT: Duration = Duration::from_secs(20);

/// How long the scenario waits for a server to take a request it has been sent.
///
/// Writing a request only puts it in the socket. This is the moment given to the server to read it
/// and start answering, so what the close finds is a connection being served rather than one about
/// to be. Nothing the scenario states depends on the wait: a server that has not started yet simply
/// has less to drain.
///
/// Every time the scenario reports is counted from the close rather than from the request, so this
/// is the offset the endpoints are built to cancel. [`crate::PAUSE`] and [`crate::SLOW`] carry it,
/// which is what lets a measurement be read against the drain without arithmetic.
pub const SETTLE: Duration = Duration::from_millis(250);

/// Holds a concrete server to what opening and closing one bound address must do.
#[ext(name = ExpectLifecycleExt)]
pub impl<This> This
where
    This: HttpServerAlg,
    This::Error: Into<LifecycleError>,
{
    /// Holds this server to the contract when the connection it served has finished.
    ///
    /// One caller asks and is answered, and only then is the server closed. This is the ordinary
    /// case: nothing is in flight, so no interpretation has anything to drain.
    ///
    /// # Errors
    ///
    /// States the first thing the server did not do: an open or close that failed, a close that
    /// did not return, or a request the reopened server did not answer.
    async fn expect_reopening<Compile>(mut self, mut compile: Compile) -> Result<(), LifecycleError>
    where
        Compile: FnMut() -> Self::Program,
    {
        // Several interpretations serve from tasks that are not `Send`, so every scenario runs on a
        // local task set. It is theirs rather than the caller's, which is why it is stated here.
        LocalSet::new()
            .run_until(async move {
                let bind = free_address()?;
                let mut open = self.open(HttpServerSetup::new(bind, compile())).await.map_err(Into::into)?;

                answered(bind).await?;
                self.expect_closing(&mut open).await?;

                // The address was released, so it can be taken again. Opening is the assertion.
                let mut reopened = self.open(HttpServerSetup::new(bind, compile())).await.map_err(Into::into)?;

                answered(bind).await?;
                self.expect_closing(&mut reopened).await
            })
            .await
    }

    /// Holds this server to the contract when the request in flight finishes before the drain.
    ///
    /// A caller asks for something taking [`crate::PAUSE`], which is less than any drain an
    /// interpretation states, and the server is closed while that answer is still being produced.
    /// Closing gracefully means this caller is answered rather than cut off, so the answer must
    /// arrive on the connection it was asked on, after the close has already returned.
    ///
    /// # Errors
    ///
    /// States the first thing the server did not do: an open or close that failed, an answer the
    /// closing server never produced, or a request the reopened server did not answer.
    async fn expect_answering_what_is_in_flight<Compile>(mut self, mut compile: Compile) -> Result<(), LifecycleError>
    where
        Compile: FnMut() -> Self::Program,
    {
        LocalSet::new()
            .run_until(async move {
                let bind = free_address()?;
                let mut open = self.open(HttpServerSetup::new(bind, compile())).await.map_err(Into::into)?;
                let mut held = hold(bind, "/pause").await?;

                // Both at once, which is the whole statement: the caller is waiting for its
                // answer at the same time as the server is being closed, and neither is allowed to
                // be sequenced after the other. Reading only once closing had returned would state
                // nothing, since by then whatever was going to happen already had.
                let (closed, delivered) = tokio::join!(self.expect_closing(&mut open), read_answer(&mut held));
                closed?;
                delivered?;
                drop(held);

                let mut reopened = self.open(HttpServerSetup::new(bind, compile())).await.map_err(Into::into)?;

                answered(bind).await?;
                self.expect_closing(&mut reopened).await
            })
            .await
    }

    /// Holds this server to the contract when the request in flight outlives the drain.
    ///
    /// The other side of what a bound means. A caller asks for something taking [`crate::SLOW`],
    /// which is longer than any drain, and the server is closed while that answer is still being
    /// produced. Closing gracefully is not closing eventually: what is still being served when the
    /// drain runs out is ended, so this caller must be left without an answer rather than handed
    /// one by a server that no longer exists.
    ///
    /// # Errors
    ///
    /// States the first thing the server did not do: an open or close that failed, an answer that
    /// arrived anyway, or a request the reopened server did not answer.
    async fn expect_ending_what_outlives_the_drain<Compile>(
        mut self,
        mut compile: Compile,
    ) -> Result<(), LifecycleError>
    where
        Compile: FnMut() -> Self::Program,
    {
        LocalSet::new()
            .run_until(async move {
                let bind = free_address()?;
                let mut open = self.open(HttpServerSetup::new(bind, compile())).await.map_err(Into::into)?;
                let mut held = hold(bind, "/slow").await?;

                let (over, ended) = tokio::join!(self.expect_ending(&mut open), read_ending(&mut held));
                over?;
                ended?;
                drop(held);

                let mut reopened = self.open(HttpServerSetup::new(bind, compile())).await.map_err(Into::into)?;

                answered(bind).await?;
                self.expect_closing(&mut reopened).await
            })
            .await
    }

    /// Measures what closing and then ending do to one request in flight, rather than stating it.
    ///
    /// The same shape as the scenarios above, reporting what it saw instead of holding the server
    /// to it. `asking_for` chooses which case is measured: `/pause` for a request that finishes
    /// inside the drain, `/slow` for one that does not.
    ///
    /// Closing and ending are measured in that order on the one server, which is how a caller
    /// wanting both would ask: take the address back, then wait for the rest.
    ///
    /// # Errors
    ///
    /// States an open, close or end that failed. What the request in flight did is measured, never
    /// an error, since either outcome is a result worth reporting.
    async fn measure_closing<Compile>(
        mut self,
        mut compile: Compile,
        asking_for: &'static str,
    ) -> Result<Measured, LifecycleError>
    where
        Compile: FnMut() -> Self::Program,
    {
        LocalSet::new()
            .run_until(async move {
                let bind = free_address()?;
                let mut open = self.open(HttpServerSetup::new(bind, compile())).await.map_err(Into::into)?;
                let mut held = hold(bind, asking_for).await?;

                let started = Instant::now();
                let (times, delivered) = tokio::join!(
                    async {
                        let closed = self.close(&mut open).await;
                        let at_close = started.elapsed();
                        let ended = self.end(&mut open).await;

                        (at_close, started.elapsed(), closed.and(ended))
                    },
                    read_outcome(&mut held, started)
                );
                times.2.map_err(Into::into)?;
                drop(held);

                let reusable = self.open(HttpServerSetup::new(bind, compile())).await;
                let measured = Measured {
                    asked: asking_for,
                    closed: times.0,
                    ended: times.1,
                    in_flight: delivered.0,
                    at: delivered.1,
                    reusable: reusable.is_ok(),
                };
                if let Ok(mut reusable) = reusable {
                    self.close(&mut reusable).await.map_err(Into::into)?;
                }

                Ok(measured)
            })
            .await
    }

    /// Closes one open server and states that closing returned inside [`LIMIT`].
    ///
    /// # Errors
    ///
    /// States a close that failed, and a close that did not return in time.
    async fn expect_closing(&mut self, open: &mut Self::Open) -> Result<(), LifecycleError> {
        let closing = Instant::now();
        let closed = tokio::time::timeout(LIMIT, self.close(open))
            .await
            .map_err(|_| io::Error::other(format!("closing did not return inside {LIMIT:?}")))?;
        closed.map_err(Into::into)?;
        bounded_by_the_server(closing, "closing")
    }

    /// Ends one open server and states that ending returned inside [`LIMIT`].
    ///
    /// # Errors
    ///
    /// States an end that failed, and an end that did not return in time.
    async fn expect_ending(&mut self, open: &mut Self::Open) -> Result<(), LifecycleError> {
        let ending = Instant::now();
        let ended = tokio::time::timeout(LIMIT, self.end(open))
            .await
            .map_err(|_| io::Error::other(format!("ending did not return inside {LIMIT:?}")))?;
        ended.map_err(Into::into)?;
        bounded_by_the_server(ending, "ending")
    }

    /// Holds this server to the contract when a connection is still open as it closes.
    ///
    /// A caller connects, is answered, and then asks for something that takes [`crate::SLOW`] to
    /// answer, which is longer than any drain an interpretation states. So the connection outlives
    /// the close rather than the close waiting it out: whoever drains hits its own bound and stops
    /// anyway. Closing must still return, and the address must still be free for the server opened
    /// after it.
    ///
    /// # Errors
    ///
    /// States the first thing the server did not do: an open or close that failed, a close the
    /// held connection kept from returning, or a request the reopened server did not answer.
    async fn expect_reopening_while_a_connection_is_held<Compile>(
        mut self,
        mut compile: Compile,
    ) -> Result<(), LifecycleError>
    where
        Compile: FnMut() -> Self::Program,
    {
        LocalSet::new()
            .run_until(async move {
                let bind = free_address()?;
                let mut open = self.open(HttpServerSetup::new(bind, compile())).await.map_err(Into::into)?;
                let held = hold(bind, "/slow").await?;

                self.expect_closing(&mut open).await?;

                let mut reopened = self.open(HttpServerSetup::new(bind, compile())).await.map_err(Into::into)?;

                // A new connection, because what becomes of the held one differs: an
                // interpretation that only drops its listener answers it in its own time, one that
                // shuts its framework down ends it unanswered. Neither is what this states, which
                // is that the held connection did not keep the address from being taken again.
                answered(bind).await?;
                drop(held);

                self.expect_closing(&mut reopened).await
            })
            .await
    }
}

/// What became of a request that was in flight when its server was closed.
///
/// Answered and ended are not the only two. A close can land between the head of an answer and the
/// end of its body, leaving the caller a status it can read and an answer it cannot use, so that is
/// stated as what it is rather than counted as either.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InFlight {
    /// The server answered in full, carrying this status.
    Answered(u16),
    /// The head arrived carrying this status, and the body was cut off before it was complete.
    CutOff(u16),
    /// The connection ended with nothing on it, so closing ran out of drain first.
    Ended,
    /// Neither happened inside [`LIMIT`], so the measurement states nothing.
    Unfinished,
}

impl InFlight {
    /// Whether the caller was left with an answer it can use.
    pub const fn is_answered(self) -> bool {
        matches!(self, Self::Answered(_))
    }
}

impl Display for InFlight {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Answered(status) => write!(f, "answered {status}"),
            Self::CutOff(status) => write!(f, "{status} cut off mid-answer"),
            Self::Ended => f.write_str("ended unanswered"),
            Self::Unfinished => f.write_str("still waiting"),
        }
    }
}

/// Reads everything a server sends on a connection, and says what it amounted to.
///
/// Answers when the first byte arrived as well as what arrived, because the two state different
/// things: one is when the server got to this request, the other whether it finished it.
async fn read_outcome(connection: &mut TcpStream, started: Instant) -> (InFlight, Duration) {
    let mut answer = Vec::new();
    let mut first = None;
    let read = tokio::time::timeout(LIMIT, async {
        let mut arriving = [0; 1024];
        // Read on rather than once: whether the body is whole is the whole question, and one read
        // can carry a head whose body never follows.
        while let Ok(read) = connection.read(&mut arriving).await {
            if read == 0 {
                break;
            }
            first.get_or_insert_with(|| started.elapsed());
            answer.extend_from_slice(&arriving[..read]);
            // Stop at the end of one whole answer rather than at the end of the connection, which
            // on a keep-alive connection is not coming.
            if states_a_whole_answer(&answer) {
                break;
            }
        }
    })
    .await;

    if read.is_err() {
        return (InFlight::Unfinished, started.elapsed());
    }

    (read_as_answer(&answer), first.unwrap_or_else(|| started.elapsed()))
}

/// Reads bytes as the answer they amount to, whole or cut off.
fn read_as_answer(answer: &[u8]) -> InFlight {
    let Some(status) = stated_status(answer) else {
        return InFlight::Ended;
    };
    if states_a_whole_answer(answer) {
        return InFlight::Answered(status);
    }

    // An answer stating no length of its own ends where the connection does, so reading this far
    // is reading all of it. Anything else stated a length it did not reach.
    match answer.windows(4).position(|four| four == b"\r\n\r\n") {
        Some(head) => {
            let head = String::from_utf8_lossy(&answer[..head]).to_lowercase();
            if stated_length(&head).is_none() && !head.contains("transfer-encoding: chunked") {
                InFlight::Answered(status)
            } else {
                InFlight::CutOff(status)
            }
        }
        // Not even the head arrived whole.
        None => InFlight::CutOff(status),
    }
}

/// Whether these bytes carry one answer, all of it.
///
/// Only where the answer stated how long it would be. One that did not is whole when the connection
/// ends, which these bytes cannot say.
fn states_a_whole_answer(answer: &[u8]) -> bool {
    let Some(head) = answer.windows(4).position(|four| four == b"\r\n\r\n") else {
        return false;
    };

    let (head, body) = answer.split_at(head + 4);
    let head = String::from_utf8_lossy(head).to_lowercase();
    match stated_length(&head) {
        Some(stated) => body.len() >= stated,
        None => head.contains("transfer-encoding: chunked") && body.ends_with(b"0\r\n\r\n"),
    }
}

/// Reads the status an answer carries, where it carries one at all.
fn stated_status(answer: &[u8]) -> Option<u16> {
    let line = answer.split(|byte| *byte == b'\r').next()?;

    str::from_utf8(line).ok()?.split_whitespace().nth(1)?.parse().ok()
}

/// Reads how long an answer says its body is, where it says at all.
fn stated_length(head: &str) -> Option<usize> {
    head.lines().find_map(|line| line.strip_prefix("content-length:")?.trim().parse().ok())
}

/// One measurement of what closing did, which reads as a row of the table it is gathered for.
#[derive(Debug, Clone, Copy)]
pub struct Measured {
    /// What the held connection asked for.
    pub asked: &'static str,
    /// How long closing took to return.
    pub closed: Duration,
    /// How long until ending returned as well, counted from the same start.
    pub ended: Duration,
    /// What became of the request in flight.
    pub in_flight: InFlight,
    /// When that became known, measured from when closing started.
    pub at: Duration,
    /// Whether the address could be taken again once closing had returned.
    pub reusable: bool,
}

impl Display for Measured {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { asked, closed, ended, in_flight, at, reusable } = self;
        // Seconds throughout rather than whatever unit each duration reads best in, so a column of
        // these can be compared down the page.
        let (closed, ended, at) = (closed.as_secs_f64(), ended.as_secs_f64(), at.as_secs_f64());

        write!(f, "{asked} | {in_flight} at {at:.4}s")?;
        write!(f, " | close {closed:.4}s | end {ended:.4}s | reusable {reusable}")
    }
}

/// Takes the mean of however many durations, which is nothing where there are none.
fn meaned(durations: impl ExactSizeIterator<Item = Duration>) -> Duration {
    let Ok(rounds) = u32::try_from(durations.len()) else {
        return Duration::ZERO;
    };

    durations.sum::<Duration>().checked_div(rounds).unwrap_or_default()
}

/// Several measurements of the same thing, taken together.
///
/// One measurement of a close is one sample of a machine, so a comparison worth reading is a mean.
/// The range comes with it, because a mean says nothing on its own about whether the rounds agreed.
#[derive(Debug, Clone, Copy)]
pub struct Averaged {
    /// What the held connection asked for.
    pub asked: &'static str,
    /// How many measurements this is the mean of.
    pub rounds: usize,
    /// How long closing took to return, meaned.
    pub closed: Duration,
    /// How long until ending returned as well, meaned.
    pub ended: Duration,
    /// The spread between the slowest and fastest close, which says how much the mean is worth.
    pub range: Duration,
    /// What became of the request in flight, which every round agreed on.
    pub in_flight: InFlight,
    /// When that became known, meaned.
    pub at: Duration,
    /// Whether the address came back, which every round agreed on.
    pub reusable: bool,
}

impl Averaged {
    /// Takes the mean of several measurements of the same thing.
    ///
    /// # Errors
    ///
    /// States rounds with nothing in them, and rounds disagreeing about what became of the request
    /// or about whether the address came back. A mean over those would hide the thing worth
    /// knowing, which is that a provider did not do the same thing twice.
    pub fn over(rounds: &[Measured]) -> Result<Self, LifecycleError> {
        let [first, rest @ ..] = rounds else {
            return Err(io::Error::other("averaging nothing").into());
        };
        if let Some(differs) =
            rest.iter().find(|round| round.in_flight != first.in_flight || round.reusable != first.reusable)
        {
            let (asked, first, differs) = (first.asked, first.in_flight, differs.in_flight);
            return Err(io::Error::other(format!("{asked} was {first} in one round and {differs} in another")).into());
        }

        let closed = rounds.iter().map(|round| round.closed);
        let slowest = closed.clone().max().unwrap_or_default();
        let quickest = closed.clone().min().unwrap_or_default();

        Ok(Self {
            asked: first.asked,
            rounds: rounds.len(),
            closed: meaned(closed),
            ended: meaned(rounds.iter().map(|round| round.ended)),
            range: slowest.saturating_sub(quickest),
            in_flight: first.in_flight,
            at: meaned(rounds.iter().map(|round| round.at)),
            reusable: first.reusable,
        })
    }
}

impl Display for Averaged {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self { asked, rounds, closed, ended, range, in_flight, at, reusable } = self;
        let (closed, ended) = (closed.as_secs_f64(), ended.as_secs_f64());
        let (range, at) = (range.as_secs_f64(), at.as_secs_f64());

        write!(f, "{asked} | {in_flight} at {at:.4}s | close {closed:.4}s | end {ended:.4}s")?;
        write!(f, " | reusable {reusable} | range {range:.4}s over {rounds}")
    }
}

/// States that what took this long was bounded by the server rather than by [`LIMIT`].
///
/// Awaiting inside the limit is not the same as being bounded by the interpretation: one that took
/// the whole limit was bounded by the scenario, and says nothing about its own drain.
fn bounded_by_the_server(since: Instant, what: &str) -> Result<(), LifecycleError> {
    if since.elapsed() >= LIMIT {
        return Err(io::Error::other(format!("{what} was bounded by the scenario rather than by the server")).into());
    }

    Ok(())
}

/// Names an address nothing is bound to.
pub(crate) fn free_address() -> Result<HttpBind, LifecycleError> {
    // Binding and dropping states the address as free; the kernel picked it, so nothing else here
    // has a reason to hold it.
    let probe = TcpListener::bind("127.0.0.1:0")?;

    Ok(HttpBind::new(probe.local_addr()?))
}

/// Asks for the readings and states that the server answered them.
pub(crate) async fn answered(bind: HttpBind) -> Result<(), LifecycleError> {
    let address = bind.address();
    let mut connection = TcpStream::connect(address).await?;
    connection.write_all(asking(address, "/items").as_bytes()).await?;

    match read_outcome(&mut connection, Instant::now()).await.0 {
        InFlight::Answered(200) => Ok(()),
        read => Err(io::Error::other(format!("the server at {address} {read}")).into()),
    }
}

/// Takes a connection the server is producing an answer on, and keeps it.
///
/// Two requests, because both matter. The first is asked and answered, which states that this
/// connection was accepted rather than left waiting in the backlog. The second asks for the slow
/// answer and is not read, so the connection is one the server is still working on. An idle
/// keep-alive connection would not do: a graceful shutdown closes those at once and drains nothing.
pub(crate) async fn hold(bind: HttpBind, asking_for: &str) -> Result<TcpStream, LifecycleError> {
    let address = bind.address();
    let mut connection = TcpStream::connect(address).await?;
    connection.write_all(asking(address, "/items").as_bytes()).await?;

    read_answer(&mut connection).await?;

    connection.write_all(asking(address, asking_for).as_bytes()).await?;
    tokio::time::sleep(SETTLE).await;

    Ok(connection)
}

/// Reads one answer from a connection, and states that the server answered it in full.
///
/// In full rather than merely begun: a head whose body was cut off leaves the caller a status and
/// nothing it can use, which is not what being answered means.
async fn read_answer(connection: &mut TcpStream) -> Result<(), LifecycleError> {
    match read_outcome(connection, Instant::now()).await.0 {
        InFlight::Answered(200) => Ok(()),
        read => Err(io::Error::other(format!("the closing server {read}")).into()),
    }
}

/// Reads the end of a connection, and states that no usable answer arrived on it.
///
/// A head cut off mid-body counts as no answer, because that is what it leaves the caller with.
/// What must not happen is a whole answer from a server that has already closed.
async fn read_ending(connection: &mut TcpStream) -> Result<(), LifecycleError> {
    match read_outcome(connection, Instant::now()).await.0 {
        InFlight::Ended | InFlight::CutOff(_) => Ok(()),
        read => Err(io::Error::other(format!("the closed server {read} anyway")).into()),
    }
}

/// States one request, left open so the connection carries whatever comes next.
fn asking(address: SocketAddr, path: &str) -> String {
    format!("GET {path} HTTP/1.1\r\nHost: {address}\r\n\r\n")
}
