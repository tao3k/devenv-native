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

### 2.4 Confidence Propagation as a Local Hypothesis

This audit did not verify a primary source for the second-round Bayesian
knowledge-graph claim. Confidence propagation should therefore be treated as a
local design hypothesis.

Design implication: a DuckDB read model may compute advisory
`derived_confidence` from relation context, but that value must not
automatically rewrite canonical `status`, promote candidate objects, or retire
accepted objects. State transitions still need repository governance.

### 2.5 Physical Code Evidence

The third-round code review strengthens the implementation feasibility claim:

1. `DuckDbLocalRelationEngine` already supports virtual Arrow and materialized
   appender registration strategies.
2. The current row-count strategy chooses virtual Arrow when
   `prefer_virtual_arrow` is enabled and row count is below
   `materialize_threshold_rows`; otherwise it chooses materialized appender.
3. `query_batches` prepares bounded DuckDB SQL and returns Arrow batches,
   making SQL-backed validation evidence physically plausible.
4. `register_materialized_relation` is a useful implementation anchor for a
   future semantic read-model pilot, but it is currently an internal method and
   should not be treated as the public SSOT contract.
5. `QTable` implements smoothing updates over episode utility, and the memory
   engine exposes read-only projection rows for host compute lanes. This is
   aligned with the read-model direction, but does not itself define semantic
   SSOT authority.

## 3. Artisan Audit Verdict

### 3.1 Pass Conditions

The RFC is recommended for approval review if these conditions hold:

1. semantic truth remains repo-native and reviewable
2. DuckDB remains a read model, not authority
3. SQL guards remain optional validation evidence until proven locally
4. derived confidence remains advisory and does not mutate canonical status
5. physical initialization of `semantic/` waits for Sovereign approval

### 3.2 Violations and Risks

1. **Identity collision**: object ids need a physical validator before seed
   objects can be authoritative.
2. **Read-model race**: any DuckDB projection refresh must prevent readers
   from observing partially refreshed semantic objects or relations.
3. **Orphaned relation endpoints**: relation targets must resolve to existing
   object ids.
4. **Projection drift**: LLM compression and review views need revision or
   staleness metadata.
5. **False precision**: confidence propagation can create a false sense of
   mathematical authority if no local calibration data exists.
6. **Poisoned-lock opacity**: the DuckDB local relation engine reports poisoned
   mutexes as string errors. This is better than panic propagation, but a later
   operational hardening pass should preserve richer panic context.
7. **Feature-gate density**: the DuckDB implementation is isolated under a
   dedicated module, but `cfg(feature = "duckdb")` remains dense inside the
   relation-engine implementation.
8. **Registration replacement race**: one Wendao `DataFusionLocalRelationEngine`
   implementation deregisters a table before re-registering it. The semantic
   read-model pilot should avoid exposing readers to a partially replaced
   relation.

### 3.3 Refinement Path

1. **Schema validator**: validate `id`, `kind`, `status`, `confidence`,
   `owners`, `provenance`, `verification`, and `relations`.
2. **Read-model pilot**: materialize accepted objects and relations into
   DuckDB with explicit source revision and projection revision metadata.
3. **Derived-confidence pilot**: define one advisory `derived_confidence` view
   and compare it against human review outcomes before using it in guards.
4. **SQL-guard pilot**: let one invariant emit SQL-backed validation evidence
   while retaining the repository `check_command` as the required gate.
5. **Qianji integration**: let one guard consume a bounded semantic-scope
   bundle after validator and read-model contracts are stable.
6. **Hot-path clone audit**: consider replacing repeated Q-table episode-id
   clones with shared identifiers such as `Arc<str>` only after profiling
   confirms the clone pressure is material.

## 4. Second-Round Calibration

The second-round additions are directionally useful but require downgrade:

1. unverified paper titles must not appear as formal research foundations
2. DuckDB synchronization must be presented as a read-model pilot
3. SQL guards must complement, not replace, repository validation commands
4. confidence propagation must be advisory until local calibration exists
5. watcher or latency claims must wait for implementation evidence
6. physical DuckDB support raises feasibility, but does not waive the approval
   gate for `semantic/`, validators, or runtime hooks

## 5. Final Verdict: Pass Recommendation With Conditions

The RFC is ready for Sovereign approval review after the second-round
calibration above. It is not yet approved for physical `semantic/`
initialization, DuckDB read-model implementation, SQL-guard enforcement, or
Qianji runtime hook integration.

## 6. Formal Research References

1. **Active Context Compression** (2026.01): [arXiv:2601.07190](https://arxiv.org/abs/2601.07190).
2. **LLM Agent Memory: A Survey** (2025): [OpenReview:KPs1EgGKcT](https://openreview.net/forum?id=KPs1EgGKcT).
3. **Mem0: Memory Layer for AI Agents** (2025.04): [arXiv:2504.19413](https://arxiv.org/abs/2504.19413).
4. **Semantic Commit: Detected Conflict Resolution** (2025.04): [arXiv:2504.09283](https://arxiv.org/html/2504.09283v1).
5. **AI Agents Need Memory Control Over More Context** (2026.01): [arXiv:2601.11653](https://arxiv.org/abs/2601.11653).
6. **MemGPT: Towards LLMs as Operating Systems** (2023.10): [arXiv:2310.08560](https://arxiv.org/abs/2310.08560).
7. **DuckDB Arrow Integration** (2021): [duckdb.org](https://duckdb.org/2021/12/03/duck-arrow).
8. **DuckDB Transactions**: [duckdb.org](https://duckdb.org/docs/current/sql/statements/transactions.html).
