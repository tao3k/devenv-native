use crate::integration_support::search_strategy_flow_candidates::SearchStrategyFlowCandidateInput;

use super::path::is_markdown_path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CandidateDiscoveryRequiredEvidence {
    pub(super) ownership_boundary: bool,
    pub(super) validation_path: bool,
    pub(super) relation_path: bool,
}

impl CandidateDiscoveryRequiredEvidence {
    pub(super) fn from_intent(intent: &str) -> Self {
        let normalized = intent.to_ascii_lowercase();
        let mentions_search_strategy_flow =
            has_text_terms(normalized.as_str(), &["search", "strategy", "flow"])
                || normalized.contains("searchstrategyflow");
        let mut required = Self {
            ownership_boundary: contains_any_text(
                normalized.as_str(),
                &[
                    "ownership",
                    "authority",
                    "boundary",
                    "governance",
                    "modularity",
                    "warning",
                    "warnings",
                    "debt",
                    "policy",
                    "auditor",
                    "agent",
                    "agents",
                    "studio",
                    "flight",
                    "materialization",
                ],
            ),
            validation_path: contains_any_text(
                normalized.as_str(),
                &[
                    "validation",
                    "validate",
                    "gate",
                    "test",
                    "tests",
                    "ci",
                    "proof",
                    "materialization",
                ],
            ),
            relation_path: contains_any_text(
                normalized.as_str(),
                &["relation", "link graph", "linkgraph"],
            ),
        };
        if mentions_search_strategy_flow && !required.has_required_bucket() {
            required = Self::all();
        }
        required
    }

    pub(super) const fn all() -> Self {
        Self {
            ownership_boundary: true,
            validation_path: true,
            relation_path: true,
        }
    }

    pub(super) const fn has_required_bucket(self) -> bool {
        self.ownership_boundary || self.validation_path || self.relation_path
    }

    pub(super) const fn bucket_count(self) -> usize {
        self.ownership_boundary as usize
            + self.validation_path as usize
            + self.relation_path as usize
    }

    pub(super) fn min_candidate_count(self) -> usize {
        (self.bucket_count() * 2).max(4)
    }

    pub(super) fn is_covered_by(self, candidates: &[SearchStrategyFlowCandidateInput]) -> bool {
        (!self.ownership_boundary
            || candidates
                .iter()
                .any(candidate_matches_ownership_boundary_evidence))
            && (!self.validation_path
                || candidates
                    .iter()
                    .any(candidate_matches_validation_path_evidence))
            && (!self.relation_path
                || candidates
                    .iter()
                    .any(candidate_matches_relation_path_evidence))
    }
}

pub(super) fn candidate_matches_ownership_boundary_evidence(
    candidate: &SearchStrategyFlowCandidateInput,
) -> bool {
    let path = candidate.relative_path.to_ascii_lowercase();
    let combined = candidate_discovery_combined_text(candidate);
    is_search_strategy_flow_owner_markdown_path(path.as_str())
        || is_governance_authority_markdown_path(path.as_str(), combined.as_str())
        || is_policy_authority_markdown_path(path.as_str(), combined.as_str())
}

pub(super) fn candidate_matches_validation_path_evidence(
    candidate: &SearchStrategyFlowCandidateInput,
) -> bool {
    let path = candidate.relative_path.to_ascii_lowercase();
    let combined = candidate_discovery_combined_text(candidate);
    is_validation_authority_markdown_path(path.as_str(), combined.as_str())
}

pub(super) fn candidate_matches_relation_path_evidence(
    candidate: &SearchStrategyFlowCandidateInput,
) -> bool {
    let path = candidate.relative_path.to_ascii_lowercase();
    let combined = candidate_discovery_combined_text(candidate);
    relation_evidence_path(path.as_str(), combined.as_str())
        || candidate
            .edge_kinds
            .iter()
            .any(|kind| has_relation_terms(kind.as_str()))
}

pub(super) fn is_search_strategy_flow_owner_markdown_path(path: &str) -> bool {
    is_markdown_path(path)
        && (path == "packages/rust/crates/xiuxian-julia-core/readme.md"
            || path.starts_with("packages/rust/crates/xiuxian-julia-core/docs/"))
}

pub(super) fn is_policy_authority_markdown_path(path: &str, combined: &str) -> bool {
    is_markdown_path(path)
        && ((path.starts_with("docs/rfcs/") && has_authority_terms(combined))
            || (path.starts_with("docs/01_core/wendao/")
                && has_search_strategy_terms(combined)
                && has_relation_terms(combined)))
}

pub(super) fn is_validation_authority_markdown_path(path: &str, combined: &str) -> bool {
    is_markdown_path(path)
        && ((path.starts_with("docs/testing/") && combined.contains("validation"))
            || is_audit_report_markdown_path(path)
            || combined.contains("evidence-calibration"))
}

pub(super) fn has_relation_terms(combined: &str) -> bool {
    combined.contains("linkgraph")
        || combined.contains("link graph")
        || combined.contains("relation")
}

fn relation_evidence_path(path: &str, combined: &str) -> bool {
    path.contains("link_graph")
        || path.contains("link-graph")
        || path.starts_with("docs/10_graph_compute/")
        || (path.contains("/wendaograph/search_strategy/") && has_relation_terms(combined))
}

fn candidate_discovery_combined_text(candidate: &SearchStrategyFlowCandidateInput) -> String {
    format!(
        "{} {} {}",
        candidate.relative_path.to_ascii_lowercase(),
        candidate.title.to_ascii_lowercase(),
        candidate.heading_anchor.to_ascii_lowercase()
    )
}

fn has_authority_terms(combined: &str) -> bool {
    combined.contains("ownership")
        || combined.contains("authority")
        || combined.contains("validation")
        || combined.contains("boundary")
}

fn has_search_strategy_terms(combined: &str) -> bool {
    combined.contains("searchstrategyflow")
        || combined.contains("search strategy")
        || combined.contains("pageindex")
        || combined.contains("page index")
}

fn has_text_terms(text: &str, terms: &[&str]) -> bool {
    terms.iter().all(|term| text.contains(term))
}

fn contains_any_text(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn is_governance_authority_markdown_path(path: &str, combined: &str) -> bool {
    is_markdown_path(path)
        && (path == "agents.md" || path.starts_with("docs/standards/"))
        && contains_any_text(
            combined,
            &[
                "governance",
                "modularity",
                "warning",
                "warnings",
                "debt",
                "policy",
                "auditor",
                "agent",
                "agents",
            ],
        )
}

fn is_audit_report_markdown_path(path: &str) -> bool {
    path.contains("-audit.")
        || path.contains("_audit.")
        || path.ends_with("-audit.md")
        || path.ends_with("_audit.md")
}
