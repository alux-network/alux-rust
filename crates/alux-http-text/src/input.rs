//! Names the input roles an endpoint reads, without extracting anything.

use crate::TextHandlerImpl;
use alux_http::HttpInputAlg;
use core::marker::PhantomData;

/// Identifies an HTTP input role in text output.
pub struct TextInputRole<Role, Input>(PhantomData<fn(Role) -> Input>);

/// Identifies path extraction in text descriptions.
pub struct PathRole;
/// Identifies query extraction in text descriptions.
pub struct QueryRole;
/// Identifies request-body extraction in text descriptions.
pub struct BodyRole;
/// Identifies form-encoded request-body extraction in text descriptions.
pub struct FormRole;
/// Identifies extraction from a body arriving as parts, in text descriptions.
pub struct MultipartRole;
/// Identifies extraction of the request body as it arrived, in text descriptions.
pub struct RawBodyRole;
/// Identifies header extraction in text descriptions.
pub struct HeaderRole;
/// Identifies cookie extraction in text descriptions.
pub struct CookieRole;
/// Identifies authentication extraction in text descriptions.
pub struct AuthRole;
/// Identifies request-context extraction in text descriptions.
pub struct ContextRole;

impl HttpInputAlg for TextHandlerImpl {
    type Path<Input> = TextInputRole<PathRole, Input>;
    type Query<Input> = TextInputRole<QueryRole, Input>;
    type Body<Input> = TextInputRole<BodyRole, Input>;
    type Form<Input> = TextInputRole<FormRole, Input>;
    type Multipart<Input> = TextInputRole<MultipartRole, Input>;
    type RawBody<Input> = TextInputRole<RawBodyRole, Input>;
    type Header<Input> = TextInputRole<HeaderRole, Input>;
    type Cookie<Input> = TextInputRole<CookieRole, Input>;
    type Auth<Input> = TextInputRole<AuthRole, Input>;
    type Context<Input> = TextInputRole<ContextRole, Input>;
}
