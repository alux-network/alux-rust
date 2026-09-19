//! Documents the shared surface, which is what the scenario holds every answer to.

use alux_http::HttpProgramExt;
use alux_http_conformance::{LABELS, Shop, ShopApiExt, WideApiExt};
use alux_http_openapi::OpenApiHandlerImpl;

#[test]
fn documents_the_shared_surface() {
    let api = OpenApiHandlerImpl::<Shop>::new();
    let route = api.compile_http(api.shop_api::<Shop>());

    assert_eq!(route.labels(), LABELS);

    // Every endpoint the scenario exercises is one a document describes, under the name it was
    // declared with.
    let document = api.document("shop", "1.0", &route);
    assert_eq!(document["paths"]["/item/{id}"]["get"]["operationId"], "shop_item");
    // A response states what the operation it answers was documented as.
    assert_eq!(
        document["paths"]["/items"]["post"]["responses"]["201"]["description"],
        "Records one reading and returns it."
    );
    assert_eq!(document["paths"]["/items"]["delete"]["responses"]["204"]["description"], "Forgets every reading.");
}

#[test]
fn compiles_the_widest_endpoint_the_specification_states() {
    // The assertion is the compile: sixteen arguments reach this interpretation's products.
    let api = OpenApiHandlerImpl::<Shop>::new();

    assert_eq!(api.compile_http(api.wide_api::<Shop>()).labels(), ["POST /wide"]);
}
