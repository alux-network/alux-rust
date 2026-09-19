//! Calls the shared surface, which is what the scenario holds every answer to.

use alux_http::HttpProgramExt;
use alux_http_conformance::{LABELS, Shop, ShopApiExt, WideApiExt};
use alux_http_typescript::TsHttpClient;
use alux_shape::Spelling;

#[test]
fn calls_the_shared_surface() {
    let client = TsHttpClient::new(Spelling::LowerCamel);
    let module = client.compile_http(client.shop_api::<Shop>());

    assert_eq!(module.labels(), LABELS);
    assert_eq!(
        module.call_names(),
        [
            "shopAdd",
            "shopAgent",
            "shopCached",
            "shopClear",
            "shopFill",
            "shopHome",
            "shopItem",
            "shopItems",
            "shopNote",
            "shopPage",
            "shopStored",
            "shopWho",
        ]
    );
}

#[test]
fn compiles_the_widest_endpoint_the_specification_states() {
    // The assertion is the compile: sixteen arguments reach this interpretation's products.
    let api = TsHttpClient::new(Spelling::LowerCamel);

    assert_eq!(api.compile_http(api.wide_api::<Shop>()).labels(), ["POST /wide"]);
}
