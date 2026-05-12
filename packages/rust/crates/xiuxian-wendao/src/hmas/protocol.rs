//! `hmas::protocol` owns Wendao hmas protocol behavior.

use serde::{Deserialize, Serialize};

/// Supported HMAS blackboard record categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HmasRecordKind {
    /// Task assignment block.
    Task,
    /// Evidence collection block.
    Evidence,
    /// Final conclusion block.
    Conclusion,
    /// Digital thread (audit trail) block.
    DigitalThread,
}

impl HmasRecordKind {
    /// Parse record kind from a heading-style label (for example `[TASK]`).
    #[must_use]
    pub fn from_header(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        let open = trimmed.find('[')?;
        let close = trimmed[open + 1..].find(']')? + open + 1;
        Self::from_label(trimmed[open + 1..close].trim())
    }

    /// Parse record kind from markdown heading text.
    #[must_use]
    pub fn from_heading_text(raw: &str) -> Option<Self> {
        Self::from_header(raw)
    }

    /// Parse record kind from fenced-code tag (for example `json hmas_task`).
    #[must_use]
    pub fn from_fence_tag(raw: &str) -> Option<Self> {
        let normalized = raw.trim().to_uppercase().replace(['-', ' '], "_");
        match normalized.as_str() {
            "HMAS_TASK" => Some(Self::Task),
            "HMAS_EVIDENCE" => Some(Self::Evidence),
            "HMAS_CONCLUSION" => Some(Self::Conclusion),
            "HMAS_DIGITAL_THREAD" => Some(Self::DigitalThread),
            _ => None,
        }
    }

    fn from_label(raw: &str) -> Option<Self> {
        let label = raw.trim().to_uppercase().replace(['-', ' '], "_");
        match label.as_str() {
            "TASK" => Some(Self::Task),
            "EVIDENCE" => Some(Self::Evidence),
            "CONCLUSION" => Some(Self::Conclusion),
            "DIGITAL_THREAD" => Some(Self::DigitalThread),
            _ => None,
        }
    }

    /// Stable lowercase code used in diagnostics/logging.
    #[must_use]
    pub const fn as_code(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Evidence => "evidence",
            Self::Conclusion => "conclusion",
            Self::DigitalThread => "digital_thread",
        }
    }
}

/// One source node reference used by evidence/digital-thread payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HmasSourceNode {
    /// Canonical node identifier in `LinkGraph`.
    pub node_id: String,
    /// Optional saliency snapshot captured at read time.
    #[serde(default)]
    pub saliency_at_time: Option<f64>,
}

/// HMAS task block payload contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HmasTaskPayload {
    /// Correlation id shared across task/evidence/conclusion/thread blocks.
    pub requirement_id: String,
    /// Human-readable objective description.
    pub objective: String,
    /// Non-negotiable constraints enforced by manager/validator.
    #[serde(default)]
    pub hard_constraints: Vec<String>,
    /// Optional assignee label for worker routing.
    #[serde(default)]
    pub assigned_to: Option<String>,
}

/// HMAS evidence block payload contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HmasEvidencePayload {
    /// Correlation id shared with the task.
    pub requirement_id: String,
    /// Evidence statement content.
    pub evidence: String,
    /// Optional list of source nodes accessed while producing evidence.
    #[serde(default)]
    pub source_nodes_accessed: Vec<HmasSourceNode>,
}

/// HMAS conclusion block payload contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HmasConclusionPayload {
    /// Correlation id shared with task/evidence/thread.
    pub requirement_id: String,
    /// Final summarized conclusion.
    pub summary: String,
    /// Confidence score in `[0.0, 1.0]`.
    pub confidence_score: f64,
    /// Constraints confirmed during final synthesis.
    #[serde(default)]
    pub hard_constraints_checked: Vec<String>,
}

/// HMAS digital-thread block payload contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HmasDigitalThreadPayload {
    /// Correlation id shared with task/evidence/conclusion.
    pub requirement_id: String,
    /// Source nodes referenced during execution.
    pub source_nodes_accessed: Vec<HmasSourceNode>,
    /// Constraints checked during execution.
    pub hard_constraints_checked: Vec<String>,
    /// Confidence score in `[0.0, 1.0]`.
    pub confidence_score: f64,
}
