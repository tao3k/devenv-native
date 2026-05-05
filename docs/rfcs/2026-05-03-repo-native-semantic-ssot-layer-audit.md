---
type: knowledge
title: "Audit: Repo-Native Semantic SSOT Layer"
category: "audit"
status: "implemented-first-slice"
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
- **Status**: First physical slice implemented; full RFC remains open.
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

This audit began as advisory review. The implementation approval was later
used to land the first physical slice: repo-native `semantic/` artifacts,
parser validation, CLI linting, the Wendao semantic-scope route, Studio
runtime serving, Qianji advisory consumption, explicit projection metadata
refresh through `wendao-client lint semantic --refresh-projections`, and
read-only lifecycle writeback preview through
`wendao-client lint semantic --lifecycle-plan`, plus explicit lifecycle apply
tooling through `wendao-client lint semantic --apply-lifecycle-plan`, and
closure-level projection freshness policy through
`wendao-client lint semantic --require-fresh-projections`. Studio now also
emits the same projection freshness policy evidence as
`semanticProjectionPolicyEvidence` in semantic-scope Flight metadata for
Qianji advisory consumption. The policy report type, policy id, metadata key,
and report construction now live in `xiuxian-wendao-parsers` so client lint and
Studio runtime metadata share the same semantic contract.
Semantic-scope Flight app metadata now also has a parser-owned envelope shared
by Studio producers and Qianji consumers, while SQL guard evidence remains
advisory JSON outside parser authority.
Qianji scheduler preflight now consumes that metadata at workflow runtime by
injecting a read-only `semanticScopeGuardTrace` into node context.
Workflow authors can also opt into explicit scheduler preflight blocking with
`semanticScopeGuardPolicy` values for blocked or review-required semantic
scope; the default remains advisory.
`wendao-client lint semantic --projection-refresh-plan` now renders the
read-only projection metadata refresh queue contract for future background
refresh workers, while artifact mutation remains explicit through
`--refresh-projections`.
`wendao-client semantic refresh-projections` now runs one explicit worker pass
that consumes that plan, applies the existing projection metadata writeback
path, and enforces post-refresh freshness.

The RFC is still not fully complete. Scheduling the one-shot refresh worker
from a real background runner, deeper workflow policy routing beyond scheduler
preflight blocking, and future Julia or DuckDB-backed derived lanes remain
outside the completed slice.

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

### 2.6 Julia Compute Augmentation Evidence

The fourth-round correction is that DuckDB is not the only feasible
augmentation lane. The existing code already has Julia compute surfaces that
fit the semantic SSOT direction:

1. Rust memory projection rows are explicitly read-only host-compute inputs.
2. `xiuxian-wendao-julia` defines staged memory compute profiles for episodic
   recall, memory gate scoring, memory plan tuning, and memory calibration.
3. The episodic-recall downcall composes Rust projection staging with Julia
   Arrow Flight transport and decoded score rows.
4. The graph-structural plugin surface owns Julia-specific semantic projection
   DTOs, route names, request and response helpers, and Arrow batch validation.
5. The existing package boundary states that `WendaoSearch.jl` augments Wendao
   graph search and is not a replacement for Wendao or the main graph store.

Design implication: Rust should own semantic authority, validators, and exact
provenance. DuckDB should own relational read-model queries. Julia should own
advisory compute evidence for graph diffusion, structural rerank, solver
experiments, calibration, and other numerically dense surfaces where Rust would
be slower to evolve.

## 3. Artisan Audit Verdict

### 3.1 Pass Conditions

The RFC is recommended for approval review if these conditions hold:

1. semantic truth remains repo-native and reviewable
2. DuckDB remains a read model, not authority
3. SQL guards remain optional validation evidence until proven locally
4. derived confidence remains advisory and does not mutate canonical status
5. Julia compute outputs remain advisory evidence rows, not canonical truth
6. physical initialization of `semantic/` is governed by explicit approval and
   repository validation

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
9. **Julia authority creep**: Julia is the right place for numerical and graph
   compute, but its outputs must not mutate canonical semantic objects without
   Rust-side validation and repository governance.
10. **Schema drift**: every Julia compute profile must keep request and response
    schema versions explicit so advisory evidence can be replayed and audited.

### 3.3 Refinement Path

1. **Schema validator**: validate `id`, `kind`, `status`, `confidence`,
   `owners`, `provenance`, `verification`, and `relations`.
2. **Read-model pilot**: materialize accepted objects and relations into
   DuckDB with explicit source revision and projection revision metadata.
3. **Derived-confidence pilot**: define one advisory `derived_confidence` view
   and compare it against human review outcomes before using it in guards.
4. **SQL-guard pilot**: let one invariant emit SQL-backed validation evidence
   while retaining the repository `check_command` as the required gate.
5. **Julia compute pilot**: export one bounded semantic subgraph or
   derived-confidence input batch to a staged Julia compute profile and import
   advisory evidence rows through versioned Arrow contracts.
6. **Qianji integration**: extend the current guard from semantic-scope and
   policy-evidence consumption toward workflow-level planning decisions after
   validator and read-model contracts are stable.
7. **Hot-path clone audit**: consider replacing repeated Q-table episode-id
   clones with shared identifiers such as `Arc<str>` only after profiling
   confirms the clone pressure is material.

## 4. Second-Round Calibration

The second-round additions are directionally useful but require downgrade:

1. unverified paper titles must not appear as formal research foundations
2. DuckDB synchronization must be presented as a read-model pilot
3. SQL guards must complement, not replace, repository validation commands
4. confidence propagation must be advisory until local calibration exists
5. watcher or latency claims must wait for implementation evidence
6. physical DuckDB support raises feasibility, but does not become semantic
   authority
7. Julia compute support raises feasibility for the weak Rust compute surfaces,
   but it must stay behind explicit Arrow schemas and advisory evidence rules

## 5. Current Verdict: First Slice Landed With Conditions

The RFC has passed the first physical implementation slice. Repo-native
semantic objects are now the authority surface; Wendao validates and serves a
scoped bundle; Qianji consumes the semantic surface as advisory context.

This does not close the full RFC. DuckDB and Julia remain derived or advisory
lanes, SQL guard evidence is not authority, and candidate promotion or
retirement apply tooling still needs a separate pass before the semantic
layer can be considered fully landed. Minimal parser governance now
prevents free-floating candidate objects by requiring `llm_suggested`
confidence and an active change-intent `candidate_suggestions` reference.
Change intents can now declare landed `status_transitions`, with parser
validation for the current target status and allowed lifecycle edge, and
explicit `promotion_targets` / `demotion_targets` for lifecycle outcomes.
`wendao-client lint semantic --lifecycle-plan` now renders a read-only
writeback preview for those lifecycle outcomes, and
`wendao-client lint semantic --apply-lifecycle-plan` can explicitly apply
pending lifecycle transitions before re-validating the repository semantic
surface. `wendao-client lint semantic --require-fresh-projections` now lets
callers enforce that active change-intent projection refresh targets are fresh
before closing a semantic change.
`wendao-client lint semantic --projection-refresh-plan` now exposes the
read-only refresh plan that a future background refresh worker can consume
without silently mutating repo-native projection artifacts.
`wendao-client semantic refresh-projections` now provides the one-shot worker
entrypoint for that contract while still routing writes through the explicit
projection metadata refresh implementation.

## 6. Formal Research References

1. **Active Context Compression** (2026.01): [arXiv:2601.07190](https://arxiv.org/abs/2601.07190).
2. **LLM Agent Memory: A Survey** (2025): [OpenReview:KPs1EgGKcT](https://openreview.net/forum?id=KPs1EgGKcT).
3. **Mem0: Memory Layer for AI Agents** (2025.04): [arXiv:2504.19413](https://arxiv.org/abs/2504.19413).
4. **Semantic Commit: Detected Conflict Resolution** (2025.04): [arXiv:2504.09283](https://arxiv.org/html/2504.09283v1).
5. **AI Agents Need Memory Control Over More Context** (2026.01): [arXiv:2601.11653](https://arxiv.org/abs/2601.11653).
6. **MemGPT: Towards LLMs as Operating Systems** (2023.10): [arXiv:2310.08560](https://arxiv.org/abs/2310.08560).
7. **DuckDB Arrow Integration** (2021): [duckdb.org](https://duckdb.org/2021/12/03/duck-arrow).
8. **DuckDB Transactions**: [duckdb.org](https://duckdb.org/docs/current/sql/statements/transactions.html).
