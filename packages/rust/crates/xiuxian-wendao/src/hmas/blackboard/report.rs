//! `hmas::blackboard::report` owns Wendao hmas blackboard report behavior.

use crate::hmas::protocol::HmasRecordKind;
use serde::{Deserialize, Serialize};

/// One HMAS validation issue item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HmasValidationIssue {
    /// 1-based line number in markdown input.
    pub line: usize,
    /// Stable issue code for programmatic checks.
    pub code: String,
    /// Human-readable validation message.
    pub message: String,
    /// Optional record kind associated with this issue.
    #[serde(default)]
    pub kind: Option<HmasRecordKind>,
}

/// Aggregate HMAS validation report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HmasValidationReport {
    /// Whether validation succeeded with zero issues.
    pub valid: bool,
    /// Number of parsed task blocks.
    pub task_count: usize,
    /// Number of parsed evidence blocks.
    pub evidence_count: usize,
    /// Number of parsed conclusion blocks.
    pub conclusion_count: usize,
    /// Number of parsed digital-thread blocks.
    pub digital_thread_count: usize,
    /// Collected validation issues.
    pub issues: Vec<HmasValidationIssue>,
}

impl HmasValidationReport {
    /// Build an initially valid empty report.
    #[must_use]
    pub fn ok() -> Self {
        Self {
            valid: true,
            task_count: 0,
            evidence_count: 0,
            conclusion_count: 0,
            digital_thread_count: 0,
            issues: Vec::new(),
        }
    }

    pub(super) fn push_issue(
        &mut self,
        line: usize,
        code: &str,
        message: impl Into<String>,
        kind: Option<HmasRecordKind>,
    ) {
        self.valid = false;
        self.issues.push(HmasValidationIssue {
            line,
            code: code.to_string(),
            message: message.into(),
            kind,
        });
    }
}
