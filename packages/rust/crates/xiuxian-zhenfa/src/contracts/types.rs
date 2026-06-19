//! JSON-RPC envelope types shared by clients and gateway handlers.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::{
    INVALID_PARAMS_CODE, INVALID_REQUEST_CODE, JSONRPC_VERSION, METHOD_NOT_FOUND_CODE,
};

fn default_jsonrpc() -> String {
    JSONRPC_VERSION.to_string()
}

fn default_params() -> Value {
    Value::Object(serde_json::Map::default())
}

/// JSON-RPC id can be string/number/null.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum JsonRpcId {
    /// String identifier.
    String(String),
    /// Integer identifier.
    Number(i64),
    /// Null identifier.
    #[default]
    Null,
}

/// Optional metadata propagated with JSON-RPC requests.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct JsonRpcMeta {
    /// Session id propagated from caller context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Trace id propagated from caller context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// Extra metadata keys.
    #[serde(flatten, default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, Value>,
}

/// Zhenfa JSON-RPC request envelope.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (`2.0`).
    #[serde(default = "default_jsonrpc")]
    pub jsonrpc: String,
    /// Fully-qualified method name (for example `wendao.search`).
    pub method: String,
    /// Request identifier.
    #[serde(default)]
    pub id: JsonRpcId,
    /// Method params payload.
    #[serde(default = "default_params")]
    pub params: Value,
    /// Optional request metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<JsonRpcMeta>,
}

impl JsonRpcRequest {
    /// Validate protocol-level request constraints.
    ///
    /// # Errors
    /// Returns a JSON-RPC error object when `jsonrpc`, `method`, or `params`
    /// violates the gateway contract.
    pub fn validate(&self) -> Result<(), JsonRpcErrorObject> {
        if self.jsonrpc != JSONRPC_VERSION {
            return Err(JsonRpcErrorObject::invalid_request(
                "jsonrpc must be exactly \"2.0\"",
            ));
        }
        if self.method.trim().is_empty() {
            return Err(JsonRpcErrorObject::invalid_request(
                "method must be a non-empty string",
            ));
        }
        if !self.params.is_object() {
            return Err(JsonRpcErrorObject::invalid_params(
                "params must be a JSON object",
            ));
        }
        Ok(())
    }
}

/// Zhenfa JSON-RPC error payload.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct JsonRpcErrorObject {
    /// JSON-RPC error code.
    pub code: i32,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured error details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcErrorObject {
    /// Build a custom JSON-RPC error object.
    #[must_use]
    pub fn new(code: i32, message: impl Into<String>, data: Option<Value>) -> Self {
        Self {
            code,
            message: message.into(),
            data,
        }
    }

    /// Invalid request helper.
    #[must_use]
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(INVALID_REQUEST_CODE, message, None)
    }

    /// Invalid params helper.
    #[must_use]
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(INVALID_PARAMS_CODE, message, None)
    }

    /// Method-not-found helper.
    #[must_use]
    pub fn method_not_found(method: impl Into<String>) -> Self {
        let method = method.into();
        Self::new(
            METHOD_NOT_FOUND_CODE,
            format!("method not found: {method}"),
            None,
        )
    }
}

/// Zhenfa JSON-RPC response envelope.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (`2.0`).
    #[serde(default = "default_jsonrpc")]
    pub jsonrpc: String,
    /// Request identifier.
    pub id: JsonRpcId,
    /// Success payload (caller-facing stripped string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// Error payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcErrorObject>,
    /// Optional transport metrics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Value>,
}

impl JsonRpcResponse {
    /// Build a success JSON-RPC response.
    #[must_use]
    pub fn success(id: JsonRpcId, result: String, metrics: Option<Value>) -> Self {
        Self {
            jsonrpc: default_jsonrpc(),
            id,
            result: Some(result),
            error: None,
            metrics,
        }
    }

    /// Build an error JSON-RPC response.
    #[must_use]
    pub fn error(id: JsonRpcId, error: JsonRpcErrorObject) -> Self {
        Self {
            jsonrpc: default_jsonrpc(),
            id,
            result: None,
            error: Some(error),
            metrics: None,
        }
    }
}
