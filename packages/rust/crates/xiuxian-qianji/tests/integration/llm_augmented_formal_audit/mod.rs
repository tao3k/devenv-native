//! Tests for LLM-augmented formal audit flow control.

#![cfg(feature = "llm")]

use async_trait::async_trait;
use futures::stream;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use xiuxian_llm::llm::client::{ChatStream, MessageRole};
use xiuxian_llm::llm::{ChatRequest, LlmClient, LlmError, LlmResult, MessageContent};
use xiuxian_qianhuan::{
    orchestrator::ThousandFacesOrchestrator,
    persona::{PersonaProfile, PersonaRegistry},
};
use xiuxian_qianji::NodeQianhuanExecutionMode;
use xiuxian_qianji::contracts::{FlowInstruction, QianjiMechanism};
use xiuxian_qianji::executors::{ContextAnnotator, LlmAugmentedAuditMechanism};
use xiuxian_qianji::{QianjiCompiler, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

mod cognitive_supervision;
mod mechanism_core;
mod provider_resolution;

struct SequencedMockLlmClient {
    responses: Arc<Mutex<Vec<String>>>,
    seen_models: Arc<Mutex<Vec<String>>>,
}

impl SequencedMockLlmClient {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            seen_models: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl LlmClient for SequencedMockLlmClient {
    async fn chat(&self, request: ChatRequest) -> LlmResult<String> {
        if let Ok(mut models) = self.seen_models.lock() {
            models.push(request.model);
        }
        let mut responses = self.responses.lock().map_err(|_| LlmError::Internal {
            message: "failed to lock llm response queue".to_string(),
        })?;
        if responses.is_empty() {
            return Ok("<score>1.0</score>".to_string());
        }
        Ok(responses.remove(0))
    }

    async fn chat_stream(&self, request: ChatRequest) -> LlmResult<ChatStream> {
        if let Ok(mut models) = self.seen_models.lock() {
            models.push(request.model);
        }
        let responses = self.responses.lock().map_err(|_| LlmError::Internal {
            message: "failed to lock llm response queue".to_string(),
        })?;
        let chunks: Vec<Result<String, LlmError>> =
            responses.iter().map(|s| Ok(s.clone())).collect();
        Ok(Box::pin(stream::iter(chunks)))
    }
}

struct RequestCaptureLlmClient {
    requests: Arc<Mutex<Vec<ChatRequest>>>,
    response: String,
}

#[async_trait]
impl LlmClient for RequestCaptureLlmClient {
    async fn chat(&self, request: ChatRequest) -> LlmResult<String> {
        self.requests
            .lock()
            .map_err(|_| LlmError::Internal {
                message: "failed to lock llm request capture".to_string(),
            })?
            .push(request);
        Ok(self.response.clone())
    }

    async fn chat_stream(&self, request: ChatRequest) -> LlmResult<ChatStream> {
        self.requests
            .lock()
            .map_err(|_| LlmError::Internal {
                message: "failed to lock llm request capture".to_string(),
            })?
            .push(request);
        Ok(Box::pin(stream::iter(vec![Ok(self.response.clone())])))
    }
}

fn make_registry() -> Arc<PersonaRegistry> {
    let mut registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "strict_teacher".to_string(),
        name: "Strict Teacher".to_string(),
        background: None,
        voice_tone: "Direct and strict.".to_string(),
        guidelines: vec!["Score rigorously.".to_string()],
        style_anchors: Vec::new(),
        cot_template: "1. Critique -> 2. Score -> 3. Decide".to_string(),
        forbidden_words: Vec::new(),
        metadata: HashMap::new(),
    });
    Arc::new(registry)
}

fn must_ok<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

fn must_f64(value: &serde_json::Value, context: &str) -> f64 {
    value
        .as_f64()
        .unwrap_or_else(|| panic!("{context}: expected numeric value"))
}

fn must_bool(value: &serde_json::Value, context: &str) -> bool {
    value
        .as_bool()
        .unwrap_or_else(|| panic!("{context}: expected boolean value"))
}

fn must_object<'a>(
    value: &'a serde_json::Value,
    context: &str,
) -> &'a serde_json::Map<String, serde_json::Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("{context}: expected object value"))
}

fn make_test_mechanism(
    llm: Arc<dyn LlmClient>,
    model: &str,
    threshold_score: f32,
    enable_cognitive_supervision: bool,
) -> LlmAugmentedAuditMechanism {
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = make_registry();

    LlmAugmentedAuditMechanism {
        annotator: ContextAnnotator {
            orchestrator,
            registry,
            persona_id: "strict_teacher".to_string(),
            template_target: Some("critique_agenda.j2".to_string()),
            execution_mode: NodeQianhuanExecutionMode::Isolated,
            input_keys: vec!["raw_facts".to_string()],
            history_key: "audit_history".to_string(),
            output_key: "annotated_prompt".to_string(),
        },
        client: llm,
        model: model.to_string(),
        threshold_score,
        max_retries: 3,
        retry_target_ids: vec!["Agenda_Steward_Proposer".to_string()],
        retry_counter_key: "audit_retry_count".to_string(),
        output_key: "audit_critique".to_string(),
        score_key: "audit_score".to_string(),
        cognitive_early_halt_threshold: 0.3,
        enable_cognitive_supervision,
    }
}

xiuxian_testing::crate_test_policy_harness!();
