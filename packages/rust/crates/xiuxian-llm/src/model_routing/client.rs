//! vLLM-SR route-decision probe client.

use reqwest::header::AUTHORIZATION;
use serde_json::json;

use super::constants::VLLM_SR_AUTO_MODEL;
use super::route_helpers::normalize_base_url;
use super::types::{WendaoModelDecision, WendaoRouteIntent};

/// OpenAI-compatible vLLM-SR route-decision probe client.
#[derive(Clone)]
pub struct VllmSrRouteDecisionClient {
    base_url: String,
    bearer_token: Option<String>,
    http: reqwest::Client,
}

impl VllmSrRouteDecisionClient {
    /// Build a route decision client.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_http_client(base_url, reqwest::Client::new())
    }

    /// Build a route decision client with an injected HTTP client.
    #[must_use]
    pub fn with_http_client(base_url: impl Into<String>, http: reqwest::Client) -> Self {
        let base_url = base_url.into();
        Self {
            base_url: normalize_base_url(base_url.as_str()),
            bearer_token: None,
            http,
        }
    }

    /// Attach an optional bearer token for vLLM-SR deployments that require it.
    #[must_use]
    pub fn with_bearer_token(mut self, bearer_token: impl Into<String>) -> Self {
        let bearer_token = bearer_token.into();
        self.bearer_token = (!bearer_token.trim().is_empty()).then_some(bearer_token);
        self
    }

    /// Return the normalized vLLM-SR base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url.as_str()
    }

    /// Obtain a route decision through vLLM-SR's OpenAI-compatible data plane.
    ///
    /// # Errors
    ///
    /// Returns an error when the HTTP request fails, vLLM-SR returns a
    /// non-success status, or the response does not include a selected model.
    pub async fn decide(
        &self,
        intent: &WendaoRouteIntent,
        selected_provider: &str,
        selected_backend_profile: &str,
    ) -> Result<WendaoModelDecision, String> {
        let endpoint = format!("{}/v1/chat/completions", self.base_url);
        let prompt = serde_json::to_string(intent)
            .map_err(|error| format!("serialize Wendao route intent: {error}"))?;
        let payload = json!({
            "model": VLLM_SR_AUTO_MODEL,
            "messages": [
                {
                    "role": "system",
                    "content": "Route this Wendao task. The response body is ignored; Wendao reads vLLM-SR routing headers."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0,
            "max_tokens": 1,
            "stream": false
        });

        let mut request = self.http.post(endpoint.as_str()).json(&payload);
        if let Some(token) = self.bearer_token.as_deref() {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        let response = request
            .send()
            .await
            .map_err(|error| format!("call vLLM-SR route probe `{endpoint}`: {error}"))?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .text()
            .await
            .map_err(|error| format!("read vLLM-SR route probe response body: {error}"))?;
        if !status.is_success() {
            return Err(format!(
                "vLLM-SR route probe `{endpoint}` returned {status}: {body}"
            ));
        }
        WendaoModelDecision::from_vllm_sr_response_parts(
            &headers,
            &body,
            selected_provider,
            selected_backend_profile,
        )
    }
}
