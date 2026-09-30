//! Closes the widest chain of output wrappers a declaration can state.

use alux_http::HttpProgramBuilder;

#[test]
fn closes_the_widest_chain_of_wrappers() {
    // The assertion is the compile: sixteen wrappers fold around the kind that closes them.
    let syntax = HttpProgramBuilder;
    let _ = syntax
        .op(())
        .result()
        .result()
        .result()
        .result()
        .result()
        .result()
        .result()
        .result()
        .result()
        .result()
        .result()
        .result()
        .result()
        .result()
        .result()
        .result()
        .json();
}
