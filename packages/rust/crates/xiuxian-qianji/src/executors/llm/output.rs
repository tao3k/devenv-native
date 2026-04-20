//! LLM analysis mechanism for high-precision reasoning.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use crate::scheduler_preflight::resolve_semantic_content;
use async_trait::async_trait;
use serde_json::json;
use std::fmt::Write as _;
use std::sync::Arc;
use xiuxian_llm::llm::{ChatMessage, ChatRequest, LlmClient};

/// Mechanism responsible for performing LLM inference based on annotated context.
pub struct LlmAnalyzer {
    /// Thread-safe client for LLM communication.
    pub client: Arc<dyn LlmClient>,
    /// Target model name.
    pub model: String,
    /// Context keys to extract and format into the prompt.
    pub context_keys: Vec<String>,
    /// The template/base prompt for the system.
    pub prompt_template: String,
    /// The output key to store the result.
    pub output_key: String,
    /// Whether to parse model output as JSON and store structured value.
    pub parse_json_output: bool,
    /// Whether to build a fallback shard plan from `repo_tree` when JSON parsing fails.
    pub fallback_repo_tree_on_parse_failure: bool,
}

fn parse_json_from_text(raw: &str) -> Option<serde_json::Value> {
    let text = raw.trim();
    if text.is_empty() {
        return None;
    }

    let strip_fence = |candidate: &str| -> String {
        let without_open = candidate
            .strip_prefix("```json")
            .or_else(|| candidate.strip_prefix("```JSON"))
            .or_else(|| candidate.strip_prefix("```"))
            .unwrap_or(candidate)
            .trim()
            .to_string();
        without_open
            .strip_suffix("```")
            .unwrap_or(&without_open)
            .trim()
            .to_string()
    };

    let mut candidates = vec![strip_fence(text)];
    let fence_stripped = candidates[0].clone();

    let list_start = fence_stripped.find('[');
    let list_end = fence_stripped.rfind(']');
    if let (Some(start), Some(end)) = (list_start, list_end)
        && end > start
    {
        candidates.push(fence_stripped[start..=end].to_string());
    }

    let obj_start = fence_stripped.find('{');
    let obj_end = fence_stripped.rfind('}');
    if let (Some(start), Some(end)) = (obj_start, obj_end)
        && end > start
    {
        candidates.push(fence_stripped[start..=end].to_string());
    }

    for candidate in candidates {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&candidate) {
            return Some(value);
        }
    }
    None
}

fn build_repo_tree_fallback_plan(context: &serde_json::Value) -> serde_json::Value {
    let repo_tree = context
        .get("repo_tree")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let mut paths = Vec::new();
    for line in repo_tree.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("./") {
            continue;
        }
        if trimmed.matches('/').count() > 1 {
            continue;
        }
        let path = trimmed.trim_start_matches("./").trim();
        if !path.is_empty() {
            paths.push(path.to_string());
        }
        if paths.len() >= 12 {
            break;
        }
    }
    if paths.is_empty() {
        paths.push(".".to_string());
    }
    json!([
        {
            "shard_id": "repository-overview",
            "paths": paths,
        }
    ])
}

fn context_non_empty_string(context: &serde_json::Value, key: &str) -> Option<String> {
    context
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn resolve_model_for_request(context: &serde_json::Value, default_model: &str) -> String {
    if let Some(explicit_override) = context_non_empty_string(context, "llm_model") {
        return explicit_override;
    }
    let default_trimmed = default_model.trim();
    if !default_trimmed.is_empty() {
        return default_trimmed.to_string();
    }
    if let Some(fallback) = context_non_empty_string(context, "llm_model_fallback") {
        return fallback;
    }
    default_trimmed.to_string()
}

#[async_trait]
impl QianjiMechanism for LlmAnalyzer {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let mut final_prompt = resolve_semantic_content(&self.prompt_template, context)?;

        // Very basic interpolation from context keys or fallback to appending
        for key in &self.context_keys {
            if let Some(val) = context.get(key) {
                let val_str = if let Some(s) = val.as_str() {
                    s.to_string()
                } else {
                    val.to_string()
                };

                let placeholder = format!("{{{{{key}}}}}");
                if final_prompt.contains(&placeholder) {
                    final_prompt = final_prompt.replace(&placeholder, &val_str);
                } else {
                    let _ = write!(final_prompt, "\n\n[{key}]:\n{val_str}");
                }
            }
        }

        let user_query = context
            .get("request")
            .or_else(|| context.get("query"))
            .and_then(|v| v.as_str())
            .unwrap_or("Proceed.");

        let request = ChatRequest::new(resolve_model_for_request(context, &self.model))
            .add_system_message(&final_prompt)
            .add_user_message(user_query)
            .with_temperature(0.1);

        let conclusion = self
            .client
            .chat(request)
            .await
            .map_err(|e| format!("LLM execution failed: {e}"))?;

        let mut data = serde_json::Map::new();
        if self.parse_json_output {
            let parsed = parse_json_from_text(&conclusion).or_else(|| {
                if self.fallback_repo_tree_on_parse_failure {
                    Some(build_repo_tree_fallback_plan(context))
                } else {
                    None
                }
            });
            data.insert(
                self.output_key.clone(),
                parsed.unwrap_or_else(|| serde_json::Value::Array(Vec::new())),
            );
            data.insert(
                format!("{}_raw", self.output_key),
                serde_json::Value::String(conclusion),
            );
        } else {
            data.insert(
                self.output_key.clone(),
                serde_json::Value::String(conclusion),
            );
        }

        Ok(QianjiOutput {
            data: serde_json::Value::Object(data),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        3.0
    }
}
