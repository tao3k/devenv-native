//! Master research workflow tests for Qianji.

#[cfg(feature = "llm")]
use async_trait::async_trait;
#[cfg(feature = "llm")]
use futures::stream;
#[cfg(feature = "llm")]
use serde_json::json;
#[cfg(feature = "llm")]
use std::collections::HashMap;
#[cfg(feature = "llm")]
use std::sync::Arc;
#[cfg(feature = "llm")]
use xiuxian_llm::llm::client::ChatStream;
#[cfg(feature = "llm")]
use xiuxian_llm::llm::{ChatRequest, LlmClient, LlmResult};
#[cfg(feature = "llm")]
use xiuxian_qianhuan::{
    orchestrator::ThousandFacesOrchestrator,
    persona::{PersonaProfile, PersonaRegistry},
};
#[cfg(feature = "llm")]
use xiuxian_qianji::{QianjiCompiler, QianjiScheduler};
#[cfg(feature = "llm")]
use xiuxian_wendao::LinkGraphIndex;

#[cfg(feature = "llm")]
struct MockLlmClient;

#[cfg(feature = "llm")]
#[async_trait]
impl LlmClient for MockLlmClient {
    async fn chat(&self, _request: ChatRequest) -> LlmResult<String> {
        Ok("Research Conclusion: Logic is verified via Synapse-Audit.".to_string())
    }

    async fn chat_stream(&self, _request: ChatRequest) -> LlmResult<ChatStream> {
        Ok(Box::pin(stream::iter(vec![Ok(
            "Research Conclusion: Logic is verified via Synapse-Audit.".to_string(),
        )])))
    }
}

#[cfg(feature = "llm")]
const MASTER_RESEARCH_TOML: &str = include_str!("../../resources/tests/master_research.toml");

#[cfg(feature = "llm")]
#[tokio::test]
async fn test_qianji_master_research_array_flow()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let index = Arc::new(LinkGraphIndex::build(temp.path())?);
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Rules".to_string(), None));

    let mut registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "artisan-engineer".to_string(),
        name: "Artisan".to_string(),
        background: None,
        voice_tone: "Precise".to_string(),
        guidelines: Vec::new(),
        style_anchors: vec![
            "milimeter-level alignment".to_string(),
            "audit trail".to_string(),
        ],
        cot_template: "T".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
    });
    let registry_arc = Arc::new(registry);
    let llm_client: Arc<dyn LlmClient> = Arc::new(MockLlmClient);

    let compiler = QianjiCompiler::new(index, orchestrator, registry_arc, Some(llm_client));
    let engine = compiler.compile(MASTER_RESEARCH_TOML)?;
    let scheduler = QianjiScheduler::new(engine);

    let result = scheduler
        .run(json!({
            "query": "Verify Trinity",
            "raw_facts": "The system enforces milimeter-level alignment and full audit trail traceability. Architectural consistency is verified.",
            "drift_score": 0.01
        }))
        .await?;

    let conclusion = result["analysis_conclusion"].as_str().unwrap_or("");
    assert!(
        conclusion.contains("Synapse-Audit"),
        "Conclusion missing: {conclusion}",
    );
    Ok(())
}
