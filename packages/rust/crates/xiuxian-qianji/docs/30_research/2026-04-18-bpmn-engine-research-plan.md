---
type: knowledge
title: "Research Plan: xiuxian-qianji-bpmn-engine Architecture and xiuxian-qianji Integration"
category: "research"
status: "draft"
authors:
  - codex
created: 2026-04-18
tags:
  - qianji
  - bpmn
  - workflow
  - spiffworkflow
  - research
---

# Research Plan: xiuxian-qianji-bpmn-engine Architecture and xiuxian-qianji Integration

## 1. Purpose

This note opens a research lane for adding a standalone
`xiuxian-qianji-bpmn-engine` crate that `xiuxian-qianji` will depend on. The current
slice is planning-only. It exists to pin the external study reference, fix the
phased reading order, and record the architectural stance before any runtime
implementation begins.

Companion design note:
[Design Note: xiuxian-qianji-bpmn-engine Runtime State and Valkey Checkpoint Model](2026-04-18-bpmn-runtime-state-and-valkey-checkpoint-design.md)
and
[Design Note: xiuxian-qianji-bpmn-engine Crate Skeleton and Host Bridge](2026-04-18-bpmn-crate-skeleton-and-host-bridge.md)
and
[Audit Note: xiuxian-qianji-bpmn-engine BPMN and DMN Parity Against SpiffWorkflow](2026-04-18-bpmn-dmn-spiff-parity-audit.md)
and
[Design Note: xiuxian-qianji-bpmn-engine Frontier Concurrency and Synchronization Semantics](2026-04-19-bpmn-frontier-concurrency-semantics.md)

## 2. External Study Reference

Primary source:

1. Repository: `https://github.com/sartography/SpiffWorkflow`
2. Observed snapshot commit for this planning slice:
   `0395cc647af3c763bba5acc14adefd5fb2c7cb55`

`SpiffWorkflow` is being read as a source-level reference for how to separate
BPMN concerns into parser, process specification, runtime workflow, serializer,
and behavior corpus. It is not being adopted as a runtime dependency.

## 3. Why This Direction Fits Qianji

`xiuxian-qianji` already owns workflow orchestration, scheduler services,
checkpoint-oriented runtime thinking, and typed contract surfaces. The current
architecture decision is that it should depend on a dedicated BPMN crate rather
than absorb a large BPMN subsystem internally.

The working architectural stance for this lane is:

1. create a standalone `xiuxian-qianji-bpmn-engine` crate
2. keep BPMN parsing, immutable IR, runtime token semantics, and checkpointing
   inside that crate
3. let `xiuxian-qianji` depend on it through thin adapters into scheduler and
   telemetry services
4. keep Flowhub as a reusable scenario-graph and contract surface rather than
   redefining it as the BPMN runtime
5. reject a Python script engine dependency even though the external study
   reference uses one
6. keep the dependency direction one-way:
   `xiuxian-qianji -> xiuxian-qianji-bpmn-engine`
7. keep explicit placeholders for future DMN support so BPMN-first slices do
   not hard-code DMN out of the crate

## 4. High-Performance Working Design

The working design for performance is not "BPMN but in Rust". It is a specific
data-shape decision:

1. parse BPMN XML once into immutable IR/specs
2. normalize BPMN ids into compact internal indices
3. precompute graph indexes during parse/build time
4. keep runtime execution on dense mutable state, not on XML trees or scattered
   dynamic maps
5. call back into the host only at explicit service/user/manual boundaries

That means the hot path should look like:

1. `Arc`-shared immutable process/package spec
2. compact per-instance runtime state for tokens, node states, join counters,
   waiting subscriptions, branch selections, and suspend reasons
3. precomputed incoming/outgoing lookup tables and boundary-event attachments
4. bounded host adapter calls into `xiuxian-qianji` scheduler and telemetry
5. reserved decision-binding slots so later DMN integration does not force an
   IR reset

The runtime should avoid:

1. reparsing BPMN on resume
2. scanning raw XML to resolve routing decisions
3. storing whole host application objects in checkpoints
4. turning checkpoint writes into the hot control-flow path on every token move

## 5. Valkey Checkpoint Direction

Checkpoint persistence is fixed to Valkey for v1.

This is aligned with the current repo reality:

1. `xiuxian-qianji` already persists workflow checkpoints in Valkey through
   `QianjiStateSnapshot`
2. runtime config already resolves checkpoint Valkey URLs through
   `QIANJI_VALKEY_URL`, `VALKEY_URL`, or `REDIS_URL`
3. Valkey is already used in this repo as runtime-state infrastructure rather
   than as retrieval storage

The BPMN crate should follow the same philosophy.

Minimum checkpoint payload should include:

1. instance or session identity
2. serializer version
3. process/spec digest
4. token frontier and waiting positions
5. node execution states
6. join, branch, and multi-instance progress counters
7. workflow variables
8. suspend or wait reason
9. monotonic checkpoint sequence

The practical rule is:

1. store runtime-recovery state in Valkey
2. keep immutable parsed specs out of checkpoint blobs
3. apply TTL for abandoned checkpoints
4. delete checkpoints explicitly on clean completion
5. reserve telemetry pub/sub for observability, not for checkpoint truth

The more concrete payload shape, key layout, and write policy are tracked in
[Design Note: xiuxian-qianji-bpmn-engine Runtime State and Valkey Checkpoint Model](2026-04-18-bpmn-runtime-state-and-valkey-checkpoint-design.md).

The intended crate layout, public API entrypoints, and host-bridge ownership are
tracked in
[Design Note: xiuxian-qianji-bpmn-engine Crate Skeleton and Host Bridge](2026-04-18-bpmn-crate-skeleton-and-host-bridge.md).

## 6. Phased Reading Plan

### Phase 0. Provenance and Reality Check

Read and record:

1. `SpiffWorkflow` top-level layout and pinned commit
2. `xiuxian-qianji` current module layout and relevant RFC/doc surfaces

Completion signal:
Imported source and host-crate seams are recorded concretely.

### Phase 1. Parser and Spec Reading

Read:

1. `SpiffWorkflow/bpmn/parser/`
2. `SpiffWorkflow/bpmn/specs/`
3. `doc/bpmn/parsing.rst`

Questions to answer:

1. What belongs in parse-time validation versus runtime-time behavior?
2. What becomes immutable process specification or IR?
3. Which parser choices are durable enough to translate into the new crate?

Completion signal:
Parser, validation, and IR boundaries are explicit.

### Phase 2. Runtime and Persistence Reading

Read:

1. `SpiffWorkflow/bpmn/workflow.py`
2. `SpiffWorkflow/bpmn/serializer/`
3. `doc/bpmn/workflows.rst`
4. `doc/bpmn/serialization.rst`
5. selected behavior and performance tests under
   `tests/SpiffWorkflow/bpmn/`

Questions to answer:

1. How are waiting states, events, subprocesses, and manual tasks handled?
2. What must `xiuxian-qianji-bpmn-engine` own natively for checkpointing and serializer
   migration?
3. What behavior corpus is strong enough to seed the first executable subset?

Completion signal:
Runtime, serializer, and test-corpus takeaways are recorded.

### Phase 3. Qianji Mapping

Read:

1. `packages/rust/crates/xiuxian-qianji/src/lib.rs`
2. `packages/rust/crates/xiuxian-qianji/src/engine/`
3. `packages/rust/crates/xiuxian-qianji/src/scheduler/`
4. `packages/rust/crates/xiuxian-qianji/src/flowhub/`
5. `packages/rust/crates/xiuxian-qianji/src/contracts/`
6. Qianji spec and workflow RFC surfaces

Questions to answer:

1. Which concerns belong in `xiuxian-qianji-bpmn-engine`?
2. Which concerns remain owned by `xiuxian-qianji`, scheduler, telemetry, and
   Flowhub?
3. What is explicitly out of scope for the first 3 slices?

Completion signal:
The crate boundary between `xiuxian-qianji-bpmn-engine` and `xiuxian-qianji` is stable
enough to implement later.

## 7. Initial Architectural Working Assumptions

These assumptions are intentionally provisional and remain auditable:

1. BPMN should live in a dedicated `xiuxian-qianji-bpmn-engine` crate, not as a
   long-lived internal folder inside `xiuxian-qianji` and not as a thin rewrite
   of Flowhub.
2. A dedicated BPMN runtime is more coherent than flattening BPMN directly into
   the current TOML manifest compiler before semantics are understood.
3. Checkpoint serialization and resume semantics must be designed alongside the
   runtime, not bolted on after parsing works.
4. The new crate should expose host-neutral contracts rather than depend back on
   `xiuxian-qianji`.
5. A bounded executable subset should land before DMN, editor support, or
   broader authoring bridges.
   Even so, explicit DMN placeholders should remain in the architecture so the
   BPMN-first slices preserve that future seam.

## 8. Audit Status

Current status:

1. research lane opened
2. external source pinned
3. architecture plan drafted
4. implementation moved forward through the bounded engine slices
5. parity audit note added to separate current bounded support from the broader
   `SpiffWorkflow` reference surface
6. bounded gateway parser and runtime support landed after the audit, and the
   audit note now reflects that narrower but real non-linear BPMN support
7. bounded `intermediateCatchEvent` support for `messageEventDefinition` and
   `signalEventDefinition` has now landed, together with event-aware wait
   registration and resume semantics
8. snapshot-style `timerEventDefinition` support for `intermediateCatchEvent`
   and one interrupting timer `boundaryEvent` on one host-blocking task have
   now landed, together with timer-aware wait registration, boundary
   attachment indexing, and LLM-friendly boundary lint diagnostics
9. one bounded same-package `callActivity` slice has now landed, together with
   parser validation for `calledElement`, checkpoint-safe parent-frame return
   semantics, and explicit rejection of embedded `subProcess` bodies and
   recursive call graphs
10. one bounded exclusive `eventBasedGateway` slice has now landed, together
    with multi-wait poll requests, checkpoint-safe competition ownership, and
    deterministic loser cancellation for message, signal, and timer wait races
11. one bounded `standardLoopCharacteristics` slice has now landed on top of
    the existing host-blocking task model, together with repeat snapshot
    metadata, sparse `standard_loops` checkpoint ownership, `testBefore` skip
    behavior, loop-maximum enforcement, and simple boolean loop conditions
12. bounded sequential and bounded parallel
    `multiInstanceLoopCharacteristics` slices have now landed for one
    host-blocking task family, together with immutable repeat snapshots,
    sparse checkpoint ownership for sequential and parallel owner state,
    repeat metadata on host-dispatch requests, zero-cardinality skip
    behavior, and interrupting boundary cleanup for multi-instance owner
    state
13. one bounded multi-instance `completionCondition` subset is now also
    landed for those same cardinality-driven sequential and parallel shapes,
    using one simple boolean variable path or one bounded counter comparison,
    while multi-instance data bindings still fail explicitly through
    engine-owned loop diagnostics rather than generic unsupported-element
    fallback
14. the bounded DMN evaluator widening and the thin
    `xiuxian-qianji` business-rule host adapter lane are now both landed
15. the next bounded move is therefore higher-level BPMN orchestration in
    `xiuxian-qianji`, not another missing bridge primitive
16. that higher-level orchestration slice is now also landed as a bounded
    `xiuxian-qianji::bpmn::runtime` facade covering bundle load, session
    ownership, checkpoint backend selection, and stable host-work-driven
    advancement without moving BPMN semantics out of `xiuxian-qianji-bpmn-engine`
17. the next bounded move should now stay above that facade and target one
    explicit scheduler, CLI, or adapter-owned execution surface rather than
    reopening parser/DMN internals first
18. that execution-surface step is now landed through a bounded
    `qianji bpmn run` CLI adapter which exercises bundle loading, session
    create-or-resume, bounded checkpoint backend selection, and stable result
    rendering without introducing scheduler ownership yet
19. the same CLI surface now also accepts one deterministic
    `--host-fixture <path>` contract keyed by stable BPMN node ids so bounded
    `serviceTask`, `userTask`, `manualTask`, and `businessRuleTask` flows can
    complete through a thin host bridge without exposing internal engine node
    indices or widening into external-event delivery
20. real external-event injection is now also landed above the same CLI/runtime
    seam: the host crate owns one wait-poll helper plus one deterministic
    `--event-fixture <path>` contract, while the engine still owns wait
    registration, winner selection, and resumed routing semantics
21. that richer event-competition follow-up slice is now also landed above the
    same CLI/runtime seam, so waiting output exposes stable BPMN wait ids,
    competition gateway ids, and deterministic event-fixture keys for
    auditability
22. focused runtime and CLI proofs now cover one explicit
    `wait_message|wait_timer` winner-selection contract end to end without
    widening ownership into the scheduler
23. that reusable host-owned BPMN execution-driver slice is now also landed
    above the same session/checkpoint seam, so later scheduler or app
    integration has one shared execution lifecycle surface instead of
    CLI-local orchestration code
24. focused runtime and CLI coverage now prove fresh execution, explicit
    fresh-context rejection, checkpoint resume, and no-progress checkpoint
    save behavior through the shared host-owned driver
25. that BPMN-specific scheduler checkpoint lifecycle slice is now also landed
    above the shared driver, with engine/store checkpoint delete support and
    one `QianjiBpmnExecutionScheduler` that cleans up terminal checkpoints
    while keeping waiting/suspended runs resumable
26. focused runtime and engine/store coverage now prove terminal checkpoint
    deletion and waiting-state retention through the shared driver plus the new
    BPMN-specific scheduler surface
27. the active next bounded move is one Valkey-backed scheduler lease-ownership
    slice above that BPMN-specific scheduler surface so host-owned runs become
    explicit single writers before any broader distributed orchestration cut
28. that Valkey-backed scheduler lease-ownership slice is now also landed,
    with owner-guarded delete, explicit lease acquire/renew/release, and
    focused host-runtime proofs for conflict, waiting retention, and terminal
    cleanup
29. generic DAG scheduler integration still remains deferred after that
    scheduler-owned lease lifecycle step, rather than reopening parser or DMN
    internals first
30. that scheduler-identity ownership slice is now also landed, with one
    BPMN-local conversion from `SchedulerAgentIdentity` into
    `QianjiBpmnSchedulerLeaseConfig`
31. that same slice also added one explicit `agent_id` requirement for
    distributed single-writer ownership plus one scheduler builder that accepts
    `SchedulerAgentIdentity` directly
32. focused BPMN runtime coverage now proves the identity-derived scheduler
    path preserves waiting retention and lease release semantics on the
    Valkey-backed path
33. that real-caller adoption slice is now also landed for `qianji bpmn run`,
    with one additive CLI seam that uses the BPMN scheduler path only when the
    runtime checkpoint backend is Valkey and a stable scheduler `agent_id`
    exists
34. the CLI output now also renders explicit `Checkpoint deleted: yes|no` so
    terminal cleanup vs retained checkpoint state is visible to users and
    future adapter callers
35. focused CLI coverage now proves both the identity-backed terminal-cleanup
    path and the role-only fallback path against a live temporary Valkey
    instance
36. that runtime selector slice is now also landed in
    `xiuxian-qianji::bpmn::runtime`, with one reusable `QianjiBpmnExecutionFacade`
    above the existing driver and scheduler types
37. the library facade now owns the same identity-aware execution choice that
    the CLI previously carried locally, so future host runtimes can adopt the
    same contract without copying CLI-local helper logic
38. focused runtime coverage now proves both the pure selection contract and
    the identity-backed terminal cleanup path directly at the library layer
39. one bounded engine-internal frontier conflict-merge slice is now also
    landed after that library selector, so same-node parallel-join arrivals
    merge before batch consumption instead of replaying every join proposal
    independently
40. that same slice keeps single-writer checkpoint ownership unchanged,
    preserves excess buffered join arrivals deterministically, and falls back
    to per-proposal replay when legacy checkpoint state lacks per-edge join
    counts
41. full `xiuxian-qianji-bpmn-engine` validation now also passes after restructuring
    the pre-existing `tests/unit/checkpoint/valkey.rs` harness blocker into a
    folder-first suite, while leaving checkpoint assertion semantics unchanged
42. the next bounded move should now be selected explicitly between broader
    node-family conflict-aware merge and higher-level host-runtime adoption,
    while generic DAG scheduler integration and broader DMN widening still
    remain deferred

The next slice should stay narrow and architecture-audited:
build on the landed orchestration facade without collapsing BPMN engine
ownership back into the host crate, then continue richer parity or
adapter/scheduler slices later.
