//! States which headers an answer carries, beside the body it states.

/// States one header an answer carries.
///
/// A name is all a header is to a program. What an interpretation does with one is the same whatever
/// the name, so a header a specification does not state is a marker a domain writes for itself, and
/// no interpretation changes.
pub trait HeaderNameAlg {
    /// The name this header is written under, as the wire spells one.
    const HEADER_NAME: &'static str;
}

macro_rules! header_names {
    ($($marker:ident => $name:literal),+ $(,)?) => {
        $(
            #[doc = concat!("States the `", $name, "` header an answer carries.")]
            #[derive(Debug, Default)]
            pub struct $marker;

            impl HeaderNameAlg for $marker {
                const HEADER_NAME: &'static str = $name;
            }
        )+
    };
}

header_names! {
    CacheControl       => "cache-control",
    ContentDisposition => "content-disposition",
    ContentLanguage    => "content-language",
    ETag               => "etag",
    Expires            => "expires",
    LastModified       => "last-modified",
    Link               => "link",
    Location           => "location",
    RetryAfter         => "retry-after",
    Vary               => "vary",
}
