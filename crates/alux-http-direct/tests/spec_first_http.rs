//! Compiles one declared surface into a surface that answers for itself, with no framework at all.
//!
//! Every other interpretation hands routing to somebody else. This one routes, so it is the one
//! that can be held to what routing means: which endpoint a request reaches, what a path binds, and
//! what is answered when nothing is declared there.

use alux_ext::ext;
use alux_http::{
    BytesOutAlg, EmptyOutAlg, HtmlOutAlg, HttpApiAlg, HttpMethod, HttpProgramExt, HttpStatus, JsonOutAlg,
    RedirectOutAlg, ResultOutAlg, StatusOutAlg, TextOutAlg, http,
};
use alux_http_direct::{DirectHandlerImpl, DirectRequest, DirectResponse, DirectRoute};
use alux_http_text::TextHandlerImpl;
use core::future::Future;
use serde::Deserialize;
use std::io::{Error as IoError, ErrorKind};

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
    /// Returns whatever is filed under a whole trailing path.
    fn filed(&self, under: String) -> impl Future<Output = String> + Send;
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

    /// Returns whatever is filed under a whole trailing path.
    async fn shop_filed(&self, under: String) -> String {
        self.filed(under).await
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
            // Whatever is filed, however deep the path goes.
            .get("/filed/{*under}", self.op(Alg::shop_filed).path::<String>().text())
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

    async fn filed(&self, under: String) -> String {
        format!("filed under {under}")
    }
}

fn shop() -> DirectRoute {
    let api = DirectHandlerImpl::new(Shop);

    api.compile_http(api.shop_api::<Shop>())
}

async fn asked(method: HttpMethod, path: &str) -> DirectResponse {
    shop().answer(DirectRequest::new(method, path)).await
}

async fn sent(method: HttpMethod, path: &str, content_type: &str, body: &str) -> DirectResponse {
    let request = DirectRequest::new(method, path).with_header("content-type", content_type).with_body(body);

    shop().answer(request).await
}

#[tokio::test]
async fn answers_every_method_the_declaration_states() {
    let items = asked(HttpMethod::Get, "/items").await;
    assert_eq!(items.status(), HttpStatus::OK);
    assert_eq!(items.text(), "[7]");

    let created = sent(HttpMethod::Post, "/items", "application/json", "7").await;
    assert_eq!(created.status(), HttpStatus::new(201));
    assert_eq!(created.text(), "7");

    let filled = sent(HttpMethod::Put, "/items", "application/x-www-form-urlencoded", "value=7").await;
    assert_eq!(filled.status(), HttpStatus::OK);
    assert_eq!(filled.text(), "7");

    let noted = sent(HttpMethod::Patch, "/items", "text/plain", "seven").await;
    assert_eq!(noted.text(), "noted seven");

    let cleared = asked(HttpMethod::Delete, "/items").await;
    assert_eq!(cleared.status(), HttpStatus::NO_CONTENT);
    assert!(cleared.body().is_empty());
}

#[tokio::test]
async fn binds_what_a_path_states_it_binds() {
    let found = asked(HttpMethod::Get, "/item/1").await;
    assert_eq!(found.status(), HttpStatus::OK);
    assert_eq!(found.text(), "7");

    let missing = asked(HttpMethod::Get, "/item/9").await;
    assert_eq!(missing.status(), HttpStatus::NOT_FOUND);
    assert_eq!(missing.text(), "no such reading");

    // A tail binds every segment that is left, however many there are.
    let filed = asked(HttpMethod::Get, "/filed/two/levels/down").await;
    assert_eq!(filed.text(), "filed under two/levels/down");
}

#[tokio::test]
async fn answers_for_itself_where_no_endpoint_is_declared() {
    let nowhere = asked(HttpMethod::Get, "/nothing").await;
    assert_eq!(nowhere.status(), HttpStatus::NOT_FOUND);

    // A path that is declared, under methods that do not include this one, is not nothing.
    let wrong_method = asked(HttpMethod::Head, "/items").await;
    assert_eq!(wrong_method.status(), HttpStatus::METHOD_NOT_ALLOWED);

    // A path that states more than the declaration reads is a different path.
    let deeper = asked(HttpMethod::Get, "/items/7").await;
    assert_eq!(deeper.status(), HttpStatus::NOT_FOUND);

    // An argument that cannot be read from the role it was declared under is the caller's mistake.
    let unreadable = asked(HttpMethod::Get, "/item/seven").await;
    assert_eq!(unreadable.status(), HttpStatus::BAD_REQUEST);
}

#[tokio::test]
async fn answers_with_what_each_output_kind_states() {
    let home = asked(HttpMethod::Get, "/home").await;
    assert_eq!(home.status(), HttpStatus::SEE_OTHER);
    assert_eq!(home.header("location"), Some("/items"));

    let page = asked(HttpMethod::Get, "/page").await;
    assert_eq!(page.header("content-type"), Some("text/html; charset=utf-8"));
    assert_eq!(page.text(), "<p>7</p>");

    let stored = asked(HttpMethod::Get, "/stored").await;
    assert_eq!(stored.header("content-type"), Some("application/octet-stream"));
    assert_eq!(stored.body(), b"seven");
}

#[test]
fn agrees_with_the_text_interpretation_on_the_same_program() {
    let text = TextHandlerImpl;
    let described = text.compile_http(text.shop_api::<Shop>());

    assert_eq!(shop().labels(), described.labels());
    assert_eq!(
        shop().labels(),
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
            "GET /filed/{*under}",
        ]
    );
}
