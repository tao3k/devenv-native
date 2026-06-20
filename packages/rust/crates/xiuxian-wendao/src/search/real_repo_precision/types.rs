use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::analyzers::RegisteredRepository;

pub(crate) const RUN_ENV: &str = "RUN_WENDAO_REAL_REPO_SEARCH_PRECISION_TEST";
pub(crate) const SYNC_MODE_ENV: &str = "WENDAO_REAL_REPO_SEARCH_PRECISION_SYNC_MODE";
pub(crate) const QUERY_KIND_ENV: &str = "WENDAO_REAL_REPO_SEARCH_PRECISION_QUERY_KIND";
pub(crate) const RESIDENT_PROOF_ENV: &str = "WENDAO_REAL_REPO_SEARCH_PRECISION_RESIDENT_PROOF";
pub(crate) const PREWARM_PROOF_ENV: &str = "WENDAO_REAL_REPO_SEARCH_PRECISION_PREWARM_PROOF";
#[cfg(feature = "julia")]
pub(crate) const MARKDOWN_SSOT_PROOF_ENV: &str =
    "WENDAO_REAL_REPO_SEARCH_PRECISION_MARKDOWN_SSOT_PROOF";
pub(crate) const DOCS_CORPUS_PROOF_ENV: &str =
    "WENDAO_REAL_REPO_SEARCH_PRECISION_DOCS_CORPUS_PROOF";

#[derive(Debug, Clone)]
pub(crate) struct RealRepoPrecisionCatalogEntry {
    pub(crate) repository: RegisteredRepository,
    pub(crate) include_dirs: Vec<String>,
    pub(crate) excluded_dirs: Vec<String>,
    pub(crate) gold_queries: Vec<RealRepoGoldQuery>,
    pub(crate) knowledge_scenarios: Vec<RealRepoKnowledgeScenario>,
}

#[derive(Debug, Clone)]
pub(crate) struct RealRepoGoldQuery {
    pub(crate) id: String,
    pub(crate) kind: RealRepoGoldQueryKind,
    pub(crate) query: String,
    pub(crate) limit: usize,
    pub(crate) must_hit_paths: Vec<String>,
    pub(crate) required_top_path: Option<String>,
    pub(crate) language_filters: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RealRepoGoldQueryKind {
    LinkGraph,
}

impl RealRepoGoldQueryKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::LinkGraph => "link_graph",
        }
    }

    pub(crate) fn parse_filter(raw: Option<&str>) -> Option<Self> {
        match raw.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) if value.eq_ignore_ascii_case("link_graph") => Some(Self::LinkGraph),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RealRepoKnowledgeScenario {
    pub(crate) id: String,
    pub(crate) kind: RealRepoKnowledgeScenarioKind,
    pub(crate) intent: String,
    pub(crate) linked_query_ids: Vec<String>,
    pub(crate) query_variants: Vec<RealRepoKnowledgeScenarioQueryVariant>,
    pub(crate) required_paths: Vec<String>,
    pub(crate) required_semantic_object_ids: Vec<String>,
    pub(crate) required_relation_paths: Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    pub(crate) authority: Option<RealRepoKnowledgeScenarioAuthorityExpectation>,
    pub(crate) forbidden_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RealRepoKnowledgeScenarioQueryVariant {
    pub(crate) query_id: String,
    pub(crate) kind: RealRepoKnowledgeScenarioQueryVariantKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RealRepoKnowledgeScenarioQueryVariantKind {
    Canonical,
    Paraphrase,
    Alias,
    Task,
}

impl RealRepoKnowledgeScenarioQueryVariantKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Canonical => "canonical",
            Self::Paraphrase => "paraphrase",
            Self::Alias => "alias",
            Self::Task => "task",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RealRepoKnowledgeScenarioKind {
    KnownItem,
    NaturalLanguageIntent,
    MultiHopRelation,
    AuthorityOrdering,
    NegativeEvidence,
    AmbiguousAlias,
    AgentTask,
}

impl RealRepoKnowledgeScenarioKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::KnownItem => "known_item",
            Self::NaturalLanguageIntent => "natural_language_intent",
            Self::MultiHopRelation => "multi_hop_relation",
            Self::AuthorityOrdering => "authority_ordering",
            Self::NegativeEvidence => "negative_evidence",
            Self::AmbiguousAlias => "ambiguous_alias",
            Self::AgentTask => "agent_task",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RealRepoKnowledgeScenarioAuthorityExpectation {
    pub(crate) preferred_path: String,
    pub(crate) competing_paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RealRepoPrecisionSyncMode {
    #[default]
    Status,
    Ensure,
    Refresh,
}

impl RealRepoPrecisionSyncMode {
    pub(crate) fn parse(raw: Option<&str>) -> Self {
        match raw.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) if value.eq_ignore_ascii_case("ensure") => Self::Ensure,
            Some(value) if value.eq_ignore_ascii_case("refresh") => Self::Refresh,
            _ => Self::Status,
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Ensure => "ensure",
            Self::Refresh => "refresh",
        }
    }

    pub(crate) const fn as_git_sync_mode(self) -> xiuxian_git_repo::SyncMode {
        match self {
            Self::Status => xiuxian_git_repo::SyncMode::Status,
            Self::Ensure => xiuxian_git_repo::SyncMode::Ensure,
            Self::Refresh => xiuxian_git_repo::SyncMode::Refresh,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RealRepoPrecisionRunOptions {
    pub(crate) enabled: bool,
    pub(crate) sync_mode: RealRepoPrecisionSyncMode,
    pub(crate) query_kind_filter: Option<RealRepoGoldQueryKind>,
    pub(crate) prewarmed_resident_only: bool,
    pub(crate) project_root: PathBuf,
    pub(crate) receipt_path: PathBuf,
    pub(crate) link_graph_cache_path: PathBuf,
}

impl RealRepoPrecisionRunOptions {
    pub(crate) fn from_env() -> Self {
        let project_root = project_root_from_env();
        let receipt_path = receipt_path_from_env(&project_root);
        let link_graph_cache_path = link_graph_cache_path_from_receipt(&receipt_path);
        Self {
            enabled: std::env::var(RUN_ENV).is_ok_and(|value| value.trim() == "1"),
            sync_mode: RealRepoPrecisionSyncMode::parse(
                std::env::var(SYNC_MODE_ENV).ok().as_deref(),
            ),
            query_kind_filter: RealRepoGoldQueryKind::parse_filter(
                std::env::var(QUERY_KIND_ENV).ok().as_deref(),
            ),
            prewarmed_resident_only: std::env::var(PREWARM_PROOF_ENV)
                .is_ok_and(|value| value.trim() == "1"),
            project_root,
            receipt_path,
            link_graph_cache_path,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub(crate) enum RealRepoPrecisionRunStatus {
    Skipped { reason: String },
    Completed(RealRepoPrecisionRunReceipt),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RealRepoPrecisionRunReceipt {
    pub(crate) schema: String,
    pub(crate) generated_at: String,
    pub(crate) sync_mode: String,
    pub(crate) query_kind_filter: String,
    pub(crate) summary: RealRepoPrecisionSummary,
    pub(crate) repositories: Vec<RealRepoPrecisionRepositoryReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct RealRepoPrecisionSummary {
    #[serde(rename = "repository_count")]
    pub(crate) repositories_total: usize,
    #[serde(rename = "materialized_repository_count")]
    pub(crate) repositories_materialized: usize,
    #[serde(rename = "skipped_repository_count")]
    pub(crate) repositories_skipped: usize,
    #[serde(rename = "query_count")]
    pub(crate) queries_total: usize,
    #[serde(rename = "passed_query_count")]
    pub(crate) queries_passed: usize,
    #[serde(rename = "failed_query_count")]
    pub(crate) queries_failed: usize,
    #[serde(rename = "knowledge_scenario_count")]
    pub(crate) knowledge_scenarios_total: usize,
    #[serde(rename = "passed_knowledge_scenario_count")]
    pub(crate) knowledge_scenarios_passed: usize,
    #[serde(rename = "failed_knowledge_scenario_count")]
    pub(crate) knowledge_scenarios_failed: usize,
    #[serde(rename = "indexed_document_count")]
    pub(crate) indexed_documents: usize,
    #[serde(rename = "indexed_markdown_document_count")]
    pub(crate) indexed_markdown_documents: usize,
    #[serde(rename = "indexed_org_document_count")]
    pub(crate) indexed_org_documents: usize,
    #[serde(rename = "indexed_total_word_count")]
    pub(crate) indexed_total_words: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RealRepoPrecisionRepositoryReceipt {
    pub(crate) repo_id: String,
    pub(crate) checkout_root: String,
    pub(crate) lifecycle: String,
    pub(crate) indexed: bool,
    pub(crate) materialize_ms: u128,
    pub(crate) link_graph_index_ms: Option<u128>,
    pub(crate) link_graph_cache_backend: Option<String>,
    pub(crate) link_graph_cache_status: Option<String>,
    pub(crate) link_graph_cache_miss_reason: Option<String>,
    pub(crate) link_graph_corpus: Option<RealRepoPrecisionLinkGraphCorpusReceipt>,
    pub(crate) markdown_knowledge_semantic_gate:
        Option<RealRepoMarkdownKnowledgeSemanticGateReceipt>,
    pub(crate) knowledge_scenarios: Vec<RealRepoKnowledgeScenarioReceipt>,
    pub(crate) query_wall_ms: u128,
    pub(crate) query_sum_ms: u128,
    pub(crate) total_ms: u128,
    pub(crate) skip_reason: Option<String>,
    pub(crate) query_receipts: Vec<RealRepoPrecisionQueryReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoPrecisionLinkGraphCorpusReceipt {
    pub(crate) document_count: usize,
    pub(crate) markdown_document_count: usize,
    pub(crate) org_document_count: usize,
    pub(crate) total_word_count: usize,
    pub(crate) path_prefix_counts: Vec<RealRepoPrecisionCorpusPathPrefixReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoPrecisionCorpusPathPrefixReceipt {
    pub(crate) prefix: String,
    pub(crate) document_count: usize,
    pub(crate) word_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoMarkdownKnowledgeSemanticGateReceipt {
    pub(crate) schema: String,
    pub(crate) semantic_root: String,
    pub(crate) linked_query_ids: Vec<String>,
    pub(crate) required_markdown_paths: Vec<String>,
    pub(crate) covered_markdown_paths: Vec<String>,
    pub(crate) required_relation_paths: Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    pub(crate) covered_relation_paths: Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    pub(crate) knowledge_scenarios: Vec<RealRepoMarkdownKnowledgeSemanticScenarioReceipt>,
    pub(crate) semantic_object_ids: Vec<String>,
    pub(crate) semantic_scope_object_count: usize,
    pub(crate) semantic_scope_relation_count: usize,
    pub(crate) page_index_node_count: usize,
    pub(crate) page_index_edge_count: usize,
    pub(crate) page_index_seed_count: usize,
    pub(crate) required_validation_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RealRepoMarkdownKnowledgeSemanticRelationPathReceipt {
    pub(crate) source: String,
    pub(crate) kind: String,
    pub(crate) target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoMarkdownKnowledgeSemanticScenarioReceipt {
    pub(crate) scenario_id: String,
    pub(crate) intent: String,
    pub(crate) linked_query_ids: Vec<String>,
    pub(crate) query_evidence: Vec<RealRepoMarkdownKnowledgeSemanticScenarioQueryEvidenceReceipt>,
    pub(crate) required_object_ids: Vec<String>,
    pub(crate) covered_object_ids: Vec<String>,
    pub(crate) required_relation_paths: Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    pub(crate) covered_relation_paths: Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    pub(crate) passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoMarkdownKnowledgeSemanticScenarioQueryEvidenceReceipt {
    pub(crate) query_id: String,
    pub(crate) query_kind: String,
    pub(crate) query_ms: u128,
    pub(crate) passed: bool,
    pub(crate) required_top_path: Option<String>,
    pub(crate) observed_top_path: Option<String>,
    pub(crate) missing_paths: Vec<String>,
    pub(crate) observed_path_count: usize,
    pub(crate) failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoKnowledgeScenarioReceipt {
    pub(crate) scenario_id: String,
    pub(crate) scenario_kind: String,
    pub(crate) intent: String,
    pub(crate) intent_frame: RealRepoKnowledgeScenarioIntentFrameReceipt,
    pub(crate) linked_query_ids: Vec<String>,
    pub(crate) query_evidence: Vec<RealRepoKnowledgeScenarioQueryEvidenceReceipt>,
    pub(crate) reasoning_tree: RealRepoKnowledgeScenarioReasoningTreeReceipt,
    pub(crate) backend_frontier: RealRepoKnowledgeScenarioBackendFrontierReceipt,
    pub(crate) query_variant_count: usize,
    pub(crate) passed_query_variant_count: usize,
    pub(crate) failed_query_variant_count: usize,
    pub(crate) query_variants: Vec<RealRepoKnowledgeScenarioQueryVariantReceipt>,
    pub(crate) required_path_count: usize,
    pub(crate) covered_required_path_count: usize,
    pub(crate) required_path_recall_bps: u32,
    pub(crate) required_path_recall_at_1_bps: u32,
    pub(crate) required_path_recall_at_3_bps: u32,
    pub(crate) required_path_recall_at_5_bps: u32,
    pub(crate) required_path_recall_at_10_bps: u32,
    pub(crate) mean_required_path_reciprocal_rank_bps: u32,
    pub(crate) best_required_path_rank: Option<usize>,
    pub(crate) required_path_ranks: Vec<RealRepoPrecisionRequiredPathRankReceipt>,
    pub(crate) required_paths: Vec<String>,
    pub(crate) covered_paths: Vec<String>,
    pub(crate) missing_paths: Vec<String>,
    pub(crate) required_semantic_object_ids: Vec<String>,
    pub(crate) covered_semantic_object_ids: Vec<String>,
    pub(crate) missing_semantic_object_ids: Vec<String>,
    pub(crate) required_relation_paths: Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    pub(crate) covered_relation_paths: Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    pub(crate) missing_relation_paths: Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    pub(crate) authority: Option<RealRepoKnowledgeScenarioAuthorityReceipt>,
    pub(crate) negative_guard: Option<RealRepoKnowledgeScenarioNegativeGuardReceipt>,
    pub(crate) failure_reasons: Vec<String>,
    pub(crate) passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoKnowledgeScenarioBackendFrontierReceipt {
    pub(crate) strategy: String,
    pub(crate) control_plane_owner: String,
    pub(crate) graph_backend: String,
    pub(crate) graph_backend_live: bool,
    pub(crate) julia_schedule_basis: String,
    pub(crate) node_count: usize,
    pub(crate) kept_node_count: usize,
    pub(crate) pruned_node_count: usize,
    pub(crate) expand_node_count: usize,
    pub(crate) subagent_judgement_node_count: usize,
    pub(crate) subagent_fanout_group_count: usize,
    pub(crate) subagent_fanout_node_count: usize,
    pub(crate) subagent_max_parallel_width: usize,
    pub(crate) subagent_context_budget_chars: usize,
    pub(crate) julia_candidate_node_count: usize,
    pub(crate) julia_dispatch_node_count: usize,
    pub(crate) julia_queue_node_count: usize,
    pub(crate) julia_fallback_node_count: usize,
    pub(crate) julia_reject_node_count: usize,
    pub(crate) strategy_flow_projection_basis: String,
    pub(crate) strategy_flow_candidate_node_count: usize,
    pub(crate) strategy_flow_transition_node_count: usize,
    pub(crate) strategy_flow_frontier_node_count: usize,
    pub(crate) strategy_flow_context_budget_chars: usize,
    pub(crate) strategy_flow_intent_complexity_class: String,
    pub(crate) strategy_flow_initial_topology: String,
    pub(crate) strategy_flow_refinement_topology: String,
    pub(crate) strategy_flow_max_planned_depth: usize,
    pub(crate) strategy_flow_loop_budget: usize,
    pub(crate) strategy_flow_cycle_candidate_node_count: usize,
    pub(crate) strategy_flow_llm_judgement_node_count: usize,
    pub(crate) selected_beam_width: usize,
    pub(crate) nodes: Vec<RealRepoKnowledgeScenarioBackendFrontierNodeReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoKnowledgeScenarioBackendFrontierNodeReceipt {
    pub(crate) node_id: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) reasoning_step_index: Option<usize>,
    pub(crate) step_kind: String,
    pub(crate) evidence_kind: String,
    pub(crate) evidence_id: String,
    pub(crate) query_id: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) relation: Option<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    pub(crate) semantic_object_id: Option<String>,
    pub(crate) disclosure_depth: usize,
    pub(crate) parallel_group: String,
    pub(crate) graph_batch_key: String,
    pub(crate) graph_score_bps: u32,
    pub(crate) authority_score_bps: u32,
    pub(crate) coverage_score_bps: u32,
    pub(crate) context_cost: usize,
    pub(crate) backend_action: String,
    pub(crate) requires_subagent_judgement: bool,
    pub(crate) subagent_prompt_hint: Option<String>,
    pub(crate) subagent_fanout_group_id: Option<String>,
    pub(crate) subagent_judgement_kind: Option<String>,
    pub(crate) subagent_priority_score_bps: Option<u32>,
    pub(crate) subagent_context_budget_chars: Option<usize>,
    pub(crate) julia_algorithm_id: Option<String>,
    pub(crate) julia_profile_id: Option<String>,
    pub(crate) julia_capability: Option<String>,
    pub(crate) julia_schedule_action: Option<String>,
    pub(crate) julia_schedule_reason: Option<String>,
    pub(crate) julia_schedule_confidence_score: Option<i32>,
    pub(crate) julia_selected_batch_size: Option<u32>,
    pub(crate) strategy_flow_candidate_id: Option<String>,
    pub(crate) strategy_flow_transition_id: Option<String>,
    pub(crate) strategy_flow_action: Option<String>,
    pub(crate) strategy_flow_score_bps: Option<u32>,
    pub(crate) strategy_flow_frontier_rank: Option<usize>,
    pub(crate) strategy_flow_context_budget_chars: Option<usize>,
    pub(crate) strategy_flow_step_role: Option<String>,
    pub(crate) strategy_flow_iteration_policy: Option<String>,
    pub(crate) strategy_flow_loop_candidate: bool,
    pub(crate) strategy_flow_requires_llm_judgement: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoKnowledgeScenarioIntentFrameReceipt {
    pub(crate) task_kind: String,
    pub(crate) anchor_terms: Vec<String>,
    pub(crate) required_evidence_kinds: Vec<String>,
    pub(crate) relation_hypotheses: Vec<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    pub(crate) authority_policy: Vec<String>,
    pub(crate) max_disclosure_depth: usize,
    pub(crate) verifier_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoKnowledgeScenarioReasoningTreeReceipt {
    pub(crate) strategy: String,
    pub(crate) passed: bool,
    pub(crate) anchor_count: usize,
    pub(crate) relation_step_count: usize,
    pub(crate) page_index_step_count: usize,
    pub(crate) source_step_count: usize,
    pub(crate) disclosure_step_count: usize,
    pub(crate) max_disclosure_depth: usize,
    pub(crate) steps: Vec<RealRepoKnowledgeScenarioReasoningTreeStepReceipt>,
    pub(crate) failure_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoKnowledgeScenarioReasoningTreeStepReceipt {
    pub(crate) step_index: usize,
    pub(crate) step_kind: String,
    pub(crate) evidence_id: String,
    pub(crate) query_id: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) relation: Option<RealRepoMarkdownKnowledgeSemanticRelationPathReceipt>,
    pub(crate) semantic_object_id: Option<String>,
    pub(crate) zero_based_rank: Option<usize>,
    pub(crate) disclosure_depth: usize,
    pub(crate) passed: bool,
    pub(crate) failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoKnowledgeScenarioQueryVariantReceipt {
    pub(crate) query_id: String,
    pub(crate) variant_kind: String,
    pub(crate) query_evidence: RealRepoKnowledgeScenarioQueryEvidenceReceipt,
    pub(crate) passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoKnowledgeScenarioQueryEvidenceReceipt {
    pub(crate) query_id: String,
    pub(crate) query_kind: String,
    pub(crate) query_ms: u128,
    pub(crate) passed: bool,
    pub(crate) required_top_path: Option<String>,
    pub(crate) observed_top_path: Option<String>,
    pub(crate) missing_paths: Vec<String>,
    pub(crate) required_path_ranks: Vec<RealRepoPrecisionRequiredPathRankReceipt>,
    pub(crate) required_path_recall_at_1_bps: u32,
    pub(crate) required_path_recall_at_3_bps: u32,
    pub(crate) required_path_recall_at_5_bps: u32,
    pub(crate) required_path_recall_at_10_bps: u32,
    pub(crate) mean_required_path_reciprocal_rank_bps: u32,
    pub(crate) best_required_path_rank: Option<usize>,
    pub(crate) observed_path_count: usize,
    pub(crate) failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoPrecisionRequiredPathRankReceipt {
    pub(crate) path: String,
    pub(crate) zero_based_rank: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoKnowledgeScenarioAuthorityReceipt {
    pub(crate) preferred_path: String,
    pub(crate) competing_paths: Vec<String>,
    pub(crate) preferred_rank: Option<usize>,
    pub(crate) earliest_competing_rank: Option<usize>,
    pub(crate) passed: bool,
    pub(crate) failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoKnowledgeScenarioNegativeGuardReceipt {
    pub(crate) forbidden_paths: Vec<String>,
    pub(crate) matched_forbidden_paths: Vec<String>,
    pub(crate) passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RealRepoPrecisionQueryReceipt {
    pub(crate) query_id: String,
    pub(crate) query_kind: String,
    pub(crate) query: String,
    pub(crate) limit: usize,
    pub(crate) query_ms: u128,
    pub(crate) passed: bool,
    pub(crate) must_hit_paths: Vec<String>,
    pub(crate) missing_paths: Vec<String>,
    pub(crate) required_top_path: Option<String>,
    pub(crate) observed_top_path: Option<String>,
    pub(crate) required_path_ranks: Vec<RealRepoPrecisionRequiredPathRankReceipt>,
    pub(crate) required_path_recall_at_1_bps: u32,
    pub(crate) required_path_recall_at_3_bps: u32,
    pub(crate) required_path_recall_at_5_bps: u32,
    pub(crate) required_path_recall_at_10_bps: u32,
    pub(crate) mean_required_path_reciprocal_rank_bps: u32,
    pub(crate) best_required_path_rank: Option<usize>,
    pub(crate) observed_paths: Vec<String>,
}

fn project_root_from_env() -> PathBuf {
    std::env::var_os("PRJ_ROOT").map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(4)
                .map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")), PathBuf::from)
        },
        PathBuf::from,
    )
}

fn receipt_path_from_env(project_root: &std::path::Path) -> PathBuf {
    let cache_home = std::env::var_os("PRJ_CACHE_HOME")
        .map_or_else(|| project_root.join(".cache"), PathBuf::from);
    cache_home
        .join("wendao")
        .join("search_precision")
        .join("real_repo_receipt.json")
}

fn link_graph_cache_path_from_receipt(receipt_path: &std::path::Path) -> PathBuf {
    receipt_path.parent().map_or_else(
        || PathBuf::from("real_repo_link_graph_cache.duckdb"),
        |parent| parent.join("real_repo_link_graph_cache.duckdb"),
    )
}
