use crate::error::RpcError;
use crate::table::MethodTable;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

/// What one call answers with, in the member order the specification presents.
#[derive(Debug, Serialize)]
struct Response {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
    id: Value,
}

impl Response {
    /// Reads one call's outcome as the response it denotes.
    fn of(id: Value, outcome: Result<Value, RpcError>) -> Self {
        match outcome {
            Ok(result) => Self { jsonrpc: "2.0", result: Some(result), error: None, id },
            Err(error) => Self { jsonrpc: "2.0", result: None, error: Some(error), id },
        }
    }
}

/// What one request document answers with: a single response, or one per non-notification call.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Answered {
    One(Response),
    Many(Vec<Response>),
}

/// Reads the request members a call supplies, or states why the document is not a request.
fn members(call: Value) -> Result<serde_json::Map<String, Value>, RpcError> {
    match call {
        Value::Object(members) => Ok(members),
        _ => Err(RpcError::invalid_request()),
    }
}

/// States whether a value can identify a call, which the specification limits to a string, number,
/// or null.
fn is_identifier(id: &Value) -> bool {
    matches!(id, Value::String(_) | Value::Number(_) | Value::Null)
}

impl MethodTable {
    /// Answers one JSON-RPC request document.
    ///
    /// Answers with nothing when the document asks for nothing: a notification, or a batch of them.
    pub async fn dispatch(&self, request: &str) -> Option<String> {
        let answered = match serde_json::from_str::<Value>(request) {
            Ok(document) => self.answer(document).await?,
            Err(_) => Answered::One(Response::of(Value::Null, Err(RpcError::parse_error()))),
        };

        Some(render(&answered))
    }

    /// Answers one parsed request document, distinguishing a batch from a single call.
    async fn answer(&self, document: Value) -> Option<Answered> {
        let Value::Array(calls) = document else {
            return self.call(document).await.map(Answered::One);
        };
        if calls.is_empty() {
            return Some(Answered::One(Response::of(Value::Null, Err(RpcError::invalid_request()))));
        }
        let mut answers = Vec::new();
        for call in calls {
            if let Some(answer) = self.call(call).await {
                answers.push(answer);
            }
        }

        (!answers.is_empty()).then_some(Answered::Many(answers))
    }

    /// Answers one call, or nothing when the call is a notification.
    async fn call(&self, call: Value) -> Option<Response> {
        let (id, outcome) = self.outcome(call).await;

        id.map(|id| Response::of(id, outcome))
    }

    /// Reads what one call asks for, answering with the identifier to respond to and the outcome.
    ///
    /// An absent identifier means a notification, which is answered by saying nothing at all — even
    /// when the call itself is malformed.
    async fn outcome(&self, call: Value) -> (Option<Value>, Result<Value, RpcError>) {
        let mut members = match members(call) {
            Ok(members) => members,
            Err(error) => return (Some(Value::Null), Err(error)),
        };
        // An absent identifier is what makes a call a notification, so it decides whether to answer
        // at all; every later step reports through it.
        let respond_to = members.remove("id");
        if respond_to.as_ref().is_some_and(|id| !is_identifier(id)) {
            return (Some(Value::Null), Err(RpcError::invalid_request()));
        }
        if members.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return (respond_to, Err(RpcError::invalid_request()));
        }
        let Some(Value::String(method)) = members.remove("method") else {
            return (respond_to, Err(RpcError::invalid_request()));
        };
        let params = members.remove("params");
        if params.as_ref().is_some_and(|params| !(params.is_array() || params.is_object() || params.is_null())) {
            return (respond_to, Err(RpcError::invalid_request()));
        }
        let Some(method) = self.get(method.as_str()).map(Arc::clone) else {
            return (respond_to, Err(RpcError::method_not_found(&method)));
        };

        (respond_to, method(params).await)
    }
}

/// Renders an answer, stating an internal error if it somehow cannot be serialized.
fn render(answered: &Answered) -> String {
    serde_json::to_string(answered).unwrap_or_else(|_| {
        r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"the answer cannot be serialized"},"id":null}"#.to_owned()
    })
}
