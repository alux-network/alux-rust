//! What Rocket routed to an endpoint, before the roles read it.

use rocket::http::HeaderMap;

/// What one matched request states, as Rocket routed it.
#[derive(Debug, Clone)]
pub struct RocketRequest {
    pub(crate) captures: Vec<String>,
    pub(crate) query: String,
    pub(crate) headers: HeaderMap<'static>,
    pub(crate) body: Vec<u8>,
}

impl RocketRequest {
    /// Returns what the path bound, in the order the path binds it.
    pub fn captures(&self) -> &[String] {
        &self.captures
    }

    /// Returns the query string this request carries.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns the headers this request carries.
    pub fn headers(&self) -> &HeaderMap<'static> {
        &self.headers
    }

    /// Returns the body this request carries.
    pub fn body(&self) -> &[u8] {
        &self.body
    }
}
