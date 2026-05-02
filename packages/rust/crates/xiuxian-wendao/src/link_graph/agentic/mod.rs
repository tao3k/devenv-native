//! Agentic graph proposal contracts, suggested-link logging, and decision audit persistence.

mod idempotency;
mod keys;
#[path = "store/mod.rs"]
mod store;
#[path = "types/mod.rs"]
mod types;

pub use idempotency::{suggested_link_signature_from_request, suggested_link_signature_from_row};
pub use store::{
    valkey_suggested_link_decide, valkey_suggested_link_decide_with_valkey,
    valkey_suggested_link_decisions_recent, valkey_suggested_link_decisions_recent_with_valkey,
    valkey_suggested_link_log, valkey_suggested_link_log_with_valkey, valkey_suggested_link_recent,
    valkey_suggested_link_recent_latest, valkey_suggested_link_recent_latest_with_valkey,
    valkey_suggested_link_recent_with_valkey,
};
pub use types::{
    LINK_GRAPH_SUGGESTED_LINK_DECISION_SCHEMA_VERSION, LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION,
    LinkGraphAgenticCandidatePair, LinkGraphAgenticExecutionConfig,
    LinkGraphAgenticExecutionResult, LinkGraphAgenticExpansionConfig,
    LinkGraphAgenticExpansionPlan, LinkGraphAgenticWorkerExecution, LinkGraphAgenticWorkerPhase,
    LinkGraphAgenticWorkerPlan, LinkGraphSuggestedLink, LinkGraphSuggestedLinkDecision,
    LinkGraphSuggestedLinkDecisionRequest, LinkGraphSuggestedLinkDecisionResult,
    LinkGraphSuggestedLinkRequest, LinkGraphSuggestedLinkState,
};
