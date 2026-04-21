use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};
use xiuxian_llm::llm::LlmClient;

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use crate::executors::ContextAnnotator;

use super::scoring::{extract_xml_score, score_to_memrl_reward};

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
