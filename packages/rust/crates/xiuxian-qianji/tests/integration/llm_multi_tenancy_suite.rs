//! Integration tests for Phase F node-level LLM multi-tenancy behavior.

#![cfg(feature = "llm")]

use async_trait::async_trait;
use futures::stream;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use xiuxian_llm::llm::client::ChatStream;
use xiuxian_llm::llm::{ChatRequest, LlmClient, LlmResult};
use xiuxian_qianhuan::{
    orchestrator::ThousandFacesOrchestrator,
    persona::{PersonaProfile, PersonaRegistry},
};
use xiuxian_qianji::QianjiCompiler;
use xiuxian_qianji::QianjiScheduler;
use xiuxian_wendao::LinkGraphIndex;

struct CapturingClient {
    seen_models: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl LlmClient for CapturingClient {
    async fn chat(&self, request: ChatRequest) -> LlmResult<String> {
        if let Ok(mut guard) = self.seen_models.lock() {
            guard.push(request.model);
        }
        Ok("ok".to_string())
    }

    async fn chat_stream(&self, request: ChatRequest) -> LlmResult<ChatStream> {
        if let Ok(mut guard) = self.seen_models.lock() {
            guard.push(request.model);
        }
        Ok(Box::pin(stream::iter(vec![Ok("ok".to_string())])))
    }
}

fn make_registry() -> Arc<PersonaRegistry> {
    let mut registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "artisan-engineer".to_string(),
        name: "Artisan".to_string(),
        background: None,
        voice_tone: "Precise".to_string(),
        guidelines: Vec::new(),
        style_anchors: Vec::new(),
        cot_template: "1. Read -> 2. Verify -> 3. Return".to_string(),
        forbidden_words: Vec::new(),
        metadata: HashMap::new(),
    });
    Arc::new(registry)
}

fn make_compiler(
    manifest: &str,
    seen_models: Arc<Mutex<Vec<String>>>,
) -> (QianjiScheduler, Arc<Mutex<Vec<String>>>) {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should work: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(temp.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let llm_client = Arc::new(CapturingClient {
        seen_models: Arc::clone(&seen_models),
    });
    let compiler = QianjiCompiler::new(index, orchestrator, make_registry(), Some(llm_client));
    let engine = compiler
        .compile(manifest)
        .unwrap_or_else(|error| panic!("manifest should compile: {error}"));
    (QianjiScheduler::new(engine), seen_models)
}

#[tokio::test]
async fn node_llm_binding_model_is_applied_per_node() {
    let manifest = r#"
name = "NodeLlmBindingModel"

[[nodes]]
id = "AnalyzerA"
task_type = "llm"
weight = 1.0
params = { output_key = "out_a" }
[nodes.llm]
provider = "openai"
model = "tenant-model-a"

[[nodes]]
id = "AnalyzerB"
task_type = "llm"
weight = 1.0
params = { output_key = "out_b" }
[nodes.llm]
provider = "openai"
model = "tenant-model-b"

[[edges]]
from = "AnalyzerA"
to = "AnalyzerB"
weight = 1.0
"#;

    let seen_models = Arc::new(Mutex::new(Vec::new()));
    let (scheduler, captured) = make_compiler(manifest, seen_models);
    let result = scheduler
        .run(json!({
            "request": "analyze",
            "annotated_prompt": "<ctx/>"
        }))
        .await
        .unwrap_or_else(|error| panic!("scheduler should run: {error}"));

    assert_eq!(result["out_a"], "ok");
    assert_eq!(result["out_b"], "ok");
    let models = captured
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default();
    assert_eq!(models, vec!["tenant-model-a", "tenant-model-b"]);
}

#[tokio::test]
async fn runtime_fallback_model_is_used_when_node_model_missing() {
    let manifest = r#"
name = "FallbackModel"

[[nodes]]
id = "Analyzer"
task_type = "llm"
weight = 1.0
params = { output_key = "out" }
"#;

    let seen_models = Arc::new(Mutex::new(Vec::new()));
    let (scheduler, captured) = make_compiler(manifest, seen_models);
    let result = scheduler
        .run(json!({
            "request": "analyze",
            "annotated_prompt": "<ctx/>",
            "llm_model_fallback": "global-runtime-model"
        }))
        .await
        .unwrap_or_else(|error| panic!("scheduler should run: {error}"));

    assert_eq!(result["out"], "ok");
    let models = captured
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default();
    assert_eq!(models, vec!["global-runtime-model"]);
}

#[tokio::test]
async fn node_model_takes_priority_over_runtime_fallback_model() {
    let manifest = r#"
name = "NodePriorityOverFallback"

[[nodes]]
id = "Analyzer"
task_type = "llm"
weight = 1.0
params = { output_key = "out" }
[nodes.llm]
provider = "openai"
model = "node-dedicated-model"
"#;

    let seen_models = Arc::new(Mutex::new(Vec::new()));
    let (scheduler, captured) = make_compiler(manifest, seen_models);
    let result = scheduler
        .run(json!({
            "request": "analyze",
            "annotated_prompt": "<ctx/>",
            "llm_model_fallback": "global-runtime-model"
        }))
        .await
        .unwrap_or_else(|error| panic!("scheduler should run: {error}"));

    assert_eq!(result["out"], "ok");
    let models = captured
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default();
    assert_eq!(models, vec!["node-dedicated-model"]);
}

#[test]
fn unsupported_node_provider_returns_compile_error() {
    let manifest = r#"
name = "UnsupportedNodeProvider"

[[nodes]]
id = "Analyzer"
task_type = "llm"
weight = 1.0
params = { output_key = "out" }
[nodes.llm]
provider = "litellm_rs"
model = "tenant-model"
base_url = "http://tenant.local/v1"
api_key_env = "TENANT_API_KEY"
"#;

    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should work: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(temp.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let compiler = QianjiCompiler::new(index, orchestrator, make_registry(), None);
    let error = compiler
        .compile(manifest)
        .err()
        .unwrap_or_else(|| panic!("manifest should fail to compile"));

    let message = error.to_string();
    assert!(message.contains("litellm_rs"));
    assert!(message.contains("not yet supported"));
}
