//! Live LLM-backed advisory execution built on top of the advisory planning bridge.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use xiuxian_llm::llm::{ChatRequest, LlmClient};
use xiuxian_testing::{AdvisoryAuditExecutor, AdvisoryAuditRequest, RoleAuditFinding};
use xiuxian_zhenfa::ZhenfaPipeline;

use super::{QianjiAdvisoryAuditExecutor, QianjiAdvisoryExecutionPlan, QianjiAdvisoryRolePlan};
#[path = "executors_formal_audit_live_advisory/critique.rs"]
mod critique;
#[path = "executors_formal_audit_live_advisory/runtime.rs"]
mod runtime;
use critique::{apply_live_critique, live_advisory_instruction};
use runtime::{LiveCognitiveMetrics, resolve_model, resolve_provider};

const DEFAULT_MODEL: &str = "gpt-5.4-mini";
const DEFAULT_TEMPERATURE: f32 = 0.1;

/// Live advisory executor that sends role plans through an `LlmClient`.
pub struct QianjiLlmAdvisoryAuditExecutor {
    /// Planning bridge reused for role resolution and snapshot assembly.
    pub planner: QianjiAdvisoryAuditExecutor,
    /// Client used to execute one critique per role.
    pub client: Arc<dyn LlmClient>,
    /// Default model used when the request does not override it.
    pub model: String,
    /// Temperature used for critique generation.
    pub temperature: f32,
    /// Whether to supervise streaming output through `ZhenfaPipeline`.
    pub enable_cognitive_supervision: bool,
    /// Coherence threshold used when cognitive supervision is enabled.
    pub cognitive_early_halt_threshold: f32,
}

impl QianjiLlmAdvisoryAuditExecutor {
    /// Create a new live advisory executor.
    #[must_use]
    pub fn new(
        planner: QianjiAdvisoryAuditExecutor,
        client: Arc<dyn LlmClient>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            planner,
            client,
            model: model.into(),
            temperature: DEFAULT_TEMPERATURE,
            enable_cognitive_supervision: false,
            cognitive_early_halt_threshold: 0.3,
        }
    }

    /// Override the critique temperature.
    #[must_use]
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Enable cognitive supervision for streaming critiques.
    #[must_use]
    pub fn with_cognitive_supervision(mut self, early_halt_threshold: f32) -> Self {
        self.enable_cognitive_supervision = true;
        self.cognitive_early_halt_threshold = early_halt_threshold;
        self
    }

    async fn execute_role_critique(
        &self,
        request: &AdvisoryAuditRequest,
        role_plan: &QianjiAdvisoryRolePlan,
    ) -> Result<(String, Option<LiveCognitiveMetrics>)> {
        let request = ChatRequest::new(resolve_model(request, self.model.as_str()))
            .add_system_message(role_plan.rendered_prompt.clone())
            .add_user_message(live_advisory_instruction(request, role_plan))
            .with_temperature(self.temperature);

        if self.enable_cognitive_supervision {
            self.execute_with_cognitive_supervision(request).await
        } else {
            self.client
                .chat(request)
                .await
                .map(|response| (response, None))
                .map_err(Into::into)
        }
    }

    async fn execute_with_cognitive_supervision(
        &self,
        request: ChatRequest,
    ) -> Result<(String, Option<LiveCognitiveMetrics>)> {
        let mut pipeline = ZhenfaPipeline::with_options(
            resolve_provider(self.model.as_str()),
            true,
            true,
            self.cognitive_early_halt_threshold,
        );
        let mut stream = self.client.chat_stream(request).await?;
        let mut accumulated = String::new();
        let mut early_halt_reason = None;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            accumulated.push_str(&chunk);

            let synthetic_line = format!(
                r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"text_delta","text":"{}"}}}}"#,
                chunk.replace('\\', "\\\\").replace('"', "\\\"")
            );
            if let Err(error) = pipeline.process_line(&synthetic_line) {
                early_halt_reason = Some(format!("pipeline violation: {error}"));
                break;
            }
            if pipeline.should_halt() {
                early_halt_reason = Some(format!(
                    "cognitive drift detected at coherence {:.2}",
                    pipeline.coherence_score()
                ));
                break;
            }
        }

        let _ = pipeline.finalize();

        Ok((
            accumulated,
            Some(LiveCognitiveMetrics {
                coherence: pipeline.coherence_score(),
                early_halt: early_halt_reason,
                distribution: pipeline.cognitive_distribution(),
            }),
        ))
    }
}

#[async_trait]
impl AdvisoryAuditExecutor for QianjiLlmAdvisoryAuditExecutor {
    async fn run(&self, request: AdvisoryAuditRequest) -> Result<Vec<RoleAuditFinding>> {
        let plan: QianjiAdvisoryExecutionPlan = self.planner.build_plan(&request).await?;
        let mut findings = QianjiAdvisoryAuditExecutor::findings_from_plan(&request, &plan);

        for (finding, role_plan) in findings.iter_mut().zip(&plan.roles) {
            let (critique_text, cognitive_metrics) =
                self.execute_role_critique(&request, role_plan).await?;
            apply_live_critique(finding, &critique_text, cognitive_metrics);
        }

        Ok(findings)
    }
}
