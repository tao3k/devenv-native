use std::collections::{HashMap, HashSet};

use super::token::{normalized_text, normalized_words, ratio, token_set_has_match};
use crate::orgize::read_model::model::AgentOrgTaskListRow;
use crate::orgize::read_model::section_lens::TaskSectionLens;

pub(super) struct RecallCandidate<'a> {
    pub(super) row: &'a AgentOrgTaskListRow,
    pub(super) lens: Option<TaskSectionLens>,
}

#[derive(Debug, Clone)]
pub(super) struct QueryEvidence {
    pub(super) raw: String,
    pub(super) normalized: String,
    pub(super) tokens: Vec<String>,
}

impl QueryEvidence {
    pub(super) fn from_query(query: &str) -> Self {
        Self {
            raw: query.trim().to_string(),
            normalized: normalized_text(query),
            tokens: normalized_words(query),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.raw.trim().is_empty()
    }
}

pub(super) struct CandidateEvidence {
    pub(super) text: String,
    pub(super) normalized: String,
    pub(super) tokens: HashSet<String>,
    pub(super) facets: Vec<OrgEvidenceFacet>,
}

pub(super) struct OrgEvidenceFacet {
    pub(super) kind: OrgEvidenceFacetKind,
    pub(super) text: String,
    pub(super) tokens: HashSet<String>,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum OrgEvidenceFacetKind {
    Identity,
    Heading,
    Lifecycle,
    Source,
    Tags,
    Planning,
    Property,
    MemoryFinality,
    MemoryClaim,
    MemoryEvidence,
    MemoryFailure,
    MemoryPreference,
    NextAction,
    Graph,
    Progress,
    Checklist,
    ChildHeadings,
}

impl OrgEvidenceFacetKind {
    pub(super) const COUNT: usize = 17;

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Heading => "heading",
            Self::Lifecycle => "lifecycle",
            Self::Source => "source",
            Self::Tags => "tags",
            Self::Planning => "planning",
            Self::Property => "property",
            Self::MemoryFinality => "memory-finality",
            Self::MemoryClaim => "memory-claim",
            Self::MemoryEvidence => "memory-evidence",
            Self::MemoryFailure => "memory-failure",
            Self::MemoryPreference => "memory-preference",
            Self::NextAction => "next-action",
            Self::Graph => "graph",
            Self::Progress => "progress",
            Self::Checklist => "checklist",
            Self::ChildHeadings => "child-headings",
        }
    }
}

pub(super) struct EvidenceCorpus {
    pub(super) document_frequency: HashMap<String, usize>,
    pub(super) document_count: usize,
}

impl EvidenceCorpus {
    pub(super) fn from_windows(query: &QueryEvidence, windows: &[CandidateEvidence]) -> Self {
        let mut document_frequency = HashMap::<String, usize>::new();
        for token in &query.tokens {
            let count = windows
                .iter()
                .filter(|window| token_set_has_match(&window.tokens, token))
                .count();
            document_frequency.insert(token.clone(), count);
        }
        Self {
            document_frequency,
            document_count: windows.len().max(1),
        }
    }

    pub(super) fn token_information(&self, token: &str) -> f32 {
        let frequency = self
            .document_frequency
            .get(token)
            .copied()
            .unwrap_or_default();
        if frequency == 0 {
            return 1.0;
        }
        let commonality = ratio(frequency, self.document_count);
        (1.0 - commonality).max(0.05)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RecallScore {
    pub(super) identity: bool,
    pub(super) phrase: bool,
    pub(super) token_coverage: f32,
    pub(super) facet_matches: usize,
    pub(super) recovery_anchor_coverage: f32,
    pub(super) facet_coverage: f32,
    pub(super) facet_signal: f32,
    pub(super) lifecycle: f32,
    pub(super) recency: f32,
    pub(super) memory_score: f32,
}

impl RecallScore {
    pub(super) fn with_memory_score(mut self, memory_score: f32) -> Self {
        self.memory_score = memory_score;
        self
    }

    pub(super) fn utility_value(self) -> f32 {
        let identity: f32 = if self.identity { 1.0 } else { 0.0 };
        let phrase: f32 = if self.phrase { 0.85 } else { 0.0 };
        (identity
            .max(phrase)
            .max(self.token_coverage)
            .max(self.recovery_anchor_coverage)
            .max(self.facet_coverage)
            .clamp(0.0, 1.0)
            + (ratio(self.facet_matches, OrgEvidenceFacetKind::COUNT) * 0.08)
            + self.facet_signal
            + self.lifecycle
            + self.recency)
            .clamp(0.0, 1.0)
    }

    pub(super) fn rank_value(self) -> f32 {
        self.utility_value().max(self.memory_score).clamp(0.0, 1.0)
    }

    pub(super) fn has_query_evidence(self) -> bool {
        self.identity
            || self.phrase
            || self.token_coverage > 0.0
            || self.facet_matches > 0
            || self.recovery_anchor_coverage > 0.0
            || self.facet_coverage > 0.0
    }
}

pub(super) struct RankedCandidate<'a, 'b> {
    pub(super) index: usize,
    pub(super) score: RecallScore,
    pub(super) candidate: &'b RecallCandidate<'a>,
}
