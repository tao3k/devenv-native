# Profile Contract

The benchmark compares retrieval profiles through a common score record. A
profile is a measured or receipt-derived strategy for exposing evidence to an
agent.

## Source Receipt

The input receipt schema is:

```text
xiuxian_wendao.real_repo_search_precision.v1
```

The package currently consumes these repository-level fields:

- `repo_id`
- `total_ms`
- `query_receipts`
- `knowledge_scenarios`

The current profile scorers read scenario-level quality facts such as:

- `passed`
- `linked_query_ids`
- `query_variants`
- `query_evidence`
- `required_path_recall_at_1_bps`
- `required_path_recall_at_3_bps`
- `required_path_recall_at_5_bps`
- `required_path_recall_at_10_bps`
- `mean_required_path_reciprocal_rank_bps`
- `reasoning_tree`
- `intent_frame`
- `backend_frontier`

The `intent_frame` field is optional for older receipts. When present, it is a
deterministic Rust-owned interpretation of the natural-language scenario
intent. The current `intent-tree-v1` profile consumes:

- `task_kind`
- `anchor_terms`
- `required_evidence_kinds`
- `relation_hypotheses`
- `authority_policy`
- `max_disclosure_depth`
- `verifier_required`

The `backend_frontier` field is optional for older receipts. When present, it
is the Rust-owned control-plane frontier contract for future
Rust/Julia/pi-wendao reasoning-tree pruning. The current
`backend-frontier-pruning-v1` profile consumes:

- `strategy`
- `control_plane_owner`
- `graph_backend`
- `graph_backend_live`
- `nodes`
- per-node `evidence_kind`
- per-node `graph_batch_key`
- per-node `parallel_group`
- per-node `context_cost`
- per-node `backend_action`
- per-node `requires_subagent_judgement`
- per-node `julia_algorithm_id`
- per-node `julia_profile_id`
- per-node `julia_schedule_action`
- per-node `julia_schedule_reason`
- per-node `strategy_flow_candidate_id`
- per-node `strategy_flow_transition_id`
- per-node `strategy_flow_action`
- per-node `strategy_flow_score_bps`
- per-node `strategy_flow_frontier_rank`
- per-node `strategy_flow_context_budget_chars`

The benchmark report aggregates those schedule facts into:

- `subagent_fanout_group_count`
- `subagent_fanout_node_count`
- `subagent_max_parallel_width`
- `subagent_context_budget_chars`
- `julia_schedule_bases`
- `julia_algorithm_count`
- `julia_profile_count`
- `julia_candidate_node_count`
- `julia_scheduled_node_count`
- `julia_dispatch_node_count`
- `julia_queue_node_count`
- `julia_fallback_node_count`
- `julia_reject_node_count`
- `strategy_flow_projection_bases`
- `strategy_flow_candidate_node_count`
- `strategy_flow_transition_node_count`
- `strategy_flow_frontier_node_count`
- `strategy_flow_context_budget_chars`
- `strategy_flow_complexity_classes`
- `strategy_flow_initial_topologies`
- `strategy_flow_refinement_topologies`
- `strategy_flow_loop_budget`
- `strategy_flow_cycle_candidate_node_count`
- `strategy_flow_llm_judgement_node_count`

When at least one selected backend-frontier node has a
`strategy_flow_frontier_rank`, the benchmark also emits
`search-strategy-flow-projection-v1`. That profile consumes the selected
SearchStrategyFlow frontier rows, their evidence kind, rank, and
materialized context cost. `strategy_flow_context_budget_chars` remains a
diagnostic budget for later subagent execution. The profile uses the same
report schema id because the fields are additive and optional.
`authority_order` and `negative_guard` rows still count as SearchStrategyFlow
candidate and transition facts, but they do not need a
`strategy_flow_frontier_rank`; the benchmark treats them as validation evidence
rather than agent-disclosure branches.

SearchStrategyFlow topology fields are planning evidence. The initial topology
is the evidence DAG produced by deterministic retrieval. The refinement
topology may remain acyclic, allow iterative graph refinement, or allow cyclic
refinement when graph revisits and LLM/subagent judgement nodes are both
present. The benchmark reports this as `Flow Loop/LLM` in Markdown.

Receipts with a different schema fail fast.

Receipts without `knowledge_scenarios` are valid source receipts, but they do
not produce knowledge profile rows or a recommended profile. This keeps
source-code-only `repo_ast` proofs distinct from knowledge retrieval
benchmarks.

## Report Schema

The output report schema is:

```text
xiuxian_wendao.knowledge_retrieval_blackbox_benchmark.v1
```

Each repository report contains:

- `repo_id`
- `source_total_ms`
- `profile_scores`
- `recommended_profile_id`
- `scenario_recommendations`

Each profile score contains:

- `profile_id`
- `scenario_count`
- `passed_scenario_count`
- `failed_scenario_count`
- `mean_recall_at_1_bps`
- `mean_recall_at_3_bps`
- `mean_recall_at_5_bps`
- `mean_recall_at_10_bps`
- `mean_reciprocal_rank_bps`
- `total_query_ms`
- `exposed_item_count`
- `exposed_path_char_count`
- `disclosure_step_count`
- `max_disclosure_depth`
- `required_evidence_kind_count`
- `observed_evidence_kind_count`
- `missing_evidence_kind_count`
- `evidence_coverage_bps`
- `context_reduction_bps`
- `scenario_scores`
- `promotion_score_bps`

Each scenario score is an additive diagnostic row with:

- `profile_id`
- `scenario_id`
- `scenario_kind`
- `passed`
- `required_evidence_kinds`
- `observed_evidence_kinds`
- `missing_evidence_kinds`
- `evidence_coverage_bps`
- `exposed_item_count`
- `exposed_path_char_count`
- `disclosure_step_count`
- `max_disclosure_depth`
- `context_reduction_bps`
- `strategy_flow_intent_complexity_class`
- `strategy_flow_initial_topology`
- `strategy_flow_refinement_topology`
- `strategy_flow_max_planned_depth`
- `strategy_flow_candidate_node_count`
- `strategy_flow_transition_node_count`
- `strategy_flow_frontier_node_count`
- `strategy_flow_loop_budget`
- `strategy_flow_cycle_candidate_node_count`
- `strategy_flow_llm_judgement_node_count`

Each scenario recommendation is an additive diagnostic row with:

- `scenario_id`
- `scenario_kind`
- `recommended_profile_id`
- `reason`
- `selected_score_bps`
- `candidate_count`

## Promotion Score

The first score is intentionally simple and deterministic:

```text
average(pass_rate, recall_at_10, reciprocal_rank)
- exposed_character_penalty
- latency_penalty
```

This score is a benchmark ranking aid, not a product policy. Correctness facts
remain visible as separate fields so a caller can reject a profile with lower
precision even if it appears cheaper.

## Scenario Recommendation

Repository-level recommendation remains useful as a summary, but it is not
precise enough for mixed knowledge workloads. The report therefore also emits
scenario-level recommendations.
The repository-level recommendation is derived from those scenario decisions
when they are available, so a repository dominated by exact known-item lookups
does not get promoted to a topology-heavy strategy just because an aggregate
profile tie-breaker exists.

The first policy is intentionally conservative:

- if flat top-k fully covers required evidence and graph/intent profiles do not
  produce a material context reduction, recommend `flat-topk`;
- if graph-first or intent-tree covers evidence that flat top-k misses,
  recommend the evidence-complete profile;
- if all candidates preserve evidence, prefer the candidate with material
  context reduction and lower exposed evidence cost.
- if SearchStrategyFlow ties or beats graph-first on quality and context cost,
  allow topology evidence such as cyclic refinement, loop budget, and
  LLM/subagent judgement nodes to break the tie in its favor.

This makes simple owner-definition or known-item lookup eligible for the cheap
path while keeping graph-first and intent-tree profiles favored for multi-hop,
authority, negative-evidence, PageIndex, and semantic SSOT scenarios.

## Extension Rules

New profile scorers must:

- preserve the current report schema identifier unless a governed compatibility
  requirement explicitly freezes and versions the report contract;
- keep missing or malformed source evidence deterministic;
- expose quality, latency, and evidence-cost fields separately;
- avoid live service calls in unit tests;
- document whether the profile is receipt-derived, Rust-backed, Julia-backed,
  or externally measured.

Future Julia-backed profiles should enter through explicit measured inputs, for
example PPR-like relatedness, community grouping, HNSW semantic fanout, or
relationship traversal receipts. Python remains the judge; Julia remains a
benchmarked implementation.

The `intent-tree-v1` profile is receipt-derived. It is the bridge between
natural-language user intent and graph-first evidence disclosure, but it does
not call an LLM, spawn agents, or replace Rust/Julia search implementations.

The `backend-frontier-pruning-v1` profile is receipt-derived too. It is the
first benchmark proof for the Rust-controlled backend frontier. Until live
Julia is wired, `graph_backend` may be `rust-baseline`; when WendaoGraph.jl is
connected, Julia scores must enter as measured node fields rather than as an
assumed improvement.

`julia_schedule_action` in current receipts is an inert scheduling projection
when `julia_schedule_basis` is `static_warm_profile_projection_v1`. It proves
that the backend frontier can be routed through existing Rust scheduler policy,
but it is not live Julia execution evidence.
