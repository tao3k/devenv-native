---
type: knowledge
title: "Design Note: qianji-bpmn-engine Frontier Concurrency and Synchronization Semantics"
category: "research"
status: "draft"
authors:
  - codex
created: 2026-04-19
tags:
  - qianji
  - bpmn
  - concurrency
  - gateway
  - runtime
  - omg
---

# Design Note: qianji-bpmn-engine Frontier Concurrency and Synchronization Semantics

## 1. Purpose

This note narrows the next bounded `qianji-bpmn-engine` slice to one question:

How should the engine align its runtime model with OMG BPMN concurrency and
synchronization semantics when a single workflow instance has multiple runnable
nodes at the same time?

The key distinction for this lane is:

1. BPMN semantic concurrency is required
2. multi-writer checkpoint concurrency is not

This note therefore does not argue for multiple distributed writers. It argues
for a better in-instance frontier model under the existing single-writer
ownership contract.

Companion notes:
[Research Plan: qianji-bpmn-engine Architecture and xiuxian-qianji Integration](2026-04-18-bpmn-engine-research-plan.md)
and
[Design Note: qianji-bpmn-engine Runtime State and Valkey Checkpoint Model](2026-04-18-bpmn-runtime-state-and-valkey-checkpoint-design.md)
and
[Audit Note: qianji-bpmn-engine BPMN and DMN Parity Against SpiffWorkflow](2026-04-18-bpmn-dmn-spiff-parity-audit.md)

Primary normative source:
[OMG BPMN 2.0.2 Specification](https://www.omg.org/spec/BPMN/2.0.2/PDF)

## 2. Normative BPMN Semantics That Matter for This Slice

### 2.1 Parallel Gateway

OMG BPMN 2.0.2 Clause 13.4.1 defines the parallel gateway as both:

1. a branching point that spawns concurrent branches
2. a merge point that synchronizes concurrent branches

For `qianji-bpmn-engine`, the important operational consequences are:

1. a parallel split is not just graph fan-out; it creates multiple active
   concurrent branches
2. a parallel join waits until there is at least one token on every incoming
   sequence flow
3. when the join fires, it consumes one token per incoming sequence flow and
   emits one token on each outgoing sequence flow
4. excess tokens are not destroyed just because the join fired once; they must
   remain represented in runtime state

This last point is important because it means the engine cannot model parallel
join semantics only as a boolean "ready/not ready" flag. It needs token-aware
frontier bookkeeping.

### 2.2 Event-Based Gateway

OMG BPMN 2.0.2 Clause 10.6.6 defines the event-based gateway as a branching
point where routing depends on which event happens, not on data expressions.

For this engine, the key consequences are:

1. the outgoing sequence flows do not carry data conditions
2. the runtime must register multiple event waits as one competition
3. one event wins the race and the losing alternatives are cancelled
4. this is semantic concurrency at the wait frontier even when only one branch
   ultimately continues

This matches the existing bounded event-competition direction, but it also
means a future frontier model must treat wait registration as a set owned by
one gateway, not as unrelated singleton waits.

### 2.3 Parallel Event-Based Gateway

The BPMN 2.0.2 spec distinguishes the parallel event-based gateway from the
ordinary event-based gateway and constrains it to process instantiation.

For the current bounded engine, the implication is:

1. do not invent a generic mid-process "parallel event race" runtime mode
2. keep the currently supported in-process event-based gateway semantics
   exclusive
3. if mid-process behavior truly needs parallel event handling, model it with
   standard parallel control-flow constructs plus explicit waits, not with a
   misread gateway type

## 3. Engine Implications

### 3.1 Semantic Concurrency Must Live Inside One Writer

The existing checkpoint design remains correct:

1. one workflow instance has one distributed writer owner at a time
2. Valkey lease ownership is about checkpoint truth and stale-writer exclusion
3. semantic concurrency happens inside that owner process as runtime state,
   not by allowing multiple checkpoint writers

This preserves deterministic resume and aligns with the existing Valkey-first
runtime-state direction.

### 3.2 Current Runtime Gap

The current engine already carries several structures that prove it is not
single-token in principle:

1. `active_tokens`
2. `joins`
3. `waits`
4. `event_competition`

However, the bounded runtime still advances around a first-token planning model
and therefore still lacks an explicit frontier proposal/reduce phase.

That means the engine currently has:

1. token-scoped ownership for multiple blocked host-work entries
2. token-scoped runnable selection so duplicate tokens at the same BPMN node do
   not get hidden behind one shared node-status bit
3. edge-aware buffered join arrivals so parallel joins do not fire early when
   duplicate arrivals come from the same incoming sequence flow
4. an explicit `BpmnFrontierSnapshot` that classifies every active token into
   deterministic frontier states such as runnable, blocked-on-host, or
   waiting-external
5. explicit frontier proposal collection that can surface every runnable token
   in deterministic token order
6. explicit deterministic reduction from those proposals to one owner action
   such as execute-batch, blocked-on-host, waiting-external, suspended, or
   stalled
7. deterministic batch consumption that re-resolves tokens by `token_id`
   before each in-batch mutation so stale snapshot indices do not misapply
   later proposals
8. one bounded conflict-aware cross-token merge model is now landed for
   same-node parallel joins, but broader node-family merge remains open

### 3.3 The Required Runtime Shape

The next architectural target should deepen that frontier-based model beyond
the landed proposal/reduction and batch-consumption seams:

1. collect all runnable token positions for the current instance
2. plan per-token transition proposals against immutable process specs
3. reduce those proposals deterministically into one owner batch
4. materialize host-dispatch work and wait registrations from the consumed
   frontier result
5. checkpoint only after the owner has one coherent post-step instance state

This keeps semantic concurrency explicit while preserving a single-writer
checkpoint truth model.

## 4. What This Means for `rayon`

`rayon` is now appropriate only for pure frontier inspection work, not for
state mutation.

The safe layering is:

1. semantics first
2. deterministic frontier planning second
3. optional CPU-parallel planning third

That means:

1. immutable frontier classification may use `rayon` when the active-token set
   is wide enough to amortize scheduling overhead
2. host-dispatch I/O should stay async
3. Valkey checkpoint and lease I/O must stay async and single-writer
4. correctness must not depend on thread scheduling order

## 5. Concrete Bounded Cases to Carry Into Tests

The next runtime slice should prove at least these cases.

### Case 1. Parallel Split with Two Host-Blocking Branches

1. one parallel gateway fans out into two leaf tasks
2. both branches become active in the same instance frontier
3. the runtime does not collapse them into one singleton pending-work slot

### Case 2. Parallel Join Synchronization

1. two branches rejoin at one parallel gateway
2. the join does not fire early
3. one token per incoming branch is consumed when the join fires

### Case 3. Event-Based Gateway Race

1. one event-based gateway fans out into multiple catch-event waits
2. the runtime records them as one competition
3. one winner continues and losing waits are cancelled deterministically

### Case 4. Excess-Token Join Behavior

1. if a join sees extra tokens on one incoming branch, they are not silently
   deleted by one join firing
2. the runtime state must still be able to represent post-fire excess token
   presence

### Case 5. Keep Multi-Instance Separate

1. gateway concurrency and multi-instance concurrency are not the same family
2. parallel multi-instance stayed out of scope for the original frontier
   slice, and should continue to be modeled as a separate owner-state family
   rather than as a gateway-special case
3. this slice should not overload gateway-frontier logic with full
   multi-instance expansion semantics

## 6. Recommended Next Slice

The next bounded implementation slice after the landed frontier
proposal/reduction seam was:

1. keep single-writer checkpoint ownership unchanged
2. keep the landed token-scoped blocked/runnable ownership, explicit frontier
   snapshots, explicit proposal collection, deterministic reduction, and
   edge-aware join buffering as the new baseline
3. widen runtime planning from reduce-to-one-owner-step into deterministic
   multi-proposal batch execution over multiple runnable tokens
4. re-resolve proposals by stable `token_id` ownership so in-batch token
   removal and index movement stay correct

The next bounded slice after that is now partially landed:

1. single-writer checkpoint ownership stayed unchanged
2. deterministic batch execution remained the baseline
3. execution now widens into one bounded conflict-aware merge for same-node
   parallel-join arrivals
4. adapter work, DMN widening, and inclusive gateways remained out of scope
5. bounded parallel multi-instance later landed as a separate owner-state
   slice on top of the same deterministic batch-execution baseline

The next bounded follow-up after the landed parallel-join merge should be:

1. keep single-writer checkpoint ownership unchanged
2. keep raw runnable proposal collection plus merge-aware batch execution as
   the new baseline
3. widen conflict-aware frontier merge only when another BPMN node family
   proves it needs aggregate semantics
4. keep adapter work, DMN widening, inclusive gateways, and parallel
   multi-instance out of scope

## 7. Final Design Stance

The correct architectural reading is:

1. BPMN requires multiple active nodes inside one workflow instance
2. `qianji-bpmn-engine` therefore needs a frontier-aware runtime model
3. the model now includes one bounded conflict-aware merge for same-node
   parallel joins on top of the earlier snapshot/planner/proposal/batch seams
4. this does not justify multiple checkpoint writers
5. each future merge rule should still be justified by BPMN semantics first,
   with `rayon` remaining limited to pure planning work
