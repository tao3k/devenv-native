//! Typed rows decoded from the `SearchStrategyFlow` Flight service.

/// Decoded `strategy_frontier` response row from `WendaoGraph`.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchStrategyFlowFrontierRow {
    /// Strategy flow identifier assigned by `WendaoGraph`.
    pub flow_id: String,
    /// Frontier row identifier.
    pub frontier_id: String,
    /// Candidate identifier in `source_path#anchor` form.
    pub candidate_id: String,
    /// Candidate revision identifier used by the Julia planner.
    pub revision_id: String,
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
    pub judgement_kind: String,
}

/// Decoded `strategy_candidates` response row from `WendaoGraph`.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchStrategyFlowServiceCandidateRow {
    /// Candidate identifier in `source_path#anchor` form.
    pub candidate_id: String,
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
    pub action_kind: String,
    /// Source candidate identifier.
    pub candidate_id: String,
    /// Target candidate identifier for compare/refine actions.
    pub target_candidate_id: String,
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
