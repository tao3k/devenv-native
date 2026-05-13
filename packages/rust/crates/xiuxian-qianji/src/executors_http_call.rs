//! Executors http call surface for `xiuxian-qianji`.

use std::collections::BTreeMap;

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use crate::scheduler_preflight::resolve_semantic_content;
use async_trait::async_trait;
use serde_json::{Value, json};

/// Mechanism responsible for one contract-validated HTTP call.
/// Semantic field boundary: this public DTO preserves externally serialized field names.
pub struct HttpCallMechanism {
    /// Stable contract id used for validation.
    pub contract: String,
    /// Authored HTTP method.
    pub method: String,
    /// Authored HTTP path or absolute URL.
    pub path: String,
    /// Optional base URL when `path` is relative.
    pub base_url: Option<String>,
    /// Authored HTTP query table.
    pub query: BTreeMap<String, Value>,
    /// Context key used to merge the response payload.
    pub output_key: String,
}

#[async_trait]
impl QianjiMechanism for HttpCallMechanism {
    async fn execute(&self, context: &Value) -> Result<QianjiOutput, String> {
        let method = resolve_semantic_content(&self.method, context)?;
        let path = resolve_semantic_content(&self.path, context)?;
        let base_url = self
            .base_url
            .as_deref()
            .map(|value| resolve_semantic_content(value, context))
            .transpose()?;
        let mut url = resolve_http_url(&path, base_url.as_deref())?;
        let query = resolve_query_pairs(&self.query, context)?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in &query {
                pairs.append_pair(key, value);
            }
        }
        let method = method
            .parse::<reqwest::Method>()
            .map_err(|error| format!("invalid HTTP method `{method}`: {error}"))?;

        let response = reqwest::Client::new()
            .request(method.clone(), url.clone())
            .send()
            .await
            .map_err(|error| format!("failed to call `{url}`: {error}"))?;
        let status = response.status();
        let body_text = response
            .text()
            .await
            .map_err(|error| format!("failed to read `{url}` response body: {error}"))?;
        let body = serde_json::from_str(&body_text).unwrap_or(Value::String(body_text.clone()));

        if !status.is_success() {
            return Err(format!(
                "HTTP call {} {} failed with {}: {}",
                method,
                url,
                status,
                body_text.trim()
            ));
        }

        Ok(QianjiOutput {
            data: json!({
                self.output_key.clone(): {
                    "contract": self.contract,
                    "transport": "http",
                    "status": status.as_u16(),
                    "body": body
                }
            }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

fn resolve_http_url(path: &str, base_url: Option<&str>) -> Result<reqwest::Url, String> {
    if let Ok(url) = reqwest::Url::parse(path) {
        return Ok(url);
    }

    let Some(base_url) = base_url else {
        return Err(format!(
            "relative HTTP path `{path}` requires `base_url` or an absolute URL"
        ));
    };
    let base = reqwest::Url::parse(base_url)
        .map_err(|error| format!("invalid HTTP base_url `{base_url}`: {error}"))?;
    base.join(path).map_err(|error| {
        format!("failed to join base_url `{base_url}` with path `{path}`: {error}")
    })
}

fn resolve_query_pairs(
    query: &BTreeMap<String, Value>,
    context: &Value,
) -> Result<Vec<(String, String)>, String> {
    query
        .iter()
        .map(|(key, value)| {
            let value = match value {
                Value::String(raw) => resolve_semantic_content(raw, context)?,
                Value::Number(number) => number.to_string(),
                Value::Bool(boolean) => boolean.to_string(),
                Value::Null => String::new(),
                _ => {
                    return Err(format!(
                        "HTTP query parameter `{key}` must be scalar, got `{value}`"
                    ));
                }
            };
            Ok((key.clone(), value))
        })
        .collect()
}
