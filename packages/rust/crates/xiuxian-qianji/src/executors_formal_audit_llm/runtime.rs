use crate::contracts::{FlowInstruction, QianjiOutput};
use futures::StreamExt;
use serde_json::{Value, json};
use xiuxian_llm::llm::ChatRequest;
use xiuxian_zhenfa::{
    CognitiveDistribution, StreamProvider, ZhenfaPipeline, ZhenfaPipelineOptions,
};

use super::api::LlmAugmentedAuditMechanism;
use super::context::{context_non_empty_string, resolve_model_for_request};
use super::scoring::FORMAL_AUDIT_XML_SCORE_CONTRACT;

/// Cognitive metrics from supervision.
pub(super) struct CognitiveMetrics {
    pub(super) coherence: f32,
    pub(super) early_halt: bool,
    pub(super) distribution: CognitiveDistribution,
}

impl LlmAugmentedAuditMechanism {
    /// Resolve the streaming provider based on model name.
    pub(super) fn resolve_provider(&self) -> StreamProvider {
        let model_lower = self.model.to_lowercase();
        if model_lower.contains("claude") || model_lower.contains("anthropic") {
            StreamProvider::Claude
        } else if model_lower.contains("gemini") {
            StreamProvider::Gemini
        } else {
            StreamProvider::Codex
        }
    }

    /// Execute LLM request with cognitive supervision.
    ///
    /// Returns the critique text and cognitive metrics.
    pub(super) async fn execute_with_cognitive_supervision(
        &self,
        request: ChatRequest,
    ) -> Result<(String, Option<CognitiveMetrics>), String> {
        let mut pipeline = ZhenfaPipeline::with_options(
            ZhenfaPipelineOptions::new(self.resolve_provider())
                .with_early_halt_threshold(self.cognitive_early_halt_threshold),
        );

        let mut stream = self
            .client
            .chat_stream(request)
            .await
            .map_err(|e| format!("Stream initiation failed: {e}"))?;

        let mut accumulated_text = String::new();
        let mut early_halt_reason: Option<String> = None;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Stream chunk error: {e}"))?;
            accumulated_text.push_str(&chunk);

            let synthetic_line = format!(
                r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"text_delta","text":"{}"}}}}"#,
                chunk.replace('\\', "\\\\").replace('"', "\\\"")
            );

            if let Err(e) = pipeline.process_line(&synthetic_line) {
                early_halt_reason = Some(format!("Cognitive Guard Violation: {e}"));
                break;
            }

            if pipeline.should_halt() {
                early_halt_reason = Some(format!(
                    "Cognitive Drift Detected (Score: {:.2})",
                    pipeline.coherence_score()
                ));
                break;
            }
        }

        let _ = pipeline.finalize();

        let metrics = CognitiveMetrics {
            coherence: pipeline.coherence_score(),
            early_halt: early_halt_reason.is_some() || pipeline.should_halt(),
            distribution: pipeline.cognitive_distribution(),
        };

        Ok((accumulated_text, Some(metrics)))
    }

    pub(super) async fn execute_audit_request(
        &self,
        context: &Value,
        prompt: &str,
        user_query: &str,
    ) -> Result<(String, Option<CognitiveMetrics>), String> {
        let request = ChatRequest::new(resolve_model_for_request(context, &self.model))
            .add_system_message(prompt)
            .add_system_message(FORMAL_AUDIT_XML_SCORE_CONTRACT)
            .add_user_message(user_query)
            .with_temperature(0.1);

        if self.enable_cognitive_supervision {
            self.execute_with_cognitive_supervision(request).await
        } else {
            self.client
                .chat(request)
                .await
                .map(|critique| (critique, None))
                .map_err(|error| format!("LLM formal audit execution failed: {error}"))
        }
    }

    pub(super) fn insert_cognitive_metrics(
        data: &mut serde_json::Map<String, Value>,
        metrics: &CognitiveMetrics,
    ) -> Option<String> {
        data.insert("_cognitive_coherence".to_string(), json!(metrics.coherence));
        data.insert(
            "_early_halt_triggered".to_string(),
            json!(metrics.early_halt),
        );
        data.insert(
            "_cognitive_distribution".to_string(),
            json!({
                "meta": metrics.distribution.meta,
                "operational": metrics.distribution.operational,
                "epistemic": metrics.distribution.epistemic,
                "instrumental": metrics.distribution.instrumental,
                "balance": metrics.distribution.balance(),
                "uncertainty_ratio": metrics.distribution.uncertainty_ratio(),
            }),
        );

        if metrics.early_halt {
            data.insert("audit_status".to_string(), json!("cognitive_drift"));
            return Some(format!(
                "Cognitive drift detected (coherence: {:.2})",
                metrics.coherence
            ));
        }

        None
    }

    pub(super) fn finalize_audit_output(
        &self,
        context: &Value,
        mut data: serde_json::Map<String, Value>,
        parsed_score: Option<f32>,
        score: f32,
    ) -> QianjiOutput {
        let failed = score < self.threshold_score;
        let retry_count = context
            .get(&self.retry_counter_key)
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0);
        let mut audit_errors: Vec<String> = Vec::new();

        if parsed_score.is_none() {
            audit_errors.push("LLM audit score missing or invalid; defaulted to 0.0.".to_string());
        }

        if let Some(memrl_episode_id) = context_non_empty_string(context, "memrl_episode_id")
            .or_else(|| context_non_empty_string(context, "episode_id"))
        {
            data.insert("memrl_episode_id".to_string(), json!(memrl_episode_id));
        }

        data.insert("audit_threshold".to_string(), json!(self.threshold_score));
        data.insert(self.retry_counter_key.clone(), json!(retry_count));

        if failed {
            let next_retry_count = retry_count.saturating_add(1);
            data.insert(self.retry_counter_key.clone(), json!(next_retry_count));
            audit_errors.push("LLM audit score below threshold.".to_string());

            if next_retry_count > self.max_retries {
                audit_errors.push(format!(
                    "LLM audit retry budget exceeded (max_retries={}).",
                    self.max_retries
                ));
                data.insert("audit_retry_exhausted".to_string(), json!(true));
                data.insert("audit_status".to_string(), json!("failed"));
                data.insert("audit_errors".to_string(), json!(audit_errors));
                return QianjiOutput {
                    data: Value::Object(data),
                    instruction: FlowInstruction::Abort(
                        "formal_audit.max_retries_exceeded".to_string(),
                    ),
                };
            }

            data.insert("audit_status".to_string(), json!("failed"));
            data.insert("audit_errors".to_string(), json!(audit_errors));
            return QianjiOutput {
                data: Value::Object(data),
                instruction: FlowInstruction::RetryNodes(self.retry_target_ids.clone()),
            };
        }

        data.insert("audit_status".to_string(), json!("passed"));
        QianjiOutput {
            data: Value::Object(data),
            instruction: FlowInstruction::Continue,
        }
    }
}
