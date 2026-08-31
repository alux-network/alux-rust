//! A program, folded into the client a caller would otherwise write by hand.

use alux_ext::ext;
use alux_jsonrpc::{JsonRpcApiAlg, JsonRpcProgramExt, jsonrpc};
use alux_jsonrpc_typescript::TsClient;
use alux_shape::{Shape, Spelling};
use core::future::Future;
use serde::Serialize;

/// What a user answer carries. The layout states its own shape.
#[derive(Serialize, Shape)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u64,
    pub display_name: String,
    pub email: Option<String>,
}

/// The domain these methods read.
trait UserAlg {
    fn user(&self, user_id: u64) -> impl Future<Output = Option<User>> + Send;
    fn count(&self) -> impl Future<Output = u64> + Send;
}

/// The operations the surface applies, as first-order values.
#[ext(name = UserOperationExt, defunc)]
impl<This> This
where
    This: UserAlg,
{
    /// One user, by identifier.
    async fn user_by_id(&self, user_id: u64) -> Option<User> {
        self.user(user_id).await
    }

    /// How many users there are.
    async fn user_total(&self) -> u64 {
        self.count().await
    }
}

/// The `user` namespace, stated before any interpreter is chosen.
#[ext(name = UserRpcExt, defunc(via = jsonrpc))]
impl<This> This
where
    This: JsonRpcApiAlg,
{
    /// Declares the user surface.
    fn user_rpc<Alg>(&self)
    where
        Alg: UserAlg,
    {
        self.methods()
            .method("user_count", self.op(Alg::user_total))
            // Decoded from a JSON array. The wire carries no names, the caller still reads them.
            .method("user_byId", self.op(Alg::user_by_id))
            // Decoded from a JSON object, so the parameter is named on the wire as well.
            .method("user_getById", self.op(Alg::user_by_id).named())
    }
}

/// A host that could answer the surface. Nothing here is applied; the client reads only the shapes.
struct Directory;

impl UserAlg for Directory {
    async fn user(&self, _user_id: u64) -> Option<User> {
        None
    }

    async fn count(&self) -> u64 {
        0
    }
}

#[test]
fn a_program_states_the_client_that_calls_it() {
    let ts = TsClient::new(Spelling::LowerCamel);
    let module = ts.compile_jsonrpc(ts.user_rpc::<Directory>()).render();

    assert_eq!(
        module,
        [
            "export interface User {",
            "  id: number",
            "  displayName: string",
            "  email: string | null",
            "}",
            "",
            "export const program = {",
            "  user_byId: method<[userId: number], User | null>(\"user_byId\", []),",
            "  user_count: method<[], number>(\"user_count\", []),",
            "  user_getById: method<[userId: number], User | null>(\"user_getById\", [\"user_id\"]),",
            "} as const",
        ]
        .join("\n"),
    );
}

#[test]
fn a_program_declares_the_methods_it_states_and_no_others() {
    let ts = TsClient::new(Spelling::LowerCamel);
    let module = ts.compile_jsonrpc(ts.user_rpc::<Directory>());

    assert_eq!(module.method_names(), ["user_byId", "user_count", "user_getById"]);
}
