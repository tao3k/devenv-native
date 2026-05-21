//! Core CCS (Context Completeness Score) calculation.
//!
//! The CCS measures how well the retrieved evidence aligns with
//! expected style anchors from the persona profile.

use super::CompensationRequest;
use super::verdict::AuditVerdict;
use crate::zhenfa_router::native::audit::AuditResult;

/// CCS threshold for accepting search results without compensation.
pub const CCS_THRESHOLD: f64 = 0.70;

/// Synapse-Audit alignment threshold (drift < 0.05 means CCS > 0.95).
const SYNAPSE_AUDIT_THRESHOLD: f32 = 0.05;

/// Evaluate alignment between search evidence and style anchors.
///
/// This implements the core CCS calculation:
/// ```text
/// CCS = 1.0 - drift_score
/// drift_score = 1.0 - (matches / total_anchors)
/// ```
///
/// # Arguments
/// * `anchors` - Required style anchors from persona profile
/// * `evidence` - Extracted text from search hits (stems, summaries)
///
/// # Returns
/// `AuditVerdict` with drift score and missing anchors
#[must_use]
pub fn evaluate_alignment(anchors: &[String], evidence: &[String]) -> AuditVerdict {
    if anchors.is_empty() {
        return AuditVerdict {
            drift_score: 0.0,
            is_aligned: true,
            missing_anchors: Vec::new(),
        };
    }

    let missing = anchors
        .iter()
        .filter(|anchor| !evidence_contains_anchor(evidence, anchor))
        .cloned()
        .collect::<Vec<_>>();
    let matches = anchors.len().saturating_sub(missing.len());

    let drift = 1.0 - bounded_ratio(matches, anchors.len()).max(0.0);

    AuditVerdict {
        drift_score: drift,
        is_aligned: drift < SYNAPSE_AUDIT_THRESHOLD,
        missing_anchors: missing,
    }
}

fn evidence_contains_anchor(evidence: &[String], anchor: &str) -> bool {
    let anchor_lower = anchor.to_lowercase();
    evidence
        .iter()
        .any(|candidate| candidate.to_lowercase().contains(&anchor_lower))
}

/// Audit search payload against provided anchors.
///
/// # Arguments
/// * `evidence` - Extracted text from search hits (stems, summaries)
/// * `anchors` - Required style anchors from persona profile
///
/// # Returns
/// `AuditResult` with CCS score and optional compensation request
#[must_use]
pub fn audit_search_payload(evidence: &[String], anchors: &[String]) -> AuditResult {
    let verdict = evaluate_alignment(anchors, evidence);
    let ccs_score = 1.0 - f64::from(verdict.drift_score);

    AuditResult {
        ccs_score,
        passed: ccs_score >= CCS_THRESHOLD,
        missing_anchors: verdict.missing_anchors.clone(),
        compensation: if ccs_score < CCS_THRESHOLD {
            Some(CompensationRequest {
                max_distance_delta: 1,
                related_limit_delta: 5,
            })
        } else {
            None
        },
    }
}

fn bounded_ratio(numerator: usize, denominator: usize) -> f32 {
    let numerator = bounded_usize_to_f32(numerator);
    let denominator = bounded_usize_to_f32(denominator);
    numerator / denominator
}

fn bounded_usize_to_f32(value: usize) -> f32 {
    u16::try_from(value).map_or(f32::from(u16::MAX), f32::from)
}

#[cfg(test)]
#[path = "../../../../tests/unit/zhenfa_router/native/audit/ccs.rs"]
mod tests;
