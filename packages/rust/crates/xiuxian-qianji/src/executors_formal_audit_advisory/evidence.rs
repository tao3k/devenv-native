use std::collections::BTreeMap;

use crate::contract_feedback::{
    AdvisoryAuditRequest, ArtifactKind, ContractFinding, EvidenceKind, FindingEvidence,
};

use super::QianjiAdvisoryRolePlan;
use super::prompt_context::RoleMixProfile;

pub(super) fn primary_finding(findings: &[ContractFinding]) -> Option<ContractFinding> {
    findings
        .iter()
        .cloned()
        .max_by_key(|finding| finding.severity)
}

pub(super) fn primary_trace_id(request: &AdvisoryAuditRequest) -> Option<String> {
    request
        .findings
        .iter()
        .find_map(|finding| finding.trace_ids.first().cloned())
        .or_else(|| {
            request
                .artifacts
                .artifacts
                .iter()
                .find(|artifact| artifact.kind == ArtifactKind::RuntimeTrace)
                .and_then(|artifact| {
                    artifact
                        .labels
                        .get("trace_id")
                        .cloned()
                        .or_else(|| Some(artifact.id.clone()))
                })
        })
}

pub(super) fn advisory_summary(
    role_plan: &QianjiAdvisoryRolePlan,
    finding_count: usize,
    primary_finding: Option<&ContractFinding>,
) -> String {
    let focus = primary_finding.map_or("contract review preparation", |finding| {
        finding.title.as_str()
    });
    format!(
        "{} prepared advisory review for {} deterministic finding(s); primary focus: {}.",
        role_plan.persona_name, finding_count, focus
    )
}

pub(super) fn advisory_labels(
    request: &AdvisoryAuditRequest,
    role_mix: &RoleMixProfile,
    role_plan: &QianjiAdvisoryRolePlan,
) -> BTreeMap<String, String> {
    let mut labels = BTreeMap::new();
    labels.insert("source_lane".to_string(), "qianji_advisory".to_string());
    labels.insert("suite_id".to_string(), request.suite_id.clone());
    labels.insert("pack_id".to_string(), request.pack_id.clone());
    labels.insert("pack_version".to_string(), request.pack_version.clone());
    labels.insert("persona_name".to_string(), role_plan.persona_name.clone());
    labels.insert(
        "snapshot_id".to_string(),
        role_plan.snapshot.snapshot_id.as_ref().to_string(),
    );
    labels.insert(
        "role_mix_profile_id".to_string(),
        role_mix.profile_id.clone(),
    );
    labels.insert(
        "prompt_chars".to_string(),
        role_plan.rendered_prompt.chars().count().to_string(),
    );
    labels
}

pub(super) fn pack_summary(request: &AdvisoryAuditRequest) -> String {
    let domains = if request.pack_domains.is_empty() {
        "none".to_string()
    } else {
        request.pack_domains.join(", ")
    };
    format!(
        "Contract suite: {}\nPack: {}@{}\nDomains: {}\nCrate: {}",
        request.suite_id,
        request.pack_id,
        request.pack_version,
        domains,
        request
            .collection_context
            .crate_name
            .as_deref()
            .unwrap_or("unknown")
    )
}

pub(super) fn findings_summary(findings: &[ContractFinding]) -> String {
    if findings.is_empty() {
        return "No deterministic contract findings were provided.".to_string();
    }

    findings
        .iter()
        .map(|finding| {
            format!(
                "- [{:?}] {}: {}",
                finding.severity, finding.title, finding.summary
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn primary_finding_summary(finding: &ContractFinding) -> String {
    let why_it_matters = if finding.why_it_matters.trim().is_empty() {
        finding.summary.as_str()
    } else {
        finding.why_it_matters.as_str()
    };
    format!(
        "Primary contract focus: {}\nWhy it matters: {}\nSuggested remediation: {}",
        finding.title,
        why_it_matters,
        if finding.remediation.trim().is_empty() {
            "No remediation provided."
        } else {
            finding.remediation.as_str()
        }
    )
}

pub(super) fn runtime_trace_artifact_summary(request: &AdvisoryAuditRequest) -> String {
    let runtime_artifacts = request
        .artifacts
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == ArtifactKind::RuntimeTrace)
        .collect::<Vec<_>>();

    if runtime_artifacts.is_empty() {
        return String::new();
    }

    runtime_artifacts
        .iter()
        .map(|artifact| {
            let trace_id = artifact
                .labels
                .get("trace_id")
                .map_or(artifact.id.as_str(), String::as_str);
            format!("Runtime trace available: {trace_id}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn runtime_trace_evidence(request: &AdvisoryAuditRequest) -> Vec<FindingEvidence> {
    request
        .artifacts
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == ArtifactKind::RuntimeTrace)
        .map(|artifact| FindingEvidence {
            kind: EvidenceKind::RuntimeTrace,
            path: artifact.path.clone(),
            locator: Some(artifact.id.clone()),
            message: artifact.labels.get("trace_id").map_or_else(
                || {
                    format!(
                        "Runtime trace artifact '{}' is available for advisory review.",
                        artifact.id
                    )
                },
                |trace_id| format!("Runtime trace available for advisory review: {trace_id}"),
            ),
        })
        .collect()
}

pub(super) fn role_mix_profile_id(request: &AdvisoryAuditRequest) -> String {
    format!(
        "contract-audit:{}:{}",
        sanitize_identifier(request.suite_id.as_str()),
        sanitize_identifier(request.pack_id.as_str())
    )
}

pub(super) fn snapshot_id(request: &AdvisoryAuditRequest, role_id: &str) -> String {
    format!(
        "{}:{}:{}",
        role_mix_profile_id(request),
        sanitize_identifier(role_id),
        "snapshot"
    )
}

pub(super) fn sanitize_identifier(raw: &str) -> String {
    let mut sanitized = String::with_capacity(raw.len());
    for character in raw.chars() {
        if character.is_ascii_alphanumeric() {
            sanitized.push(character.to_ascii_lowercase());
        } else {
            sanitized.push('-');
        }
    }
    sanitized
}
