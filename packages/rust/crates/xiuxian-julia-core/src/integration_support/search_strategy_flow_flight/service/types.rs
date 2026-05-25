//! Typed rows decoded from the `SearchStrategyFlow` Flight service.

use serde::Serialize;
use std::fmt;

macro_rules! string_newtype {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Borrow the stable string value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consume the wrapper and return the stable string value.
            #[must_use]
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
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

        impl PartialEq<$name> for &str {
            fn eq(&self, other: &$name) -> bool {
                *self == other.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

string_newtype!(SearchStrategyFlowActionKind);
string_newtype!(SearchStrategyFlowCandidateId);
string_newtype!(SearchStrategyFlowFlowId);
string_newtype!(SearchStrategyFlowFrontierId);
string_newtype!(SearchStrategyFlowJudgementKind);
string_newtype!(SearchStrategyFlowRevisionId);

/// Decoded `strategy_frontier` response row from `WendaoGraph`.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchStrategyFlowFrontierRow {
    /// Strategy flow identifier assigned by `WendaoGraph`.
    pub flow_id: SearchStrategyFlowFlowId,
    /// Frontier row identifier.
    pub frontier_id: SearchStrategyFlowFrontierId,
    /// Candidate identifier in `source_path#anchor` form.
    pub candidate_id: SearchStrategyFlowCandidateId,
    /// Candidate revision identifier used by the Julia planner.
    pub revision_id: SearchStrategyFlowRevisionId,
    /// Frontier rank emitted by the strategy flow.
    pub rank: i64,
    /// Whether this candidate is selected for the frontier.
    pub selected: bool,
    /// Final Julia strategy score.
    pub final_score: f64,
    /// Planner action associated with this frontier row.
    pub action: String,
    /// Context budget consumed by selected rows.
    pub context_budget: i64,
    /// Judgement bucket assigned by the strategy flow.
    pub judgement_kind: SearchStrategyFlowJudgementKind,
}

/// Decoded `strategy_candidates` response row from `WendaoGraph`.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchStrategyFlowServiceCandidateRow {
    /// Candidate identifier in `source_path#anchor` form.
    pub candidate_id: SearchStrategyFlowCandidateId,
    /// Candidate action assigned by the strategy flow.
    pub action: String,
    /// Candidate action reason.
    pub reason: String,
    /// Final Julia strategy score.
    pub final_score: f64,
    /// Evidence coverage score.
    pub evidence_coverage: f64,
    /// Graph score.
    pub graph_score: f64,
    /// Authority score.
    pub authority_score: f64,
    /// Semantic score.
    pub semantic_score: f64,
    /// Structural score.
    pub structural_score: f64,
    /// Context cost.
    pub context_cost: i64,
    /// Whether the candidate was blocked by a guard.
    pub blocked: bool,
}

/// Decoded `strategy_planner_actions` response row from `WendaoGraph`.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchStrategyFlowServicePlannerActionRow {
    /// Planner action kind.
    pub action_kind: SearchStrategyFlowActionKind,
    /// Source candidate identifier.
    pub candidate_id: SearchStrategyFlowCandidateId,
    /// Target candidate identifier for compare/refine actions.
    pub target_candidate_id: SearchStrategyFlowCandidateId,
    /// Whether the action allows another strategy loop.
    pub cycle_allowed: bool,
    /// Whether the action requires LLM judgement.
    pub requires_llm_judgement: bool,
    /// Planner action score.
    pub score: f64,
    /// Context budget attached to the action.
    pub context_budget: i64,
    /// Planner action reason.
    pub reason: String,
}

/// Decoded `SearchStrategyFlow` response bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchStrategyFlowServiceResponse {
    /// Candidate scoring rows.
    pub candidates: Vec<SearchStrategyFlowServiceCandidateRow>,
    /// Transition row count.
    pub transition_count: usize,
    /// Frontier rows.
    pub frontier: Vec<SearchStrategyFlowFrontierRow>,
    /// Planner action rows.
    pub planner_actions: Vec<SearchStrategyFlowServicePlannerActionRow>,
}

/// Result of one negotiated `SearchStrategyFlow` Flight service roundtrip.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchStrategyFlowServiceRoundtrip {
    /// Runtime Flight route selected for the exchange.
    pub flight_route: String,
    /// Decoded response bundle returned by `WendaoGraph`.
    pub response: SearchStrategyFlowServiceResponse,
    /// Decoded frontier rows returned by `WendaoGraph`.
    pub rows: Vec<SearchStrategyFlowFrontierRow>,
}
