//! Compiles one declared surface into axum, and holds it to what the description says it is.
//!
//! This is the second executable interpreter of the same specification. Nothing here restates a
//! route, a method, a path spelling, an input role, or an output kind: the declaration is the one a
//! Poem service would be compiled from, and axum answers it.

#![allow(async_fn_in_trait)]

use alux_ext::ext;
use alux_http::{
    BytesOutAlg, EmptyOutAlg, HtmlOutAlg, HttpApiAlg, HttpProgramExt, JsonOutAlg, RedirectOutAlg, ResultOutAlg,
    StatusOutAlg, TextOutAlg, http,
};
use alux_http_axum::AxumHandlerImpl;
use alux_http_text::TextHandlerImpl;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use axum::response::Response;
use core::future::Future;
use serde::Deserialize;
use std::io::{Error as IoError, ErrorKind};
use tower::ServiceExt;

/// Carries one reading sent by a caller.
#[derive(Debug, Deserialize)]
struct Amount {
    value: u32,
}

/// Keeps whatever readings the domain has been told about.
trait ShopAlg {
    /// Returns the reading recorded for `id`, or why it could not be read.
    fn item(&self, id: u32) -> impl Future<Output = Result<u32, IoError>> + Send;
    /// Returns every reading.
    fn items(&self) -> impl Future<Output = Vec<u32>> + Send;
    /// Records `value` and returns it.
    fn add(&self, value: u32) -> impl Future<Output = u32> + Send;
    /// Notes what was sent and says what it made of it.
    fn note(&self, note: String) -> impl Future<Output = String> + Send;
    /// Forgets every reading.
    fn clear(&self) -> impl Future<Output = ()> + Send;
    /// Returns where the readings actually live.
    fn home(&self) -> impl Future<Output = String> + Send;
    /// Returns the readings as a page.
    fn page(&self) -> impl Future<Output = String> + Send;
    /// Returns the readings as they are stored.
    fn stored(&self) -> impl Future<Output = Vec<u8>> + Send;
}

#[ext(name = ShopOperationExt, defunc)]
impl<This> This
where
    This: ShopAlg,
{
    /// Returns one identified reading, or why it could not be read.
    async fn shop_item(&self, id: u32) -> Result<u32, IoError> {
        self.item(id).await
    }

    /// Returns every reading.
    async fn shop_items(&self) -> Vec<u32> {
        self.items().await
    }

    /// Records one reading and returns it.
    async fn shop_add(&self, value: u32) -> u32 {
        self.add(value).await
    }

    /// Records one reading sent as a form and returns it.
    async fn shop_fill(&self, amount: Amount) -> u32 {
        self.add(amount.value).await
    }

    /// Notes what was sent and says what it made of it.
    async fn shop_note(&self, note: String) -> String {
        self.note(note).await
    }

    /// Forgets every reading.
    async fn shop_clear(&self) {
        self.clear().await;
    }

    /// Returns where the readings actually live.
    async fn shop_home(&self) -> String {
        self.home().await
    }

    /// Returns the readings as a page.
    async fn shop_page(&self) -> String {
        self.page().await
    }

    /// Returns the readings as they are stored.
    async fn shop_stored(&self) -> Vec<u8> {
        self.stored().await
    }
}

#[ext(name = ShopApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg
        + JsonOutAlg
        + TextOutAlg
        + HtmlOutAlg
        + BytesOutAlg
        + EmptyOutAlg
        + RedirectOutAlg
        + StatusOutAlg
        + ResultOutAlg,
{
    /// Declares the whole surface: every method, every body role, every output kind it uses.
    fn shop_api<Alg>(&self)
    where
        Alg: ShopAlg,
    {
        self.routes()
            // One identified reading, its id taken from the path in Poem's spelling.
            .get("/item/:id", self.op(Alg::shop_item).path::<u32>().json().result())
            // Every reading.
            .get("/items", self.op(Alg::shop_items).json())
            // A recording sent as a document, which creates something and says so.
            .post("/items", self.op(Alg::shop_add).body::<u32>().json().status::<201>())
            // The same recording, sent as a form.
            .put("/items", self.op(Alg::shop_fill).form::<Amount>().json())
            // A note, taken exactly as it arrived.
            .patch("/items", self.op(Alg::shop_note).raw_body::<String>().text())
            // A removal, which answers with nothing at all.
            .delete("/items", self.op(Alg::shop_clear).empty())
            // Where the readings actually live.
            .get("/home", self.op(Alg::shop_home).redirect())
            // The readings as a page.
            .get("/page", self.op(Alg::shop_page).html())
            // The readings as they are stored.
            .get("/stored", self.op(Alg::shop_stored).bytes())
    }
}

struct Shop;

impl ShopAlg for Shop {
    async fn item(&self, id: u32) -> Result<u32, IoError> {
        match id {
            1 => Ok(7),
            _ => Err(IoError::new(ErrorKind::NotFound, "no such reading")),
        }
    }

    async fn items(&self) -> Vec<u32> {
        vec![7]
    }

    async fn add(&self, value: u32) -> u32 {
        value
    }

    async fn note(&self, note: String) -> String {
        format!("noted {note}")
    }

    async fn clear(&self) {}

    async fn home(&self) -> String {
        "/items".to_owned()
    }

    async fn page(&self) -> String {
        "<p>7</p>".to_owned()
    }

    async fn stored(&self) -> Vec<u8> {
        b"seven".to_vec()
    }
}

fn shop() -> Router {
    let api = AxumHandlerImpl::new(Shop);

    api.compile_http(api.shop_api::<Shop>()).into_axum()
}

async fn call(request: Request<Body>) -> Response {
    shop().oneshot(request).await.unwrap()
}

async fn read(response: Response) -> String {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();

    String::from_utf8(body.to_vec()).unwrap()
}

fn sent(method: Method, path: &str, content_type: &str, body: &'static str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(body))
        .unwrap()
}

fn asked(method: Method, path: &str) -> Request<Body> {
    Request::builder().method(method).uri(path).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn answers_every_method_the_declaration_states() {
    let items = call(asked(Method::GET, "/items")).await;
    assert!(items.status().is_success());
    assert_eq!(read(items).await, "[7]");

    let created = call(sent(Method::POST, "/items", "application/json", "7")).await;
    assert_eq!(created.status(), StatusCode::CREATED);
    assert_eq!(read(created).await, "7");

    let filled = call(sent(Method::PUT, "/items", "application/x-www-form-urlencoded", "value=7")).await;
    assert!(filled.status().is_success());
    assert_eq!(read(filled).await, "7");

    let noted = call(sent(Method::PATCH, "/items", "text/plain", "seven")).await;
    assert!(noted.status().is_success());
    assert_eq!(read(noted).await, "noted seven");

    let cleared = call(asked(Method::DELETE, "/items")).await;
    assert_eq!(cleared.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn reads_a_path_written_in_another_router_s_spelling() {
    // The declaration writes `:id`, which axum's router never reads; the program states a segment.
    let found = call(asked(Method::GET, "/item/1")).await;
    assert!(found.status().is_success());
    assert_eq!(read(found).await, "7");

    let missing = call(asked(Method::GET, "/item/9")).await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(read(missing).await, "no such reading");
}

#[tokio::test]
async fn answers_with_what_each_output_kind_states() {
    let home = call(asked(Method::GET, "/home")).await;
    assert_eq!(home.status(), StatusCode::SEE_OTHER);
    assert_eq!(home.headers().get(header::LOCATION).unwrap(), "/items");

    let page = call(asked(Method::GET, "/page")).await;
    assert_eq!(page.headers().get(header::CONTENT_TYPE).unwrap(), "text/html; charset=utf-8");
    assert_eq!(read(page).await, "<p>7</p>");

    let stored = call(asked(Method::GET, "/stored")).await;
    assert_eq!(read(stored).await, "seven");
}

#[test]
fn agrees_with_the_text_interpretation_on_the_same_program() {
    let axum = AxumHandlerImpl::new(Shop);
    let text = TextHandlerImpl;

    let described = text.compile_http(text.shop_api::<Shop>());
    let compiled = axum.compile_http(axum.shop_api::<Shop>());

    assert_eq!(compiled.labels(), described.labels());
    assert_eq!(
        compiled.labels(),
        [
            "GET /item/{id}",
            "GET /items",
            "POST /items",
            "PUT /items",
            "PATCH /items",
            "DELETE /items",
            "GET /home",
            "GET /page",
            "GET /stored",
        ]
    );
}
