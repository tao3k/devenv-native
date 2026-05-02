//! Integration coverage for the live LLM-backed contract-feedback pipeline.

#![cfg(feature = "llm")]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use futures::stream;
#[path = "support/workspace.rs"]
mod workspace;
use serde_json::json;
use xiuxian_llm::llm::client::ChatStream;
use xiuxian_llm::llm::{ChatRequest, LlmClient, LlmError, LlmResult};
use xiuxian_qianhuan::{PersonaRegistry, ThousandFacesOrchestrator};
use xiuxian_qianji::contract_feedback::{
    AdvisoryAuditPolicy, ArtifactKind, CollectedArtifact, CollectedArtifacts, CollectionContext,
    ContractExecutionMode, ContractFinding, ContractRunConfig, ContractSuite, EvidenceKind,
    FindingConfidence, FindingEvidence, FindingExamples, FindingMode, FindingSeverity, RulePack,
    RulePackDescriptor,
};
use xiuxian_qianji::{
    QianjiLiveContractFeedbackOptions, QianjiLiveContractFeedbackRuntime,
    run_and_persist_contract_feedback_flow_with_live_advisory,
    run_contract_feedback_flow_with_live_advisory, sovereign::InMemoryContractFeedbackSink,
};

#[derive(Debug, Clone)]
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

    fn seen_models(&self) -> Vec<String> {
        self.seen_models
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
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

#[derive(Debug, Clone, Copy)]
struct FakeRestDocsRulePack;

impl RulePack for FakeRestDocsRulePack {
    fn descriptor(&self) -> RulePackDescriptor {
        RulePackDescriptor {
            id: "rest_docs",
            version: "v1",
            domains: &["rest", "documentation"],
            default_mode: FindingMode::Deterministic,
        }
    }

    fn collect(&self, _ctx: &CollectionContext) -> Result<CollectedArtifacts> {
        let mut artifacts = CollectedArtifacts::default();
        artifacts.push(CollectedArtifact {
            id: "openapi".to_string(),
            kind: ArtifactKind::OpenApiDocument,
            path: Some(PathBuf::from("openapi.yaml")),
            content: json!({
                "openapi": "3.1.0",
                "paths": {
                    "/api/search": {
                        "get": {
                            "responses": {
                                "200": { "description": "ok" }
                            }
                        }
                    }
                }
            }),
            labels: std::collections::BTreeMap::new(),
        });
        artifacts.push(CollectedArtifact {
            id: "trace-zhenfa-1".to_string(),
            kind: ArtifactKind::RuntimeTrace,
            path: Some(PathBuf::from("trace.jsonl")),
            content: json!({
                "provider": "zhenfa",
                "events": 4
            }),
            labels: std::collections::BTreeMap::from([(
                "trace_id".to_string(),
                "trace-zhenfa-1".to_string(),
            )]),
        });
        Ok(artifacts)
    }

    fn evaluate(&self, _artifacts: &CollectedArtifacts) -> Result<Vec<ContractFinding>> {
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
            "Without a business-facing purpose, callers cannot infer endpoint intent.".to_string();
        finding.remediation = "Add a summary and one example request.".to_string();
        finding.examples = FindingExamples {
            good: vec!["summary: Retrieves a ranked set of knowledge hits.".to_string()],
            bad: vec!["summary: <missing>".to_string()],
        };
        finding
            .labels
            .insert("domain".to_string(), "rest".to_string());
        finding.evidence.push(FindingEvidence {
            kind: EvidenceKind::OpenApiNode,
            path: Some(PathBuf::from("openapi.yaml")),
            locator: Some("$.paths./api/search.get".to_string()),
            message: "GET /api/search is missing summary text.".to_string(),
        });
        Ok(vec![finding])
    }
}

fn test_suite() -> ContractSuite {
    let mut suite = ContractSuite::new("contracts", "v1");
    suite.register_rule_pack(Box::new(FakeRestDocsRulePack));
    suite
}

fn test_context() -> CollectionContext {
    CollectionContext {
        suite_id: "contracts".to_string(),
        crate_name: Some("xiuxian-wendao".to_string()),
        workspace_root: Some(workspace::workspace_root()),
        labels: std::collections::BTreeMap::from([
            (
                "session_id".to_string(),
                "session-contract-feedback".to_string(),
            ),
            ("llm_model".to_string(), "gpt-5.4-mini".to_string()),
        ]),
    }
}

fn live_config() -> ContractRunConfig {
    let mut config = ContractRunConfig {
        execution_mode: ContractExecutionMode::Advisory,
        generated_at: "2026-03-17T21:00:00Z".to_string(),
        ..ContractRunConfig::default()
    };
    config.set_advisory_policy_for_pack(
        "rest_docs",
        AdvisoryAuditPolicy {
            enabled: true,
            requested_roles: vec!["strict_teacher".to_string()],
            min_severity: FindingSeverity::Warning,
        },
    );
    config
}

fn must_ok<T, E: std::fmt::Display>(result: std::result::Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

#[tokio::test]
async fn live_contract_feedback_flow_exports_live_advisory_findings_to_wendao_entries() {
    let client = Arc::new(MockAdvisoryLlmClient::new(
        "",
        vec![
            r#"{"summary":"Live critique: endpoint contract is ambiguous.","why_it_matters":"Agents need a stable purpose statement.","remediation":"Add a summary and example.","severity":"critical","confidence":"high","evidence_excerpt":"summary field is missing in the contract.","good_example":"summary: Retrieves a ranked set of knowledge hits.","bad_example":"summary: <missing>"}"#,
        ],
    ));
    let result = must_ok(
        run_contract_feedback_flow_with_live_advisory(
            &test_suite(),
            &test_context(),
            &live_config(),
            Arc::new(ThousandFacesOrchestrator::new(
                "Contract Kernel".to_string(),
                None,
            )),
            Arc::new(PersonaRegistry::with_builtins()),
            client.clone(),
            QianjiLiveContractFeedbackOptions {
                cognitive_early_halt_threshold: Some(0.2),
                ..QianjiLiveContractFeedbackOptions::default()
            },
        )
        .await,
        "live contract feedback flow should succeed",
    );

    assert_eq!(result.report.stats.total, 2);
    assert_eq!(result.report.stats.deterministic, 1);
    assert_eq!(result.report.stats.advisory, 1);

    let Some(advisory_finding) = result
        .report
        .findings
        .iter()
        .find(|finding| finding.mode == FindingMode::Advisory)
    else {
        panic!("report should contain one live advisory finding");
    };
    assert_eq!(advisory_finding.severity, FindingSeverity::Critical);
    assert_eq!(advisory_finding.confidence, FindingConfidence::High);
    assert_eq!(
        advisory_finding.summary,
        "Live critique: endpoint contract is ambiguous."
    );
    assert_eq!(
        advisory_finding
            .labels
            .get("execution_mode")
            .map(String::as_str),
        Some("live_llm")
    );
    assert_eq!(
        advisory_finding
            .labels
            .get("cognitive_monitoring")
            .map(String::as_str),
        Some("enabled")
    );

    let Some(advisory_entry) = result.knowledge_entries.iter().find(|entry| {
        entry.metadata.get("labels").is_some_and(|labels| {
            labels
                .get("execution_mode")
                .is_some_and(|value| value == "live_llm")
        })
    }) else {
        panic!("knowledge export should include one live advisory entry");
    };
    assert_eq!(
        advisory_entry.title,
        "[REST-R001] Live critique: endpoint contract is ambiguous."
    );
    assert!(advisory_entry.tags.iter().any(|tag| tag == "decision:fail"));
    assert!(
        advisory_entry
            .metadata
            .get("trace_ids")
            .is_some_and(|value| value == &json!(["trace-zhenfa-1"]))
    );
    assert_eq!(client.seen_models(), vec!["gpt-5.4-mini".to_string()]);
}

#[tokio::test]
async fn live_run_and_persist_contract_feedback_flow_persists_live_entries_through_sink() {
    let client = Arc::new(MockAdvisoryLlmClient::new(
        "",
        vec![
            r#"{"summary":"Live critique: endpoint contract is ambiguous.","why_it_matters":"Agents need a stable purpose statement.","remediation":"Add a summary and example.","severity":"critical","confidence":"high","evidence_excerpt":"summary field is missing in the contract.","good_example":"summary: Retrieves a ranked set of knowledge hits.","bad_example":"summary: <missing>"}"#,
        ],
    ));
    let sink = InMemoryContractFeedbackSink::new();

    let result = must_ok(
        run_and_persist_contract_feedback_flow_with_live_advisory(
            &test_suite(),
            &test_context(),
            &live_config(),
            QianjiLiveContractFeedbackRuntime::new(
                Arc::new(ThousandFacesOrchestrator::new(
                    "Contract Kernel".to_string(),
                    None,
                )),
                Arc::new(PersonaRegistry::with_builtins()),
                client,
            ),
            QianjiLiveContractFeedbackOptions::default(),
            &sink,
        )
        .await,
        "live run-and-persist contract feedback flow should succeed",
    );

    assert_eq!(result.run.report.stats.total, 2);
    assert_eq!(result.persisted_entry_ids.len(), 2);
    assert_eq!(sink.len(), 2);
    assert_eq!(
        sink.entries()
            .into_iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        result.persisted_entry_ids
    );
}
