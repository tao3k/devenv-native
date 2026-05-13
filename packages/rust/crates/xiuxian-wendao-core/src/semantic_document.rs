//! Semantic document and cognitive trace contracts for `LinkGraphIndex`.

use std::sync::Arc;

macro_rules! semantic_string_type {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Default)]
        pub struct $name(String);

        impl $name {
            /// Borrows the serialized value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&String> for $name {
            fn from(value: &String) -> Self {
                Self(value.to_owned())
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.as_str() == *other
            }
        }
    };
}

semantic_string_type!(SemanticAnchorId, "Stable semantic anchor identifier.");
semantic_string_type!(SemanticDocId, "Stable semantic document identifier.");
semantic_string_type!(
    SemanticDocumentPath,
    "Repository-relative semantic document path."
);
semantic_string_type!(CognitiveTraceId, "Stable cognitive trace identifier.");
semantic_string_type!(CognitiveSessionId, "Cognitive trace session identifier.");
semantic_string_type!(CognitiveNodeId, "Cognitive trace node identifier.");

/// Typed semantic document exported from `LinkGraphIndex` for downstream retrieval runtimes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkGraphSemanticDocument {
    /// Stable anchor identifier used to recover semantic paths.
    pub anchor_id: SemanticAnchorId,
    /// Canonical document identifier owning this semantic document.
    pub doc_id: SemanticDocId,
    /// Relative markdown path for traceability.
    pub path: SemanticDocumentPath,
    /// Semantic document kind used by downstream document-scope filters.
    pub kind: LinkGraphSemanticDocumentKind,
    /// Complete logical ancestry path recovered from `PageIndex`.
    pub semantic_path: Vec<String>,
    /// Text payload exported for semantic indexing.
    pub content: Arc<str>,
    /// Optional source line range when the document maps to one concrete section.
    pub line_range: Option<(usize, usize)>,
}

/// Semantic document kind exported from `LinkGraphIndex`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkGraphSemanticDocumentKind {
    /// One document-level summary row.
    Summary,
    /// One section-level semantic row derived from `PageIndex`.
    Section,
    /// Agent reasoning trace captured during workflow execution (V6.1 Sovereign Memory).
    CognitiveTrace,
}

impl LinkGraphSemanticDocumentKind {
    /// Return the canonical metadata label used by vector retrieval adapters.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::Section => "section",
            Self::CognitiveTrace => "cognitive_trace",
        }
    }
}

/// Cognitive trace artifact for sovereign memory (V6.1).
///
/// Represents a persistent reasoning trace that connects Intent -> Reasoning ->
/// Outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct CognitiveTraceRecord {
    /// Unique identifier for this trace.
    pub trace_id: CognitiveTraceId,
    /// Session identifier from Qianji execution.
    pub session_id: Option<CognitiveSessionId>,
    /// Node identifier from the compiled flow graph.
    pub node_id: CognitiveNodeId,
    /// The original user intent/prompt.
    pub intent: String,
    /// Aggregated reasoning content (thoughts + text deltas).
    pub reasoning: Arc<str>,
    /// Final outcome or conclusion.
    pub outcome: Option<Arc<str>>,
    /// Associated commit hash if the trace led to code changes.
    pub commit_sha: Option<String>,
    /// Timestamp when the trace was captured.
    pub timestamp_ms: u64,
    /// Cognitive coherence score during execution.
    pub coherence_score: Option<f32>,
    /// Whether early halt was triggered.
    pub early_halt_triggered: bool,
}

impl CognitiveTraceRecord {
    /// Create a new cognitive trace record.
    #[must_use]
    pub fn new(
        trace_id: impl Into<CognitiveTraceId>,
        session_id: Option<CognitiveSessionId>,
        node_id: impl Into<CognitiveNodeId>,
        intent: String,
    ) -> Self {
        Self {
            trace_id: trace_id.into(),
            session_id,
            node_id: node_id.into(),
            intent,
            reasoning: Arc::<str>::from(""),
            outcome: None,
            commit_sha: None,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .try_into()
                .unwrap_or(0),
            coherence_score: None,
            early_halt_triggered: false,
        }
    }

    /// Convert to a semantic document for Wendao ingestion.
    #[must_use]
    pub fn to_semantic_document(
        &self,
        doc_id: impl Into<SemanticDocId>,
        path: impl Into<SemanticDocumentPath>,
    ) -> LinkGraphSemanticDocument {
        LinkGraphSemanticDocument {
            anchor_id: format!("trace:{}", self.trace_id.as_str()).into(),
            doc_id: doc_id.into(),
            path: path.into(),
            kind: LinkGraphSemanticDocumentKind::CognitiveTrace,
            semantic_path: vec![
                "Cognitive Traces".to_string(),
                self.node_id.as_str().to_string(),
            ],
            content: self.reasoning.clone(),
            line_range: None,
        }
    }
}
