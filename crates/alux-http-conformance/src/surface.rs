//! One declared surface, and the domain it reads.
//!
//! This is user code, written once. Every interpretation compiles this same declaration, which is
//! what makes their agreement evidence rather than coincidence.

use alux_ext::ext;
use alux_http::{
    BytesOutAlg, CacheControl, ChunksAlg, ChunksExt, EmptyOutAlg, FromPartsAlg, HeaderOutAlg, HtmlOutAlg, HttpApiAlg,
    JsonOutAlg, NamedValuesAlg, PartAlg, RedirectOutAlg, ResultOutAlg, StatusOutAlg, StreamOutAlg, TextOutAlg, http,
};
use alux_shape::Shape;
use core::convert::Infallible;
use core::fmt::Display;
use core::future::Future;
use serde::{Deserialize, Serialize};
use std::io::{Error as IoError, ErrorKind};

/// Who a caller says they are, sent as cookies.
#[derive(Debug, Serialize, Deserialize, Shape)]
pub struct Session {
    /// Which session the caller is in.
    pub session: String,
}

/// What a caller said about themselves in the headers they sent.
#[derive(Debug, Serialize, Deserialize, Shape)]
pub struct Agent {
    /// What the caller says they are.
    pub user_agent: String,
}

impl NamedValuesAlg for Session {}

impl NamedValuesAlg for Agent {}

/// One reading, sent as a form.
#[derive(Debug, Serialize, Deserialize, Shape)]
pub struct Amount {
    /// What was read.
    pub value: u32,
}

/// Keeps whatever readings the domain has been told about.
pub trait ShopAlg {
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
    /// Returns who the caller is in, as they said.
    fn who(&self, session: String) -> impl Future<Output = String> + Send;
    /// Returns what the caller says they are.
    fn agent(&self, agent: String) -> impl Future<Output = String> + Send;
    /// Returns the readings and how long they may be kept.
    fn cached(&self) -> impl Future<Output = (String, Vec<u32>)> + Send;
}

/// Derives the operations the shared surface exposes.
#[ext(name = ShopOperationExt, defunc)]
pub impl<This> This
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

    /// Returns who the caller says they are.
    async fn shop_who(&self, session: Session) -> String {
        self.who(session.session).await
    }

    /// Returns what the caller says they are.
    async fn shop_agent(&self, agent: Agent) -> String {
        self.agent(agent.user_agent).await
    }

    /// Returns the readings, and how long a caller may keep them.
    async fn shop_cached(&self) -> (String, Vec<u32>) {
        self.cached().await
    }
}

/// Declares the shared surface, which every interpretation compiles unchanged.
#[ext(name = ShopApiExt, defunc(via = http))]
pub impl<This> This
where
    This: HttpApiAlg
        + HeaderOutAlg
        + JsonOutAlg
        + TextOutAlg
        + HtmlOutAlg
        + BytesOutAlg
        + EmptyOutAlg
        + RedirectOutAlg
        + StatusOutAlg
        + ResultOutAlg,
{
    /// Declares the surface every interpretation is held to.
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
            // Who the caller says they are, taken from the cookies they sent.
            .get("/session", self.op(Alg::shop_who).cookie::<Session>().text())
            // What the caller says they are, taken from the headers they sent.
            .get("/agent", self.op(Alg::shop_agent).in_header::<Agent>().text())
            // Every reading, and how long a caller may keep it.
            .get("/cached", self.op(Alg::shop_cached).json().out_header::<CacheControl>())
    }
}

/// The reference domain every interpretation is held to.
#[derive(Debug, Default, Clone, Copy)]
pub struct Shop;

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

    async fn who(&self, session: String) -> String {
        format!("known as {session}")
    }

    async fn agent(&self, agent: String) -> String {
        format!("sent by {agent}")
    }

    async fn cached(&self) -> (String, Vec<u32>) {
        ("max-age=60".to_owned(), vec![7])
    }
}

/// Every label the declared surface states, in declaration order.
pub const LABELS: &[&str] = &[
    "GET /item/{id}",
    "GET /items",
    "POST /items",
    "PUT /items",
    "PATCH /items",
    "DELETE /items",
    "GET /home",
    "GET /page",
    "GET /stored",
    "GET /session",
    "GET /agent",
    "GET /cached",
];

/// Reads however many arguments a caller states.
pub trait WideAlg {
    /// Returns what the domain makes of everything stated.
    fn wide(&self, stated: Vec<String>) -> impl Future<Output = String> + Send;
}

/// Derives the one operation the widest surface exposes.
#[ext(name = WideOperationExt, defunc)]
pub impl<This> This
where
    This: WideAlg,
{
    /// Returns what the domain makes of sixteen stated arguments.
    #[allow(clippy::too_many_arguments)]
    async fn wide_all(
        &self,
        first: String,
        second: String,
        third: String,
        fourth: String,
        fifth: String,
        sixth: String,
        seventh: String,
        eighth: String,
        ninth: String,
        tenth: String,
        eleventh: String,
        twelfth: String,
        thirteenth: String,
        fourteenth: String,
        fifteenth: String,
        sixteenth: String,
    ) -> String {
        let stated = vec![
            first, second, third, fourth, fifth, sixth, seventh, eighth, ninth, tenth, eleventh, twelfth, thirteenth,
            fourteenth, fifteenth, sixteenth,
        ];

        self.wide(stated).await
    }
}

/// Declares the widest endpoint the specification states, which every interpretation compiles.
///
/// Sixteen is what the products accumulate to, so one endpoint reading sixteen arguments is what
/// holds every interpretation to the same width. One role is repeated because a role a framework
/// reads once is a framework's business, and what is being stated here is the width.
#[ext(name = WideApiExt, defunc(via = http))]
pub impl<This> This
where
    This: HttpApiAlg + TextOutAlg,
{
    /// Declares one endpoint reading sixteen arguments.
    fn wide_api<Alg>(&self)
    where
        Alg: WideAlg,
    {
        self.routes().post(
            "/wide",
            self.op(Alg::wide_all)
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .raw_body::<String>()
                .text(),
        )
    }
}

impl WideAlg for Shop {
    async fn wide(&self, stated: Vec<String>) -> String {
        stated.len().to_string()
    }
}

/// A body the domain produces a piece at a time.
///
/// Nothing here names a stream type. The domain states what a chunk is and how the next one is
/// taken, and whichever interpretation carries the answer chooses how the bytes actually move.
#[derive(Debug, Default)]
pub struct Ticks {
    left: Vec<&'static str>,
}

impl ChunksAlg for Ticks {
    type Chunk = Vec<u8>;
    type Error = Infallible;

    async fn next_chunk(&mut self) -> Option<Result<Self::Chunk, Self::Error>> {
        self.left.pop().map(|tick| Ok(tick.as_bytes().to_vec()))
    }
}

/// Answers with a body it produces a piece at a time.
pub trait TicksAlg {
    /// Returns what the domain has to say, a piece at a time.
    fn ticks(&self) -> impl Future<Output = Ticks> + Send;
}

impl TicksAlg for Shop {
    async fn ticks(&self) -> Ticks {
        Ticks { left: vec!["three", "two", "one"] }
    }
}

/// Derives the one operation the streamed surface exposes.
#[ext(name = TicksOperationExt, defunc)]
pub impl<This> This
where
    This: TicksAlg,
{
    /// Returns what the domain has to say, a piece at a time.
    async fn shop_ticks(&self) -> Ticks {
        self.ticks().await
    }
}

/// Declares the streamed surface, which every interpretation carrying a produced body compiles.
#[ext(name = StreamApiExt, defunc(via = http))]
pub impl<This> This
where
    This: HttpApiAlg + StreamOutAlg,
{
    /// Declares one endpoint answering with a body produced over time.
    fn stream_api<Alg>(&self)
    where
        Alg: TicksAlg,
    {
        self.routes().get("/ticks", self.op(Alg::shop_ticks).stream())
    }
}

/// What a caller sent as parts, read the same way by every interpretation.
///
/// Nothing here names a reader. The domain states what it makes of a sequence of parts, and each
/// interpretation hands it whichever reader it has.
#[derive(Debug, Default)]
pub struct Upload {
    /// What each part was sent under, and what it carried.
    pub stated: Vec<String>,
}

impl<Parts> FromPartsAlg<Parts> for Upload
where
    Parts: ChunksAlg + Send,
    Parts::Error: Display + Send,
    Parts::Chunk: PartAlg + Send,
    <Parts::Chunk as PartAlg>::Content: ChunksAlg<Chunk = Vec<u8>> + Send + 'static,
    <<Parts::Chunk as PartAlg>::Content as ChunksAlg>::Error: Display,
{
    type Error = String;

    async fn from_parts(mut parts: Parts) -> Result<Self, Self::Error> {
        let mut stated = Vec::new();
        while let Some(part) = parts.next_chunk().await {
            let part = part.map_err(|error| error.to_string())?;
            let name = part.part_name().unwrap_or_default().to_owned();
            let carried = part.part_content().gathered().await.map_err(|error| error.to_string())?;
            let carried = String::from_utf8_lossy(&carried.concat()).into_owned();
            stated.push(format!("{name}={carried}"));
        }

        Ok(Self { stated })
    }
}

/// Reads a body a caller sent as parts.
pub trait UploadAlg {
    /// Returns what the domain makes of everything the parts stated.
    fn uploaded(&self, stated: Vec<String>) -> impl Future<Output = String> + Send;
}

impl UploadAlg for Shop {
    async fn uploaded(&self, stated: Vec<String>) -> String {
        stated.join(",")
    }
}

/// Derives the one operation the parts surface exposes.
#[ext(name = UploadOperationExt, defunc)]
pub impl<This> This
where
    This: UploadAlg,
{
    /// Returns what the domain makes of a body sent as parts.
    async fn shop_upload(&self, upload: Upload) -> String {
        self.uploaded(upload.stated).await
    }
}

/// Declares the parts surface, which every interpretation reading a body as parts compiles.
#[ext(name = MultipartApiExt, defunc(via = http))]
pub impl<This> This
where
    This: HttpApiAlg + TextOutAlg,
{
    /// Declares one endpoint reading a body that arrives as parts.
    fn multipart_api<Alg>(&self)
    where
        Alg: UploadAlg,
    {
        self.routes().post("/upload", self.op(Alg::shop_upload).multipart::<Upload>().text())
    }
}
