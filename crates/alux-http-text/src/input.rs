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
/// Identifies header extraction in text descriptions.
pub struct HeaderRole;
/// Identifies authentication extraction in text descriptions.
pub struct AuthRole;
/// Identifies request-context extraction in text descriptions.
pub struct ContextRole;

impl HttpInputAlg for TextHandlerImpl {
    type Path<Input> = TextInputRole<PathRole, Input>;
    type Query<Input> = TextInputRole<QueryRole, Input>;
    type Body<Input> = TextInputRole<BodyRole, Input>;
    type Header<Input> = TextInputRole<HeaderRole, Input>;
    type Auth<Input> = TextInputRole<AuthRole, Input>;
    type Context<Input> = TextInputRole<ContextRole, Input>;
}
