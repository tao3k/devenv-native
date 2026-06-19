---
type: knowledge
title: "Wendao Plan Consolidation (2026)"
category: "plans"
tags:
  - wendao
  - link-graph
  - rollout
  - execution
saliency_base: 8.0
decay_rate: 0.02
metadata:
  title: "Wendao Plan Consolidation (2026)"
---

# Wendao Plan Consolidation (2026)

> Status: Active (core milestones landed; hardening in progress)  
> Date: February 20, 2026  
> Program: `xiuxian-wendao` LinkGraph and agentic graph evolution

## 1. Purpose

Unify LinkGraph-related plan documents into one execution entrypoint so implementation does not drift across parallel drafts.

## 1.1 Flexible Context

This consolidation plan governs execution across:

- `[[LinkGraph PPR Algorithm Spec|docs/01_core/wendao/ppr-algorithm.md]]`
- `[[Rust Context Injection + Memory Self-Evolution + Reflection|docs/01_core/memory/injection-evolution.md]]`

## 2. Canonical Document Set

| Role                                  | Document                                     | Ownership                                          |
| ------------------------------------- | -------------------------------------------- | -------------------------------------------------- |
| Retrieval algorithm source of truth   | `docs/01_core/wendao/ppr-algorithm.md`       | `xiuxian-wendao` LinkGraph core (`src/link_graph`) |
| Agentic proposal and promotion policy | `docs/01_core/memory/injection-evolution.md` | Rust context injection and memory evolution policy |

Conflict policy:

1. Retrieval behavior conflict -> resolve in `ppr-algorithm.md`.
2. Agentic lifecycle conflict -> resolve in `injection-evolution.md`.
3. Citation or terminology mismatch -> resolve in the active package documentation, then propagate.

## 3. Unified Execution Backlog (Wendao)

1. W0 Contracts (Done)

- Add PPR-related schema fields and keep Python/Rust model parity.
- Gate:
  `uv run pytest packages/python/foundation/tests/unit/api/test_link_graph_search_options_schema.py -q`

2. W1 PPR Kernel (Done)

- Implement PPR scorer in `packages/rust/crates/xiuxian-wendao/src/link_graph/index`.
- Gate:
  `cargo test -p xiuxian-wendao --test test_link_graph`

3. W2 Replace `related` BFS Path (Done)

- Route related retrieval through PPR ranking.
- Gate:
  `cargo test -p xiuxian-wendao --test test_link_graph test_link_graph_neighbors_related_metadata_and_toc`

4. W3 Subgraph Partition and Fusion (Done)

- Add divide-and-conquer path for large graph queries with bounded resource budgets.
- Runtime guards are wired (`max_candidates`, `max_partitions`, `time_budget_ms`) and
  exposed in diagnostics (`candidate_cap`, `candidate_capped`, `timed_out`).
- Verbose monitor phases are wired:
  - `link_graph.related.ppr`
  - `link_graph.related.subgraph.partition`
  - `link_graph.related.subgraph.fusion`
- Gate:
  `just gate-wendao-ppr`

5. W4 Agentic Graph Evolution (Done)

- Keep all suggested edges provisional first, add promotion traceability.
- Phase A completed: passive suggested-link logging in Valkey stream with schema contract:
  - `packages/rust/crates/xiuxian-wendao/resources/xiuxian_wendao.link_graph.suggested_link.v1.schema.json`
  - `packages/rust/crates/xiuxian-wendao/src/link_graph/agentic/*`
- Phase C groundwork landed:
  - transition API (`provisional -> promoted/rejected`) with decision audit stream:
    - `valkey_suggested_link_decide(_with_valkey)`
    - `valkey_suggested_link_decisions_recent(_with_valkey)`
  - latest-state read path for proposal surfaces:
    - `valkey_suggested_link_recent_latest(_with_valkey)`
  - CLI operator commands:
    - `wendao agentic log`
    - `wendao agentic recent`
    - `wendao agentic decide`
    - `wendao agentic decisions`
- Phase C completed (engine-level materialization):
  - promoted suggestions are applied as verified query-time graph overlays;
  - no markdown rewrite is required for promoted links to become query-visible;
  - affected retrieval surfaces:
    - `wendao search` graph filters/paths
    - `wendao neighbors`
    - `wendao related`
  - verbose observability surfaces:
    - `wendao search --verbose` (phases + monitor bottlenecks)
    - `wendao neighbors --verbose` (overlay phase + monitor bottlenecks)
    - `wendao related --verbose` (PPR/partition/fusion phases + overlay phase + monitor bottlenecks)
- Phase B completed (engine-level):
  - Provisional suggested-link retrieval is resolved in core search runtime (config-first).
  - Engine-level hybrid policy now consumes provisional suggestions to:
    - boost matching document scores in ranking;
    - inject matched provisional candidates into result rows when lexical rows are absent.
  - CLI remains a thin override surface:
    - `wendao search --include-provisional[=true|false] --provisional-limit <N>`.
- Phase D scheduler slice landed (Rust core):
  - bounded expansion planner with runtime budgets:
    - `max_workers`
    - `max_candidates`
    - `max_pairs_per_worker`
    - `time_budget_ms`
  - CLI operator planning command:
    - `wendao agentic plan --query <q> [budget overrides]`.
- Phase D execution runtime landed (Rust core):
  - bounded execution with worker-level telemetry:
    - processed/persisted/failed proposal counts
    - duplicate-skip and retry-attempt counters
    - worker elapsed milliseconds
    - worker phase timeline (`prepare/dedupe/persist/total`)
    - estimated prompt/completion token placeholders
  - config-first execution defaults:
    - `link_graph.agentic.execution.worker_time_budget_ms`
    - `link_graph.agentic.execution.persist_suggestions_default`
    - `link_graph.agentic.execution.persist_retry_attempts`
    - `link_graph.agentic.execution.idempotency_scan_limit`
    - `link_graph.agentic.execution.relation`
    - `link_graph.agentic.execution.agent_id`
    - `link_graph.agentic.execution.evidence_prefix`
  - CLI operator execution command:
    - `wendao agentic run --query <q> [budget overrides] [--persist[=true|false]] [--verbose]`.
- 2026-02-21 hardening evidence:
  - `cargo test -p xiuxian-wendao --test test_link_graph --test test_link_graph_agentic --test test_link_graph_agentic_expansion --test test_wendao_cli`
  - `cargo test -p xiuxian-wendao`
- Gate:
  proposal and promotion boundary tests in `xiuxian-wendao` plus schema validation in
  foundation layer.

6. W5 Default Rollout (Done for related path)

- Default related retrieval behavior is PPR-only.
- No compatibility ranking path is retained.

7. W6 Markdown AST Configuration Parser (Done)

- Implement a Rust-native Markdown AST traversal using the existing `comrak` crate (currently used in `xiuxian-wendao`) to extract Org-Mode style HTML properties (`<!-- id: "...", type: "..." -->`) and fenced code blocks (`jinja2`) bound to specific heading nodes.
- Store extracted blocks (Personas, Templates, Skill Manuals) into the `xiuxian-wendao` memory index using the extracted `id` as the $O(1)$ primary key.
- Provide a zero-export read interface so downstream context engines can pull configuration directly from the graph memory.
- **Deep Dive**: See [[docs/01_core/wendao/architecture/id-resolution-mechanism.md|ID Resolution Mechanism]] for the $O(1)$ technical implementation details.
- Evidence: `packages/rust/crates/xiuxian-wendao/src/enhancer/markdown_config.rs` implemented and verified via `cargo nextest`.

8. W7 Unified HTTP Gateway Integration (In Progress)

- Instead of `xiuxian-wendao` building its own isolated HTTP server, it will integrate with the centralized `xiuxian-zhenfa` (阵法) service.
- **Router Registration:** Wendao will expose a standard trait (e.g., `ZhenfaRouter`) that `xiuxian-zhenfa` mounts onto its high-performance `axum` backend.
- **Decoupling Context Injection:** The zero-export interface (from W6) will be served over this central HTTP Matrix, allowing runtime consumers to request persona profiles, templates, and Skill Manuals via standard REST/JSON contracts instead of direct memory linking.
- Evidence: `packages/rust/crates/xiuxian-zhenfa` core networking layer and contracts bootstrapped.

- Instead of `xiuxian-wendao` building its own isolated HTTP server, it will integrate with the centralized `xiuxian-zhenfa` (阵法) service.
- **Router Registration:** Wendao will expose a standard trait (e.g., `ZhenfaRouter`) that `xiuxian-zhenfa` mounts onto its high-performance `axum` backend.
- **Decoupling Context Injection:** The zero-export interface (from W6) will be served over this central HTTP Matrix, allowing runtime consumers to request persona profiles, templates, and Skill Manuals via standard REST/JSON contracts instead of direct memory linking.

## 4. Change Control Rules

1. No skill-layer implementation of graph core algorithms.
2. Python stays as binding/adapter for `xiuxian-wendao` runtime behavior.
3. All plan updates must include:

- changed command(s),
- changed gate(s),
- changed owner module(s).
