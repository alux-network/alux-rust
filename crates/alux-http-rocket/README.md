# alux-http-rocket

`alux-http-rocket` interprets an [`alux-http`](https://docs.rs/alux-http) program as executable
[Rocket](https://docs.rs/rocket) routes.

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_rocket::RocketHandlerImpl;

let api = RocketHandlerImpl::new(App::new());
let rocket = api.compile_http(api.status_api::<App>()).mount(rocket::build());

rocket.launch().await?;
```

Rocket requires every mounted route to have a method. An endpoint without a method therefore cannot
be mounted. Rocket responses also borrow the request, so this interpreter builds the response value
at the route boundary and returns it to Rocket.

## Accepting

Rocket binds its own listener inside `launch` and offers no way to hand it one, and no entry point that takes a single connection. The listener, the accept loop and the runtime are all internal, so nothing about serving can be driven from outside.

That shapes both ends of the lifecycle here. Starting learns that the address is serving from an `on_liftoff` fairing, because Rocket is the only thing that knows when it bound. Stopping goes through `Shutdown::notify`, because there is no listener to drop. Rocket also warns that it runs inside a custom runtime, which costs only its last-resort termination of a runaway task: its graceful and forced connection shutdown still apply.

Most of that is inherited. Rocket 0.5 builds on hyper 0.14, whose `Server` takes the listener and spawns each connection itself, and Rocket drives it through [`Server::builder`](https://github.com/rwf2/Rocket/blob/v0.5.1/core/lib/src/server.rs#L509). hyper 0.14 did expose a per-connection entry point in [`Http::serve_connection`](https://github.com/hyperium/hyper/blob/v0.14.32/src/server/conn.rs#L665), so the closed loop was a choice rather than something forced, but once `Server` is the thing being driven there is no connection to reach and no task to wait on. hyper 1 split those apart again, which is what lets [`alux-http-hyper`](https://docs.rs/alux-http-hyper) own the loop and hold every connection it accepts.

## Signals

This interpreter sets `ctrlc: false` and an empty `signals` set in Rocket's `ShutdownConfig`, because whoever opened the server is the one who decides when it closes.

Left at its defaults, Rocket installs process-wide handlers and answers Ctrl-C and SIGTERM itself. An application built on `alux-http` never sees either, so its own shutdown never runs and `kill` or `pkill` appears to do nothing at all. Ending a server is what `HttpServerAlg::close` and `HttpServerAlg::end` are for, and a signal is the application's to interpret.

## Logging

Rocket writes a launch banner and a line per request to stdout, at `log_level` `normal` in debug and `critical` in release. This crate sets it to `off`: what an application says is the application's, and a benchmark reading its own output has nothing else in it.

## Closing

Closing and ending are one call here, because Rocket's shutdown releases the address and drains connections together and cannot be asked for only the first. Both are `Shutdown::notify` followed by waiting for `launch` to return. How long that takes is set by two configuration values that read as one budget and are not:

`grace` is the drain. While it runs, each connection keeps serving normally, so a request in flight is still being answered. When it expires the connection is shut down whatever it was doing. This crate sets 5 seconds, the same drain the other interpretations state.

`mercy` begins where grace ends, and is not more time for the request. It bounds the orderly close of the socket, flushing what is pending and closing the write half, before the socket is dropped outright. This crate sets 0, because a connection whose request has just been abandoned has nothing left to flush.

Closing returns later than both, at `grace + mercy + 1`. The extra second is Rocket's own, and its [comment](https://github.com/rwf2/Rocket/blob/v0.5.1/core/lib/src/server.rs#L565) says why: hyper's server future resolves before the responses it started have finished, so Rocket cannot observe when its connections are done. It waits the periods out, adds a buffer, and then checks whether every task has dropped its reference to the server. A failed check is the `Shutdown failed: outstanding background I/O` warning.
