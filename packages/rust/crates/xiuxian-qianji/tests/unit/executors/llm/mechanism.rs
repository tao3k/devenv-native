use crate::contracts::{FlowInstruction, QianjiMechanism};
use crate::executors::LlmAnalyzer;
use async_trait::async_trait;
use futures::stream;
use serde_json::json;
use std::sync::Arc;
use xiuxian_llm::llm::client::ChatStream;
use xiuxian_llm::llm::{ChatRequest, LlmClient, LlmError};
use xiuxian_zhenfa::StreamProvider;

/// Mock LLM client that returns predefined responses.
struct MockLlmClient {
    responses: Vec<String>,
}

impl MockLlmClient {
    fn new(responses: Vec<&str>) -> Self {
        Self {
            responses: responses.into_iter().map(String::from).collect(),
        }
    }
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn chat(&self, _request: ChatRequest) -> Result<String, LlmError> {
        self.responses
            .first()
            .cloned()
            .ok_or(LlmError::EmptyTextChoice)
    }

    async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatStream, LlmError> {
        let chunks: Vec<Result<String, LlmError>> =
            self.responses.iter().map(|s| Ok(s.clone())).collect();
        Ok(Box::pin(stream::iter(chunks)))
    }
}

#[tokio::test]
async fn llm_analyzer_resolves_claude_provider() {
    let client = Arc::new(MockLlmClient::new(vec!["test response"]));
    let analyzer = LlmAnalyzer {
        client,
        model: "claude-3-opus".to_string(),
    };
    assert_eq!(analyzer.resolve_provider(), StreamProvider::Claude);
}

#[tokio::test]
async fn llm_analyzer_resolves_anthropic_provider() {
    let client = Arc::new(MockLlmClient::new(vec!["test response"]));
    let analyzer = LlmAnalyzer {
        client,
        model: "anthropic-claude".to_string(),
    };
    assert_eq!(analyzer.resolve_provider(), StreamProvider::Claude);
}

#[tokio::test]
async fn llm_analyzer_resolves_gemini_provider() {
    let client = Arc::new(MockLlmClient::new(vec!["test response"]));
    let analyzer = LlmAnalyzer {
        client,
        model: "gemini-pro".to_string(),
    };
    assert_eq!(analyzer.resolve_provider(), StreamProvider::Gemini);
}

#[tokio::test]
async fn llm_analyzer_resolves_codex_provider() {
    let client = Arc::new(MockLlmClient::new(vec!["test response"]));
    let analyzer = LlmAnalyzer {
        client,
        model: "gpt-4".to_string(),
    };
    assert_eq!(analyzer.resolve_provider(), StreamProvider::Codex);
}

#[tokio::test]
async fn llm_analyzer_executes_streaming() {
    let client = Arc::new(MockLlmClient::new(vec!["Hello ", "world!"]));
    let analyzer = LlmAnalyzer {
        client,
        model: "claude-3-opus".to_string(),
    };

    let context = json!({
        "annotated_prompt": "You are a helpful assistant.",
        "query": "Say hello."
    });

    let result = analyzer.execute(&context).await;
    assert!(result.is_ok());

    let output = result.unwrap_or_else(|err| panic!("analyzer should succeed: {err}"));
    assert_eq!(output.instruction, FlowInstruction::Continue);
    assert!(output.data["analysis_conclusion"].is_string());
    assert!(output.data["cognitive_metrics"]["coherence"].is_number());
}

#[tokio::test]
async fn llm_analyzer_includes_cognitive_metrics() {
    let client = Arc::new(MockLlmClient::new(vec!["Test response"]));
    let analyzer = LlmAnalyzer {
        client,
        model: "claude-3-opus".to_string(),
    };

    let context = json!({
        "annotated_prompt": "You are a helpful assistant.",
        "query": "Test query."
    });

    let output = analyzer
        .execute(&context)
        .await
        .unwrap_or_else(|err| panic!("analyzer should succeed: {err}"));

    let metrics = &output.data["cognitive_metrics"];
    assert!(metrics["coherence"].is_number());
    assert!(metrics["early_halt_triggered"].is_boolean());
    assert!(metrics["distribution"]["meta"].is_number());
    assert!(metrics["distribution"]["operational"].is_number());
    assert!(metrics["distribution"]["epistemic"].is_number());
    assert!(metrics["distribution"]["instrumental"].is_number());
    assert!(metrics["distribution"]["balance"].is_number());
    assert!(metrics["distribution"]["uncertainty_ratio"].is_number());
}

#[tokio::test]
async fn llm_analyzer_returns_weight() {
    let client = Arc::new(MockLlmClient::new(vec!["test"]));
    let analyzer = LlmAnalyzer {
        client,
        model: "test-model".to_string(),
    };
    assert!((analyzer.weight() - 3.0).abs() < 0.001);
}

#[tokio::test]
async fn llm_analyzer_handles_missing_prompt() {
    let client = Arc::new(MockLlmClient::new(vec!["test response"]));
    let analyzer = LlmAnalyzer {
        client,
        model: "claude-3-opus".to_string(),
    };

    let context = json!({
        "query": "Test query."
    });

    let result = analyzer.execute(&context).await;
    assert!(result.is_err());
    let Err(err) = result else {
        panic!("analyzer should fail when annotated_prompt is missing");
    };
    assert!(err.contains("Missing 'annotated_prompt'"));
}

#[tokio::test]
async fn llm_analyzer_uses_default_query() {
    let client = Arc::new(MockLlmClient::new(vec!["response"]));
    let analyzer = LlmAnalyzer {
        client,
        model: "claude-3-opus".to_string(),
    };

    let context = json!({
        "annotated_prompt": "You are a helpful assistant."
    });

    let result = analyzer.execute(&context).await;
    assert!(result.is_ok());
}
