//! LLM-feature integration tests for compiler dispatch route coverage.

#![cfg(feature = "llm")]
#![cfg(feature = "wendao-integration")]

use async_trait::async_trait;
use futures::stream;
use std::path::Path;
use std::sync::Arc;
use xiuxian_llm::llm::client::ChatStream;
use xiuxian_llm::llm::{ChatRequest, LlmClient, LlmResult};
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_qianji::QianjiCompiler;
use xiuxian_wendao::LinkGraphIndex;

const LLM_TASK_MANIFEST: &str = r#"
name = "LlmDispatchFeature"

[[nodes]]
id = "Analyzer"
task_type = "llm"
weight = 1.0
params = { prompt = "Analyze", output_key = "analysis" }
"#;

const FORMAL_AUDIT_LLM_MANIFEST: &str = r#"
name = "FormalAuditLlmDispatchFeature"

[[nodes]]
id = "Teacher"
task_type = "formal_audit"
weight = 1.0
params = { retry_targets = ["Steward"], threshold_score = 0.8, max_retries = 2 }
[nodes.qianhuan]
persona_id = "strict_teacher"
template_target = "critique_agenda.j2"
[nodes.llm]
model = "test-model"
"#;

const FORMAL_AUDIT_LLM_WITHOUT_RETRY_TARGETS_MANIFEST: &str = r#"
name = "FormalAuditLlmWithoutRetryTargetsDispatchFeature"

[[nodes]]
id = "Teacher"
task_type = "formal_audit"
weight = 1.0
params = { threshold_score = 0.8, max_retries = 2 }
[nodes.qianhuan]
persona_id = "strict_teacher"
template_target = "critique_agenda.j2"
[nodes.llm]
model = "test-model"
"#;

struct StubLlmClient;

#[async_trait]
impl LlmClient for StubLlmClient {
    async fn chat(&self, _request: ChatRequest) -> LlmResult<String> {
        Ok("ok".to_string())
    }

    async fn chat_stream(&self, _request: ChatRequest) -> LlmResult<ChatStream> {
        Ok(Box::pin(stream::iter(vec![Ok("ok".to_string())])))
    }
}

fn build_compiler_with_client(
    index_root: &Path,
    llm_client: Option<Arc<xiuxian_qianji::QianjiLlmClient>>,
) -> Result<QianjiCompiler, Box<dyn std::error::Error>> {
    let index = Arc::new(LinkGraphIndex::build(index_root)?);
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Rules".to_string(), None));
    let registry = Arc::new(PersonaRegistry::with_builtins());
    Ok(QianjiCompiler::new(
        index,
        orchestrator,
        registry,
        llm_client,
    ))
}

#[test]
fn compiler_dispatches_llm_task_with_global_llm_client() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let llm_client: Arc<xiuxian_qianji::QianjiLlmClient> = Arc::new(StubLlmClient);
    let compiler = build_compiler_with_client(temp.path(), Some(llm_client))?;
    let engine = compiler.compile(LLM_TASK_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_llm_augmented_formal_audit_with_global_llm_client()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let llm_client: Arc<xiuxian_qianji::QianjiLlmClient> = Arc::new(StubLlmClient);
    let compiler = build_compiler_with_client(temp.path(), Some(llm_client))?;
    let engine = compiler.compile(FORMAL_AUDIT_LLM_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_rejects_llm_augmented_formal_audit_with_positive_max_retries_and_no_retry_targets()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let llm_client: Arc<xiuxian_qianji::QianjiLlmClient> = Arc::new(StubLlmClient);
    let compiler = build_compiler_with_client(temp.path(), Some(llm_client))?;
    let error = compiler
        .compile(FORMAL_AUDIT_LLM_WITHOUT_RETRY_TARGETS_MANIFEST)
        .err()
        .unwrap_or_else(|| panic!("llm formal_audit without retry_targets should fail"));
    let message = error.to_string();
    assert!(message.contains("formal_audit.retry_targets"));
    assert!(message.contains("max_retries"));
    Ok(())
}

#[test]
fn compiler_rejects_llm_task_when_global_client_is_missing()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler_with_client(temp.path(), None)?;
    let error = compiler
        .compile(LLM_TASK_MANIFEST)
        .err()
        .unwrap_or_else(|| panic!("llm task should fail without a client"));
    let message = error.to_string();
    assert!(message.contains("LLM client not provided to compiler"));
    Ok(())
}
