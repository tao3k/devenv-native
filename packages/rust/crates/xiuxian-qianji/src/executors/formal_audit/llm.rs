use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use crate::executors::ContextAnnotator;
use async_trait::async_trait;
use futures::StreamExt;
use serde_json::{Value, json};
use std::sync::Arc;
use xiuxian_llm::llm::{ChatRequest, LlmClient};
use xiuxian_zhenfa::{StreamProvider, ZhenfaPipeline, ZhenfaTransmuter};

const FORMAL_AUDIT_XML_SCORE_CONTRACT: &str = "Return XML only. Include exactly one numeric <score> tag with a value in [0.0, 1.0]. Include a <reason> tag that explains the score. Do not emit Markdown or prose outside XML.";

fn context_non_empty_string(context: &Value, key: &str) -> Option<String> {
    context
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn resolve_model_for_request(context: &Value, default_model: &str) -> String {
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

fn extract_xml_score(text: &str) -> Option<f32> {
    ZhenfaTransmuter::get_tag_f32(text, "score")
}

fn score_to_memrl_reward(score: f32) -> f32 {
    score.clamp(0.0, 1.0)
}

/// LLM-driven formal audit controller (Synaptic Flow V2).
///
/// This mechanism implements cognitive supervision during audit:
/// - Real-time coherence monitoring during LLM streaming
/// - Early-halt detection for cognitive drift
/// - Cognitive distribution metrics in output
pub struct LlmAugmentedAuditMechanism {
    /// Node-local context annotator used to generate critique prompts.
    pub annotator: ContextAnnotator,
    /// LLM client used for critique generation.
    pub client: Arc<dyn LlmClient>,
    /// Default model name used unless context override is present.
    pub model: String,
    /// Score threshold below which retry is required.
    pub threshold_score: f32,
    /// Maximum allowed retries before hard stop to prevent runaway loops.
    pub max_retries: u32,
    /// Target nodes to trigger if audit score is below threshold.
    pub retry_target_ids: Vec<String>,
    /// Context key used to persist retry counter across loop iterations.
    pub retry_counter_key: String,
    /// Output key used for raw critique text.
    pub output_key: String,
    /// Output key used for numeric score extraction.
    pub score_key: String,
    /// Early-halt threshold for cognitive coherence (0.0 to disable).
    pub cognitive_early_halt_threshold: f32,
    /// Whether to enable cognitive monitoring.
    pub enable_cognitive_supervision: bool,
}

impl LlmAugmentedAuditMechanism {
    fn extract_prompt<'a>(
        &self,
        data: &'a serde_json::Map<String, Value>,
    ) -> Result<&'a str, String> {
        data.get(&self.annotator.output_key)
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "LlmAugmentedAuditMechanism missing annotated prompt at key `{}`",
                    self.annotator.output_key
                )
            })
    }

    /// Resolve the streaming provider based on model name.
    fn resolve_provider(&self) -> StreamProvider {
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
    async fn execute_with_cognitive_supervision(
        &self,
        request: ChatRequest,
    ) -> Result<(String, Option<CognitiveMetrics>), String> {
        // Initialize the Cognitive Pipeline
        let mut pipeline = ZhenfaPipeline::with_options(
            self.resolve_provider(),
            true, // validate_xsd
            true, // monitor_cognitive
            self.cognitive_early_halt_threshold,
        );

        // Start streaming
        let mut stream = self
            .client
            .chat_stream(request)
            .await
            .map_err(|e| format!("Stream initiation failed: {e}"))?;

        let mut accumulated_text = String::new();
        let mut early_halt_reason: Option<String> = None;

        // In-flight cognitive supervision loop
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Stream chunk error: {e}"))?;
            accumulated_text.push_str(&chunk);

            // Process through ZhenfaPipeline for cognitive analysis
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

    async fn execute_audit_request(
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

    fn insert_cognitive_metrics(
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

    fn finalize_audit_output(
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

/// Cognitive metrics from supervision.
struct CognitiveMetrics {
    coherence: f32,
    early_halt: bool,
    distribution: xiuxian_zhenfa::CognitiveDistribution,
}

#[async_trait]
impl QianjiMechanism for LlmAugmentedAuditMechanism {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let annotation_output = self.annotator.execute(context).await?;
        let Value::Object(mut data) = annotation_output.data else {
            return Err("LlmAugmentedAuditMechanism expected annotation output object".to_string());
        };

        let prompt = self.extract_prompt(&data)?;
        let user_query = context
            .get("request")
            .or_else(|| context.get("query"))
            .or_else(|| context.get("raw_facts"))
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("Critique the agenda and emit an XML <score> tag.");
        let (critique, cognitive_metrics) = self
            .execute_audit_request(context, prompt, user_query)
            .await?;

        let parsed_score = extract_xml_score(&critique);
        let score = parsed_score.unwrap_or(0.0);

        data.insert(self.output_key.clone(), Value::String(critique));
        data.insert(self.score_key.clone(), json!(score));
        data.insert(
            "memrl_reward".to_string(),
            json!(score_to_memrl_reward(score)),
        );
        data.insert("memrl_signal_source".to_string(), json!("formal_audit.llm"));

        if let Some(metrics) = cognitive_metrics.as_ref()
            && let Some(reason) = Self::insert_cognitive_metrics(&mut data, metrics)
        {
            return Ok(QianjiOutput {
                data: Value::Object(data),
                instruction: FlowInstruction::Abort(reason),
            });
        }

        Ok(self.finalize_audit_output(context, data, parsed_score, score))
    }

    fn weight(&self) -> f32 {
        2.0
    }
}
