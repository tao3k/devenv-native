use std::fmt::Write as _;

use async_trait::async_trait;
use xiuxian_llm::llm::ChatRequest;

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use crate::scheduler_preflight::resolve_semantic_content;

use super::api::StreamingLlmAnalyzer;
use super::output::{
    build_repo_tree_fallback_plan, parse_json_from_text, resolve_model_for_request,
};

#[async_trait]
impl QianjiMechanism for StreamingLlmAnalyzer {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let mut final_prompt = resolve_semantic_content(&self.prompt_template, context)?;

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
            .add_system_message(final_prompt)
            .add_user_message(user_query)
            .with_temperature(0.1);

        let conclusion = self
            .client
            .chat(request)
            .await
            .map_err(|e| format!("LLM execution failed: {e}"))?;

        let pipeline = self.create_pipeline();
        let coherence_score = pipeline.coherence_score();
        let _cognitive_distribution = pipeline.cognitive_distribution();
        let early_halt_triggered = pipeline.should_halt();

        let mut data = serde_json::Map::new();

        if self.output_flags.parse_json_output {
            let parsed = parse_json_from_text(&conclusion).or_else(|| {
                if self.output_flags.fallback_repo_tree_on_parse_failure {
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

        data.insert(
            "_cognitive_coherence".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(f64::from(coherence_score))
                    .unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );
        data.insert(
            "_early_halt_triggered".to_string(),
            serde_json::Value::Bool(early_halt_triggered),
        );

        let instruction = if early_halt_triggered {
            FlowInstruction::Abort(
                "Early halt triggered due to low cognitive coherence".to_string(),
            )
        } else {
            FlowInstruction::Continue
        };

        Ok(QianjiOutput {
            data: serde_json::Value::Object(data),
            instruction,
        })
    }

    fn weight(&self) -> f32 {
        3.0
    }
}
