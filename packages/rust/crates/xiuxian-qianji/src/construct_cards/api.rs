//! Api surface for `xiuxian-qianji`.

use serde::Serialize;

/// Lifecycle status for a construct-card contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructStatus {
    /// Stable enough for downstream compilers to depend on.
    Stable,
    /// Available as guidance while the contract is still being hardened.
    Draft,
}

impl ConstructStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Draft => "draft",
        }
    }
}

/// One diagnostic mapping for a construct card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConstructLintMapping {
    /// Diagnostic code emitted by qianji lint or an aligned host/runtime check.
    pub diagnostic: &'static str,
    /// Human and LLM readable repair guidance.
    pub repair: &'static str,
}

/// One executable construct card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConstructCard {
    /// Stable construct id used by CLI and downstream compilers.
    pub id: &'static str,
    /// Short display title.
    pub title: &'static str,
    /// BPMN or DMN domain.
    pub domain: &'static str,
    /// Lifecycle status.
    pub status: ConstructStatus,
    /// Compact index summary.
    pub summary: &'static str,
    /// When an LLM should choose this construct.
    pub purpose: &'static str,
    /// Required preconditions or neighboring constructs.
    pub requires: &'static [&'static str],
    /// Supported bounded forms.
    pub allows: &'static [&'static str],
    /// Explicit anti-patterns.
    pub forbids: &'static [&'static str],
    /// Minimal BPMN or DMN scaffold for this construct.
    pub example: &'static str,
    /// Lint diagnostic repair hints connected to this construct.
    pub lint_mappings: &'static [ConstructLintMapping],
    /// Follow-up cards that are commonly useful with this card.
    pub next_cards: &'static [&'static str],
}

/// Compact machine-readable index entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConstructIndexEntry {
    /// Stable construct id.
    pub id: &'static str,
    /// BPMN or DMN domain.
    pub domain: &'static str,
    /// Lifecycle status.
    pub status: ConstructStatus,
    /// Compact summary.
    pub summary: &'static str,
}
