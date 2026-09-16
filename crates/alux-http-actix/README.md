# alux-http-actix

`alux-http-actix` interprets an [`alux-http`](https://docs.rs/alux-http) program as executable
[actix-web](https://docs.rs/actix-web) routes.

```rust ignore
use actix_web::{App, HttpServer};
use alux_http::HttpProgramExt;
use alux_http_actix::ActixHandlerImpl;

let api = ActixHandlerImpl::new(App::new());
let configure = api.compile_http(api.status_api::<App>()).into_actix();

HttpServer::new(move || App::new().configure(configure.clone())).bind(("0.0.0.0", 3000))?.run().await?;
```

actix-web creates its services again for every worker, and an `actix_web::Route` cannot be cloned or
reused. The interpreter therefore keeps the logic for constructing each endpoint and rebuilds the
routes for every worker. Shared state reaches a handler as the semantic context from the program,
not as `actix_web::Data`, so framework types stay out of the specification.
