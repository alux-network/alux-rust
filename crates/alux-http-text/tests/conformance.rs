//! Describes the shared surface, which is what the scenario holds every answer to.

use alux_http::HttpProgramExt;
use alux_http_conformance::{LABELS, Shop, ShopApiExt, WideApiExt};
use alux_http_text::TextHandlerImpl;

#[test]
fn describes_the_shared_surface() {
    let text = TextHandlerImpl;

    assert_eq!(text.compile_http(text.shop_api::<Shop>()).labels(), LABELS);
}

#[test]
fn compiles_the_widest_endpoint_the_specification_states() {
    // The assertion is the compile: sixteen arguments reach this interpretation's products.
    let api = TextHandlerImpl;

    assert_eq!(api.compile_http(api.wide_api::<Shop>()).labels(), ["POST /wide"]);
}
