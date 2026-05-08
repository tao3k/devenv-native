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
- `promotion_score_bps`

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

## Extension Rules

New profile scorers must:

- preserve the existing report schema unless a new version is explicitly
  introduced;
- keep missing or malformed source evidence deterministic;
- expose quality, latency, and evidence-cost fields separately;
- avoid live service calls in unit tests;
- document whether the profile is receipt-derived, Rust-backed, Julia-backed,
  or externally measured.

Future Julia-backed profiles should enter through explicit measured inputs, for
example PPR-like relatedness, community grouping, HNSW semantic fanout, or
relationship traversal receipts. Python remains the judge; Julia remains a
benchmarked implementation.
