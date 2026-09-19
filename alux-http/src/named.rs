//! States which arguments can be read from a collection of names and values.

use std::collections::{BTreeMap, HashMap};

/// States that an argument is read from a collection of names and values.
///
/// A query string, a header collection, and a cookie collection carry names and values. Every
/// interpretation presents one as a product, so an argument read from one has to be a product too:
/// a value stating no names of its own is read by nobody, and would fail on the first request that
/// reached it.
///
/// Nothing in a type says whether it is a product, so the type says it here. That turns the mistake
/// into one the compiler catches rather than one a caller finds:
///
/// ```compile_fail
/// use alux_http::HttpProgramBuilder;
///
/// let builder = HttpProgramBuilder;
/// // A query string carries names and values, and `String` states none.
/// let _ = builder.op(()).query::<String>();
/// ```
///
/// ```
/// use alux_http::{HttpProgramBuilder, NamedValuesAlg};
///
/// /// What a caller narrows a search by, stated as the query string carries it.
/// struct Filters {
///     since: u64,
/// }
///
/// impl NamedValuesAlg for Filters {}
///
/// let builder = HttpProgramBuilder;
/// let _ = builder.op(()).query::<Filters>();
/// ```
pub trait NamedValuesAlg {}

/// An association from names to values is what a collection of them is, whatever it maps to.
impl<Name, Value> NamedValuesAlg for BTreeMap<Name, Value> {}

/// An association from names to values is what a collection of them is, whatever it maps to.
impl<Name, Value, State> NamedValuesAlg for HashMap<Name, Value, State> {}
