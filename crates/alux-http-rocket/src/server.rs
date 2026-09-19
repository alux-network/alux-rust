use crate::RocketRoute;
use alux_http::{HttpServerAlg, HttpServerSetup};
use rocket::config::{LogLevel, Shutdown as ShutdownConfig};
use rocket::fairing::AdHoc;
#[cfg(unix)]
use std::collections::HashSet;
use std::io;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

type RocketServerError = Box<dyn std::error::Error + Send + Sync>;

/// Bounds how long Rocket finishes outstanding requests before it ends them, in seconds.
///
/// The same bound the other interpretations state, so a request still being served this long after
/// a close is ended wherever it was made. Rocket's default is 2, which would end one sooner here
/// than anywhere else.
const GRACE: u32 = 5;

/// Bounds how long Rocket then finishes outstanding connection I/O, in seconds.
///
/// Nothing, because it is spent after the grace period, by which point every request has been
/// answered or ended. Rocket takes about a second of its own past these two either way, so closing
/// returns a second after the grace period rather than at it. Its default is 3.
const MERCY: u32 = 0;

/// Serves a Rocket route at the address its setup names.
#[derive(Debug, Default)]
pub struct RocketServer;

/// Carries an open Rocket server that is accepting requests.
pub struct RocketOpen {
    shutdown: rocket::Shutdown,
    task: JoinHandle<Result<(), RocketServerError>>,
}

impl Drop for RocketOpen {
    fn drop(&mut self) {
        self.shutdown.clone().notify();
        self.task.abort();
    }
}

/// Shuts Rocket down if the open that launched it is cancelled.
///
/// This open is the only one that awaits after spawning, so cancelling it would otherwise leave
/// the address held by a task with no handle.
struct Launching(Option<rocket::Shutdown>);

impl Launching {
    /// Disarms the guard once the open has finished.
    fn launched(&mut self) {
        self.0 = None;
    }
}

impl Drop for Launching {
    fn drop(&mut self) {
        if let Some(shutdown) = self.0.take() {
            shutdown.notify();
        }
    }
}

/// Reads a Rocket failure as text, which is also what keeps it from aborting the process.
fn rocket_error(error: &rocket::Error) -> RocketServerError {
    io::Error::other(error.to_string()).into()
}

impl HttpServerAlg for RocketServer {
    type Program = RocketRoute;
    type Open = RocketOpen;
    type Error = RocketServerError;

    async fn open(&mut self, setup: HttpServerSetup<Self::Program>) -> Result<Self::Open, Self::Error> {
        let (bind, route) = setup.into_parts();
        let (listening, mut bound) = oneshot::channel();
        let rocket = route
            .mount(rocket::custom(rocket::Config {
                address: bind.address().ip(),
                port: bind.address().port(),
                // Rocket writes a launch banner and a line per request to stdout. Whoever opened
                // this server states what it says, so it says nothing.
                log_level: LogLevel::Off,
                shutdown: ShutdownConfig {
                    grace: GRACE,
                    mercy: MERCY,
                    // Whoever opened this server decides when it closes, so Rocket takes no
                    // signals. Left on, it installs process-wide handlers and answers Ctrl-C and
                    // SIGTERM itself, which stops an application from ever seeing them.
                    ctrlc: false,
                    #[cfg(unix)]
                    signals: HashSet::new(),
                    ..ShutdownConfig::default()
                },
                ..rocket::Config::default()
            }))
            // Rocket binds inside `launch`, so liftoff is the only reliable signal that the
            // address is held. Probing it instead would answer for whatever server is there.
            .attach(AdHoc::on_liftoff("alux-http-rocket bound", move |_| {
                Box::pin(async move {
                    let _ = listening.send(());
                })
            }))
            .ignite()
            .await
            .map_err(|error| rocket_error(&error))?;
        let shutdown = rocket.shutdown();
        let mut launching = Launching(Some(shutdown.clone()));
        let mut task =
            tokio::task::spawn_local(async move { rocket.launch().await.map(|_| ()).map_err(|e| rocket_error(&e)) });

        // Rocket either lifts off or stops; stopping first carries the reason it failed.
        let open = tokio::select! {
            result = &mut task => match result? {
                Ok(()) => Err(io::Error::other("Rocket stopped before binding").into()),
                Err(error) => Err(error),
            },
            Ok(()) = &mut bound => Ok(RocketOpen { shutdown, task }),
        };
        launching.launched();

        open
    }

    async fn close(&mut self, open: &mut Self::Open) -> Result<(), Self::Error> {
        open.shutdown.clone().notify();
        if !open.task.is_finished() {
            let _ = (&mut open.task).await;
        }

        Ok(())
    }

    // Nothing here can tell the address being released from the drain being over, because both
    // happen inside the framework's own shutdown. So ending is closing, which satisfies both.
    async fn end(&mut self, open: &mut Self::Open) -> Result<(), Self::Error> {
        self.close(open).await
    }
}
