//! `zhenfa_router::native::sentinel::types` owns Wendao native sentinel types behavior.

use chrono;
use serde::{Deserialize, Serialize};

/// A signal indicating that source code changes may affect documentation.
///
/// This struct captures the relationship between a changed source file and
/// documents that contain `:OBSERVE:` patterns potentially referencing it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDriftSignal {
    /// The source file that changed.
    pub source_path: String,
    /// File stem used for heuristic matching.
    pub file_stem: String,
    /// Documents with observations that may reference this source.
    pub affected_docs: Vec<AffectedDoc>,
    /// Confidence level of the drift detection.
    pub confidence: DriftConfidence,
    /// Timestamp of the detection.
    pub timestamp: String,
}

/// A document potentially affected by source code changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedDoc {
    /// Document ID (stem or full path).
    pub doc_id: String,
    /// The observation pattern that matched the source file.
    pub matching_pattern: String,
    /// Language of the observation.
    pub language: String,
    /// Line number of the observation in the document.
    pub line_number: Option<usize>,
    /// Node ID where the observation was found.
    pub node_id: String,
}

/// Confidence level for drift detection.
///
/// Ordered: `Low < Medium < High` for comparison operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DriftConfidence {
    /// Low confidence: fuzzy heuristic match only.
    Low,
    /// Medium confidence: pattern contains related keywords.
    Medium,
    /// High confidence: pattern explicitly references the file/symbol.
    High,
}

impl std::fmt::Display for DriftConfidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

impl SemanticDriftSignal {
    /// Create a new semantic drift signal.
    #[must_use]
    pub fn new(source_path: impl Into<String>, file_stem: impl Into<String>) -> Self {
        let timestamp = chrono::Utc::now().to_rfc3339();
        Self {
            source_path: source_path.into(),
            file_stem: file_stem.into(),
            affected_docs: Vec::new(),
            confidence: DriftConfidence::Low,
            timestamp,
        }
    }

    /// Add an affected document to the signal.
    pub fn add_affected_doc(&mut self, doc: AffectedDoc) {
        self.affected_docs.push(doc);
    }

    /// Update confidence based on match quality.
    pub fn update_confidence(&mut self, confidence: DriftConfidence) {
        self.confidence = confidence;
    }

    /// Generate a human-readable summary.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Semantic drift in '{}' may affect {} doc(s): {}",
            self.file_stem,
            self.affected_docs.len(),
            self.affected_docs
                .iter()
                .map(|d| d.doc_id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    /// Convert to streaming event payload.
    #[must_use]
    pub fn to_streaming_payload(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

impl AffectedDoc {
    /// Create a new affected document record.
    #[must_use]
    pub fn new(
        doc_id: impl Into<String>,
        matching_pattern: impl Into<String>,
        language: impl Into<String>,
        node_id: impl Into<String>,
    ) -> Self {
        Self {
            doc_id: doc_id.into(),
            matching_pattern: matching_pattern.into(),
            language: language.into(),
            line_number: None,
            node_id: node_id.into(),
        }
    }

    /// Set the line number.
    #[must_use]
    pub fn with_line(mut self, line: usize) -> Self {
        self.line_number = Some(line);
        self
    }
}
