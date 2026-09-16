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
