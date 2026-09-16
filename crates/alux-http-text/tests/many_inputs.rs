//! One endpoint reads sixteen arguments, and stating them costs what stating one of them costs.
//!
//! Each recorded role extends the extractor product and the argument product in the same position,
//! so what this guards is that the two stay in step all the way up. The text interpretation extracts
//! nothing, which is what lets every role be mixed here without a transport deciding which of them
//! may appear more than once.

#![allow(async_fn_in_trait)]

use alux_ext::ext;
use alux_http::{HttpApiAlg, HttpProgramExt, JsonOutAlg, NamedValuesAlg, http};
use alux_http_text::TextHandlerImpl;
use core::future::Future;

/// What a caller states in a collection of names and values.
struct Stated {
    // The text interpretation reads nothing, so this member is stated and never taken.
    #[allow(dead_code)]
    stated: String,
}

impl NamedValuesAlg for Stated {}

/// Reads whatever the domain holds, however many ways a caller states it.
trait ReadingAlg {
    /// The value one reading denotes.
    type Reading;

    /// Returns the reading the arguments state.
    fn read(&self) -> impl Future<Output = Self::Reading> + Send;
}

#[ext(name = ReadOperationExt, defunc)]
impl<This> This
where
    This: ReadingAlg,
{
    /// Returns the reading sixteen stated arguments select.
    #[allow(clippy::too_many_arguments)]
    async fn read_all(
        &self,
        first: u8,
        second: Stated,
        third: u32,
        fourth: u64,
        fifth: i8,
        sixth: Stated,
        seventh: Stated,
        eighth: i64,
        ninth: bool,
        tenth: Stated,
        eleventh: String,
        twelfth: u8,
        thirteenth: u16,
        fourteenth: Stated,
        fifteenth: Stated,
        sixteenth: String,
    ) -> This::Reading {
        let _ = (first, second, third, fourth, fifth, sixth, seventh, eighth);
        let _ = (ninth, tenth, eleventh, twelfth, thirteenth, fourteenth, fifteenth, sixteenth);

        self.read().await
    }
}

#[ext(name = ReadApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + JsonOutAlg,
{
    /// Declares one endpoint reading sixteen arguments, across every role a caller can state.
    fn read_api<Alg>(&self)
    where
        Alg: ReadingAlg,
    {
        self.routes().get(
            "/read/:first",
            self.op(Alg::read_all)
                .path::<u8>()
                .query::<Stated>()
                .body::<u32>()
                .form::<u64>()
                .raw_body::<i8>()
                .in_header::<Stated>()
                .auth::<Stated>()
                .context::<i64>()
                .path::<bool>()
                .query::<Stated>()
                .body::<String>()
                .form::<u8>()
                .raw_body::<u16>()
                .in_header::<Stated>()
                .auth::<Stated>()
                .context::<String>()
                .json(),
        )
    }
}

/// Names a domain for the declaration to be read against.
struct Domain;

impl ReadingAlg for Domain {
    type Reading = u32;

    async fn read(&self) -> u32 {
        7
    }
}

#[test]
fn one_endpoint_reads_sixteen_stated_arguments() {
    let text = TextHandlerImpl;
    let described = text.compile_http(text.read_api::<Domain>());
    let line = described.lines().first().cloned().unwrap_or_default();

    assert!(line.starts_with("### GET /read/{first}"), "{line}");
    // The extractor product and the argument product stay in step, sixteen deep.
    assert_eq!(line.matches("alux_http_text::input::TextInputRole").count(), 16, "{line}");
    assert_eq!(line.matches("Role,").count(), 16, "{line}");
}
