---
type: knowledge
title: "RFC: Repo-Native Semantic SSOT Layer"
category: "rfc"
status: "implemented-first-slice"
authors:
  - codex
created: 2026-05-03
tags:
  - rfc
  - semantic-ssot
  - wendao
  - qianji
  - llm-agent
  - projection
  - governance
metadata:
  title: "RFC: Repo-Native Semantic SSOT Layer"
---

# RFC: Repo-Native Semantic SSOT Layer

## 1. Summary

This RFC proposes a repo-native semantic single-source-of-truth layer for
Xiuxian Artisan Workshop.

The core decision is:

1. canonical semantic truth should be represented as explicit repository
   objects and typed relations
2. those objects and relations should carry minimal governance metadata:
   status, owner, provenance, and verification
3. Wendao should query semantic objects and relation subgraphs, not only
   document chunks, snippets, and file paths
4. Qianji should derive execution context from task-scoped semantic subgraphs,
   not from scattered documents or unstructured retrieval alone
5. human, LLM, review, and operations views should become projections from
   the same semantic truth layer

This is not a request to add a feature module. It is a proposal to make the
repository's shared truth explicit enough for LLM agents, workflow engines,
retrieval systems, reviewers, and operators to consume different views without
forking the underlying meaning.

### 1.1 Implementation Status

As of 2026-05-05, the first physical slice is implemented:

1. canonical semantic artifacts live under `semantic/`
2. `xiuxian-wendao-parsers` validates semantic objects, projections, and
   change intents
3. `wendao-client lint semantic` validates the repository-native semantic
   surface
4. Wendao Flight exposes the transport-only `/analysis/semantic-scope` route
5. Studio provides the real route provider by loading repo semantic artifacts
   and returning Arrow rows plus full bundle metadata
6. Qianji consumes semantic scope, change-intent, SQL-guard, and projection
   policy evidence as advisory planning context without owning semantic truth
7. projection source revisions can be refreshed explicitly with
   `wendao-client lint semantic --refresh-projections`
8. candidate semantic objects must remain `llm_suggested` and be governed by
   active change-intent `candidate_suggestions` entries
9. semantic change intents can declare landed `status_transitions` that the
   parser validates against current repo facts and allowed lifecycle edges
10. candidate promotion and object demotion outcomes are explicit
    `promotion_targets` and `demotion_targets` entries, cross-checked against
    landed status transitions
11. `wendao-client lint semantic --lifecycle-plan` renders a read-only
    lifecycle writeback preview for validated promotion, demotion, and other
    status-transition outcomes
12. `wendao-client lint semantic --apply-lifecycle-plan` applies pending
    lifecycle transitions explicitly, including candidate promotion metadata
    writeback, before re-validating the repo-native semantic surface
13. `wendao-client lint semantic --require-fresh-projections` enforces a
    parser-owned closure-level policy that active change-intent projection
    refresh targets are fresh
14. Studio emits the same projection freshness policy evidence through
    semantic-scope Flight metadata as `semanticProjectionPolicyEvidence`,
    using the shared parser-owned policy report contract
15. semantic-scope Flight app metadata now uses a parser-owned envelope
    contract shared by Studio producers and Qianji consumers; SQL guard
    evidence stays advisory JSON and projection freshness evidence stays typed
    by the semantic parser contract
16. Qianji scheduler preflight can inject a read-only
    `semanticScopeGuardTrace` into workflow context when the run context
    carries Wendao semantic-scope metadata, giving mechanisms advisory
    semantic status, issues, validations, and projection evidence. It also
    injects `semanticScopeGuardRoute`, a compact routing decision with the
    configured policy, guard status, execution outcome, and recommended
    semantic action for downstream mechanisms.
17. Qianji scheduler preflight supports explicit `semanticScopeGuardPolicy`
    values: `advisory`, `block_on_blocked`, and
    `block_on_review_required`. The default remains advisory, while
    workflow authors can opt into preflight blocking for unresolved or
    review-required semantic scope.
18. `wendao-client lint semantic --projection-refresh-plan` renders a
    read-only, parser-owned projection metadata refresh plan. This gives a
    future background refresh worker an explicit queue contract while keeping
    actual projection artifact mutation behind `--refresh-projections`.
19. `wendao-client semantic refresh-projections` runs an explicit projection
    metadata refresh worker over repo-native semantic artifacts. Its default
    mode remains a single pass, while `--interval-secs` and `--max-runs` let a
    supervised process run the same worker as a bounded or long-running
    recurring runner. `--require-clean-worktree` lets supervised starts refuse
    to write projection metadata when the root git worktree already has
    pending changes. Each pass uses the parser-owned refresh plan, applies the
    existing explicit projection writeback path, renders the post-refresh plan,
    and enforces projection freshness before returning success.
20. `process-compose` now packages that runner as `wendao-semantic-refresh`.
    The process delegates to managed scripts under
    `scripts/channel/processes/wendao-semantic-refresh/`, writes pid/log state
    under the project runtime root, builds the existing `wendao-client` binary
    by default, and runs `semantic refresh-projections --require-clean-worktree`
    with `WENDAO_SEMANTIC_REFRESH_INTERVAL_SECS` and
    `WENDAO_SEMANTIC_REFRESH_MAX_RUNS` operator controls. It has no downstream
    service dependency and does not make projections authoritative.
21. Qianji router nodes can now opt into workflow-level semantic guard route
    consumption with `semantic_guard_route = true` or an explicit
    `semantic_guard_route_key`. When enabled, a
    `semanticScopeGuardRoute.recommendedAction` value matching a configured
    branch selects that branch before probabilistic fallback. The default
    router path remains unchanged and semantic truth remains read-only context.
22. Qianji now has a checked-in guard route-aware workflow fixture at
    `packages/rust/crates/xiuxian-qianji/resources/tests/semantic_guard_route_branch.toml`.
    The integration test compiles that ordinary TOML manifest and proves stale
    semantic scope selects the `review_required` branch while leaving
    `continue` and `blocked` branches inactive.
23. `qianji template --semantic-guard-route` now renders that workflow shape as
    an operator-facing TOML template. The command is authoring support only:
    Qianji still reads semantic guard-route context at runtime and does not
    own canonical semantic artifacts.
24. `wendao-client lint semantic --read-model-summary` now renders an
    advisory row/table summary for the provisional semantic read model,
    including `semantic_objects`, `semantic_relations`, and
    `semantic_projection_state` counts. The summary is read-only and keeps
    repo-native semantic artifacts as the authority source.
25. `wendao-client semantic query-read-model --query SQL` now executes
    read-only SQL against the same provisional semantic read-model tables and
    returns text, JSON, or pretty JSON query payloads. This gives operators a
    direct evidence query surface without changing semantic authority.

The full RFC is not complete. Remaining work includes wider rollout of
semantic guard route-aware real workflows, DuckDB-backed materialized read
model expansion, and future Julia compute expansion. Those remain advisory or
derived lanes; they do not change repo-native authority.

## 2. Alignment

### 2.1 Stable References

1. [Documentation Design](../DOCUMENTATION_DESIGN.md)
2. [Xiuxian-Zhixing Theoretical Foundations](../99_llm/xiuxian_zhixing_theory.md)
3. [RFC: Wendao Memory Layer Boundaries](2026-04-05-wendao-memory-layer-boundaries-rfc.md)
4. [RFC: DuckDB as a Bounded In-Process Analytic Lane for Wendao and Qianji](2026-04-08-wendao-qianji-duckdb-bounded-analytics-rfc.md)
5. [Wendao SPEC](../01_core/wendao/SPEC.md)

### 2.2 External Evidence and Research Constraints

The external literature supports the direction only as design evidence, not as
approval authority:

1. [Active Context Compression](https://arxiv.org/abs/2601.07190) and
   [AI Agents Need Memory Control Over More Context](https://arxiv.org/abs/2601.11653)
   both argue that long-running agents need bounded, actively managed context
   rather than unbounded transcript replay.
2. [Mem0](https://arxiv.org/abs/2504.19413) supports structured long-term
   memory and graph-backed retrieval as useful agent memory patterns.
3. [Semantic Commit](https://arxiv.org/abs/2504.09283) supports impact
   analysis, semantic conflict detection, and human acceptance when updating
   AI memory or intent specifications.
4. [DuckDB Arrow integration](https://duckdb.org/2021/12/03/duck-arrow) and
   [DuckDB transaction semantics](https://duckdb.org/docs/current/sql/statements/transactions.html)
   support DuckDB as a plausible read-model substrate, but do not make DuckDB
   the authority source.

Any claim about repo-specific latency, confidence propagation, SQL-guard
coverage, or synchronization semantics must be proven by local implementation
evidence before it becomes canonical.

## 3. Problem Statement

The project already has three unusually strong foundations:

1. Qianji provides a bounded and increasingly governable execution plane.
2. Wendao is moving beyond document search toward a unified knowledge and
   query substrate.
3. The documentation hierarchy already recognizes that human readers and LLM
   agents need different surfaces.

The missing layer is a shared semantic truth model that can answer:

1. which components, decisions, and invariants a task affects
2. which semantic objects a workflow operates on
3. which canonical objects should be cut into an LLM execution context
4. what a change means semantically, beyond which files changed
5. how human, LLM, review, and operations views share one truth source

## 4. Critical Frame: The LLM-Agent Paradigm Shift

The new paradigm is not simply "RAG plus agents". That framing is too weak.
RAG gives an agent relevant text. Workflow gives an agent steps. Neither gives
the agent a governed model of what the repository believes to be true.

An LLM agent needs a semantic operating surface:

1. objects it can name without ambiguity
2. relations it can traverse without guessing
3. invariants it can treat as constraints
4. status and provenance it can use to rank trust
5. verification requirements it can follow before declaring closure
6. projections that match the current task and role

## 5. Design Principles

### 5.1 Semantic Truth Must Be Explicit

Important architecture objects must have stable identities. A component,
decision, invariant, or task should be addressable as an object, not only as a
heading inside a document or a phrase inside a search result.

### 5.2 Narrative Remains Valuable

The semantic layer should not reduce design to bare JSON. The body of an
artifact can remain narrative because humans and LLMs both need context. The
canonical truth, however, must live in structured fields and typed relations
that validators can read.

### 5.3 Projection Is Not Authority

Human docs, LLM compression views, review views, and operations views are
read models. They should be generated or maintained from semantic objects, but
they must not become independent truth sources.

### 5.4 Execution Graph Is Separate From Semantic Graph

Qianji's execution graph describes how work moves through steps, guards, and
handoffs. The semantic graph describes which objects exist, what they mean,
what constrains them, and what changed. Qianji should consume semantic
subgraphs; it should not become the ontology owner.

### 5.5 LLM Output Is Never an Authority Source

LLM output may propose objects, relations, impact summaries, and projection
updates. It is not authoritative until accepted through repository governance.

### 5.6 Repo-Native First

The first implementation should use versioned repository artifacts. A database
can later materialize fast read models, but the authoritative source should be
reviewable in git and auditable by repository validators.

### 5.7 Start With Stable Architecture Objects

The first ontology should cover durable architecture-level objects:

1. components
2. decisions
3. invariants
4. tasks or changes

## 6. Proposed Object Model

### 6.1 Object Kinds

The initial object kinds should be:

| Kind        | Meaning                                                                            |
| ----------- | ---------------------------------------------------------------------------------- |
| `component` | A stable subsystem, crate, service, runtime surface, or governed package boundary. |
| `decision`  | A durable architectural choice with rationale and rejected alternatives.           |
| `invariant` | A rule that must remain true across implementation and review.                     |
| `task`      | A bounded work item, migration slice, or change intent tied to semantic impact.    |

### 6.2 Required Object Fields

Every canonical semantic object should carry:

| Field          | Purpose                                                                                |
| -------------- | -------------------------------------------------------------------------------------- |
| `id`           | Stable semantic identifier.                                                            |
| `kind`         | Object kind.                                                                           |
| `title`        | Human-readable label.                                                                  |
| `status`       | Minimal lifecycle state.                                                               |
| `confidence`   | [NEW] Trust score (0.0-1.0) and source type (human_signed, llm_suggested, verified).   |
| `owners`       | Accountable maintainers, teams, agents, or packages.                                   |
| `provenance`   | Source documents, code paths, RFCs, tests, or prior decisions that justify the object. |
| `verification` | Required validation evidence and optional `check_command` for automated audits.        |
| `relations`    | Explicit outgoing relation declarations.                                               |

Optional pilot fields may be introduced only after schema approval:

| Field                | Purpose                                                                                                               |
| -------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `derived_confidence` | Read-model confidence computed from relation context. It is advisory and must not overwrite canonical `status`.       |
| `sql_guard`          | Optional query-backed validation evidence against a DuckDB read model. It complements required repository validation. |

### 6.3 Status Vocabulary

The initial status vocabulary should be small:

| Status       | Meaning                                                                       |
| ------------ | ----------------------------------------------------------------------------- |
| `draft`      | Proposed but not accepted.                                                    |
| `candidate`  | [NEW] LLM-suggested object awaiting human sign-off or automated verification. |
| `active`     | Current authoritative truth.                                                  |
| `superseded` | Replaced by another object.                                                   |
| `deprecated` | Still present but should not be used for new work.                            |
| `retired`    | No longer active and not expected to return.                                  |

Current parser governance requires each `candidate` object to use
`confidence.source: llm_suggested` and to be named by an active semantic
change intent `candidate_suggestions` entry. Promotion from `candidate` to
`active` remains a repository-governed workflow. Change intents may declare
landed `status_transitions`; the checked-out object status must match the
transition target status, and the parser validates the lifecycle edge without
mutating any object. Promotion and demotion outcomes must also be named
explicitly in `promotion_targets` and `demotion_targets` so lifecycle closure
is reviewable without inferring intent from status alone.

### 6.4 Example Shape

```yaml
id: component.wendao.query-substrate
kind: component
title: Wendao Query Substrate
status: active
confidence:
  score: 1.0
  source: human_signed
  last_audit: 2026-05-03
owners:
  - package: packages/rust/crates/xiuxian-wendao
provenance:
  - docs/rfcs/2026-03-26-wendao-query-engine-rfc.md
  - docs/rfcs/2026-04-08-wendao-qianji-duckdb-bounded-analytics-rfc.md
verification:
  required:
    - cargo test -p xiuxian-wendao
  check_command: "direnv exec . cargo test -p xiuxian-wendao --lib"
relations:
  - kind: constrains
    target: invariant.wendao.flight-boundary-remains-external
  - kind: consumed_by
    target: component.qianji.execution-plane
```

## 7. Read-Model and Compute Pilots

DuckDB can become a high-performance read model for semantic objects, and Julia
can become a high-performance compute augmentation lane, only after the
repository-native object schema is accepted. Neither should become the write
authority for semantic truth.

### 7.2 Responsibility Split

The physical split should be:

1. Rust owns canonical artifacts, schema validation, provenance, status
   transitions, repository validation commands, and final authority decisions.
2. DuckDB owns bounded relational read models and SQL-backed validation
   evidence over accepted semantic projections.
3. Julia owns advisory compute over compact Arrow-shaped projections when the
   computation benefits from graph, numerical, solver, or calibration
   libraries.
4. Wendao owns query/projection orchestration and evidence disclosure.
5. Qianji consumes bounded semantic scope and validation evidence, but does not
   own ontology truth.

### 7.3 DuckDB Code-Backed Feasibility

The current DuckDB codebase makes the read-model pilot materially more feasible
than a purely speculative design:

1. `DuckDbLocalRelationEngine` already supports two request-scoped relation
   registration strategies: virtual Arrow views and materialized Arrow
   appender tables.
2. The strategy selection already uses row-count-aware routing through
   `prefer_virtual_arrow` and `materialize_threshold_rows`.
3. `query_batches` already prepares bounded DuckDB SQL and returns Arrow record
   batches, which is sufficient for a SQL-backed validation evidence pilot.

This does not make SQL guards authoritative. It means the first pilot can reuse
existing relation-engine capabilities instead of introducing a new database
subsystem.

### 7.4 Julia Code-Backed Feasibility

The current Julia-facing codebase makes compute augmentation feasible without
making Julia a second SSOT:

1. The memory engine already exports read-only projection rows for host compute
   lanes, including current `q_value` and recall counters.
2. `xiuxian-wendao-julia` already defines staged memory compute profiles for
   episodic recall, memory gate scoring, plan tuning, and calibration.
3. The Julia memory downcall path already composes Rust projection staging with
   Arrow Flight request/response contracts.
4. The graph-structural plugin surface already owns Julia-specific semantic
   projection DTOs, route names, request-row helpers, response-row helpers, and
   Arrow batch validation for mixed-graph structural routes.

This keeps Rust responsible for ownership, schema validation, and provenance
while allowing Julia to provide advisory numerical, solver, graph-diffusion,
reranking, and calibration evidence through bounded contracts.

### 7.5 Pilot Contract

1. materializing accepted semantic objects into a provisional
   `semantic_objects` table
2. materializing accepted relations into a provisional `semantic_relations`
   edge table
3. attaching source revision, projection revision, and staleness metadata to
   every read-model row
4. running bounded SQL queries that produce validation evidence for invariants
5. refreshing the read model through a transaction or snapshot-swap discipline
   so readers never observe a partially refreshed graph
6. exporting bounded semantic subgraphs or derived read-model rows to Julia
   compute lanes only through versioned Arrow contracts
7. accepting Julia outputs only as advisory evidence rows, never as canonical
   object state

The first pilot should also observe these constraints:

1. treat `register_materialized_relation` as an implementation anchor, not as
   the public SSOT contract, unless a dedicated public API is approved
2. avoid promising a dedicated watcher, refresh latency, or recursive
   confidence propagation until local evidence exists
3. audit repeated registration behavior before reusing table names across
   concurrent query windows
4. keep repository `check_command` validation as the required gate while SQL
   guards remain evidence-producing read-model queries
5. keep Julia compute outputs subordinate to Rust-owned schema validation,
   provenance, and Sovereign approval

Current implementation evidence: `xiuxian-wendao-sql` already projects
validated semantic repositories into `semantic_objects`, `semantic_relations`,
and `semantic_projection_state`, and `wendao-client lint semantic
--read-model-summary` exposes those row counts as advisory operator context.
`wendao-client semantic query-read-model --query SQL` also exposes bounded
read-only SQL queries over those tables. This is not yet a DuckDB
materialization or Julia compute slice.

## 8. Proposed Relation Model

### 8.1 Initial Relation Kinds

| Relation      | Meaning                                                |
| ------------- | ------------------------------------------------------ |
| `contains`    | Source owns or includes target as a subpart.           |
| `depends_on`  | Source requires target to remain valid.                |
| `constrains`  | Source imposes a rule on target.                       |
| `implements`  | Source implements a decision, interface, or invariant. |
| `governs`     | Source defines policy for target.                      |
| `affects`     | Source change or task impacts target.                  |
| `validates`   | Source provides validation evidence for target.        |
| `supersedes`  | Source replaces target.                                |
| `projects_to` | Source object contributes to a derived view.           |
| `consumed_by` | Source is consumed by target.                          |

## 9. Projection System

The semantic layer should support multiple views as projections.

### 9.1 LLM Compression View

Audience: model context windows.

Shape: high-density, low-ambiguity context bundle:

1. object ids
2. relation triples
3. invariant summaries
4. status and provenance labels
5. confidence scores
6. exact document or code anchors for reopening evidence

## 10. Wendao and Qianji Integration

Wendao should become the semantic-object-first query surface. It should be
able to return:

1. canonical semantic objects
2. typed relation neighborhoods
3. task-scoped subgraphs
4. projection payloads
5. provenance and verification evidence
6. unresolved endpoint or stale-projection diagnostics

Qianji should execute against task-scoped semantic subgraphs. A bounded work
surface should declare:

1. semantic task id
2. intended touched objects
3. expected relation changes
4. affected invariants
5. required validations
6. allowed projection outputs

This keeps Qianji's execution graph separate from the semantic graph while
still letting Qianji guards reason over governed semantic scope.

## 11. Change Governance

Every nontrivial change should be able to declare semantic intent:

1. touched objects
2. changed relations
3. affected invariants
4. required validations
5. intended projections to refresh
6. landed status transitions, if any
7. promotion and demotion targets, if any
8. candidate LLM-generated suggestions, if any

Validators should reject changes when:

1. a touched object id does not exist
2. relation endpoints cannot be resolved
3. relation kinds are unknown
4. status transitions violate lifecycle rules
5. promotion or demotion targets do not match the relevant status transition
6. affected invariants have no required validation evidence
7. generated projections are stale and not explicitly marked stale
8. LLM-generated suggestions are treated as canonical without acceptance
9. candidate objects are not governed by active change-intent suggestions

## 12. Minimal First Slice

1. define the semantic object frontmatter schema
2. define relation-kind validation
3. seed 8-12 canonical objects
4. add validation for object ids and relation endpoints
5. produce one LLM compression projection

Suggested seed objects:

1. `component.qianji.execution-plane`
2. `component.wendao.query-substrate`
3. `component.docs.projection-system`
4. `component.valkey.runtime-state-spine`
5. `decision.semantic-ssot.repo-native-first`
6. `decision.semantic-ssot.projections-are-read-models`
7. `invariant.llm-output-is-not-authority`
8. `invariant.execution-graph-is-not-semantic-graph`
9. `invariant.valkey-is-not-semantic-authority`
10. `task.semantic-ssot.object-schema-pilot`

## 13. Approval Questions

This RFC asks for approval on these decisions:

1. Should the project adopt a repo-native semantic SSOT layer as a first-class
   architecture direction?
2. Should the first object kinds be limited to `component`, `decision`,
   `invariant`, and `task`?
3. Should canonical semantic truth live outside generated projections and
   outside Valkey?
4. Should Qianji consume semantic subgraphs from Wendao instead of directly
   deriving execution context from scattered docs and chunks?
5. Should the preferred physical root be `semantic/` rather than
   `docs/semantic/objects/`?
6. Should DuckDB be treated as a read-model pilot only, with repo-native
   artifacts remaining authoritative?
7. Should SQL guards remain optional validation evidence until the first local
   pilot establishes the contract?
