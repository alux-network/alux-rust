# HTTP providers

This project declares one HTTP API and compiles it with each HTTP interpreter. The portable
specification is in [`src/lib.rs`](src/lib.rs) and [`src/spec.rs`](src/spec.rs); the single
[`src/main.rs`](src/main.rs) owns the application implementation. Each framework crate owns the
open and close interpretation of the route it compiled, and how much of that it can own is set out
under [How each provider closes](#how-each-provider-closes).

The example runs eight servers from one `main` function. Seven are fixed interpreter instances,
each serving the declaration on its own port. The eighth server is the dynamic front door on port
3000, served by whichever provider was switched to last:

```text
port 3000 -> locked provider object -> hyper at startup
                         |
                         +-- axum, warp, poem, salvo, actix, or rocket after a switch
```

At startup, a local control loop sends a hyper setup to `alux-http::server`. The example compiles its
declaration with `DirectHandlerImpl` and hands the resulting route to `HyperServer`; the other
providers do the same with their own handler and server values. A switch requested through a fixed
provider becomes another setup in the same command stream. The lifecycle manager closes the open
handle, opens the selected framework server, and reports the transition only after the
replacement has bound the address. Thus the same port is served by a different framework after the
switch; the fixed servers on their own ports are not restarted. Switching from the dynamic provider
itself is a separate case because it would be asking the server handling the request to unload
itself.

The running instances are, ordered the way the comparison below orders them:

| Provider | Address |
| --- | --- |
| _\<dynamic\>_ | `http://127.0.0.1:3000/api` |
| hyper | `http://127.0.0.1:3001/api` |
| axum | `http://127.0.0.1:3002/api` |
| warp | `http://127.0.0.1:3003/api` |
| Poem | `http://127.0.0.1:3004/api` |
| Salvo | `http://127.0.0.1:3005/api` |
| Actix Web | `http://127.0.0.1:3006/api` |
| Rocket | `http://127.0.0.1:3007/api` |

Run the whole example with one main:

```sh
cargo run -p http-providers
```

Query every running provider:

```sh
for port in {3000..3007}; do curl "http://127.0.0.1:$port/api"; echo; done
```

Read the documentation interpretations from any running provider:

```sh
curl http://127.0.0.1:3000/openapi-spec
curl http://127.0.0.1:3000/ts-spec
```

Switch the dynamic front door through a fixed provider, then query it again:

```sh
curl --request POST http://127.0.0.1:3001/fw --header 'content-type: application/json' --data '"poem"'
curl http://127.0.0.1:3000/api
```

`GET /api` reports the provider currently serving that port. Send `POST /fw` to a fixed provider
with a JSON provider name such as `"poem"`; it replaces the provider serving port 3000 and returns
the dynamic front-door address, `http://127.0.0.1:3000/api`. `GET /openapi-spec` and `GET /ts-spec`
read the same declaration as an OpenAPI document and a TypeScript client. The switch endpoint,
provider API, and documentation outputs come from that one declaration.

## How each provider closes

Every provider closes gracefully: it stops accepting, releases the address, answers what it is already serving, and ends whatever is still serving when its drain runs out. They reach that in two different ways.

All seven answer a request that finishes inside the drain and end one that does not. That is the contract `HttpServerAlg` states, and it is what `tests/lifecycle.rs` holds each of them to.

The contract has two halves, and they are separate calls. `close` resolves once the address is released, and says nothing about requests still being served. `end` resolves once those are finished too, answered or ended on the drain. A caller replacing one server with another at the same address wants the first; a caller shutting down wants the second.

One caller, two requests on one connection. It asks `GET /items` first and reads the answer, which is what states that this connection was accepted and being served rather than left waiting in the backlog. It then asks a second time and does not read, so the server is still producing that answer when the close begins. That second request is the one the table reports, and it is either `GET /pause`, which answers 1s after the close begins and so inside the 5s drain, or `GET /slow`, which would answer 30s after and so nowhere near it.

The close and the caller's read then run concurrently rather than one after the other. Sequencing them would state nothing: by the time a close has returned, whatever was going to happen to the request already has.

Every time below is counted from the moment closing started, and both endpoints are built to answer a stated time after that rather than after the request, so a measurement can be read against the drain as it stands. Each figure is the median of three samples.

**Accept loop** says how much of the accept loop a framework leaves under the caller's control, which is what decides the cost of supporting it at all. It is read off each framework's API rather than measured; everything to its right is timed.

What happens, in order, in each case:

```text
/pause, answering 1s after the close begins, inside the 5s drain

  0s                    1s                                    5s
  |---------------------|-------------------------------------|
  close begins          ✅ answered, head and body            drain would expire,
  close returns 🟢      close returns 🟡 🟠 🔴                nothing left to end
                        end returns

/slow, answering 30s after the close begins, well past the drain

  0s                                        5s                          30s
  |-----------------------------------------|---------------------------|
  close begins                              drain expires               would have answered
  close returns 🟢                          🚫 request ended
                                            close returns 🟡 🟠 🔴
                                            end returns
```

So ✅ is an answer the caller can use, and 🚫 is the connection closing with nothing on it. A close landing between those two, leaving a head and body partially returned is not tested.

| Provider | Accept loop | `/pause` ✅ at 1s | | `/slow` 🚫 at 5s | |
| --- | --- | --- | --- | --- | --- |
| | | `close` | `end` | `close` | `end` |
| [hyper](https://github.com/hyperium/hyper) | 🟢 **composable** | **66 µs** | 0.9999s | **81 µs** | 5.0009s |
| [axum](https://github.com/tokio-rs/axum) | 🟢 **composable** | **69 µs** | 1.0003s | **75 µs** | 5.0013s |
| [warp](https://github.com/seanmonstar/warp) | 🟢 **composable** | **76 µs** | 1.0002s | **73 µs** | 5.0013s |
| [Poem](https://github.com/poem-web/poem) | 🟡 bindable | 0.9997s | 0.9999s | 5.0016s | 5.0015s |
| [Salvo](https://github.com/salvo-rs/salvo) | 🟡 bindable | 1.0003s | 1.0004s | 5.0019s | 5.0016s |
| [Actix Web](https://github.com/actix/actix-web) | 🟠 opaque | 1.0016s | 1.0013s | 5.0066s | 5.0057s |
| [Rocket](https://github.com/rwf2/Rocket) | 🔴 sealed | 1.0003s | 1.0003s | 6.0009s | 6.0011s |

A framework has a **closed accept loop** when it owns the loop that takes new connections off the listening socket and exposes no way to hand it a single connection. It can be started and asked to stop, and that is all: there is no function to call when the socket becomes readable, so the loop can only ever be its own, on a task the caller never created. How much a closed loop takes with it is the ladder above.

- **🟢 composable.** The framework states a service, a function from a request to an answer, and nothing else. The loop is written outside it, so it can be driven by any runtime and composed with anything. hyper is the base case: [`HyperConnections`](../../crates/alux-http-hyper/src/connections.rs) binds, accepts and spawns a task per connection, and axum and warp reuse it unchanged because both become a tower service. The listener is held here for its whole life, so releasing the address is dropping it and the moment it comes free is observable. That is the sub-millisecond `close`, and it is why `close` and `end` are separate calls at all. Supporting another framework of this kind costs a route compiler and nothing more.
- **🟡 bindable.** The loop is closed, but the framework accepts a listener the caller binds. Binding is therefore the caller's: a port already taken fails here, before the framework is involved. Releasing it is not. The framework holds the listener from then on and drops it somewhere inside its own shutdown, so the only thing to await is that shutdown finishing, which is also when the drain is over. Hence one call: `close` and `end` resolve together.
- **🟠 opaque.** The framework accepts a listener the caller binds, then serves it from runtimes the caller cannot reach. No handle here touches a connection, so beyond that first bind the whole lifecycle goes through the framework's own, and `close` and `end` resolve together for the same reason.
- **🔴 sealed.** The framework binds its own listener and exposes neither it nor the loop. Even "it is bound now" has to arrive through a callback, and cancelling an open before it binds needs a guard to shut down what was already launched.

Every row reaches the two outcomes the header records: the request inside the drain is answered, the one past it is ended, with `end` returning alongside it. What the ladder decides is `close`. Where the loop is composable it returns in microseconds, because releasing the address is dropping the listener and nothing else has to happen first. Where the loop is closed, `close` and `end` are one call: the framework's shutdown does both, and there is no way to ask it for only the address.

Rocket is the one row where a number needs explaining. It ends the request past the drain a second after the drain expires, because it waits [`grace + mercy + 1`](https://github.com/rwf2/Rocket/blob/v0.5.1/core/lib/src/server.rs#L565) and only the first two can be set, so the trailing second is the hardcoded one. Actix Web's own [one second poll](https://github.com/actix/actix-net/blob/server-v2.9.5/actix-server/src/worker.rs#L38), which is how a worker checks whether its connections are done, is not visible here because both cases end on a second.

Ctrl-C and `kill` both stop the example, which takes one fix. Rocket and Actix Web install signal handlers by default and answer those signals themselves, so `main` would keep waiting for a signal it never gets. [`alux-http-rocket`](../../crates/alux-http-rocket/README.md#signals) and [`alux-http-actix`](../../crates/alux-http-actix/README.md#signals) turn that off.

Every provider is a dependency of this project, so one command runs the whole table:

```sh
cargo bench -p http-providers --bench closing
```

A composable `close` is CPU-bound and measured one case at a time; every other case here blocks until a drain expires, so the seven run at the same time and cost what the longest of them costs. That split is `BenchAlg::group` against `BenchParallelAlg::together`, stated by [`alux-bench`](../../alux-bench/README.md) and interpreted by [`alux-bench-direct`](../../crates/alux-bench-direct/README.md).

That measures rather than states, which is why it is only run when asked for. What each provider *must* do is stated by `tests/lifecycle.rs` in each provider crate, and those run on every `cargo test`.

Each drain is 5 seconds, set by the crate serving it: the argument to `HyperConnections::serve` for hyper, axum and warp, the timeout passed to Poem's `run_with_graceful_shutdown` and Salvo's `stop_graceful`, Actix Web's `shutdown_timeout`, and Rocket's `grace`. Setting all seven to one number is what makes the measurement a comparison, and finding where to set it is most of the work of supporting a framework at all.

Where the loop is composable, the drain is run here. `HyperConnections` keeps the handle of every connection it spawns, so closing aborts the accepting task, tells each connection to shut down gracefully, and ends whatever is still serving when the time runs out. The frameworks contribute only the service it calls, which is axum's [`Router`](https://github.com/tokio-rs/axum/blob/main/axum/src/routing/mod.rs), [`warp::service`](https://github.com/seanmonstar/warp/blob/master/src/service.rs), or a hyper service directly, driven over the connection by hyper-util's [`auto::Builder`](https://github.com/hyperium/hyper-util/blob/master/src/server/conn/auto/mod.rs).

Where it is closed, the drain is a number handed to the framework's own shutdown, and each states it in its own vocabulary. Poem takes a timeout beside its [shutdown signal](https://github.com/poem-web/poem/blob/master/poem/src/server.rs#L242), Salvo breaks out of its accept loop and then waits on [`alive_connections`](https://github.com/salvo-rs/salvo/blob/v0.96.0/crates/core/src/server.rs#L476), Rocket's [`Shutdown::notify`](https://github.com/rwf2/Rocket/blob/v0.5.1/core/lib/src/shutdown.rs#L89) trips a token its server watches, and Actix Web's [`ServerCommand::Stop`](https://github.com/actix/actix-net/blob/server-v2.9.5/actix-server/src/server.rs#L261) fans the request out to every worker, waiting for them only when it is graceful.

Dropping an open server without closing it is the forceful version of the same thing, and it is what ends the fixed providers when the example stops. Each `Drop` aborts the task, and the three that need it also signal their handle first, because a `JoinHandle` that is merely dropped detaches its task rather than cancelling it. A drop cannot wait, so it frees the address without promising when.

## How long one handover takes

One handover is `close` then `open` on the same address. Cells are handovers per second, with the time for one in brackets. Three cases:

- **no request**: `close` then `open`.
- **req OK**: `GET /items`, read the 200, then `close` and `open`.
- **req OK 30 sec**: `GET /slow` sent and not read, then `close` and `open`. It responds 30s after the close starts, past the 5s drain.

| Provider | Accept loop | no request | req OK | req OK 30 sec |
| --- | --- | --- | --- | --- |
| [hyper](https://github.com/hyperium/hyper) | 🟢 **composable** | **213k /s (4.71 µs)** | **26.9k /s (37.2 µs)** | **9.16k /s (109 µs)** |
| [axum](https://github.com/tokio-rs/axum) | 🟢 **composable** | **136k /s (7.35 µs)** | **24.6k /s (40.6 µs)** | **7.50k /s (133 µs)** |
| [warp](https://github.com/seanmonstar/warp) | 🟢 **composable** | **202k /s (4.95 µs)** | **28.6k /s (35.0 µs)** | **8.78k /s (114 µs)** |
| [Poem](https://github.com/poem-web/poem) | 🟡 bindable | 33.3k /s (30.0 µs) | 15.1k /s (66.0 µs) | 0.20 /s (5.0017 s) |
| [Salvo](https://github.com/salvo-rs/salvo) | 🟡 bindable | 145k /s (6.92 µs) | 29.0k /s (34.4 µs) | 0.20 /s (5.0017 s) |
| [Actix Web](https://github.com/actix/actix-web) | 🟠 opaque | 753 /s (1.33 ms) | 673 /s (1.49 ms) | 0.20 /s (5.0070 s) |
| [Rocket](https://github.com/rwf2/Rocket) | 🔴 sealed | 21.7k /s (46.0 µs) | 8.22k /s (122 µs) | 0.17 /s (6.0018 s) |

The **req OK 30 sec** column is what `close` and `end` being separate calls buys. A composable loop's `close` returns without waiting for the drain, so a request still being served costs a handover rather than a drain. The other four have one call that waits, so there a handover is a drain.

Actix Web is over 1 ms in every column, because each `open` starts a worker per core and each `close` stops them. Salvo is in the range of the composable three with no request, and pays the drain only when a request is being served.

Run it with:

```sh
cargo bench -p http-providers --bench handover
```

Three samples per case, each spending a tenth of a second: a handover of a free address takes microseconds, so a sample runs thousands of them, while one that blocks on the drain takes longer than the spend, so a sample runs one. The first two groups are CPU-bound and measured one case at a time; the third blocks until the drain expires, so its seven run at the same time.

## Switching under load

```sh
cargo bench -p http-providers --bench switching
```

Eight clients send `POST /fw` at once, each naming the next provider in turn, so port 3000 is closed
and opened again thousands of times over ten samples of fifteen seconds. What the numbers say is how
fast one provider hands the port to the next; what the run says is that none of those handovers fails
with the address still in use. `close` resolves only once the address is released, so the next `open`
finds it free.

The front door keeps serving throughout. Ask it while the benchmark runs:

```sh
curl http://127.0.0.1:3000/api
```

and a request landing between two switches is answered by whichever provider holds the port at that
moment.
