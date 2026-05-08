# Reasoning Tree Backend Frontier

:PROPERTIES:
:ID: feat-reasoning-tree-backend-frontier
:PARENT: [[../index|Wendao DocOS Kernel: Map of Content]]
:TAGS: feature, search, reasoning-tree, julia, subagent
:STATUS: ACTIVE
:END:

## Purpose

Wendao's reasoning tree should not become a flat list of documents for an LLM
to sort. The backend must first expose a compact, scored frontier that can be
processed in parallel by Rust and Julia before external subagents spend LLM
tokens on uncertain branches.

The target architecture is:

```text
Rust expands evidence frontier
-> Julia ranks and prunes graph candidates
-> Rust selects a compact beam under budget
-> pi-wendao subagents judge uncertain branches in parallel
-> Rust merges decisions with SSOT, authority, and negative guards
-> next reasoning-tree depth
```

This keeps truth in repository evidence while allowing external orchestration
systems such as [`pi-wendao`](https://github.com/tao3k/pi-wendao) to use
subagent parallelism for high-value reasoning.

## Ownership

Rust owns the control plane:

- reasoning-tree session state
- frontier queues
- evidence collection
- stable ordering
- budget and deadline accounting
- cancellation and merge policy
- SSOT, authority, and negative-guard enforcement

Julia owns graph computation when live integration is enabled:

- PPR-like relatedness
- community grouping
- large relationship traversal
- frontier rerank
- diversity and MMR-style compression
- batched graph scoring

`pi-wendao` owns LLM/subagent execution:

- branch judgement
- parallel branch exploration
- uncertainty explanation
- prompt/model policy
- workflow state

The subagent result is advisory. It can say `keep`, `prune`, `expand`, or
`need_more_evidence`, but it cannot override Rust-enforced SSOT authority,
negative guards, or source provenance.

## First Slice

The first implementation slice adds an additive backend-frontier receipt to
the real-repo precision harness. It is not a live Julia call yet.

The receipt records:

- `strategy`: the frontier contract version
- `control_plane_owner`: currently `rust`
- `graph_backend`: currently `rust-baseline`, later replaceable by a measured
  WendaoGraph.jl backend
- `julia_schedule_basis`: currently `static_warm_profile_projection_v1`, which
  means the schedule action is an inert projection through existing scheduler
  policy, not a live Julia call
- node counts by `keep`, `prune`, and `expand`
- subagent judgement candidate count
- subagent fanout group count, fanout node count, maximum parallel width, and
  context budget
- Julia candidate, dispatch, queue, fallback, and reject counts
- SearchStrategyFlow projection basis, candidate count, transition count,
  selected frontier count, and projected context budget
- SearchStrategyFlow intent complexity, initial topology, refinement topology,
  planned depth, loop budget, cycle-candidate count, and LLM/subagent
  judgement count
- `selected_beam_width`
- per-node evidence kind, path/relation/object payload, graph batch key,
  parallel group, graph score, authority score, coverage score, context cost,
  backend action, whether a subagent judgement is required, subagent fanout
  group, judgement kind, priority score, context budget, Julia algorithm id,
  Julia profile id, Julia capability, schedule action, schedule reason,
  confidence score, selected batch size, SearchStrategyFlow candidate id,
  transition id, action, score, frontier rank, context budget, step role,
  iteration policy, loop-candidate flag, and LLM-judgement flag

This gives the Python benchmark enough structure to score deterministic
backend pruning before `pi-wendao` runs real subagents.

## Julia Schedule Projection

The current schedule projection maps frontier evidence kinds to existing
`WendaoGraph.jl` algorithm catalog entries:

- `anchor_query` -> `relationship_search.hnsw_semantic_fanout`
- `relation_path` -> `relationship_search.ppr_like_relatedness`
- `page_index_seed` -> `page_index.reasoning_frontier`
- `source_path` -> `relationship_search.graph_search_ranking`

Authority-order and negative-guard nodes stay Rust-owned. They are not
scheduled to Julia because they enforce truth boundaries rather than numeric
ranking.

The projection uses static warm profile facts only to prove that the frontier
shape can pass through the existing orchestrator scheduling policy. Promotion
to a live Julia backend requires replacing those static facts with measured
readiness, host-probe, latency, queue, and error-rate evidence.

## Algorithm Model

The intended long-term algorithm is evidence-gated multi-agent beam search:

```text
branch_score =
  evidence_coverage
+ authority_match
+ relation_path_match
+ negative_guard_match
+ graph_relatedness
+ subagent_branch_judgement
- context_cost
- expansion_cost
- uncertainty_penalty
```

Rust and Julia reduce the candidate space before LLM work starts. Subagents
judge only uncertain or high-value frontier branches, then Rust merges the
result into the next deterministic frontier.

The current additive fanout hints make that later subagent step deterministic:
each uncertain branch carries a fanout group id, judgement kind, priority score,
and bounded context budget. External orchestrators can run those groups in
parallel, but Rust still owns the merge and guard policy.

## Promotion Signals

This lane is ready to advance from `rust-baseline` to live Julia only when:

- real-repo query and scenario pass counts stay unchanged
- backend-frontier profile preserves evidence coverage
- context exposure drops or stays bounded versus graph-first
- `pi-wendao` scenarios show useful flat/top-k versus frontier distinctions
- Julia graph scores are recorded as measured inputs, not assumed defaults
- subagent judgement remains advisory and cannot bypass backend guards
