//! Audit verdict and result types.

use serde::{Deserialize, Serialize};

/// Audit verdict returned by alignment evaluation.
#[derive(Debug, Clone)]
pub struct AuditVerdict {
    /// Drift score in [0, 1], where lower is better aligned.
    pub drift_score: f32,
    /// Whether the evaluated content is considered aligned.
    pub is_aligned: bool,
    /// Missing anchor terms detected during evidence scan.
    pub missing_anchors: Vec<String>,
}

/// Namespace boundary: this public name is scoped by its module owner.
/// Result of audit evaluation with optional compensation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    /// CCS score achieved (0.0-1.0).
    pub ccs_score: f64,
    /// Whether the result passed the threshold.
    pub passed: bool,
    /// Missing anchors that triggered failure.
    pub missing_anchors: Vec<String>,
    /// Suggested compensation parameters (if failed).
    pub compensation: Option<super::CompensationRequest>,
}
