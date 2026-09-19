//! States which request method an endpoint answers on.
//!
//! A method is a value rather than a capability function. An interpreter witnesses every method by
//! interpreting one value, so declaring a method neither adds an obligation to an interpreter nor
//! grows the selector algebra.

/// Names the request method a declaration marker denotes.
pub trait HttpMethodAlg {
    /// The method this marker selects.
    const METHOD: HttpMethod;
}

macro_rules! http_methods {
    ($($declaration:ident => $marker:ident, $label:literal),+ $(,)?) => {
        /// Names one HTTP request method.
        ///
        /// These are the standard request methods, which every major framework routes natively. A
        /// surface that answers on an extension method states it as the closest standard method,
        /// because a method no framework can express is a method no interpreter could witness.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum HttpMethod {
            $(
                #[doc = concat!("Selects the `", $label, "` request method.")]
                $marker,
            )+
        }

        impl HttpMethod {
            /// Lists every request method the specification names, in declaration order.
            pub const ALL: &'static [Self] = &[$(Self::$marker),+];

            /// Returns the token this method is written as on the wire.
            pub const fn label(self) -> &'static str {
                match self {
                    $(Self::$marker => $label,)+
                }
            }
        }

        $(
            #[doc = concat!("Identifies a `", $label, "` endpoint declaration.")]
            #[derive(Debug, Default)]
            pub struct $marker;

            impl HttpMethodAlg for $marker {
                const METHOD: HttpMethod = HttpMethod::$marker;
            }
        )+
    };
}

with_http_methods!(http_methods);
