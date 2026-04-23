//! LLM analysis mechanism with real-time cognitive supervision.
//!
//! This mechanism integrates `ZhenfaPipeline` for in-flight cognitive
//! monitoring and early-halt capabilities during LLM streaming.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use futures::StreamExt;
use serde_json::json;
use std::sync::Arc;
use xiuxian_llm::llm::{ChatRequest, LlmClient};
use xiuxian_zhenfa::{StreamProvider, ZhenfaPipeline};

/// Mechanism responsible for performing supervised LLM inference.
///
/// This mechanism implements real-time cognitive sovereignty protection:
/// - In-flight token-by-token cognitive monitoring
/// - Early-halt detection for low coherence or logical drift
/// - XSD validation for structured outputs
/// - Cognitive distribution metrics in output
pub struct LlmAnalyzer {
    /// Thread-safe client for LLM communication.
    pub client: Arc<dyn LlmClient>,
    /// Target model name.
    pub model: String,
}

impl LlmAnalyzer {
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
}

#[async_trait]
impl QianjiMechanism for LlmAnalyzer {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let prompt = context
            .get("annotated_prompt")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'annotated_prompt'")?;
        let user_query = context
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("Summarize facts.");

        // Build request using litellm-rs builder pattern
        let request = ChatRequest::new(&self.model)
            .add_system_message(prompt)
            .add_user_message(user_query)
            .with_temperature(0.1);

        // Initialize the Cognitive Pipeline for in-flight supervision
        let mut pipeline = ZhenfaPipeline::new(self.resolve_provider());

        // Start the sovereign streaming loop
        let mut stream = self
            .client
            .chat_stream(request)
            .await
            .map_err(|e| format!("Stream initiation failed: {e}"))?;

        // Track accumulated text for final output
        let mut accumulated_text = String::new();
        let mut early_halt_reason: Option<String> = None;

        // In-flight cognitive supervision loop
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Stream chunk error: {e}"))?;

            // Accumulate text
            accumulated_text.push_str(&chunk);

            // Process through ZhenfaPipeline for cognitive analysis
            // Note: process_line expects NDJSON format, so we create a synthetic line
            let synthetic_line = format!(
                r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"text_delta","text":"{}"}}}}"#,
                chunk.replace('\\', "\\\\").replace('"', "\\\"")
            );

            if let Err(e) = pipeline.process_line(&synthetic_line) {
                // Cognitive guard violation - immediate abort
                early_halt_reason = Some(format!("Cognitive Guard Violation: {e}"));
                break;
            }

            // Real-time early-halt check
            if pipeline.should_halt() {
                early_halt_reason = Some(format!(
                    "Cognitive Drift Detected (Score: {:.2})",
                    pipeline.coherence_score()
                ));
                break;
            }
        }

        // Finalize the pipeline
        let _ = pipeline.finalize();

        // Extract cognitive metrics
        let coherence_score = pipeline.coherence_score();
        let cognitive_distribution = pipeline.cognitive_distribution();
        let should_halt = pipeline.should_halt();

        // Build cognitive metrics output
        let cognitive_metrics = json!({
            "coherence": coherence_score,
            "early_halt_triggered": early_halt_reason.is_some() || should_halt,
            "distribution": {
                "meta": cognitive_distribution.meta,
                "operational": cognitive_distribution.operational,
                "epistemic": cognitive_distribution.epistemic,
                "instrumental": cognitive_distribution.instrumental,
                "balance": cognitive_distribution.balance(),
                "uncertainty_ratio": cognitive_distribution.uncertainty_ratio(),
            }
        });

        // Determine flow instruction based on cognitive state
        let instruction = if let Some(ref reason) = early_halt_reason {
            FlowInstruction::Abort(reason.clone())
        } else if should_halt {
            FlowInstruction::Abort("Cognitive drift detected: early-halt triggered".to_string())
        } else {
            FlowInstruction::Continue
        };

        Ok(QianjiOutput {
            data: json!({
                "analysis_conclusion": accumulated_text,
                "cognitive_metrics": cognitive_metrics,
            }),
            instruction,
        })
    }

    fn weight(&self) -> f32 {
        3.0
    }
}

#[cfg(test)]
#[path = "../tests/unit/executors/llm/mechanism.rs"]
mod tests;
