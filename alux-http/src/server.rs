//! States HTTP server lifecycle independently of the framework that serves a surface.
//!
//! [`HttpServerAlg`] is the tagless-final specification: an interpreter chooses its executable
//! surface, open-server handle, and error types, then gives meaning to opening, closing and ending.
//! [`HttpServerExt::lifecycle`] derives streaming access from those primitives.

use alux_ext::ext;
use core::future::Future;
use core::pin::pin;
use futures::future::{Either, select};
use futures::{Stream, StreamExt, stream};
use std::net::SocketAddr;

/// Interprets the lifecycle of one bound HTTP surface.
pub trait HttpServerAlg {
    /// Carries the executable HTTP surface this server serves.
    type Program;
    /// Carries a concrete open server.
    type Open;
    /// States a failure while opening or closing a server.
    type Error;

    /// Opens the address a setup names and resolves only once it is ready to serve.
    ///
    /// Must leave no address bound if dropped before it resolves, so a call still binding can be
    /// cancelled.
    fn open(&mut self, setup: HttpServerSetup<Self::Program>) -> impl Future<Output = Result<Self::Open, Self::Error>>;

    /// Closes an open server and resolves only once its address is released.
    ///
    /// Nothing already being served is waited for. Those requests are answered, or ended when this
    /// interpretation's drain runs out, which may happen after this has resolved. So closing is
    /// what a caller wants to take the address back, and [`Self::end`] is what it wants to know
    /// the server is done.
    ///
    /// Leaves `open` available when closing fails, so a manager can retain the state it still owns
    /// rather than pretending the server disappeared.
    fn close(&mut self, open: &mut Self::Open) -> impl Future<Output = Result<(), Self::Error>>;

    /// Closes an open server and resolves once nothing it accepted is still being served either.
    ///
    /// A request already being served is answered, and one still being served when this
    /// interpretation's drain runs out is ended. So ending neither abandons whoever is mid-request
    /// nor waits on them without a bound.
    ///
    /// An interpretation that cannot tell the two apart resolves this where [`Self::close`]
    /// resolves, which satisfies both: the address is released and the drain is over.
    ///
    /// Leaves `open` available when ending fails, for the reason [`Self::close`] does.
    fn end(&mut self, open: &mut Self::Open) -> impl Future<Output = Result<(), Self::Error>>;
}

/// Names the socket address at which an HTTP surface is bound.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HttpBind(SocketAddr);

impl HttpBind {
    /// Names one socket address to bind.
    pub const fn new(address: SocketAddr) -> Self {
        Self(address)
    }

    /// Returns the socket address named.
    pub const fn address(self) -> SocketAddr {
        self.0
    }
}

impl From<SocketAddr> for HttpBind {
    fn from(address: SocketAddr) -> Self {
        Self::new(address)
    }
}

/// States one executable HTTP surface together with the address at which it is served.
#[derive(Debug)]
pub struct HttpServerSetup<Surface> {
    bind: HttpBind,
    surface: Surface,
}

impl<Surface> HttpServerSetup<Surface> {
    /// States that `surface` is served at `bind`.
    pub const fn new(bind: HttpBind, surface: Surface) -> Self {
        Self { bind, surface }
    }

    /// Returns the address at which the surface is served.
    pub const fn bind(&self) -> HttpBind {
        self.bind
    }

    /// Separates the bound address from the surface it serves.
    pub fn into_parts(self) -> (HttpBind, Surface) {
        (self.bind, self.surface)
    }
}

/// Requests the desired lifecycle state of one bound HTTP server.
#[derive(Debug)]
pub enum HttpServerCommand<Surface> {
    /// Requests that this setup be the server currently open.
    Open(HttpServerSetup<Surface>),
    /// Requests that no server remain open.
    Close,
}

/// Records the lifecycle transition an HTTP server interpreter made.
#[derive(Debug, PartialEq)]
pub enum HttpServerEvent<Error> {
    /// Records that a surface began serving at an address.
    Opened(HttpBind),
    /// Records that an open server released one address before another was bound.
    Replaced {
        /// Names the address the previous server closed.
        closed: HttpBind,
        /// Names the address the replacement server opened.
        opened: HttpBind,
    },
    /// Records that an open server released its address.
    Closed(HttpBind),
    /// Records that the requested transition could not be completed.
    Failed {
        /// Names the address of the transition that failed.
        bind: HttpBind,
        /// Carries the interpreter's failure.
        error: Error,
    },
}

/// How one open ended: with the server it made, or with the command that cancelled it.
enum Opening<Open, Error, Program> {
    /// What the server answered, open or failed.
    Made(Result<Open, Error>),
    /// The command that arrived while the open was still binding.
    Cancelled(HttpServerCommand<Program>),
}

/// Derives streaming access to a concrete HTTP server's lifecycle.
#[ext(name = HttpServerExt)]
pub impl<This> This
where
    This: HttpServerAlg,
{
    /// Interprets lifecycle commands as the ordered transitions this server makes.
    fn lifecycle<Commands>(self, commands: Commands) -> impl Stream<Item = HttpServerEvent<Self::Error>>
    where
        Commands: Stream<Item = HttpServerCommand<Self::Program>> + Unpin,
    {
        stream::unfold((self, commands, None, None), |(mut server, mut commands, mut open, mut pending)| async move {
            loop {
                let command = match pending.take() {
                    Some(command) => command,
                    None => commands.next().await?,
                };

                match command {
                    HttpServerCommand::Open(setup) => {
                        let bind = setup.bind();
                        let closed = match open.take() {
                            Some((closed, mut active)) => match server.close(&mut active).await {
                                Ok(()) => Some(closed),
                                Err(error) => {
                                    return Some((
                                        HttpServerEvent::Failed { bind: closed, error },
                                        (server, commands, Some((closed, active)), pending),
                                    ));
                                }
                            },
                            None => None,
                        };

                        // The next command cancels an open that has not bound yet, so a close
                        // does not wait for the address to be taken first.
                        let opening = {
                            let opens = pin!(server.open(setup));
                            match select(opens, commands.next()).await {
                                Either::Left((made, _)) => Opening::Made(made),
                                Either::Right((Some(next), _)) => Opening::Cancelled(next),
                                Either::Right((None, opens)) => Opening::Made(opens.await),
                            }
                        };

                        let made = match opening {
                            Opening::Made(made) => made,
                            Opening::Cancelled(next) => {
                                pending = Some(next);
                                // The previous address was released before this open was
                                // cancelled, so report that much.
                                let Some(closed) = closed else { continue };

                                return Some((HttpServerEvent::Closed(closed), (server, commands, None, pending)));
                            }
                        };

                        let (event, active) = match made {
                            Ok(active) => {
                                let event = match closed {
                                    Some(closed) => HttpServerEvent::Replaced { closed, opened: bind },
                                    None => HttpServerEvent::Opened(bind),
                                };
                                (event, Some((bind, active)))
                            }
                            Err(error) => (HttpServerEvent::Failed { bind, error }, None),
                        };

                        return Some((event, (server, commands, active, pending)));
                    }
                    HttpServerCommand::Close => {
                        let Some((bind, mut active)) = open.take() else {
                            continue;
                        };

                        let (event, active) = match server.close(&mut active).await {
                            Ok(()) => (HttpServerEvent::Closed(bind), None),
                            Err(error) => (HttpServerEvent::Failed { bind, error }, Some((bind, active))),
                        };

                        return Some((event, (server, commands, active, pending)));
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;
    use futures::channel::oneshot::{self, Receiver, Sender};
    use futures::{StreamExt, executor::block_on, stream};
    use std::rc::Rc;

    #[derive(Debug, PartialEq)]
    struct TestError;

    struct TestServer;

    struct TestOpen;

    impl HttpServerAlg for TestServer {
        type Program = &'static str;
        type Open = TestOpen;
        type Error = TestError;

        async fn open(&mut self, _setup: HttpServerSetup<Self::Program>) -> Result<Self::Open, Self::Error> {
            Ok(TestOpen)
        }

        async fn close(&mut self, _open: &mut Self::Open) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn end(&mut self, _open: &mut Self::Open) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    struct FailingServer {
        fails_once: bool,
    }

    impl HttpServerAlg for FailingServer {
        type Program = &'static str;
        type Open = TestOpen;
        type Error = TestError;

        async fn open(&mut self, _setup: HttpServerSetup<Self::Program>) -> Result<Self::Open, Self::Error> {
            Ok(TestOpen)
        }

        async fn close(&mut self, _open: &mut Self::Open) -> Result<(), Self::Error> {
            if self.fails_once {
                self.fails_once = false;
                Err(TestError)
            } else {
                Ok(())
            }
        }

        async fn end(&mut self, open: &mut Self::Open) -> Result<(), Self::Error> {
            self.close(open).await
        }
    }

    fn bind(port: u16) -> HttpBind {
        HttpBind::new(SocketAddr::from(([127, 0, 0, 1], port)))
    }

    #[test]
    fn streams_ordered_lifecycle_transitions() {
        let commands = stream::iter([
            HttpServerCommand::Open(HttpServerSetup::new(bind(3000), "first")),
            HttpServerCommand::Open(HttpServerSetup::new(bind(3001), "second")),
            HttpServerCommand::Close,
        ]);

        let events = block_on(TestServer.lifecycle(commands).collect::<Vec<_>>());

        assert_eq!(
            events,
            [
                HttpServerEvent::Opened(bind(3000)),
                HttpServerEvent::Replaced { closed: bind(3000), opened: bind(3001) },
                HttpServerEvent::Closed(bind(3001)),
            ]
        );
    }

    #[test]
    fn ignores_a_close_when_nothing_is_open() {
        let commands = stream::iter([HttpServerCommand::<&'static str>::Close]);
        let events = block_on(TestServer.lifecycle(commands).collect::<Vec<_>>());

        assert!(events.is_empty());
    }

    /// Binds only when told to, which is what lets a test ask for something while an open waits.
    struct SlowServer {
        /// Resolves when the open named by `blocks` may bind; never, where the sender is held.
        binding: Option<Receiver<()>>,
        /// Names which open waits, counted from the first.
        blocks: usize,
        opened: usize,
        bound: Rc<Cell<usize>>,
    }

    impl SlowServer {
        fn new(blocks: usize, bound: &Rc<Cell<usize>>) -> (Self, Sender<()>) {
            let (binds, binding) = oneshot::channel();
            (Self { binding: Some(binding), blocks, opened: 0, bound: bound.clone() }, binds)
        }
    }

    impl HttpServerAlg for SlowServer {
        type Program = &'static str;
        type Open = TestOpen;
        type Error = TestError;

        async fn open(&mut self, _setup: HttpServerSetup<Self::Program>) -> Result<Self::Open, Self::Error> {
            if self.opened == self.blocks
                && let Some(binding) = self.binding.take()
            {
                let _ = binding.await;
            }
            self.opened += 1;
            self.bound.set(self.bound.get() + 1);

            Ok(TestOpen)
        }

        async fn close(&mut self, _open: &mut Self::Open) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn end(&mut self, _open: &mut Self::Open) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn cancels_an_open_that_has_not_bound_when_something_else_is_asked() {
        let bound = Rc::new(Cell::new(0));
        let (server, _binds) = SlowServer::new(0, &bound);
        let commands = stream::iter([
            HttpServerCommand::Open(HttpServerSetup::new(bind(3000), "never binds")),
            HttpServerCommand::<&'static str>::Close,
        ]);

        let events = block_on(server.lifecycle(commands).collect::<Vec<_>>());

        // The close was answered while the address was being taken, so it never was taken.
        assert!(events.is_empty());
        assert_eq!(bound.get(), 0, "the cancelled open bound an address");
    }

    #[test]
    fn states_the_release_when_a_replacement_open_is_cancelled() {
        let bound = Rc::new(Cell::new(0));
        let (server, _binds) = SlowServer::new(1, &bound);
        let commands = stream::iter([
            HttpServerCommand::Open(HttpServerSetup::new(bind(3000), "binds")),
            HttpServerCommand::Open(HttpServerSetup::new(bind(3001), "never binds")),
            HttpServerCommand::Close,
        ]);

        let events = block_on(server.lifecycle(commands).collect::<Vec<_>>());

        // The first address was released before the second open was cancelled, and releasing it is
        // a transition whoever reads them is told about.
        assert_eq!(events, [HttpServerEvent::Opened(bind(3000)), HttpServerEvent::Closed(bind(3000))]);
        assert_eq!(bound.get(), 1, "the cancelled open bound an address");
    }

    #[test]
    fn retains_a_server_after_its_close_fails() {
        let commands = stream::iter([
            HttpServerCommand::Open(HttpServerSetup::new(bind(3000), "first")),
            HttpServerCommand::Close,
            HttpServerCommand::Open(HttpServerSetup::new(bind(3001), "second")),
        ]);
        let server = FailingServer { fails_once: true };
        let events = block_on(server.lifecycle(commands).collect::<Vec<_>>());

        assert_eq!(
            events,
            [
                HttpServerEvent::Opened(bind(3000)),
                HttpServerEvent::Failed { bind: bind(3000), error: TestError },
                HttpServerEvent::Replaced { closed: bind(3000), opened: bind(3001) },
            ]
        );
    }
}
