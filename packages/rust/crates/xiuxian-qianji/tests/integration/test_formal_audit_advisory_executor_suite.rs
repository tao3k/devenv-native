//! Focused coverage for the formal-audit advisory executor bridge.

use std::path::PathBuf;
use std::sync::Arc;

#[path = "support/workspace.rs"]
mod workspace;
use serde_json::json;
use xiuxian_qianhuan::{PersonaRegistry, ThousandFacesOrchestrator};
use xiuxian_qianji::contract_feedback::{
    AdvisoryAuditExecutor, AdvisoryAuditRequest, ArtifactKind, CollectedArtifact,
    CollectedArtifacts, CollectionContext, ContractFinding, EvidenceKind, FindingConfidence,
    FindingEvidence, FindingExamples, FindingMode, FindingSeverity,
};
use xiuxian_qianji::executors::QianjiAdvisoryAuditExecutor;

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
        path: Some(PathBuf::from("openapi.yaml")),
        locator: Some("$.paths./nodes.post".to_string()),
        message: "POST /nodes is missing summary text.".to_string(),
    });

    let mut artifacts = CollectedArtifacts::default();
    let mut runtime_labels = std::collections::BTreeMap::new();
    runtime_labels.insert("trace_id".to_string(), "trace-zhenfa-1".to_string());
    artifacts.push(CollectedArtifact {
        id: "runtime-trace-1".to_string(),
        kind: ArtifactKind::RuntimeTrace,
        path: Some(PathBuf::from("trace.jsonl")),
        content: json!({
            "provider": "zhenfa",
            "events": 4
        }),
        labels: runtime_labels,
    });

    let mut labels = std::collections::BTreeMap::new();
    labels.insert("session_id".to_string(), "session-42".to_string());

    AdvisoryAuditRequest {
        suite_id: "contracts".to_string(),
        pack_id: "rest_docs".to_string(),
        pack_version: "v1".to_string(),
        pack_domains: vec!["rest".to_string(), "documentation".to_string()],
        findings: vec![finding],
        artifacts,
        collection_context: CollectionContext {
            suite_id: "contracts".to_string(),
            crate_name: Some("xiuxian-wendao".to_string()),
            workspace_root: Some(workspace_root()),
            labels,
        },
        requested_roles: vec!["strict_teacher".to_string(), "artisan-engineer".to_string()],
    }
}

fn workspace_root() -> PathBuf {
    workspace::workspace_root()
}

fn must_ok<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

#[tokio::test]
async fn advisory_executor_builds_role_mix_and_snapshots() {
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = Arc::new(PersonaRegistry::with_builtins());
    let executor = QianjiAdvisoryAuditExecutor::new(orchestrator, registry);

    let plan = must_ok(
        executor.build_plan(&advisory_request()).await,
        "advisory plan should build",
    );

    assert_eq!(plan.role_mix.roles.len(), 2);
    assert_eq!(plan.role_mix.roles[0].role, "strict_teacher");
    assert_eq!(plan.role_mix.roles[1].role, "artisan-engineer");
    assert_eq!(plan.roles.len(), 2);
    for role in &plan.roles {
        must_ok(role.snapshot.validate(), "snapshot should validate");
        assert!(role.rendered_prompt.contains("<system_prompt_injection>"));
        assert_eq!(
            role.snapshot
                .role_mix
                .as_ref()
                .map(|mix| mix.profile_id.as_str()),
            Some(plan.role_mix.profile_id.as_str())
        );
    }
}

#[tokio::test]
async fn advisory_executor_exports_role_findings_with_trace_and_snapshot_metadata() {
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = Arc::new(PersonaRegistry::with_builtins());
    let executor = QianjiAdvisoryAuditExecutor::new(orchestrator, registry);

    let findings = must_ok(
        AdvisoryAuditExecutor::run(&executor, advisory_request()).await,
        "advisory executor should produce role findings",
    );

    assert_eq!(findings.len(), 2);
    for finding in findings {
        assert_eq!(finding.rule_id.as_deref(), Some("REST-R001"));
        assert_eq!(finding.trace_id.as_deref(), Some("trace-zhenfa-1"));
        assert_eq!(
            finding.labels.get("source_lane").map(String::as_str),
            Some("qianji_advisory")
        );
        assert!(finding.labels.contains_key("snapshot_id"));
        assert!(finding.evidence.iter().any(|evidence| {
            evidence.kind == EvidenceKind::RuntimeTrace
                && evidence.message.contains("trace-zhenfa-1")
        }));
        assert!(finding.evidence.iter().any(|evidence| {
            evidence.kind == EvidenceKind::DerivedInvariant
                && evidence
                    .locator
                    .as_deref()
                    .is_some_and(|locator| locator.contains("snapshot"))
        }));
    }
}
