//! Public dmn model reference contracts for BPMN/DMN engine integration.

use std::sync::Arc;

/// Future DMN binding kind associated with one BPMN node or evaluation request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmnBindingKind {
    /// A logical decision identifier reference.
    DecisionRef,
}

/// In-memory DMN source input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmnSourceFile {
    /// Source identifier used for diagnostics.
    pub source_id: String,
    /// Raw XML or DMN content.
    pub contents: String,
}

impl DmnSourceFile {
    /// Creates a DMN source input.
    #[must_use]
    pub fn new(source_id: impl Into<String>, contents: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            contents: contents.into(),
        }
    }
}

/// Placeholder link from BPMN to a future DMN decision artifact.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnDecisionRef {
    /// Stable decision identifier.
    pub decision_id: Arc<str>,
    /// Optional source or namespace identifier.
    pub source_id: Option<Arc<str>>,
    /// Binding kind used for the reference.
    pub binding: DmnBindingKind,
}

impl DmnDecisionRef {
    /// Creates one decision reference placeholder.
    #[must_use]
    pub fn new(decision_id: impl AsRef<str>) -> Self {
        Self {
            decision_id: Arc::<str>::from(decision_id.as_ref()),
            source_id: None,
            binding: DmnBindingKind::DecisionRef,
        }
    }

    /// Adds an optional source identifier.
    #[must_use]
    pub fn with_source_id(mut self, source_id: impl AsRef<str>) -> Self {
        self.source_id = Some(Arc::<str>::from(source_id.as_ref()));
        self
    }
}
