//! What a warp filter gathered, before the roles read it.
//!
//! warp matches a request by composing filters, so what reaches an endpoint is what those filters
//! extracted rather than a request object. This carries exactly that.

use warp::http::HeaderMap;

/// What one matched request states, as warp's filters gathered it.
#[derive(Debug, Clone)]
pub struct WarpRequest {
    pub(crate) captures: Vec<String>,
    pub(crate) query: String,
    pub(crate) headers: HeaderMap,
    pub(crate) body: Vec<u8>,
}

impl WarpRequest {
    /// Returns what the path bound, in the order the path binds it.
    pub fn captures(&self) -> &[String] {
        &self.captures
    }

    /// Returns the query string this request carries.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns the headers this request carries.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Returns the body this request carries.
    pub fn body(&self) -> &[u8] {
        &self.body
    }
}
