use serde::Deserialize;
use xiuxian_testing::{AdvisoryAuditRequest, FindingConfidence, FindingSeverity, RoleAuditFinding};

use super::QianjiAdvisoryRolePlan;
use super::runtime::LiveCognitiveMetrics;

#[derive(Debug, Deserialize)]
struct LiveRoleCritiquePayload {
    summary: Option<String>,
    why_it_matters: Option<String>,
    remediation: Option<String>,
    severity: Option<String>,
    confidence: Option<String>,
    evidence_excerpt: Option<String>,
    good_example: Option<String>,
    bad_example: Option<String>,
}

pub(super) fn live_advisory_instruction(
    request: &AdvisoryAuditRequest,
    role_plan: &QianjiAdvisoryRolePlan,
) -> String {
    let primary_title = request
        .findings
        .first()
        .map_or("contract review", |finding| finding.title.as_str());
    format!(
        "Review contract suite '{suite_id}' pack '{pack_id}' as role '{role_id}' ({persona_name}). \
Return one JSON object only with keys: summary, why_it_matters, remediation, severity, confidence, \
evidence_excerpt, good_example, bad_example. Use severity in [info, warning, error, critical] and \
confidence in [low, medium, high]. Focus on the primary issue '{primary_title}' and the evidence \
already provided in the system prompt. Do not wrap the JSON in markdown.",
        suite_id = request.suite_id,
        pack_id = request.pack_id,
        role_id = role_plan.role_id,
        persona_name = role_plan.persona_name,
    )
}

pub(super) fn apply_live_critique(
    finding: &mut RoleAuditFinding,
    critique_text: &str,
    cognitive_metrics: Option<LiveCognitiveMetrics>,
) {
    finding
        .labels
        .insert("execution_mode".to_string(), "live_llm".to_string());
    finding.push_message_evidence(format!("Live advisory critique: {}", critique_text.trim()));

    if let Some(payload) = parse_live_payload(critique_text) {
        if let Some(summary) = payload.summary.filter(|value| !value.trim().is_empty()) {
            finding.summary = summary;
        }
        if let Some(why_it_matters) = payload
            .why_it_matters
            .filter(|value| !value.trim().is_empty())
        {
            finding.why_it_matters = why_it_matters;
        }
        if let Some(remediation) = payload.remediation.filter(|value| !value.trim().is_empty()) {
            finding.remediation = remediation;
        }
        if let Some(severity) = payload.severity.as_deref().and_then(parse_severity) {
            finding.severity = severity;
        }
        if let Some(confidence) = payload.confidence.as_deref().and_then(parse_confidence) {
            finding.confidence = confidence;
        }
        if let Some(evidence_excerpt) = payload
            .evidence_excerpt
            .filter(|value| !value.trim().is_empty())
        {
            finding.push_message_evidence(evidence_excerpt);
        }
        if let Some(good_example) = payload
            .good_example
            .filter(|value| !value.trim().is_empty())
        {
            finding.examples.good.push(good_example);
        }
        if let Some(bad_example) = payload.bad_example.filter(|value| !value.trim().is_empty()) {
            finding.examples.bad.push(bad_example);
        }
    }

    if let Some(metrics) = cognitive_metrics {
        finding.labels.insert(
            "cognitive_coherence".to_string(),
            format!("{:.3}", metrics.coherence),
        );
        finding
            .labels
            .insert("cognitive_monitoring".to_string(), "enabled".to_string());
        if let Some(reason) = metrics.early_halt.as_ref() {
            finding
                .labels
                .insert("cognitive_early_halt".to_string(), "true".to_string());
            finding.push_message_evidence(reason.clone());
        }
        finding.push_message_evidence(format!(
            "Cognitive distribution meta={:.3}, operational={:.3}, epistemic={:.3}, instrumental={:.3}, balance={:.3}, uncertainty_ratio={:.3}",
            metrics.distribution.meta,
            metrics.distribution.operational,
            metrics.distribution.epistemic,
            metrics.distribution.instrumental,
            metrics.distribution.balance(),
            metrics.distribution.uncertainty_ratio(),
        ));
    }
}

fn parse_live_payload(critique_text: &str) -> Option<LiveRoleCritiquePayload> {
    serde_json::from_str::<LiveRoleCritiquePayload>(critique_text.trim())
        .ok()
        .or_else(|| {
            let start = critique_text.find('{')?;
            let end = critique_text.rfind('}')?;
            serde_json::from_str::<LiveRoleCritiquePayload>(&critique_text[start..=end]).ok()
        })
}

fn parse_severity(raw: &str) -> Option<FindingSeverity> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "info" => Some(FindingSeverity::Info),
        "warning" | "warn" => Some(FindingSeverity::Warning),
        "error" => Some(FindingSeverity::Error),
        "critical" => Some(FindingSeverity::Critical),
        _ => None,
    }
}

fn parse_confidence(raw: &str) -> Option<FindingConfidence> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "high" => Some(FindingConfidence::High),
        "medium" | "med" => Some(FindingConfidence::Medium),
        "low" => Some(FindingConfidence::Low),
        _ => None,
    }
}
