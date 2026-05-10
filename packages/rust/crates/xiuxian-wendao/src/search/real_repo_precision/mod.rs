//! Opt-in real-repository search precision harness.

#[path = "catalog.rs"]
mod catalog;
#[path = "evaluate.rs"]
mod evaluate;
#[path = "frontier.rs"]
mod frontier;
#[path = "harness.rs"]
mod harness;
#[path = "intent_frame.rs"]
mod intent_frame;
#[path = "receipt.rs"]
mod receipt;
#[path = "scenario_matrix.rs"]
mod scenario_matrix;
#[cfg(feature = "julia")]
#[path = "semantic_gate.rs"]
mod semantic_gate;
#[path = "types.rs"]
mod types;

pub(crate) use catalog::default_real_repo_precision_catalog;
pub(crate) use evaluate::evaluate_gold_query_paths;
pub(crate) use harness::{
    run_real_repo_precision_harness, run_real_repo_precision_harness_with_options,
};
pub(crate) use scenario_matrix::evaluate_knowledge_scenario_matrix;
#[cfg(feature = "julia")]
pub(crate) use semantic_gate::{
    RealRepoMarkdownKnowledgeSemanticGateEvaluation,
    attach_markdown_knowledge_semantic_query_evidence, evaluate_markdown_knowledge_semantic_gate,
};
#[cfg(feature = "julia")]
pub(crate) use types::MARKDOWN_SSOT_PROOF_ENV;
pub(crate) use types::{
    DOCS_CORPUS_PROOF_ENV, PREWARM_PROOF_ENV, RESIDENT_PROOF_ENV, RealRepoGoldQuery,
    RealRepoGoldQueryKind, RealRepoKnowledgeScenario,
    RealRepoKnowledgeScenarioAuthorityExpectation, RealRepoKnowledgeScenarioKind,
    RealRepoKnowledgeScenarioQueryVariant, RealRepoKnowledgeScenarioQueryVariantKind,
    RealRepoKnowledgeScenarioReceipt, RealRepoMarkdownKnowledgeSemanticGateReceipt,
    RealRepoMarkdownKnowledgeSemanticRelationPathReceipt, RealRepoPrecisionCatalogEntry,
    RealRepoPrecisionQueryReceipt, RealRepoPrecisionRunOptions, RealRepoPrecisionRunStatus,
    RealRepoPrecisionSyncMode,
};

#[cfg(test)]
#[path = "../../../tests/unit/search/real_repo_precision/mod.rs"]
mod tests;
