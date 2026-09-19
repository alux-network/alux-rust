//! A running example server, and the switches a benchmark drives through it.
//!
//! Starting the server and waiting for it to answer is setup, so a bench does it off the clock and
//! measures only the switches. What a switch is: one `POST /fw` naming the next provider, answered
//! by whichever provider is serving the front door at that moment.

use crate::{BoxError, run_until};
use alux_bench::BenchRounds;
use core::time::Duration;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Instant;

/// The providers a switch names, in the order the README lists them.
const PROVIDERS: [&str; 7] = ["hyper", "axum", "warp", "poem", "salvo", "actix", "rocket"];

/// How many clients send the switches at once.
const CLIENTS: u64 = 8;

/// Where the switches are asked for, which is the first fixed provider's own address.
const ADDRESS: &str = "127.0.0.1:3001";

/// How long the server is given to answer before starting it is called a failure.
const STARTS: Duration = Duration::from_secs(5);

/// A running example server, stopped when it is dropped.
pub struct Switching {
    result: mpsc::Receiver<Result<(), BoxError>>,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<thread::JoinHandle<Result<(), BoxError>>>,
}

impl Switching {
    /// Starts the example and answers once it is serving.
    ///
    /// # Panics
    ///
    /// Panics where the server stops before it answers, or does not answer before starting it
    /// times out.
    pub fn started() -> Self {
        let mut serving = Self::starting();
        serving.wait_for_server().expect("the example serves");

        serving
    }

    /// Drives `rounds` switches, split evenly between the clients sending at once, and answers how
    /// long they took.
    ///
    /// Every client takes its own share of the rounds, so what is measured is the switches the
    /// server answered rather than the clients waiting for each other.
    pub fn switches(&self, rounds: BenchRounds) -> Duration {
        let round = Instant::now();
        thread::scope(|scope| {
            for client in 0..CLIENTS {
                let first = rounds.count() * client / CLIENTS;
                let last = rounds.count() * (client + 1) / CLIENTS;
                scope.spawn(move || {
                    for index in first..last {
                        let provider = PROVIDERS[index as usize % PROVIDERS.len()];
                        assert!(switched(provider), "the front door switches to {provider}");
                    }
                });
            }
        });

        round.elapsed()
    }

    /// Starts the example on a runtime of its own, without waiting for it.
    fn starting() -> Self {
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let (completed, result) = mpsc::channel();
        let thread = thread::spawn(move || -> Result<(), BoxError> {
            let result =
                tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(run_until(async {
                    let _ = stopped.await;
                }));
            let _ = completed.send(result);
            Ok(())
        });

        Self { result, stop: Some(stop), thread: Some(thread) }
    }

    /// Asks for the front door until it answers, or until the server says why it cannot.
    fn wait_for_server(&mut self) -> io::Result<()> {
        let deadline = Instant::now() + STARTS;
        while Instant::now() < deadline {
            if request("GET", "/api", b"") {
                return Ok(());
            }
            match self.result.try_recv() {
                Ok(Ok(())) => return Err(io::Error::other("the example stopped before it served")),
                Ok(Err(error)) => return Err(io::Error::other(error)),
                Err(TryRecvError::Disconnected) => return Err(io::Error::other("the example's thread stopped")),
                Err(TryRecvError::Empty) => {}
            }
            thread::sleep(Duration::from_millis(100));
        }

        Err(io::Error::new(io::ErrorKind::TimedOut, "the example did not serve"))
    }
}

impl Drop for Switching {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Asks the front door to switch to `provider`, and answers whether it did.
fn switched(provider: &str) -> bool {
    request("POST", "/fw", format!("\"{provider}\"").as_bytes())
}

/// Sends one request to the front door, and answers whether it was answered with a 200.
fn request(method: &str, path: &str, body: &[u8]) -> bool {
    let Ok(mut stream) =
        TcpStream::connect_timeout(&ADDRESS.parse().expect("the front door's address"), Duration::from_secs(2))
    else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let asked = format!(
        "{method} {path} HTTP/1.1\r\nHost: {ADDRESS}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    if stream.write_all(asked.as_bytes()).is_err() || stream.write_all(body).is_err() {
        return false;
    }

    let mut answer = [0; 1024];
    stream.read(&mut answer).is_ok_and(|read| answer[..read].starts_with(b"HTTP/1.1 200"))
}
