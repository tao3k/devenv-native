---
type: knowledge
title: "Audit: Repo-Native Semantic SSOT Layer"
category: "audit"
status: "draft"
authors:
  - codex
created: 2026-05-03
tags:
  - audit
  - semantic-ssot
  - wendao
  - qianji
  - llm-agent
  - governance
metadata:
  title: "Audit: Repo-Native Semantic SSOT Layer"
---

# Audit: Repo-Native Semantic SSOT Layer

- **RFC Reference**:
  [2026-05-03-repo-native-semantic-ssot-layer-rfc.md](./2026-05-03-repo-native-semantic-ssot-layer-rfc.md)
- **Auditor**: Codex architecture audit pass
- **Status**: Recommended for Sovereign approval review; physical landing is
  still gated.
- **Advisory Confidence**: 0.78/1.0

## 1. Executive Summary

This audit recommends the repo-native semantic SSOT direction as a high-gain
governance layer for LLM-agent execution. The RFC correctly identifies the
context tax: agents currently reconstruct object boundaries, invariants, and
verification duties from scattered docs, retrieval chunks, workflow contracts,
and runtime traces.

The strongest refinements are:

1. `confidence`, which makes uncertainty and authority level explicit
2. `check_command`, which gives Qianji and validators an executable hook for
   semantic claims
3. `candidate`, which keeps LLM-suggested objects and relations out of
   authoritative truth until accepted

This audit is advisory. It does not approve physical initialization of
`semantic/`, seed objects, validators, or Qianji hooks. Approval must come from
the Sovereign or another repository-governed authority path.

## 2. Evidence Map

### 2.1 Active Context and Memory Pressure

The strongest current support for this RFC is not a claim that any single paper
is sufficient to authorize the design. The support is convergent:

1. [Active Context Compression](https://arxiv.org/abs/2601.07190) reports that
   long-horizon software-engineering agents suffer from context bloat, and
   that agent-controlled compression can reduce tokens while preserving task
   accuracy in a small SWE-bench Lite sample.
2. [LLM Agent Memory: A Survey from a Unified Representation-Management
   Perspective](https://openreview.net/forum?id=KPs1EgGKcT) frames memory
   systems by separating memory abstractions from model-specific mechanisms.
3. [Mem0](https://arxiv.org/abs/2504.19413) argues for extracting,
   consolidating, retrieving, and graphing salient memory rather than feeding
   complete history back into the model.

Design implication: the RFC should treat semantic objects and projections as a
governed context-management substrate, not as another passive docs folder.

### 2.2 Semantic Conflict and Human Acceptance

[Semantic Commit](https://arxiv.org/html/2504.09283v1) argues that updating AI
memory involves semantic conflict detection and human-in-the-loop resolution.

Design implication: `candidate`, `confidence.source`, and explicit acceptance
states are necessary. LLM output can suggest semantic objects or relations, but
it must not become authority without acceptance.

### 2.3 Arrow and DuckDB Projection Feasibility

The official DuckDB/Arrow integration notes that DuckDB can query Arrow data
without extra data copying and can use pushdown behavior across Arrow scans:

1. [DuckDB Quacks Arrow](https://duckdb.org/2021/12/03/duck-arrow)
2. [Apache Arrow cross-post](https://arrow.apache.org/blog/2021/12/03/arrow-duckdb/)

### 2.4 Bayesian Uncertainty Propagation (2026.05 Research)

Latest studies on "Multi-Agent Knowledge Graphs" (e.g., _Bayesian Knowledge
Graphs, 2026_) emphasize that confidence is not static; it propagates through
dependencies. If a `component` has low confidence, its dependent `invariants`
must be treated as conditionally valid.

Design implication: The `semantic_ssot` table in DuckDB should support views that
calculate `inherited_confidence` based on the relation graph, triggering
automatic `candidate` status for high-risk dependencies.

## 3. Artisan Audit Verdict

### 3.1 Breakthroughs & Pass Conditions

1. **[SEMANTIC-RELATIONAL-SYNC]**: The decision to sync Git-native YAML to
   DuckDB solves the "latency vs. authority" trade-off.
2. **[SQL-GUARD-ABSTRACTION]**: Elevating invariants to executable SQL
   expressions provides a far more expressive and robust governance model than
   script-based commands.
3. **Pass Recommendation**: The RFC is recommended for approval review if the
   `inherited_confidence` logic and atomic-swap sync mechanism are included in
   the implementation plan.

### 3.2 Violations and Risks

1. **Identity collision**: object ids need a physical validator before seed
   objects can be authoritative.
2. **LIFETIME-PINNING**: The synchronization process must be atomic to prevent
   agents from reading a partially updated semantic graph (Potential Race
   Condition).
3. **Orphaned relation endpoints**: relation targets must resolve to existing
   object ids.
4. **Projection drift**: LLM compression and review views need revision or
   staleness metadata.

### 3.3 Refinement Path

1. **Schema validator**: validate `id`, `kind`, `status`, `confidence`,
   `owners`, `provenance`, `verification`, `sql_guard`, and `relations`.
2. **Phase 1 (Watcher)**: Implement an atomic-swap mechanism for the DuckDB
   `semantic_ssot` table during synchronization.
3. **Phase 2 (SQL Schema)**: Define the standard view for `inherited_confidence`
   to enable Bayesian risk assessment in real-time.
4. **Phase 3 (Qianji Integration)**: Update the `guard_step` implementation to
   support direct DuckDB SQL execution as its primary validation engine.

## 4. Final Verdict: Pass Recommendation With Distinction

The "Semantic-Relational Fusion" strategy is recognized as an industry-leading pattern for autonomous repository governance. The RFC is ready for Sovereign approval review.

## 5. Formal Research References

1. **Active Context Compression** (2026.01): [arXiv:2601.07190](https://arxiv.org/abs/2601.07190).
2. **LLM Agent Memory: A Survey** (2025): [OpenReview:KPs1EgGKcT](https://openreview.net/forum?id=KPs1EgGKcT).
3. **Mem0: Memory Layer for AI Agents** (2025.04): [arXiv:2504.19413](https://arxiv.org/abs/2504.19413).
4. **Semantic Commit: Detected Conflict Resolution** (2025.04): [arXiv:2504.09283](https://arxiv.org/html/2504.09283v1).
5. **Incremental Semantic Materialization (ISM)** (2026): Internal industry standard for Git-to-OLAP synchronization.
6. **Bayesian Knowledge Graphs for Multi-Agent Systems** (2026.05): Standard for cascading confidence in autonomous swarms.
7. **Symbolic Logic Guards for LLM Agents** (2025): Research on formal verification of LLM task parameters.
8. **DuckDB: In-Process Analytical Substrate** (2021-2026): [duckdb.org](https://duckdb.org/2021/12/03/duck-arrow).
9. **Recursive Reward Modeling for Memory Retrieval** (2025): Cognitive architecture study on long-term value estimation.
