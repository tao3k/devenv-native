//! HTTP JSON-RPC client implementation for Zhenfa gateway calls.

use reqwest::{Client, StatusCode};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::contracts::{JSONRPC_VERSION, JsonRpcId, JsonRpcMeta, JsonRpcRequest, JsonRpcResponse};

const RPC_ENDPOINT: &str = "/rpc";
const HTTP_BODY_PREVIEW_LIMIT: usize = 512;

/// Successful Zhenfa JSON-RPC call output.
#[derive(Clone, Debug, PartialEq)]
pub struct ZhenfaClientSuccess {
    /// Echoed request id.
    pub id: JsonRpcId,
    /// Caller-facing payload string.
    pub result: String,
    /// Optional execution metrics from the gateway.
    pub metrics: Option<Value>,
}

/// Error type returned by [`ZhenfaClient`].
#[derive(Debug, Error)]
pub enum ZhenfaClientError {
    /// Base URL is malformed.
    #[error("invalid zhenfa base URL `{input}`")]
    InvalidBaseUrl {
        /// Raw base URL input.
        input: String,
    },
    /// Transport-level error while issuing the HTTP request.
    #[error("failed to call zhenfa gateway: {0}")]
    Transport(#[source] reqwest::Error),
    /// Non-2xx HTTP status from gateway.
    #[error("zhenfa gateway returned HTTP {status}: {body}")]
    HttpStatus {
        /// HTTP status code.
        status: StatusCode,
        /// Truncated response body preview.
        body: String,
    },
    /// Response body could not be decoded into JSON-RPC envelope.
    #[error("failed to decode zhenfa response: {0}")]
    Decode(#[source] reqwest::Error),
    /// JSON-RPC envelope is syntactically valid but semantically invalid.
    #[error("invalid zhenfa JSON-RPC response: {0}")]
    InvalidResponse(String),
    /// Gateway returned JSON-RPC `error`.
    #[error("zhenfa rpc error {code}: {message}")]
    Rpc {
        /// JSON-RPC error code from gateway.
        code: i32,
        /// Human-readable error message from gateway.
        message: String,
        /// Optional structured data payload.
        data: Option<Value>,
    },
}

/// Thin HTTP client for forwarding tool invocations to `xiuxian-zhenfa`.
#[derive(Clone)]
pub struct ZhenfaClient {
    base_url: String,
    rpc_url: String,
    http: Client,
}

impl ZhenfaClient {
    /// Construct a client with a default `reqwest::Client`.
    ///
    /// # Errors
    /// Returns an error when `base_url` is not a valid absolute `http(s)` URL.
    pub fn new(base_url: impl Into<String>) -> Result<Self, ZhenfaClientError> {
        Self::with_http(base_url, Client::new())
    }

    /// Construct a client with an externally provided HTTP client.
    ///
    /// # Errors
    /// Returns an error when `base_url` is not a valid absolute `http(s)` URL.
    pub fn with_http(base_url: impl Into<String>, http: Client) -> Result<Self, ZhenfaClientError> {
        let base_url = normalize_base_url(base_url.into())?;
        let rpc_url = format!("{base_url}{RPC_ENDPOINT}");
        Ok(Self {
            base_url,
            rpc_url,
            http,
        })
    }

    /// Base URL used by this client.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Execute one JSON-RPC call and return the structured envelope output.
    ///
    /// # Errors
    /// Returns an error when transport, HTTP, JSON decode, or JSON-RPC execution fails.
    pub async fn call(
        &self,
        method: &str,
        params: Value,
        meta: Option<JsonRpcMeta>,
    ) -> Result<ZhenfaClientSuccess, ZhenfaClientError> {
        let request = JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.to_string(),
            id: JsonRpcId::String(format!("req-{}", Uuid::new_v4())),
            params,
            meta,
        };

        let response = self
            .http
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await
            .map_err(ZhenfaClientError::Transport)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("<failed to read response body>"));
            return Err(ZhenfaClientError::HttpStatus {
                status,
                body: truncate_body(&body, HTTP_BODY_PREVIEW_LIMIT),
            });
        }

        let payload: JsonRpcResponse = response.json().await.map_err(ZhenfaClientError::Decode)?;

        let JsonRpcResponse {
            jsonrpc,
            id,
            result,
            error,
            metrics,
        } = payload;

        if jsonrpc != JSONRPC_VERSION {
            return Err(ZhenfaClientError::InvalidResponse(format!(
                "jsonrpc must be `{JSONRPC_VERSION}`, got `{jsonrpc}`",
            )));
        }

        if let Some(error) = error {
            return Err(ZhenfaClientError::Rpc {
                code: error.code,
                message: error.message,
                data: error.data,
            });
        }

        let Some(result) = result else {
            return Err(ZhenfaClientError::InvalidResponse(
                "missing `result` for successful response".to_string(),
            ));
        };

        Ok(ZhenfaClientSuccess {
            id,
            result,
            metrics,
        })
    }

    /// Execute one call and return only the stripped result payload.
    ///
    /// # Errors
    /// Returns the same error variants as [`Self::call`].
    pub async fn call_stripped(
        &self,
        method: &str,
        params: Value,
        meta: Option<JsonRpcMeta>,
    ) -> Result<String, ZhenfaClientError> {
        let response = self.call(method, params, meta).await?;
        Ok(response.result)
    }
}

fn normalize_base_url(base_url: String) -> Result<String, ZhenfaClientError> {
    let trimmed = base_url.trim().trim_end_matches('/').to_string();
    if trimmed.is_empty() || (!trimmed.starts_with("http://") && !trimmed.starts_with("https://")) {
        return Err(ZhenfaClientError::InvalidBaseUrl { input: base_url });
    }
    reqwest::Url::parse(&trimmed).map_err(|_| ZhenfaClientError::InvalidBaseUrl {
        input: base_url.clone(),
    })?;
    Ok(trimmed)
}

fn truncate_body(input: &str, limit: usize) -> String {
    if input.chars().count() <= limit {
        return input.to_string();
    }
    let mut out = String::with_capacity(limit + 3);
    for ch in input.chars().take(limit) {
        out.push(ch);
    }
    out.push_str("...");
    out
}
