//! Names the output conversion an endpoint selects, without converting anything.

use crate::TextHandlerImpl;
use alux_http::{FileOutAlg, JsonOutAlg, OutputAlg};

/// Interprets JSON output selection in text descriptions.
pub struct TextJsonOutput;

impl<From> OutputAlg<From> for TextJsonOutput {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

/// Interprets streamed-file output selection in text descriptions.
pub struct TextFileOutput;

impl<From> OutputAlg<From> for TextFileOutput {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

impl JsonOutAlg for TextHandlerImpl {
    type Json<From> = TextJsonOutput;
}

impl FileOutAlg for TextHandlerImpl {
    type File<From> = TextFileOutput;
}
