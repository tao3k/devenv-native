//! Integration coverage for the live LLM-backed advisory executor.

#![cfg(feature = "llm")]

use async_trait::async_trait;
use futures::stream;
use serde_json::json;
use std::sync::{Arc, Mutex};
use xiuxian_llm::llm::client::ChatStream;
use xiuxian_llm::llm::{ChatRequest, LlmClient, LlmError, LlmResult};
use xiuxian_qianhuan::{PersonaRegistry, ThousandFacesOrchestrator};
use xiuxian_qianji::contract_feedback::{
    AdvisoryAuditExecutor, AdvisoryAuditRequest, ArtifactKind, CollectedArtifact,
    CollectedArtifacts, CollectionContext, ContractFinding, EvidenceKind, FindingConfidence,
    FindingEvidence, FindingExamples, FindingMode, FindingSeverity,
};
use xiuxian_qianji::executors::{QianjiAdvisoryAuditExecutor, QianjiLlmAdvisoryAuditExecutor};

struct MockAdvisoryLlmClient {
    response: String,
    stream_chunks: Vec<String>,
    seen_models: Arc<Mutex<Vec<String>>>,
}

impl MockAdvisoryLlmClient {
    fn new(response: &str, stream_chunks: Vec<&str>) -> Self {
        Self {
            response: response.to_string(),
            stream_chunks: stream_chunks.into_iter().map(ToString::to_string).collect(),
            seen_models: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl LlmClient for MockAdvisoryLlmClient {
    async fn chat(&self, request: ChatRequest) -> LlmResult<String> {
        if let Ok(mut seen_models) = self.seen_models.lock() {
            seen_models.push(request.model);
        }
        Ok(self.response.clone())
    }

    async fn chat_stream(&self, request: ChatRequest) -> LlmResult<ChatStream> {
        if let Ok(mut seen_models) = self.seen_models.lock() {
            seen_models.push(request.model);
        }
        let chunks = self
            .stream_chunks
            .iter()
            .cloned()
            .map(Ok::<String, LlmError>)
            .collect::<Vec<_>>();
        Ok(Box::pin(stream::iter(chunks)))
    }
}

fn advisory_request() -> AdvisoryAuditRequest {
    let mut finding = ContractFinding::new(
        "REST-R001",
        "rest_docs",
        FindingSeverity::Error,
        FindingMode::Deterministic,
        "Missing endpoint purpose",
        "The endpoint is missing a purpose description.",
    );
    finding.confidence = FindingConfidence::High;
    finding.trace_ids.push("trace-zhenfa-1".to_string());
    finding.why_it_matters =
        "Without a clear purpose, the endpoint contract becomes ambiguous.".to_string();
    finding.remediation = "Add a summary and one request example.".to_string();
    finding.examples = FindingExamples {
        good: vec!["summary: Creates a knowledge node.".to_string()],
        bad: vec!["summary: <missing>".to_string()],
    };
    finding.evidence.push(FindingEvidence {
        kind: EvidenceKind::OpenApiNode,
        path: None,
        locator: Some("$.paths./nodes.post".to_string()),
        message: "POST /nodes is missing summary text.".to_string(),
    });

    let mut artifacts = CollectedArtifacts::default();
    let mut runtime_labels = std::collections::BTreeMap::new();
    runtime_labels.insert("trace_id".to_string(), "trace-zhenfa-1".to_string());
    artifacts.artifacts.push(CollectedArtifact {
        id: "runtime-trace-1".to_string(),
        kind: ArtifactKind::RuntimeTrace,
        path: None,
        labels: runtime_labels,
        content: json!({
            "events": [
                { "type": "Status", "text": "gateway warmup" },
                { "type": "TextDelta", "text": "missing summary detected" }
            ]
        }),
    });

    AdvisoryAuditRequest {
        suite_id: "wendao-gateway".to_string(),
        pack_id: "rest_docs".to_string(),
        pack_version: "v1".to_string(),
        pack_domains: vec!["rest".to_string(), "docs".to_string()],
        findings: vec![finding],
        artifacts,
        collection_context: CollectionContext {
            crate_name: Some("xiuxian-wendao".to_string()),
            labels: std::collections::BTreeMap::from([(
                "session_id".to_string(),
                "session-rest-docs".to_string(),
            )]),
            ..CollectionContext::default()
        },
        requested_roles: vec!["strict_teacher".to_string()],
    }
}

#[tokio::test]
async fn live_advisory_executor_parses_role_json_output() {
    let planner = QianjiAdvisoryAuditExecutor::new(
        Arc::new(ThousandFacesOrchestrator::new(
            "Contract Kernel".to_string(),
            None,
        )),
        Arc::new(PersonaRegistry::with_builtins()),
    );
    let client = Arc::new(MockAdvisoryLlmClient::new(
        r#"{
            "summary":"Role critique: endpoint contract is ambiguous.",
            "why_it_matters":"Agents cannot infer intent safely.",
            "remediation":"Document the endpoint purpose and add an example.",
            "severity":"critical",
            "confidence":"high",
            "evidence_excerpt":"summary field is missing in the contract.",
            "good_example":"summary: Creates a knowledge node.",
            "bad_example":"summary: <missing>"
        }"#,
        vec![],
    ));

    let executor =
        QianjiLlmAdvisoryAuditExecutor::new(planner, client, "gpt-5.4-mini").with_temperature(0.0);
    let findings = executor
        .run(advisory_request())
        .await
        .unwrap_or_else(|error| panic!("live advisory executor should succeed: {error}"));

    assert_eq!(findings.len(), 1);
    let finding = &findings[0];
    assert_eq!(finding.role_id, "strict_teacher");
    assert_eq!(finding.trace_id.as_deref(), Some("trace-zhenfa-1"));
    assert_eq!(finding.severity, FindingSeverity::Critical);
    assert_eq!(finding.confidence, FindingConfidence::High);
    assert_eq!(
        finding.summary,
        "Role critique: endpoint contract is ambiguous."
    );
    assert!(
        finding
            .evidence
            .iter()
            .any(|evidence| evidence.message.contains("summary field is missing"))
    );
    assert_eq!(
        finding.labels.get("execution_mode").map(String::as_str),
        Some("live_llm")
    );
}

#[tokio::test]
async fn live_advisory_executor_records_cognitive_metrics_when_streaming() {
    let planner = QianjiAdvisoryAuditExecutor::new(
        Arc::new(ThousandFacesOrchestrator::new(
            "Contract Kernel".to_string(),
            None,
        )),
        Arc::new(PersonaRegistry::with_builtins()),
    );
    let client = Arc::new(MockAdvisoryLlmClient::new(
        "",
        vec![
            r#"{"summary":"Streaming critique.","why_it_matters":"Need coherence evidence.","remediation":"Keep JSON stable.","severity":"warning","confidence":"medium"}"#,
        ],
    ));

    let executor = QianjiLlmAdvisoryAuditExecutor::new(planner, client, "gpt-5.4-mini")
        .with_cognitive_supervision(0.2);
    let findings = executor
        .run(advisory_request())
        .await
        .unwrap_or_else(|error| {
            panic!("live advisory executor should stream successfully: {error}")
        });

    let finding = &findings[0];
    assert_eq!(finding.summary, "Streaming critique.");
    assert_eq!(
        finding
            .labels
            .get("cognitive_monitoring")
            .map(String::as_str),
        Some("enabled")
    );
    assert!(
        finding
            .evidence
            .iter()
            .any(|evidence| evidence.message.contains("Cognitive distribution"))
    );
}
