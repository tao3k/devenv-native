//! Integration coverage for the Qianji contract-feedback pipeline.

#![cfg(feature = "wendao-integration")]

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
#[path = "support/workspace.rs"]
mod workspace;
use serde_json::json;
use xiuxian_qianhuan::{PersonaRegistry, ThousandFacesOrchestrator};
use xiuxian_qianji::contract_feedback::{
    AdvisoryAuditPolicy, ArtifactKind, CollectedArtifact, CollectedArtifacts, CollectionContext,
    ContractExecutionMode, ContractFinding, ContractRunConfig, ContractSuite, EvidenceKind,
    FindingConfidence, FindingEvidence, FindingExamples, FindingMode, FindingSeverity,
    NoopAdvisoryAuditExecutor, RulePack, RulePackDescriptor,
};
use xiuxian_qianji::{
    executors::QianjiAdvisoryAuditExecutor, run_and_persist_contract_feedback_flow,
    run_contract_feedback_flow, sovereign::InMemoryContractFeedbackSink,
};

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
        labels: std::collections::BTreeMap::from([(
            "session_id".to_string(),
            "session-contract-feedback".to_string(),
        )]),
    }
}

fn base_config() -> ContractRunConfig {
    ContractRunConfig {
        execution_mode: ContractExecutionMode::Advisory,
        generated_at: "2026-03-17T20:00:00Z".to_string(),
        ..ContractRunConfig::default()
    }
}

fn must_ok<T, E: std::fmt::Display>(result: std::result::Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

#[tokio::test]
async fn contract_feedback_flow_exports_deterministic_report_to_wendao_entries() {
    let result = must_ok(
        run_contract_feedback_flow(
            &test_suite(),
            &test_context(),
            &base_config(),
            &NoopAdvisoryAuditExecutor,
        )
        .await,
        "deterministic contract feedback flow should succeed",
    );

    assert_eq!(result.report.stats.total, 1);
    assert_eq!(result.report.stats.deterministic, 1);
    assert_eq!(result.knowledge_batch.entries.len(), 1);
    assert_eq!(result.knowledge_entries.len(), 1);

    let entry = &result.knowledge_entries[0];
    assert_eq!(entry.title, "[REST-R001] Missing endpoint purpose");
    assert_eq!(entry.source.as_deref(), Some("openapi.yaml"));
    assert!(entry.tags.iter().any(|tag| tag == "contract_feedback"));
    assert!(entry.tags.iter().any(|tag| tag == "decision:fail"));
    assert!(entry.tags.iter().any(|tag| tag == "pack:rest_docs"));
    assert!(
        entry
            .metadata
            .get("suite_id")
            .is_some_and(|value| value == "contracts")
    );
    assert!(
        entry
            .metadata
            .get("trace_ids")
            .is_some_and(|value| value == &json!(["trace-zhenfa-1"]))
    );
}

#[tokio::test]
async fn contract_feedback_flow_keeps_advisory_exports_unique_and_wendao_ready() {
    let executor = QianjiAdvisoryAuditExecutor::new(
        Arc::new(ThousandFacesOrchestrator::new(
            "Contract Kernel".to_string(),
            None,
        )),
        Arc::new(PersonaRegistry::with_builtins()),
    );
    let mut config = base_config();
    config.set_advisory_policy_for_pack(
        "rest_docs",
        AdvisoryAuditPolicy {
            enabled: true,
            requested_roles: vec!["strict_teacher".to_string()],
            min_severity: FindingSeverity::Warning,
        },
    );

    let result = must_ok(
        run_contract_feedback_flow(&test_suite(), &test_context(), &config, &executor).await,
        "advisory-backed contract feedback flow should succeed",
    );

    assert_eq!(result.report.stats.total, 2);
    assert_eq!(result.report.stats.deterministic, 1);
    assert_eq!(result.report.stats.advisory, 1);
    assert_eq!(result.knowledge_batch.entries.len(), 2);
    assert_eq!(result.knowledge_entries.len(), 2);

    let ids = result
        .knowledge_entries
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<HashSet<_>>();
    assert_eq!(ids.len(), result.knowledge_entries.len());

    let Some(advisory_entry) = result.knowledge_entries.iter().find(|entry| {
        entry.metadata.get("labels").is_some_and(|labels| {
            labels
                .get("source_lane")
                .is_some_and(|value| value == "qianji_advisory")
        })
    }) else {
        panic!("knowledge export should include one qianji advisory entry");
    };

    assert!(
        advisory_entry
            .id
            .contains("::advisory::openapi.yaml::role:strict_teacher")
    );
    assert!(advisory_entry.tags.iter().any(|tag| tag == "decision:fail"));
    assert!(
        advisory_entry
            .metadata
            .get("trace_ids")
            .is_some_and(|value| value == &json!(["trace-zhenfa-1"]))
    );
}

#[tokio::test]
async fn run_and_persist_contract_feedback_flow_persists_knowledge_entries_through_sink() {
    let sink = InMemoryContractFeedbackSink::new();

    let result = must_ok(
        run_and_persist_contract_feedback_flow(
            &test_suite(),
            &test_context(),
            &base_config(),
            &NoopAdvisoryAuditExecutor,
            &sink,
        )
        .await,
        "run-and-persist contract feedback flow should succeed",
    );

    assert_eq!(result.run.report.stats.total, 1);
    assert_eq!(result.persisted_entry_ids.len(), 1);
    assert_eq!(sink.len(), 1);
    assert_eq!(
        result.persisted_entry_ids[0],
        result.run.knowledge_entries[0].id
    );
    assert_eq!(sink.entries()[0].id, result.persisted_entry_ids[0]);
}
