//! States what a response answers with, and what a failure means to a caller.

use core::convert::Infallible;
use std::io::{Error as IoError, ErrorKind};

/// Names the status a response is answered with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HttpStatus(u16);

impl HttpStatus {
    /// Answers that the request succeeded.
    pub const OK: Self = Self(200);
    /// Answers that the request succeeded and states no body.
    pub const NO_CONTENT: Self = Self(204);
    /// Answers that what was asked for is at another location.
    pub const SEE_OTHER: Self = Self(303);
    /// Answers that the request could not be read.
    pub const BAD_REQUEST: Self = Self(400);
    /// Answers that the caller is known and not allowed.
    pub const FORBIDDEN: Self = Self(403);
    /// Answers that nothing is at this location.
    pub const NOT_FOUND: Self = Self(404);
    /// Answers that something is at this location, but not under this method.
    pub const METHOD_NOT_ALLOWED: Self = Self(405);
    /// Answers that the service failed for a reason the caller cannot act on.
    pub const INTERNAL: Self = Self(500);

    /// Returns the status this code names.
    pub const fn new(code: u16) -> Self {
        Self(code)
    }

    /// Returns the code this status is written as on the wire.
    pub const fn code(self) -> u16 {
        self.0
    }

    /// Returns whether this status names a successful answer.
    pub const fn is_success(self) -> bool {
        200 <= self.0 && self.0 < 300
    }
}

/// States what one failure means to a caller.
///
/// A handler that can fail returns a `Result`, and its endpoint answers with what the failure means
/// rather than with the failure itself. That meaning is HTTP vocabulary rather than domain
/// vocabulary, which is why a domain states its failures and this states how they are answered.
pub trait HttpErrorAlg {
    /// Every status this failure can be answered with.
    ///
    /// An interpretation that runs asks a failure what it means and is given one status. An
    /// interpretation that only reads a program has no failure to ask, so what an endpoint can
    /// answer with has to be stated by the type rather than by a value.
    const HTTP_STATUSES: &'static [HttpStatus];

    /// Returns the status this failure is answered with.
    fn http_status(&self) -> HttpStatus;

    /// Returns the message this failure is answered with.
    fn http_message(&self) -> String;
}

impl HttpErrorAlg for Infallible {
    const HTTP_STATUSES: &'static [HttpStatus] = &[];

    fn http_status(&self) -> HttpStatus {
        match *self {}
    }

    fn http_message(&self) -> String {
        match *self {}
    }
}

impl HttpErrorAlg for IoError {
    const HTTP_STATUSES: &'static [HttpStatus] = &[HttpStatus::NOT_FOUND, HttpStatus::FORBIDDEN, HttpStatus::INTERNAL];

    fn http_status(&self) -> HttpStatus {
        match self.kind() {
            ErrorKind::NotFound => HttpStatus::NOT_FOUND,
            ErrorKind::PermissionDenied => HttpStatus::FORBIDDEN,
            _ => HttpStatus::INTERNAL,
        }
    }

    fn http_message(&self) -> String {
        self.to_string()
    }
}
