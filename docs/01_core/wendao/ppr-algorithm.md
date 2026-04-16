---
type: knowledge
title: "LinkGraph PPR Algorithm Spec (HippoRAG + GRAG Calibrated)"
category: "plans"
tags:
  - wendao
  - link-graph
  - ppr
  - retrieval
  - hipporag
  - grag
saliency_base: 8.5
decay_rate: 0.02
metadata:
  title: "LinkGraph PPR Algorithm Spec (HippoRAG + GRAG Calibrated)"
---

# LinkGraph PPR Algorithm Spec (HippoRAG + GRAG Calibrated)

> **Status:** In execution (xiuxian-wendao core track)  
> **Target Version:** 2026.Q1  
> **Date:** February 20, 2026
> **Primary Owner:** `packages/rust/crates/xiuxian-wendao/src/link_graph`
> **Program Index:** `docs/01_core/wendao/roadmap.md`

## 0. Role in the Merged Wendao Plan Set

This document is the algorithmic source of truth for `xiuxian-wendao` LinkGraph retrieval.

Scope rules:

1. Retrieval math, ranking, and related-query behavior must be defined here.
2. Runtime governance references (Omega, memory gates) may consume this contract but should not redefine it.
3. Agentic graph construction documents may add proposal/promotion behavior, but cannot override this retrieval baseline.

## 0.1 Hard Constraints

> [!WARNING]
>
> 1. Related-note retrieval baseline is PPR-only; no BFS compatibility path should be reintroduced.
> 2. Retrieval math and ranking are Rust-owned in `xiuxian-wendao`; adapters must not fork algorithm behavior.
> 3. Any policy or agentic extension must consume this contract, not redefine it.

## 1. Citation Calibration

| Scope                                | Source                                        | Validation                                  | Use in Omni                                                |
| ------------------------------------ | --------------------------------------------- | ------------------------------------------- | ---------------------------------------------------------- |
| Single-step graph retrieval with PPR | [HippoRAG](https://arxiv.org/abs/2405.14831)  | Findings of EMNLP 2024 (per arXiv comments) | Core ranking for `find_related` and graph-side rerank.     |
| Large-graph retrieval scaling        | [GRAG](https://arxiv.org/abs/2405.16506)      | Verified (arXiv preprint)                   | Divide-and-conquer subgraph retrieval before final fusion. |
| Graph/vector hybrid entity alignment | [HybridRAG](https://arxiv.org/abs/2408.04948) | Verified (arXiv preprint)                   | Keep graph-first policy with bounded vector escalation.    |

## 2. Current Baseline (Code Reality)

Implemented today:

- `path_fuzzy`, `fts`, `exact`, `re` match strategies exist in Rust search.
- `search_planned` and policy-level `path_fuzzy` routing already work.
- tree-level section retrieval controls are implemented end-to-end (`scope`, `max_heading_level`,
  `max_tree_hops`, `collapse_to_doc`, `edge_types`, `per_doc_section_cap`, `min_section_words`).
- parser, filters, and section-aware scoring are wired in `xiuxian-wendao` search pipeline.
- `related` retrieval is PPR-based in Rust traversal/search paths (BFS fallback removed).
- `wendao related --verbose` exposes PPR diagnostics (`iteration_count`, `final_residual`,
  `candidate_count`, `candidate_cap`, `candidate_capped`, `graph_node_count`, `subgraph_count`,
  `time_budget_ms`, `timed_out`, `partition_max_node_count`,
  `partition_min_node_count`, `partition_avg_node_count`, `partition_duration_ms`,
  `kernel_duration_ms`, `fusion_duration_ms`, `total_duration_ms`,
  `subgraph_mode`, `horizon_restricted`).
- seed-driven subgraph partition + score fusion is implemented in Rust PPR kernel.
- monitor-phase level events are emitted in verbose payload:
  - `link_graph.related.ppr`
  - `link_graph.related.subgraph.partition`
  - `link_graph.related.subgraph.fusion`
- benchmark runner supports ambiguous stem auto-resolution with path-biased retry, so
  `README`-style seeds are usable in regression gates.

Remaining work:

- collect sustained green gate evidence (`gate-wendao-ppr-report`) and complete explicit sign-off before default scope switch to `mixed`.

## 3. Target Retrieval Algorithm

### 3.1 Problem Statement

Current related retrieval is hop-based and distance-sorted. This misses global influence and multi-path evidence. We move to a PPR-scored ranking over graph topology.

### 3.2 Graph Model

Use the existing schema-less note graph:

- node: note/document stem;
- edge: explicit markdown/wiki link;
- optional edge weight from structural priors (for example bidirectional or hub signals).

### 3.3 Seed Construction

Build seed vector from:

1. graph hits produced by `search_planned`;
2. optional hybrid seeds from Librarian/vector stage (policy-controlled);
3. normalized score mass across selected seeds.

### 3.4 Single-Step PPR Retrieval

Compute one PPR diffusion and rank by steady-state scores:

\[
r\_{t+1} = (1-\alpha)\,s + \alpha\,P^\top r_t
\]

Where:

- \(s\): normalized seed vector,
- \(P\): transition matrix from weighted edges,
- \(\alpha\): damping factor (default `0.85`),
- stop by tolerance or max iterations.

Return Top-K nodes excluding direct seed echoes when configured.

### 3.5 Divide-and-Conquer Subgraph Mode (Large Graphs)

For large corpora:

1. construct candidate subgraphs around seed/frontier constraints;
2. run PPR per subgraph in parallel;
3. fuse candidates by normalized score + source/subgraph confidence.

This keeps latency stable as graph size grows.

## 4. Contracts and Observability

Add/extend common contracts with:

- PPR config (`alpha`, `max_iter`, `tol`, `subgraph_mode`);
- runtime outputs (`iteration_count`, `residual_error`, `candidate_count`, `candidate_cap`,
  `candidate_capped`, `subgraph_count`, `time_budget_ms`, `timed_out`,
  `partition_max_node_count`, `partition_min_node_count`, `partition_avg_node_count`,
  `partition_duration_ms`, `kernel_duration_ms`, `fusion_duration_ms`, `total_duration_ms`);
- monitor phases:
  - `link_graph.related.ppr`
  - `link_graph.related.subgraph.partition`
  - `link_graph.related.subgraph.fusion`

All payloads stay schema-validated through shared contracts.

## 5. Xiuxian-Wendao Execution Plan (Primary Track)

1. **WG0: Contract Lock (schema + bindings) — Done**

- extend shared contracts for PPR options/results:
  - `packages/rust/crates/xiuxian-wendao/resources/xiuxian_wendao.link_graph.search_options.v2.schema.json`
  - `packages/rust/crates/xiuxian-wendao/resources/xiuxian_wendao.link_graph.record.v1.schema.json`
  - `packages/rust/crates/xiuxian-wendao/resources/xiuxian_wendao.link_graph.valkey_cache_snapshot.v1.schema.json`
  - `packages/rust/crates/xiuxian-wendao/resources/xiuxian_wendao.link_graph.stats.cache.v1.schema.json`
  - `packages/rust/crates/xiuxian-wendao/resources/xiuxian_wendao.link_graph.saliency.v1.schema.json`
- mirror fields in Python/Rust models:
  - `packages/python/foundation/src/omni/rag/link_graph/models.py`
  - `packages/rust/crates/xiuxian-wendao/src/link_graph/models.rs`

2. **WG1: Rust PPR Kernel — Done**

- add a dedicated PPR module under `xiuxian_wendao::link_graph::index`
  (no skill-layer implementation).
- expose deterministic top-k scoring API for `related` path reuse.

3. **WG2: Replace `related` BFS Path — Done**

- route `related` through PPR ranking in search/traversal pipeline.
- no compatibility BFS ranking path is retained.

4. **WG3: Large-Graph Subgraph Partition — Done**

- implement bounded partition + parallel PPR + score fusion.
- add hard guards for timeout, candidate cap, and memory growth.
- expose partition/fusion monitor phases in CLI verbose output and diagnostics payloads.

5. **WG4: Graph-First Policy Coupling — Done**

- confidence-based graph-first decision is now emitted by Rust `search_planned_payload` with:
  - `requested_mode`, `selected_mode`, `reason`,
  - `graph_hit_count`, `source_hint_count`,
  - `graph_confidence_score`, `graph_confidence_level`,
  - schema-aligned `retrieval_plan` (`xiuxian_wendao.link_graph.retrieval_plan.v1`).
- Python policy now treats Rust payload as the single decision authority:
  - valid payload decision (`requested_mode` matches caller mode): trust payload `selected_mode` + `reason`;
  - payload mode conflict: force `vector_only` in hybrid path with `reason=graph_policy_mode_conflict`;
  - missing payload decision: force `vector_only` in hybrid path with `reason=graph_policy_missing`;
  - for `graph_only` requests, selected mode remains `graph_only`; reasons are
    `graph_only_search_timeout`, `graph_only_payload_overridden`,
    `graph_only_payload_mode_conflict`, or `graph_only_policy_missing`.
- escalation remains bounded by runtime config:
  - `link_graph.retrieval.mode`,
  - `link_graph.retrieval.candidate_multiplier`,
  - `link_graph.retrieval.max_sources`,
  - `link_graph.retrieval.hybrid_min_hits`,
  - `link_graph.retrieval.hybrid_min_top_score`,
  - `link_graph.retrieval.graph_rows_per_source`.
- CLI verbose output now includes an explicit `link_graph.search.policy` phase.

6. **WG5: Rollout and Default Switch — Done (core default)**

- pass perf + contract gates on `graph_only` path.
- default related retrieval is PPR ranking.
- compatibility ranking path is removed to enforce one runtime behavior.

## 6. Acceptance Gates

Functional:

- deterministic top-k for fixed seed/graph fixtures;
- contract tests for options and response fields;
- graceful fallback when graph is empty or corrupted.

Performance:

- p95 non-regression for existing search paths;
- bounded memory growth under repeated related queries;
- large-graph partition mode stays within configured timeout budgets.

Executable gate commands (current + planned):

- `just gate-wendao-ppr`
- `cargo test -p xiuxian-wendao --test test_link_graph`
- `uv run python scripts/benchmark_wendao_related.py --root . --stem README --runs 5 --warm-runs 1 --no-build --ppr-subgraph-mode auto`
- `uv run pytest packages/python/foundation/tests/unit/api/test_link_graph_search_options_schema.py -q`

## 7. Section Graph: Implemented Scope and Hardening

### 7.1 Goal

Support heading/subheading as first-class graph nodes so retrieval can target sub-sections, while preventing graph noise from polluting gate decisions.

### 7.2 Node and Edge Taxonomy

Node kinds:

- `doc`: markdown file-level node (existing stem-level identity).
- `section`: heading/subheading node (for example `doc_stem#h2:performance-tuning`).

Edge kinds:

- `structural`:
  - `doc -> section` (`contains`)
  - `section -> parent_section` (`parent`)
  - reverse edges are materialized for traversal symmetry.
- `semantic`:
  - explicit user-authored links between docs/sections (markdown links, wiki links, anchors).
- `provisional`:
  - inferred links from agentic enrichment and cross-note semantic inference.
- `verified`:
  - promoted provisional links that passed gate checks and persistence policy.

### 7.3 Gate Policy (Anti-Pollution)

Policy rules:

- Structural edges are deterministic parser output and do not enter the 3-in-1 gate.
- Explicit semantic edges do not enter the gate by default.
- Only provisional edges are evaluated by 3-in-1 gate.
- Provisional edges are persisted with TTL and quota controls:
  - per-source-node cap;
  - per-document cap;
  - automatic expiration unless promoted to `verified`.

This keeps gate bandwidth focused on uncertain links only.

### 7.4 Retrieval Filters (Tree-Level Controls)

Extend `filters` with tree controls:

| Field                 | Type                                                       | Default    | Purpose                                               |
| --------------------- | ---------------------------------------------------------- | ---------- | ----------------------------------------------------- |
| `scope`               | `doc_only \| section_only \| mixed`                        | `doc_only` | Select node granularity in recall/search.             |
| `max_heading_level`   | `integer (1..6)`                                           | `6`        | Ignore deep headings above configured depth.          |
| `max_tree_hops`       | `integer (>=0)`                                            | `2`        | Bound structural traversal around section seeds.      |
| `collapse_to_doc`     | `boolean`                                                  | `true`     | Merge section hits into doc-level output when needed. |
| `edge_types`          | `array<structural \| semantic \| provisional \| verified>` | all        | Restrict traversal/ranking edge set.                  |
| `per_doc_section_cap` | `integer (>=1)`                                            | `3`        | Prevent one document from dominating section hits.    |
| `min_section_words`   | `integer (>=0)`                                            | `24`       | Filter low-information headings.                      |

### 7.5 Ranking Adjustments for Section Graph

PPR ranking should be edge-aware:

- apply edge-type priors (`verified >= semantic > structural > provisional`);
- normalize by local branching factor to reduce hub domination;
- apply per-doc section cap before final fusion;
- if `collapse_to_doc=true`, aggregate section scores into doc score:
  - `score_doc = max(section_scores) + beta * sum(top_m_section_scores)`.

### 7.6 Contract Mapping (Schema/Python/Rust)

Shared schema changes:

- `packages/rust/crates/xiuxian-wendao/resources/xiuxian_wendao.link_graph.search_options.v2.schema.json`
  - add new filter fields in `filters` object:
    - `scope`, `max_heading_level`, `max_tree_hops`, `collapse_to_doc`, `edge_types`,
      `per_doc_section_cap`, `min_section_words`.
- `packages/rust/crates/xiuxian-wendao/resources/xiuxian_wendao.link_graph.valkey_cache_snapshot.v1.schema.json`
  - enforce Rust cache snapshot contract for index payload persistence.
- `packages/rust/crates/xiuxian-wendao/resources/xiuxian_wendao.link_graph.stats.cache.v1.schema.json`
  - enforce persistent graph stats cache payload contract.
- `packages/rust/crates/xiuxian-wendao/resources/xiuxian_wendao.link_graph.saliency.v1.schema.json`
  - enforce GraphMem saliency state contract.

Python contract changes:

- `packages/python/foundation/src/omni/rag/link_graph/models.py`
  - add `LinkGraphScope` and `LinkGraphEdgeType` literals/enums;
  - extend `LinkGraphSearchFilters` dataclass with tree-control fields;
  - include serialization in `to_record()`.

Rust contract changes:

- `packages/rust/crates/xiuxian-wendao/src/link_graph/models.rs`
  - add `LinkGraphScope` and `LinkGraphEdgeType` enums;
  - extend `LinkGraphSearchFilters` with matching fields;
  - keep `serde(deny_unknown_fields)` for strict validation.

### 7.7 Parser and Indexing Changes

Parsing/indexing behavior:

- parse markdown into section nodes and heading hierarchy edges;
- keep stable section ids via normalized heading slug + heading level;
- keep org-mode slot reserved but disabled for now;
- ignore attachment-style links (`![...](file.png)`, `[...](file.pdf)`) and embedded
  wikilinks (`![[...]]`) from structural edge generation;
- store node kind and edge kind in graph index so retrieval can filter by scope and edge types.

### 7.8 Tests Required

Contract tests:

- schema validation tests for each new filter field.

Rust unit/integration tests:

- section node extraction from markdown headings;
- structural edge correctness (`contains`, `parent`, reverse edges);
- attachment/embed filtering correctness (no edge pollution);
- edge-type filtered retrieval;
- `collapse_to_doc` aggregation semantics;
- anti-pollution caps (`per_doc_section_cap`, `min_section_words`).

Python tests:

- `LinkGraphSearchFilters.to_record()` coverage for new fields;
- policy tests proving only provisional edges enter gate path.

### 7.9 Rollout Plan

1. Contracts landed (schema + Python + Rust structs). (Done)
2. Parser/indexer section graph landed. (Done)
3. Retrieval filters and ranking controls landed. (Done)
4. Keep `knowledge.recall` default conservative while collecting perf evidence (`doc_only` default), enforced by unified gate command `just gate-wendao-ppr`. (Done)
5. Keep default retrieval scope at `doc_only` until rollout sign-off criteria are met.
6. Rollout sign-off criteria:
   - `just gate-wendao-ppr` passes for 7 consecutive CI runs.
   - `just gate-wendao-ppr-mixed-canary` reports `top3_rate >= 0.90` in the same 7-run window.
   - `just validate-wendao-ppr-reports` passes, validating JSON artifact contracts for both base gate and mixed canary outputs.
7. Only after all criteria are green, switch runtime default scope from `doc_only` to `mixed`.
8. Use `just wendao-ppr-rollout-status` to compute and persist sustained-green readiness (`wendao_rollout_status.json`) from current reports plus previous CI status.
9. CI is now configured with `XIUXIAN_WENDAO_ROLLOUT_STRICT_READY=1` (hard gate).
10. While strict is enabled, previous rollout status must be fetched from `run-status=completed` history (not `success` only), so failed pre-signoff runs still contribute to streak progression.
