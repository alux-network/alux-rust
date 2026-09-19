# alux-http-conformance

`alux-http-conformance` states one [`alux-http`](https://docs.rs/alux-http) surface and the scenario
every interpretation of it must satisfy.

Two interpretations can be compared only when they compile the same declaration and answer the same
requests. This crate provides that shared evidence: one declaration, compiled by the interpretation
under test, and one set of exchanges describing the required methods, paths, media types, and
statuses. The exchanges know nothing about the domain or any framework.

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_conformance::{Shop, ShopApiExt, expect};
use alux_http_direct::DirectHandlerImpl;

let api = DirectHandlerImpl::new(Shop);
let surface = api.compile_http(api.shop_api::<Shop>());

expect(&surface).await.unwrap();
```

An interpretation that answers with a framework's own response states an adapter for
[`AnswerAlg`](AnswerAlg), which is the only place a framework is named.


## Opening and closing

The other half is for interpretations that serve rather than answer: what opening and closing one bound address must do. `ExpectLifecycleExt` states it against the real thing, over a socket. A server is opened, a client uses it, the server is closed, and another is opened at the same address. Opening the second one is the assertion, because it can only succeed if closing really released the address.

Three scenarios, which differ in what the close finds. `expect_reopening` closes with nothing in flight. `expect_reopening_while_a_connection_is_held` closes while a caller is still waiting on `GET /slow`, which takes longer to answer than any interpretation waits before closing anyway, so the connection outlives the close instead of the close waiting it out. `expect_answering_what_is_in_flight` closes while a caller is waiting on `GET /pause`, which takes less than any drain, so the close has time to finish it and the answer must reach that caller. `expect_ending_what_outlives_the_drain` closes while a caller is waiting on `GET /slow`, which takes more, so the caller must be left without an answer rather than handed one by a server that no longer exists.

The last two are the same statement from both sides, and each is run with the close and the caller's read racing rather than sequenced, because sequencing them would state nothing: by the time a close has returned, whatever was going to happen already has.

Each scenario runs on a local task set of its own, because several interpretations serve from tasks that are not `Send`. That is the interpretation's business rather than the caller's, so a test states only the server and the route it serves:

```rust ignore
use alux_http::HttpProgramExt;
use alux_http_conformance::{ExpectLifecycleExt, LifecycleApiExt, Shop};
use alux_http_salvo::{SalvoHandlerImpl, SalvoRoute, SalvoServer};

fn route() -> SalvoRoute {
    let api = SalvoHandlerImpl::new(Shop);

    api.compile_http(api.lifecycle_api::<Shop>())
}

#[tokio::test]
async fn reopens_the_address_it_closed() {
    SalvoServer.expect_reopening(route).await.unwrap();
}

#[tokio::test]
async fn reopens_the_address_it_closed_while_a_connection_is_held() {
    SalvoServer.expect_reopening_while_a_connection_is_held(route).await.unwrap();
}

#[tokio::test]
async fn answers_a_request_in_flight_when_it_closes() {
    SalvoServer.expect_answering_what_is_in_flight(route).await.unwrap();
}

#[tokio::test]
async fn ends_a_request_that_outlives_the_drain_when_it_closes() {
    SalvoServer.expect_ending_what_outlives_the_drain(route).await.unwrap();
}
```

`expect_closing` and `expect_ending` are the two the scenarios build on, and either can be used alone: each acts on one open server and states that the call returned inside `LIMIT`. Which one a scenario reaches for is the point of the split, and `expect_ending_what_outlives_the_drain` needs the second: closing alone releases the address and leaves the drain running, so nothing would end the request.

`measure_closing` is the same shape as the last two scenarios, reporting what it saw instead of holding the server to it. It answers a `Measured`, which reads as a row:

```text
alux-http-salvo | /pause | close returned 1.00s | answered 200 at 1.00s     | reusable true
alux-http-salvo | /slow  | close returned 5.00s | ended unanswered at 5.00s | reusable true
```

The `http-providers` example depends on every interpretation, so it gathers all seven in one place, in `benches/closing.rs`. It is a benchmark rather than a test because it measures rather than states.
