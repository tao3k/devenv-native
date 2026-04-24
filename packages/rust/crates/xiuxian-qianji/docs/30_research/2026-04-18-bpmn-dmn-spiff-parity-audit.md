---
type: knowledge
title: "Audit Note: qianji-bpmn-engine BPMN and DMN Parity Against SpiffWorkflow"
category: "research"
status: "draft"
authors:
  - codex
created: 2026-04-18
tags:
  - qianji
  - bpmn
  - dmn
  - spiffworkflow
  - audit
---

# Audit Note: qianji-bpmn-engine BPMN and DMN Parity Against SpiffWorkflow

## 1. Purpose

This note records a source-grounded audit of the current
`qianji-bpmn-engine` crate against the imported `SpiffWorkflow` reference
implementation.

It answers one narrow question:

Does the current `qianji-bpmn-engine` align with the BPMN and DMN syntax surface
that `SpiffWorkflow` currently parses and executes?

The answer is no.

More precisely:

1. BPMN support is not aligned.
2. DMN support is not aligned.
3. The BPMN gap is still structural, although several bounded gateway, wait,
   timer, boundary, message-task, and nested call-activity slices have now
   landed.
4. The DMN gap is also large, but it must be described carefully because
   `SpiffWorkflow` itself is not a full OMG DMN implementation.

Companion planning note:
[Research Plan: qianji-bpmn-engine Architecture and xiuxian-qianji Integration](2026-04-18-bpmn-engine-research-plan.md)

Companion design notes:
[Design Note: qianji-bpmn-engine Runtime State and Valkey Checkpoint Model](2026-04-18-bpmn-runtime-state-and-valkey-checkpoint-design.md)
and
[Design Note: qianji-bpmn-engine Crate Skeleton and Host Bridge](2026-04-18-bpmn-crate-skeleton-and-host-bridge.md)

## 2. Evidence Baseline

### 2.1 Current Engine Evidence

The current crate documents itself as a bounded subset:

1. `packages/rust/crates/qianji-bpmn-engine/src/lib.rs` now states that bounded
   `parallelGateway` split/join semantics and deterministic
   `exclusiveGateway` routing with simple boolean-path or numeric-comparison
   outgoing `sequenceFlow` `conditionExpression` values plus one optional
   `default` flow, plus one bounded structured `inclusiveGateway` subset with
   the same condition/default routing rules plus one matching linear join
   fragment, plus bounded
   `intermediateCatchEvent` waits backed by `messageEventDefinition`,
   `signalEventDefinition`, and snapshot-style `timerEventDefinition`, plus
   one bounded `receiveTask` message-wait shell, plus one bounded `sendTask`
   host-dispatch shell, plus one interrupting timer, message, or signal
   `boundaryEvent` on one host-blocking task, plus one bounded
   interrupting timer, message, or signal `boundaryEvent` on one embedded
   subprocess owner, plus one bounded
   non-interrupting timer, message, or signal `boundaryEvent` on one
   non-repeating or bounded sequential or parallel multi-instance
   host-blocking task, plus
   one bounded
   embedded `subProcess` body with
   exactly one nested `startEvent`
   and at least one nested `endEvent`, plus one bounded `<transaction>` shell
   with exactly one nested `startEvent` and at least one nested `endEvent`,
   plus one bounded embedded subprocess owner that may expose one
   interrupting timer/message/signal `boundaryEvent` plus one or more
   interrupting error `boundaryEvent` nodes, where the interrupting parent
   timer/message/signal boundary may cancel the child shell before
   restoring the parent frame, one or more nested error ends may each
   restore the parent frame while preserving variable mutations and route
   through every matching parent error boundary including one catch-all
   boundary, while normal completion and either supported interrupting
   winner cancel the non-selected sibling boundaries, plus one bounded
   transaction cancel path composed of
   one interrupting
   cancel `boundaryEvent` attached to that transaction shell and one nested
   cancel end that restores the parent frame and rolls back
   transaction-local variable mutations, plus one bounded transaction owner
   that may expose one interrupting cancel `boundaryEvent` plus one or more
   interrupting error `boundaryEvent` nodes, where one or more nested error
   ends may each restore the parent frame while preserving transaction-local
   variable mutations and route through every matching parent error boundary
   including one catch-all boundary, while normal completion and cancel
   routing cancel the non-selected sibling boundaries, plus one bounded
   transaction cancel
   compensation subset where compensable activities may bind one explicit
   compensation handler and cancel routing replays those handlers in reverse
   completion order before the parent cancel boundary fires, plus one
   synchronous throw-compensation `endEvent` subset that either uses explicit
   `activityRef` targeting or omits `activityRef` for default reverse replay,
   plus one synchronous throw-compensation
   `intermediateThrowEvent` subset that either uses explicit `activityRef`
   targeting or omits `activityRef` for default reverse replay inside that
   same transaction shell before normal sequence-flow routing resumes, plus
   one bounded same-package `callActivity`, plus one bounded top-level
   `errorEventDefinition` end path that terminates the instance in failed
   state, plus bounded
   `standardLoopCharacteristics`, plus bounded sequential and bounded
   parallel `multiInstanceLoopCharacteristics` with integer
   `loopCardinality` on one host-blocking task family are supported, and those
   same multi-instance shapes may now also carry one bounded
   `completionCondition` using either one simple boolean variable path or one
   bounded counter comparison, plus one bounded collection-backed data-binding
   subset using `loopDataInputRef`, `inputDataItem`, optional
   `loopDataOutputRef`, and `outputDataItem`, while broader unstructured
   inclusive gateways, compensation event subprocesses, asynchronous
   throw-compensation intermediate forms, asynchronous
   throw-compensation end-event forms, more than one cancel boundary on one
   transaction owner, broader error propagation beyond those bounded
   transaction, embedded-subprocess, same-package `callActivity`, and
   top-level terminal-failure paths, broader non-interrupting boundary
   families on subprocess-like owners, richer BPMN
   orchestration, broader
   multi-BPMN import/dependency handling, and broader FEEL or script-backed
   temporal behavior remain deferred.
2. `packages/rust/crates/qianji-bpmn-engine/src/parser/import.rs` now accepts
   `parallelGateway`, `exclusiveGateway`, `intermediateCatchEvent`, one
   bounded `receiveTask`/`sendTask` message-task family, one bounded
   interrupting timer/message/signal `boundaryEvent` family including one
   bounded same-package `callActivity` owner subset, plus one bounded
   non-interrupting timer/message/signal subset on one non-repeating,
   bounded standard-loop, or bounded sequential or parallel multi-instance
   host-blocking task, one bounded embedded
   `subProcess` body family, one bounded `<transaction>` shell family, one bounded
   `callActivity` family, bounded `standardLoopCharacteristics`, and bounded
   sequential or bounded
   parallel `multiInstanceLoopCharacteristics`, with message, signal, and
   timer event definitions inside the currently supported wait shapes and
   message-task validation surface, plus bounded error event definitions for
   one embedded subprocess owner that may expose one or more error
   boundaries and one interrupting timer/message/signal boundary on that
   same owner, plus bounded cancel and error event definitions for one
   transaction owner that may expose one cancel boundary plus one or more
   error boundaries, plus one synchronous
   top-level `endEvent` subset with one `errorEventDefinition` that may
   terminate one executable process in failed state, plus one synchronous
   throw-compensation `endEvent` subset that either uses explicit
   `activityRef` targeting or omits `activityRef` for default reverse replay,
   plus one synchronous throw-compensation
   `intermediateThrowEvent` subset that either uses explicit `activityRef`
   targeting or omits `activityRef` for default reverse replay inside that
   same transaction shell.
3. `packages/rust/crates/qianji-bpmn-engine/src/runtime/lifecycle.rs` now
   supports bounded multi-token routing for parallel split/join, bounded
   exclusive condition-driven routing using simple boolean-path or
   numeric-comparison outgoing `sequenceFlow` `conditionExpression` values
   plus one optional `default` flow, plus one bounded structured inclusive
   split/join subset with the same condition/default routing rules and one
   matching linear join fragment, plus deterministic wait registration for intermediate
   message/signal/timer catch events, one bounded `receiveTask` message wait
   shell, one bounded `sendTask` host-dispatch shell, one interrupting timer,
   message, or signal boundary path, one bounded non-interrupting timer,
   message, or signal boundary path on one non-repeating, bounded
   standard-loop, or bounded sequential or parallel multi-instance
   host-blocking task that opens one concurrent boundary branch while the
   original task stays active, and
   parent-frame enter/return
   semantics for one bounded
   same-package `callActivity` plus one bounded embedded `subProcess` body,
   plus one bounded same-package `callActivity` interrupting external-boundary
   path where one parent timer/message/signal boundary stays armed while the
   called child process runs and may cancel that child process before
   restoring the parent frame,
   plus one bounded embedded subprocess error path that restores the parent
   frame, preserves variable mutations, routes through every matching parent
   error boundary including one catch-all boundary, and cancels non-selected
   sibling boundaries on normal completion or error routing, plus one bounded
   same-package `callActivity` error path that restores the parent frame,
   preserves variable mutations, routes through every matching parent error
   boundary including one catch-all boundary, and cancels non-selected sibling
   boundaries on normal completion or error routing, plus one bounded
   top-level error end path that marks the instance failed terminally,
   preserves merged variables, and clears remaining frontier state, plus one
   bounded transaction cancel path that restores the parent frame,
   rolls back transaction-local variable mutations, and routes through the
   parent cancel boundary, plus one bounded transaction boundary-ownership
   slice where one transaction owner may route one thrown error through every
   matching parent error boundary, preserve transaction-local variable
   mutations, and cancel non-selected sibling boundaries on normal completion,
   cancel routing, or error routing, plus one bounded transaction cancel
   compensation path that replays explicit compensation handlers in reverse
   completion order before the parent cancel boundary fires, plus one
   synchronous throw-compensation `endEvent` path that either replays one
   referenced compensable activity or replays every already compensable
   activity in reverse completion order, plus one synchronous
   throw-compensation `intermediateThrowEvent` path that either replays one
   referenced compensable activity or replays every already compensable
   activity in reverse completion order before normal downstream
   sequence-flow routing resumes inside that same transaction shell, plus
   bounded standard-loop re-entry and skip semantics, plus bounded
   sequential and bounded parallel multi-instance
   re-entry, repeat-context propagation, zero-cardinality skip, one bounded
   early-completion path via `completionCondition`, plus bounded collection
   input and output bindings with checkpoint-safe per-iteration overlays and
   deterministic output aggregation, and interrupting-boundary cleanup on one
   host-blocking task family.
   `businessRuleTask` can now also execute locally when the package carries a
   matching engine-owned DMN decision definition, while still deferring
   broader unstructured inclusive semantics, broader non-interrupting
   boundary families on standard-loop or sequential multi-instance task
   owners or subprocess-like owners, more than one cancel
   boundary on one transaction owner, broader transaction error propagation,
   broader multi-BPMN import/dependency handling, and broader FEEL or
   script-backed gateway condition semantics.
4. `packages/rust/crates/qianji-bpmn-engine/src/parser/package.rs` now exposes
   one bounded parser-owned `BpmnBundleSnapshot` contract plus
   `parse_bpmn_bundle(...)`, allowing one BPMN source plus optional DMN
   sources to populate the package registry deterministically.
5. `packages/rust/crates/qianji-bpmn-engine/src/dmn/parse.rs` accepts one
   decision and one decision table only.
6. `packages/rust/crates/qianji-bpmn-engine/src/dmn/evaluate.rs` supports
   `UNIQUE` and `COLLECT` only, with wildcard matching, literal equality,
   numeric unary comparisons, bounded numeric ranges, ISO date comparisons and
   ranges, ISO local datetime comparisons and ranges, and bounded ISO time
   comparisons and ranges.
7. `packages/rust/crates/qianji-bpmn-engine/src/lint/bpmn.rs` and
   `packages/rust/crates/qianji-bpmn-engine/src/lint/dmn.rs` explicitly guide
   callers toward this bounded subset, including LLM-friendly repair prompts
   for invalid bounded `receiveTask`/`sendTask` message bindings.

### 2.2 SpiffWorkflow Evidence

The imported `SpiffWorkflow` snapshot is pinned in the research lane to commit
`0395cc647af3c763bba5acc14adefd5fb2c7cb55`.

Evidence came from:

1. upstream README feature statements
2. `SpiffWorkflow/spiff/parser/process.py`
3. `SpiffWorkflow/bpmn/serializer/config.py`
4. `SpiffWorkflow/dmn/parser/DMNParser.py`
5. `SpiffWorkflow/dmn/engine/DMNEngine.py`
6. representative BPMN and DMN tests under `tests/SpiffWorkflow/`
7. the public BPMN support page on ReadTheDocs:
   [SpiffWorkflow BPMN Supported Elements](https://spiffworkflow.readthedocs.io/en/latest/bpmn/supported.html)

## 3. BPMN Parity Matrix

| BPMN family                                                                                                             | SpiffWorkflow evidence                                                                                                                                              | Current engine parse status                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | Current engine runtime status                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | Severity | Recommended next slice                                                         |
| ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- | ------------------------------------------------------------------------------ |
| Core linear flow: `startEvent`, `endEvent`, `serviceTask`, `userTask`, `manualTask`, `businessRuleTask`, `sequenceFlow` | Covered by `SpiffWorkflow/spiff/parser/process.py`; also stated in ReadTheDocs tasks list                                                                           | Supported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | Supported for linear single-frontier routing; `businessRuleTask` can now execute locally when the parser-owned bundle snapshot or later callers register one matching engine-owned DMN definition, but adapter wiring remains incomplete                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | baseline | keep stable while widening richer shapes                                       |
| Gateways: exclusive, inclusive, parallel, event-based                                                                   | Declared in ReadTheDocs; serializer config includes `ExclusiveGateway`, `InclusiveGateway`, `ParallelGateway`, `EventBasedGateway`; tests cover gateway families    | `parallelGateway`, `exclusiveGateway`, one structured `inclusiveGateway` subset, and one bounded exclusive `eventBasedGateway` shape are supported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | bounded `parallelGateway` split/join, bounded `exclusiveGateway` routing with simple boolean-path or numeric-comparison branch conditions plus one optional `default` flow, one structured `inclusiveGateway` split/join subset with the same condition/default rules plus one matching linear join fragment, and one bounded event-based winner-takes-all wait race are supported; broader unstructured inclusive joins and broader FEEL/script-backed gateway conditions remain unsupported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | P1       | broader inclusive-gateway reachability and broader gateway-condition semantics |
| Intermediate, boundary, timer, message, signal, escalation, error, cancel events                                        | Parser registrations and serializer config include boundary and intermediate events; tests cover timer, message, boundary, escalation, cancel, event-based gateways | Bounded `intermediateCatchEvent` support is now present for `messageEventDefinition`, `signalEventDefinition`, and `timerEventDefinition`; one interrupting timer, message, or signal `boundaryEvent` attached to one host-blocking task is also supported; one interrupting timer, message, or signal `boundaryEvent` attached to one bounded embedded subprocess owner is also supported, including one bounded mixed-owner shape with that single interrupting timer/message/signal boundary plus one or more interrupting error boundaries on the same owner; one interrupting timer, message, or signal `boundaryEvent` attached to one bounded same-package `callActivity` owner is also supported, including one bounded mixed-owner shape with that single interrupting timer/message/signal boundary plus one or more interrupting error boundaries on the same owner; one interrupting timer, message, or signal `boundaryEvent` attached to one bounded transaction shell owner is also supported, including one bounded mixed-owner shape with that single interrupting timer/message/signal boundary plus one interrupting cancel boundary on the same owner, one bounded mixed-owner shape with that single interrupting timer/message/signal boundary plus one or more interrupting error boundaries on the same owner, and one bounded mixed-owner shape with that single interrupting timer/message/signal boundary plus one interrupting cancel boundary plus one or more interrupting error boundaries on the same owner; one bounded non-interrupting timer, message, or signal `boundaryEvent` attached to one non-repeating or bounded standard-loop, sequential multi-instance, or parallel multi-instance host-blocking task is also supported; one bounded transaction owner may now expose one cancel boundary plus one or more error boundaries; one bounded embedded subprocess owner may now expose one or more interrupting error boundaries; one bounded same-package `callActivity` owner may now expose one or more interrupting error boundaries; one bounded top-level `endEvent` with `errorEventDefinition` is also supported; one bounded transaction cancel compensation subset with one explicit compensation-handler marker is also supported; and one bounded throw-compensation subset is now supported for nested `endEvent` and `intermediateThrowEvent` shapes inside that same transaction shell, with targeted or default replay allowed on both throw shapes, while either throw shape may stay synchronous or set `waitForCompletion=\"false\"` inside the bounded subset | Intermediate message/signal/timer waits now register and resume through the engine-owned wait shell; one interrupting timer, message, or signal boundary path can cancel blocked host work; one interrupting timer, message, or signal boundary path can also stay armed on one embedded subprocess owner while the child shell runs, then cancel that child shell and restore the parent frame onto the selected boundary route; one bounded embedded-subprocess mixed-owner shape can now let either that armed timer/message/signal boundary or one matching parent error boundary win while clearing the non-selected owner-level waits and sibling boundaries; one interrupting timer, message, or signal boundary path can also stay armed on one bounded same-package `callActivity` owner while the called child process runs, then cancel that child process and restore the parent frame onto the selected boundary route; one bounded same-package `callActivity` mixed-owner shape can now let either that armed timer/message/signal boundary or one matching parent error boundary win while clearing the non-selected owner-level waits and sibling boundaries; one interrupting timer, message, or signal boundary path can also stay armed on one bounded transaction shell owner while the child shell runs, then cancel that child shell and restore the parent frame onto the selected boundary route; one bounded transaction mixed-owner shape can now let either that armed timer/message/signal boundary, the parent cancel boundary with rollback, or one matching parent error boundary win while explicitly clearing owner waits and cancelling non-selected sibling boundaries, and the combined mixed-owner shape may keep one interrupting cancel boundary and one or more interrupting error boundaries adjacent to that single external boundary on the same transaction owner; one bounded non-interrupting timer, message, or signal boundary path can keep the original task blocked while opening one concurrent boundary branch on one non-repeating or bounded standard-loop, sequential multi-instance, or parallel multi-instance host-blocking task; one bounded `eventBasedGateway` can race those waits and cancel the losing siblings; one bounded top-level error end path can fail the instance terminally while preserving merged variables and clearing remaining frontier state; one bounded transaction cancel path can restore the parent frame, roll back transaction-local variable mutations, and route through the parent cancel boundary; one bounded transaction cancel compensation path can replay explicit compensation handlers in reverse completion order before that parent cancel boundary fires; one bounded transaction error path can restore the parent frame, preserve transaction-local variable mutations, route through every matching parent error boundary including one catch-all boundary, and cancel non-selected sibling boundaries; one bounded embedded-subprocess error path can restore the parent frame, preserve variable mutations, route through every matching parent error boundary including one catch-all boundary, and cancel non-selected sibling boundaries; one bounded same-package `callActivity` error path can restore the parent frame, preserve variable mutations, route through every matching parent error boundary including one catch-all boundary, and cancel non-selected sibling boundaries; and one bounded throw-compensation path can now replay the referenced compensable activity either before transaction-shell completion from a nested `endEvent` or before normal downstream sequence-flow routing resumes from a nested `intermediateThrowEvent`, with default replay draining every already compensable activity in reverse completion order for either throw shape, bounded asynchronous intermediate routing letting the compensation queue drain while downstream sequence flow continues, and bounded asynchronous end-event routing letting the parent scope resume while detached compensation replay finishes | P1       | broader event families                                                         |
| Script, send, and receive tasks                                                                                         | Registered in `SpiffWorkflow/spiff/parser/process.py`; tests cover script and event-driven workflows                                                                | Bounded `sendTask` and `receiveTask` are now supported when they carry exactly one message binding through task-level `messageRef` or one nested `messageEventDefinition`; bounded `scriptTask` is now also supported when it preserves one optional `scriptFormat` attribute and one optional nested `<bpmn:script>` body                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | `receiveTask` now reuses the engine-owned message-wait shell, `sendTask` now reuses the host-dispatch shell with preserved message metadata, `scriptTask` now reuses that same host-dispatch shell with preserved bounded script metadata, and `xiuxian-qianji` now wires both `sendTask` and `scriptTask` host work through the callback bridge                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               | P1       | correlations and broader collaboration-aware message routing                   |
| Subprocess, call activity, transaction subprocess                                                                       | Registered in parser and serializer config; tests cover call activity and nested processes                                                                          | One bounded embedded `subProcess` body with exactly one nested `startEvent` and at least one nested `endEvent`, one bounded embedded subprocess owner with one or more interrupting error boundaries plus one interrupting timer/message/signal boundary, one bounded `<transaction>` shell with exactly one nested `startEvent` and at least one nested `endEvent`, one bounded transaction owner with either one interrupting timer/message/signal boundary on its own, one interrupting timer/message/signal boundary plus one interrupting cancel boundary, one interrupting timer/message/signal boundary plus one or more interrupting error boundaries, one interrupting timer/message/signal boundary plus one interrupting cancel boundary plus one or more interrupting error boundaries, or one cancel boundary plus one or more error boundaries, one bounded transaction cancel compensation subset with one explicit compensation-handler marker per compensable activity, one bounded throw-compensation subset with targeted or default replay on nested `endEvent` or nested `intermediateThrowEvent` where either throw shape may stay synchronous or use bounded `waitForCompletion=\"false\"` routing, one bounded `callActivity` that targets another executable process in the same BPMN package, and one bounded same-package `callActivity` owner with one interrupting timer/message/signal boundary plus one or more interrupting error boundaries are supported; compensation event subprocesses and broader transaction error propagation remain unsupported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | The runtime can enter any of those bounded nested shapes through the same child-process frame model, suspend there, restore the parent frame on child completion, execute one bounded transaction cancel path with rollback through the parent boundary, execute one bounded transaction cancel compensation path that replays explicit compensation handlers in reverse completion order before the parent cancel boundary, execute one bounded transaction error path without rollback through every matching parent error boundary, execute one bounded transaction interrupting external-boundary path through one parent-owned timer/message/signal wait, execute one bounded transaction mixed-owner path through either one parent-owned timer/message/signal wait, the parent cancel boundary with rollback, or one matching parent error boundary, including the combined same-owner external-plus-cancel-plus-error subset, execute one bounded embedded-subprocess error path through every matching parent error boundary, execute one bounded embedded-subprocess interrupting external-boundary path through one parent-owned timer/message/signal wait, execute one bounded embedded-subprocess mixed-owner path through either one parent-owned timer/message/signal wait or one matching parent error boundary, execute one bounded same-package `callActivity` error path through every matching parent error boundary, execute one bounded same-package `callActivity` interrupting external-boundary path through one parent-owned timer/message/signal wait, execute one bounded same-package `callActivity` mixed-owner path through either one parent-owned timer/message/signal wait or one matching parent error boundary, cancel non-selected sibling boundaries on the same supported owner, and execute one bounded throw-compensation path that replays the referenced compensable activity either before normal completion from a nested `endEvent` or before normal downstream sequence-flow routing resumes from a nested `intermediateThrowEvent`, with default replay draining every already compensable activity in reverse completion order for either throw shape, the bounded asynchronous intermediate subset keeping the queue detached while downstream routing continues, and the bounded asynchronous end-event subset letting the parent scope resume while detached compensation replay drains, but recursive call graphs, compensation event subprocesses, broader nested subprocess families, and broader mixed transaction-shell boundary families remain unsupported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | P1       | broader transaction error propagation                                          |
| Standard loop and multi-instance tasks                                                                                  | ReadTheDocs lists loop, parallel multi-instance, sequential multi-instance; tests cover both loop and multi-instance                                                | Bounded `standardLoopCharacteristics`, bounded sequential `multiInstanceLoopCharacteristics isSequential="true"`, and bounded parallel `multiInstanceLoopCharacteristics` with omitted or `isSequential="false"` plus integer `loopCardinality` are now supported on one service/user/manual/business-rule task family; one bounded `completionCondition` subset is now also supported on those same multi-instance shapes, and one bounded collection-backed data-binding subset using `loopDataInputRef`, `inputDataItem`, optional `loopDataOutputRef`, and `outputDataItem` is now supported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | Standard loop now supports `testBefore` skip, loop-maximum re-entry, simple boolean conditions such as `done` or `not done`, and one owner-level non-interrupting boundary branch that stays armed across bounded re-entry until the boundary wins or the final completion clears the owner; sequential multi-instance now supports checkpoint-safe sequential re-entry, repeat-context propagation, zero-cardinality skip, bounded `completionCondition` early-stop, interrupting-boundary cleanup, one owner-level non-interrupting boundary branch that stays armed across iteration handoff until the boundary wins or the final pending iteration completes, and collection-backed iteration overlays/output aggregation; bounded parallel multi-instance now supports single-writer token fan-out, per-iteration repeat-context propagation, zero-cardinality skip, bounded `completionCondition` sibling cancellation, interrupting-boundary cleanup, one owner-level non-interrupting boundary branch that stays armed until the boundary wins or the final pending iteration completes, and collection-backed iteration overlays/output aggregation                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | P1       | transaction subprocess and richer nested orchestration                         |
| Collaboration, pools, lanes, messages, correlations                                                                     | README mentions pools and lanes; tests cover collaboration, correlations, and swimlanes                                                                             | Unsupported document family                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | Unsupported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | P1       | collaboration and lane metadata slice after core execution parity              |
| Data object, data store, IO specification                                                                               | ReadTheDocs lists data object and data store; tests cover data object, data store reference, and IO spec                                                            | Unsupported document family                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | Unsupported workflow-data binding model                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | P1       | data binding slice after core control flow                                     |
| Schema validation and broader source-bundle handling                                                                    | Spiff has BPMN and DMN validators and dependency discovery; tests cover invalid workflows and dependency detection                                                  | `parse_bpmn_package(...)` still rejects schema validation and multi-BPMN bundles, but one bounded `BpmnBundleSnapshot` with exactly one BPMN source plus optional DMN sources is now supported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | not applicable                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | P2       | parser completeness and import/dependency slice                                |

## 4. BPMN Audit Interpretation

The BPMN mismatch is not just a parser whitelist problem.

The deeper issue is that the current engine has been optimized around one
bounded runtime shape:

1. one BPMN package with one active process frame at a time plus a bounded
   parent-frame `call_stack`
2. compact multi-token state only for bounded gateway routing inside the active
   frame
3. one active wait or host-blocking boundary in the current frame, with parent
   frames checkpointed rather than co-running
4. explicit host blocking only at leaf task nodes

That runtime shape is coherent for the landed v1 slice, but it means the
following `SpiffWorkflow` families remain blocked even if parser support were
added mechanically:

1. broader transaction error propagation beyond the landed bounded
   throw-compensation subsets,
   and richer nested orchestration beyond the landed bounded embedded
   `subProcess`, bounded `<transaction>` shell, one bounded transaction owner
   with one cancel boundary plus one or more error boundaries, one bounded
   transaction cancel compensation subset, and `callActivity` shapes
2. multi-instance expansion and richer repeatable-task aggregation
3. broader event families beyond the bounded message/signal/timer race shape
   plus the landed bounded `receiveTask`/`sendTask` message-task family
4. lane-aware or collaboration-aware execution surfaces
5. broader condition-driven gateway branching beyond the bounded simple
   boolean-path or numeric-comparison exclusive subset

The correct reading is therefore:

1. parser parity is still partial rather than broad
2. runtime parity is stronger than before, but still materially narrower than
   `SpiffWorkflow`
3. BPMN implementation order must still be driven by runtime semantics, not by tag
   count

## 5. DMN Parity Matrix

| DMN capability                                                                          | Current engine status                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | SpiffWorkflow status                                                                                         | Nuance                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | Severity |
| --------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| One DMN source with one decision                                                        | Supported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | Supported                                                                                                    | Both implementations handle the simple case                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | baseline |
| Multiple decisions per source                                                           | Supported within the bounded parser subset                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | Broader file/version handling exists, but the parser remains bounded in structure                            | Current engine now materializes multiple decision definitions from one source while both implementations still remain narrower than full DMN schema/version breadth                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | baseline |
| One decision with one decision table                                                    | Supported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | Supported                                                                                                    | This is the strongest shared surface                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | baseline |
| Multiple decision tables inside one decision                                            | Unsupported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | DMN parser comment explicitly says it assumes one decision table within a decision                           | Not a parity gap worth prioritizing because upstream is also bounded here                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | P3       |
| Hit policies `UNIQUE` and `COLLECT`                                                     | Supported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | Supported                                                                                                    | This is the strongest overlapping execution surface                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | baseline |
| Other hit policies such as `FIRST`, `PRIORITY`, `ANY`, `RULE ORDER`, `OUTPUT ORDER`     | Unsupported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | Not implemented in the active `HitPolicy` enum even though schema files mention them                         | Do not overstate upstream support                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | P3       |
| Literal equality with strings, numbers, booleans, `null`, wildcard `-`                  | Supported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | Supported indirectly through script evaluation                                                               | Current engine is stricter and easier to reason about                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | baseline |
| FEEL-like expressions, comparison operators, range syntax, and script-backed predicates | Partially supported: numeric unary comparisons, bounded numeric ranges, ISO date literals, ISO date comparisons, bounded ISO date ranges, ISO local datetime literals, ISO local datetime comparisons, bounded ISO local datetime ranges, RFC3339 offset-aware datetime literals, comparisons, and bounded ranges, signed ISO 8601 day-time duration literals, comparisons, and bounded ranges, signed ISO 8601 year-month duration literals, comparisons, and bounded ranges, plus ISO time literals, ISO time comparisons, and bounded ISO time ranges are now supported, but broader FEEL and script-backed predicates remain unsupported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | Supported through the script engine in `DMNEngine.evaluate(...)`; tests cover ranges, comparisons, and dates | The gap narrowed further, but upstream still proves materially broader evaluator semantics                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | P1       |
| Date decisions and richer temporal predicates                                           | Partially supported: ISO date-only equality, comparisons, and bounded ranges plus ISO local datetime equality, comparisons, and bounded ranges plus RFC3339 offset-aware datetime equality, comparisons, and bounded ranges plus one bounded mixed local-vs-offset UTC normalization rule that now also covers datetime literal equality in addition to comparisons and ranges, plus signed ISO 8601 day-time duration equality, comparisons, and bounded ranges including bounded fractional day-time forms such as `duration("P1.5D")`, `duration("P1,5D")`, `duration("PT1.5H")`, `duration("PT1,5H")`, `duration("PT1.5M")`, `duration("PT1,5M")`, `duration("PT1.5S")`, and `duration("PT1,5S")`, plus signed ISO 8601 year-month duration equality, comparisons, and bounded ranges, plus ISO time-only equality, comparisons, and bounded ranges are now supported                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | Covered by python-engine tests with broader datetime semantics                                               | Local support is still intentionally narrower than upstream and currently excludes trailing-lower-unit fractional forms such as `duration("PT1.5H30S")`, fractional year-month duration literals, mixed year-month/day-time duration forms, and script-backed temporal functions                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | P1       |
| DMN schema/version parsing                                                              | Partially supported: one non-executable document snapshot can now scan namespaced/versioned DMN roots plus decision headers, model-version hints, top-level `import`, `itemDefinition`, `inputData`, `knowledgeSource`, `businessKnowledgeModel`, `decisionService`, `organizationUnit`, `performanceIndicator`, `textAnnotation`, `association`, `elementCollection`, `dmndi:DMNDI`, and `group` counts, plus bounded top-level `itemDefinition` metadata with one direct `itemComponent` placeholder layer, plus bounded top-level `inputData` metadata with one optional direct `variable` placeholder layer, plus bounded top-level `knowledgeSource` metadata, plus bounded top-level `decisionService` metadata with direct `outputDecision`, `encapsulatedDecision`, `inputDecision`, and `inputData` href placeholders, plus bounded top-level `businessKnowledgeModel` metadata with one optional invocable `variable` placeholder, one bounded `encapsulatedLogic` function-definition placeholder, and one direct body `literalExpression` placeholder, plus bounded top-level `organizationUnit` metadata, plus bounded top-level `performanceIndicator` metadata, plus bounded top-level `textAnnotation` metadata with one direct text payload, plus bounded top-level `association` metadata with one direct `associationDirection` / `sourceRef` / `targetRef` placeholder layer, plus bounded top-level `elementCollection` metadata, plus bounded top-level `group` metadata, plus bounded top-level `dmndi:DMNDI` metadata with one direct `DMNDiagram` placeholder layer carrying bounded diagram ids, direct shape/edge counts, direct `DMNShape` / `DMNEdge` placeholder metadata bounded to optional `id` plus `dmnElementRef`, one optional direct `DMNShape.isListedInputData` boolean, one optional direct `DMNShape.isCollapsed` boolean, one optional direct `dc:Bounds` placeholder under `DMNShape` bounded to one optional x/y/width/height contract, one repeated direct `di:waypoint` placeholder list under `DMNEdge` bounded to optional x/y pairs, one optional direct `DMNLabel` placeholder bounded to one optional label id plus one optional direct `dc:Bounds` placeholder plus one optional direct text payload, and one optional direct `DMNDecisionServiceDividerLine` placeholder under `DMNShape` bounded to one repeated direct `di:waypoint` placeholder list with optional x/y pairs, decision-owned `allowedAnswers`, `decisionMaker`, and `decisionOwner` counts, direct decision-owned requirement counts for `informationRequirement`, `knowledgeRequirement`, and `authorityRequirement`, nested requirement-target counts for `requiredInput`, `requiredDecision`, `requiredKnowledge`, and `requiredAuthority`, direct decision-owned requirement reference metadata with parent requirement kind, direct reference kind, and href, direct decision-logic counts for `literalExpression`, `context`, `invocation`, `relation`, `functionDefinition`, and `list`, direct invocation metadata for invoked literal-expression text plus binding parameter/argument placeholders, and non-executable direct functionDefinition metadata for function kind, formal parameters, and body literal-expression placeholders; executable parsing now also rejects non-`definitions` roots, missing or unsupported DMN model namespaces, top-level `<import>` declarations, preserves bounded executable clause `typeRef` metadata on `inputExpression` and `output` clauses, can materialize one bounded direct `literalExpression` decision for literal, variable-path, or whitespace-delimited numeric `path +/- number` execution, can materialize one bounded direct `list` decision whose direct children are bounded literal-expression items, can materialize one bounded direct `context` decision whose ordered entries are bounded literal-expression bodies with optional variable names and an optional final unnamed result entry, can materialize one bounded direct `relation` decision whose direct rows contain one bounded literal-expression cell per direct column, can materialize one bounded direct `invocation` decision, parser-owned bundle loading can now materialize bounded package-owned same-source BKM and decision-service registries from top-level metadata, and local package-aware DMN runtime can now execute one bounded same-source `decisionService` as a thin alias to one or more direct local `outputDecision` targets, validates preserved same-source `encapsulatedDecision` / `inputDecision` / `inputData` refs before running that alias, and still executes one bounded same-source invocation seam against the BKM registry while preserving or consuming one bounded executable same-source `requiredKnowledge` href constraint for that invocation target, but it still performs no full schema validation and broader boxed-expression execution remains deferred | DMN 1.0, 1.2, and 1.3 schema/version handling exists in `BpmnDmnParser` and version tests                    | The crate now has one real placeholder surface for later adapter work and for construct-aware lint diagnostics across the main unsupported type-model, business-context, annotation, document-structure, diagram-interchange, group-artifact, decision-metadata, governance-metadata, decision-logic, metadata-only DRD artifact, and decision-dependency shapes seen in the current research lane, while the latest business-context, item-definition, text-annotation, association metadata, element-collection metadata, group metadata, DMNDI metadata, diagram-element metadata, listed-input-data shape metadata, direct-shape bounds metadata, direct-shape `isCollapsed` metadata, direct-label placeholder metadata, direct-label bounds metadata, direct-label text metadata, direct-edge waypoint metadata, direct decision-service divider-line metadata, allowed-answers, decision-governance, root-artifact, requirement-target href metadata, requirement-edge, document-root, import validation, non-executable item-definition metadata, non-executable input-data metadata, non-executable knowledge-source metadata, non-executable decision-service reference metadata, non-executable business-knowledge-model invocable metadata, non-executable business-context metadata, non-executable text-annotation metadata, non-executable document-structure metadata, non-executable DMNDI metadata, executable clause-type metadata, direct invocation metadata evidence, non-executable functionDefinition metadata evidence, bounded direct-literal execution, bounded direct-list execution, bounded direct-context execution, bounded direct-relation execution, bounded direct-invocation execution, bounded same-source `requiredKnowledge` invocation-target constraints, bounded same-source `requiredDecision` recursion, bounded same-source `requiredInput` alias binding, bounded top-level BKM invocable snapshot capture, bounded package-owned BKM registry materialization, bounded package-owned decision-service registry materialization, and one bounded same-source decision-service alias runtime seam with direct local multi-output context support closes real parser/lint/runtime safety gaps before broader dependency resolution or broader schema parity, but full parser/schema parity is still materially incomplete | P2       |
| BPMN `businessRuleTask` to DMN execution integration                                    | Partially supported: engine-owned package registries can execute locally, parser-owned bundle snapshots can now populate those registries, and `xiuxian-qianji` now owns a bounded host adapter for missing-definition fallback                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | Integrated parser-to-engine binding exists                                                                   | The unconditional host-only gap is closed, parser-owned registration now exists, and the host adapter now exists in bounded form, but full BPMN scheduler/CLI orchestration is still missing                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | P1       |

## 6. DMN Audit Interpretation

The DMN result needs one important guardrail:

`SpiffWorkflow` is broader than the current engine, but it is not evidence of
full DMN standard coverage.

The audit-proven statement is narrower:

1. `qianji-bpmn-engine` currently implements a deliberately bounded DMN
   contract.
2. `SpiffWorkflow` implements a materially richer DMN parse and evaluation
   surface than that bounded contract, even after the local numeric,
   ISO-date, ISO-local-datetime, mixed local-vs-offset UTC coercion,
   signed day-time duration, signed year-month duration, and ISO-time
   comparison/range widening cuts.
3. `SpiffWorkflow` still does not prove full DMN standard parity because its
   decision-table model and active hit-policy enum remain limited.

That distinction matters because otherwise the lane may overreact and try to
port speculative DMN completeness before the adapter and runtime seams are
ready.

## 7. Lint and Adapter Implications

The current linter contract is aligned with the bounded engine, not with
`SpiffWorkflow`.

This is correct for now.

It means:

1. `qianji lint --bpmn` is currently an engine-subset validator, not a
   `SpiffWorkflow`-parity validator, even though it now emits task-specific
   repair guidance for the bounded `receiveTask`/`sendTask` message-task
   family.
2. `qianji lint --dmn` is currently a bounded decision-table validator with
   construct-aware placeholder diagnostics, not a FEEL-capable DMN validator.
3. `xiuxian-qianji` adapter work should not advertise richer BPMN or DMN
   support until the engine slices land first.

For the later `xiuxian-qianji` adapter lane, the audit implies that parser
parity and runtime parity must move together enough that the CLI does not tell
LLM tooling to "repair" files into shapes the runtime still cannot execute.

## 8. Recommended Post-Audit Slice Order

The recommended order is driven by execution semantics rather than by document
surface breadth.

1. Frontier concurrency semantics slice
   This slice should align the runtime with OMG BPMN expectations for multiple
   simultaneously runnable nodes under one workflow-instance owner, instead of
   keeping first-token planning and singleton pending host work as an implicit
   limitation.
2. Broader DMN evaluator widening slice
   One non-executable DMN document snapshot with namespace/version hints is
   now landed together with the bounded numeric, ISO-date,
   ISO-local-datetime, one bounded mixed local-vs-offset UTC coercion rule,
   RFC3339 offset-aware datetime, signed ISO 8601 day-time duration,
   signed ISO 8601 year-month duration, and ISO-time comparison/range cuts.
   Fractional duration seconds, mixed-family duration forms, broader FEEL
   semantics, and executable schema validation remain deferred.
3. `xiuxian-qianji` higher-level BPMN orchestration slice
   The bounded host adapter plus CLI runtime facade are now landed, so the
   next host-side gap is scheduler-owned distributed orchestration and
   broader writer-ownership adoption beyond the current bounded CLI/runtime
   execution surface.
4. Transaction error semantics and richer nested orchestration slice
   This should widen beyond the landed bounded embedded `subProcess`,
   bounded `<transaction>` shell, bounded transaction cancel path, and
   `callActivity` ownership model.
5. Collaboration, lane, and data-binding slice
   This should follow after the core execution model is stable.
6. Schema/version/import-completeness slice
   This should harden parser completeness once runtime semantics are no longer
   moving rapidly.

## 9. Final Audit Verdict

The current engine is not aligned with `SpiffWorkflow` BPMN or DMN syntax
coverage.

The strongest precise statement supported by source evidence is:

1. `qianji-bpmn-engine` currently matches a bounded BPMN subset that now
   includes linear flow plus deterministic `parallelGateway` and
   `exclusiveGateway` support, one bounded exclusive `eventBasedGateway`
   race, together with bounded intermediate message/signal/timer waits, one
   bounded `receiveTask` message-wait shell, one bounded `sendTask`
   host-dispatch shell, one interrupting timer/message/signal boundary path, one bounded
   non-interrupting timer/message/signal boundary path on one non-repeating, bounded
   standard-loop, or bounded sequential or parallel multi-instance host-blocking
   task, one bounded embedded `subProcess` body, one bounded `<transaction>`
   shell, one bounded transaction cancel path, one bounded same-package
   `callActivity`, one bounded
   `standardLoopCharacteristics` shape, and bounded sequential or bounded
   parallel cardinality-driven `multiInstanceLoopCharacteristics`
   shapes with one bounded `completionCondition` subset plus one bounded
   collection-backed data-binding subset, plus one engine-owned local
   `businessRuleTask` path when the package already carries a matching DMN
   definition, plus one parser-owned bundle snapshot path that can register
   bounded DMN definitions into that package, but still covers only a small
   part of `SpiffWorkflow`.
2. `qianji-bpmn-engine` currently matches only the bounded core of
   `SpiffWorkflow` DMN support: one decision table with `UNIQUE` or `COLLECT`,
   plus wildcard/literal matching, numeric unary comparisons, bounded numeric
   ranges, ISO date literals, ISO date comparisons, bounded ISO date ranges,
   ISO local and RFC3339 offset-aware datetime literals/comparisons/ranges,
   signed ISO 8601 day-time duration literals/comparisons/ranges, signed ISO
   8601 year-month duration literals/comparisons/ranges, and ISO time
   literals, ISO time comparisons, and bounded ISO time ranges.
3. The next useful implementation target is not "all missing tags". It is the
   runtime semantics needed to unlock multi-instance expansion, richer nested
   orchestration, and richer DMN behavior in an order that preserves
   checkpoint and host-bridge integrity.

## 10. Follow-up After DMN Unsupported Construct Classification Slice

The parity note needs one additional precision update after the latest bounded
DMN placeholder slice.

What changed:

1. the crate still does not execute `decisionService` or direct
   `literalExpression` decisions, so there is no claim of broader executable
   parity here
2. the non-executable DMN snapshot surface can now classify those constructs
   explicitly, which closes a real adapter/lint gap against the broader
   versioned DMN documents present in the imported `SpiffWorkflow` research
   lane
3. `qianji lint --dmn` can now tell LLM-driven repair flows not to fabricate
   decision-table logic from `decisionService` metadata or from direct
   `literalExpression` bodies, which is materially safer than the earlier
   generic missing-decision or missing-table guidance
4. executable schema validation, broader FEEL semantics, `decisionService`
   execution, and `literalExpression` execution all remain explicitly
   deferred

## 10.1 Follow-up After DMN Literal Expression Runtime Slice

The earlier unsupported-construct statement is now narrowed for direct
`literalExpression` decisions.

What changed:

1. one direct decision-owned `<literalExpression><text>` body can now parse and
   execute when the text is a supported literal, one variable path, or one
   whitespace-delimited numeric `path +/- number` operation
2. runtime output is object-shaped as `{ "<decision_id>": <value> }`, preserving
   the existing BPMN `businessRuleTask` object merge contract
3. `qianji lint --dmn` now accepts the supported direct-literal subset and emits
   `dmn.unsupported_literal_expression_subset` only when broader FEEL text is
   encountered
4. broader `context`, `invocation`, `functionDefinition`, broader `relation`,
   broader list children, full FEEL arithmetic, and `decisionService`
   execution remain explicitly deferred

## 10.2 Follow-up After DMN List Expression Runtime Slice

The earlier placeholder-only direct `list` statement is now narrowed.

What changed:

1. one direct decision-owned `<list>` body can now parse and execute when every
   direct child is a supported bounded `<literalExpression>` item
2. runtime output is object-shaped as `{ "<decision_id>": [<values>...] }`,
   preserving the existing BPMN `businessRuleTask` object merge contract
3. `qianji lint --dmn` now accepts the supported direct-list subset and emits
   `dmn.unsupported_list_expression_subset` for unsupported item text or
   `dmn.unsupported_list_child` for non-literal direct list children
4. nested lists, broader context entries, `invocation`,
   `functionDefinition`, broader relation cells, full FEEL arithmetic, and
   `decisionService` execution remain explicitly deferred

## 10.3 Follow-up After DMN Context Expression Runtime Slice

The earlier placeholder-only direct `context` statement is now narrowed.

What changed:

1. one direct decision-owned `<context>` body can now parse and execute when
   every direct `<contextEntry>` contains optional variable metadata plus one
   bounded `<literalExpression>` body
2. context entries execute in source order; named entries become local
   variables visible to later entries, and one final unnamed entry returns the
   decision value
3. runtime output is object-shaped as `{ "<decision_id>": <value> }`, preserving
   the existing BPMN `businessRuleTask` object merge contract
4. `qianji lint --dmn` now accepts the supported direct-context subset and
   emits `dmn.unsupported_context_expression_subset` for unsupported entry text
   or `dmn.unsupported_context_child` for children outside the bounded
   context-entry shape
5. nested contexts, invocation, function-definition, broader relation cells,
   nested-list, full FEEL arithmetic, and `decisionService` execution remain
   explicitly deferred

## 11. Follow-up After DMN Context and Invocation Classification Slice

The parity note needed one more precision update after this bounded DMN
placeholder slice. The runtime follow-up below now narrows the executable
subset.

What changed:

1. direct `context` execution is limited to the bounded context-entry subset
   recorded in the follow-up above, while direct `invocation` execution is now
   limited to one narrower callable seam described below
2. the non-executable DMN snapshot surface can still classify those constructs
   explicitly, and later snapshot-evidence slices also preserve direct
   invocation function-expression and binding placeholders for lint and audit
   evidence against the broader versioned DMN documents present in the imported
   `SpiffWorkflow` research lane
3. `qianji lint --dmn` can now tell LLM-driven repair flows not to flatten
   context entries or to claim broader invocation parity when the local
   callable contract is missing, which is materially safer than the earlier
   generic missing-table guidance
4. executable schema validation, broader FEEL semantics, broader `context`
   execution, and broader invocation semantics still remain explicitly
   deferred

## 11.1 Follow-up After DMN Invocation Snapshot Evidence Slice

The earlier direct `invocation` placeholder statement is now obsolete. The
crate now owns one bounded executable direct invocation seam.

What changed:

1. direct decision-owned `<invocation>` is now executable only when the
   invoked text resolves to exactly one same-source top-level
   `businessKnowledgeModel` by id or invocable `variable` name
2. the local runtime binds explicit named parameters through supported
   literal-expression arguments and evaluates one supported direct
   `encapsulatedLogic` literal-expression body from the target BKM
3. the decision snapshot still preserves invoked-expression text plus binding
   parameter and argument evidence, and `qianji lint --dmn` now blocks
   invocation shapes that parse but do not satisfy that bounded local callable
   contract
4. standalone public DMN evaluation still does not execute invocation without
   package-owned same-source BKM context, and broader called-function
   resolution, import handling, standalone `requiredKnowledge` execution,
   decision-service invocation, full schema validation, and broader FEEL
   semantics remain explicitly deferred

## 11.2 Follow-up After DMN Required-Knowledge Runtime Contract Slice

The earlier required-knowledge placeholder statement also needs narrowing after
the latest local-runtime cut.

What changed:

1. executable DMN decision definitions now preserve direct same-source
   `<knowledgeRequirement><requiredKnowledge href="#..."/></knowledgeRequirement>`
   edges alongside the already-landed invocation seam
2. the local runtime now resolves those direct same-source BKM hrefs through
   the package-owned BKM registry and uses them as an invocation-side target
   constraint when the decision already has direct local `<invocation>` logic
3. `qianji lint --dmn` now blocks invocation shapes whose target, while still
   locally resolvable, falls outside the explicitly declared same-source
   required-knowledge edges
4. required-knowledge-only decisions still do not materialize missing local
   decision logic automatically, and broader knowledge-requirement recursion,
   imports, decision-service calls, full schema validation, and broader FEEL
   callable semantics remain explicitly deferred

## 12. Follow-up After DMN Relation Classification Slice

The parity note needs one more precision update after the latest bounded DMN
placeholder slice.

What changed:

1. at this placeholder slice, the crate still did not execute direct
   `relation` decisions, so there was still no claim of broader executable
   parity here
2. the non-executable DMN snapshot surface can now classify that construct
   explicitly, which closes another real adapter/lint gap against the
   broader versioned DMN documents present in the imported `SpiffWorkflow`
   research lane
3. `qianji lint --dmn` could then tell LLM-driven repair flows not to flatten
   direct `relation` rows into guessed decision-table logic before the bounded
   runtime subset existed, which was materially safer than the earlier generic
   missing-table guidance
4. executable schema validation, broader FEEL semantics, and `relation`
   execution remained explicitly deferred until the bounded runtime slice below

## 12.1 Follow-up After DMN Relation Expression Runtime Slice

The earlier placeholder-only direct `relation` statement is now narrowed.

What changed:

1. one direct decision-owned `<relation>` body can now parse and execute when
   it contains direct columns and every direct row has one bounded
   `<literalExpression>` cell per column
2. runtime output is object-shaped as
   `{ "<decision_id>": [{ "<column_key>": <cell_value>, ... }, ...] }`,
   preserving the existing BPMN `businessRuleTask` object merge contract
3. `qianji lint --dmn` now accepts the supported direct-relation subset and
   emits `dmn.unsupported_relation_expression_subset` for unsupported cell text
   or `dmn.unsupported_relation_child` for children outside the bounded
   column/row shape
4. nested relations, broader boxed cell expressions, imports, DRD dependency
   execution, full schema validation, and broader FEEL semantics remain
   explicitly deferred

## 13. Follow-up After DMN Function and List Classification Slice

The parity note needs one more precision update after the latest bounded DMN
placeholder slice.

What changed:

1. the crate still does not execute direct `functionDefinition` decisions or
   broader direct-list children, so there is still no claim of broader
   executable parity here
2. the non-executable DMN snapshot surface can now classify those constructs
   explicitly, which closes the remaining direct-expression adapter/lint gap
   exposed by the official DMN 1.3 schema after the earlier
   `literalExpression`, `context`, `invocation`, and `relation` cuts
3. `qianji lint --dmn` can now tell LLM-driven repair flows not to inline
   function bodies or flatten unsupported direct list children into guessed
   decision-table logic, which is materially safer than the earlier generic
   missing-table guidance
4. executable schema validation, broader FEEL semantics,
   `functionDefinition` execution, and broader nested list execution all remain
   explicitly deferred

## 13.1 Follow-up After DMN Function Definition Snapshot Evidence Slice

The earlier direct `functionDefinition` placeholder statement is now more
precise.

What changed:

1. direct decision-owned `<functionDefinition>` remains non-executable, so there
   is no claim of function-runtime parity
2. the non-executable decision snapshot now preserves the function-definition
   id, kind, direct formal parameters, and direct body literal-expression text
3. `qianji lint --dmn` now includes that function-definition evidence in
   `dmn.unsupported_function_definition_decision`, so LLM repair flows can
   preserve parameter and body shape instead of guessing decision-table rules
4. function body evaluation, business-knowledge-model execution, imports, DRD
   dependency execution, full schema validation, and broader FEEL semantics
   remain explicitly deferred

## 14. Follow-up After DMN Document Root Validation Slice

The parity note needs one more precision update after the latest bounded DMN
document-validation slice.

What changed:

1. the crate still does not perform full DMN XSD validation, import
   validation, or diagram-interchange validation, so there is still no claim
   of broad schema-completeness parity here
2. the executable parser now rejects non-`definitions` roots and missing or
   unsupported DMN model namespaces before decision parsing continues, which
   closes one real document-level safety gap exposed by broader versioned DMN
   sources in the imported `SpiffWorkflow` research lane
3. `qianji lint --dmn` can now tell LLM-driven repair flows about document-level
   root and model-namespace failures directly, which is materially safer than
   misclassifying those cases as missing-decision-table or other placeholder
   issues
4. full schema validation, broader FEEL semantics, import handling, and DI
   semantics all remain explicitly deferred

## 15. Follow-up After DMN Import Validation Slice

The parity note needs one more precision update after the latest bounded DMN
document-validation slice.

What changed:

1. the crate still does not resolve DMN imports, execute imported decisions,
   or evaluate imported item definitions, so there is still no claim of
   cross-document execution parity here
2. the non-executable DMN snapshot surface can now classify top-level import
   counts and preserve bounded import metadata explicitly, and the executable
   parser now rejects top-level `<import>` declarations before decision-table
   execution begins, which closes one real document-dependency safety gap
   exposed by imported `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows not to delete
   top-level imports blindly just to force local parsing, which is materially
   safer than silently ignoring cross-document dependency declarations
4. DMN import resolution, full schema validation, broader FEEL semantics,
   and item-definition execution all remain explicitly deferred

## 16. Follow-up After DMN Requirement Edge Classification Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not resolve DMN dependency edges, execute upstream
   decision requirements, or evaluate broader DRD semantics, so there is
   still no claim of dependency-resolution parity here
2. the non-executable DMN snapshot surface can now classify direct
   `informationRequirement`,
   `knowledgeRequirement`, and
   `authorityRequirement` counts explicitly, which closes another real
   adapter/lint gap exposed by the versioned `SpiffWorkflow` DMN research
   fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows not to
   fabricate local decision-table rules from requirement edges alone, which
   is materially safer than the earlier generic missing-table fallback
4. DMN dependency resolution, full schema validation, broader FEEL semantics,
   item-definition execution, and broader DRD execution all remain
   explicitly deferred

## 17. Follow-up After DMN Requirement Target Guidance Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not resolve DMN dependency edges or execute broader
   DRD semantics, so there is still no claim of dependency-resolution parity
   here
2. the non-executable DMN snapshot surface can now classify nested
   `requiredInput`,
   `requiredDecision`,
   `requiredKnowledge`, and
   `requiredAuthority` counts explicitly, which closes another real
   adapter/lint gap exposed by the versioned `SpiffWorkflow` DMN research
   fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows whether a
   requirement-edge-only decision points at input data, another decision, a
   business knowledge model, or an authority surface, which is materially
   safer than broader edge-only wording
4. DMN dependency resolution, full schema validation, broader FEEL semantics,
   item-definition execution, and broader DRD execution all remain
   explicitly deferred

## 18. Follow-up After DMN Root Artifact Guidance Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not execute top-level `inputData`,
   `knowledgeSource`, or `businessKnowledgeModel` artifacts directly, so
   there is still no claim of broader DRD execution parity here
2. the non-executable DMN root snapshot surface can now classify those
   top-level artifact counts explicitly, and it now preserves bounded
   top-level `inputData` metadata plus one optional direct `variable`
   placeholder layer, which closes another real adapter/lint gap exposed by
   metadata-only `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows whether a
   metadata-only DMN document is exposing top-level input data, knowledge
   sources, or business knowledge models, and for input-data-only documents
   it can now surface the bounded preserved input-data metadata directly,
   which is materially safer than the earlier generic missing-decision
   wording
4. DRD execution, DMN dependency resolution, full schema validation, broader
   FEEL semantics, and item-definition execution all remain explicitly
   deferred

## 18.1 Follow-up After DMN Requirement Reference Snapshot Evidence Slice

The parity note needs one more precision update after the latest bounded DMN
requirement-reference evidence slice.

What changed:

1. the crate still does not resolve or execute DRD dependency edges, so there
   is still no claim of dependency-resolution parity here
2. the non-executable DMN decision snapshot surface now preserves direct
   requirement target hrefs with parent requirement kind and target reference
   kind
3. `qianji lint --dmn` can now tell LLM-driven repair flows exactly which
   `requiredInput`, `requiredDecision`, `requiredKnowledge`, or
   `requiredAuthority` href was present instead of exposing counts only
4. DRD execution, href resolution, import resolution, full schema validation,
   and broader FEEL semantics all remain explicitly deferred

## 18.2 Follow-up After DMN Information Requirement Contract Slice

The parity note needs one more precision update after the latest bounded
executable DMN parser-contract slice.

What changed:

1. the crate still does not resolve or execute DRD dependency edges, so there
   is still no claim of local DRD runtime parity here
2. parsed executable `DmnDecisionDefinition` values now preserve direct
   `requiredInput` and `requiredDecision` href placeholders in source order
3. package-level DMN decision lookup keeps that parser-owned dependency
   contract intact, which closes another real bridge gap before later local
   dependency evaluation work
4. DRD execution, href resolution, import resolution, full schema validation,
   and broader FEEL semantics all remain explicitly deferred

## 18.3 Follow-up After DMN Required-Decision Local Resolution Slice

The parity note needs one more precision update after the latest bounded local
DMN runtime slice.

What changed:

1. the crate now consumes the executable `informationRequirement` contract for
   direct same-source `requiredDecision` edges during local BPMN
   `businessRuleTask` DMN execution
2. local evaluation resolves those upstream decisions recursively, overlays
   their output objects into the current decision input scope, and then
   evaluates the selected decision body
3. missing or non-local required-decision hrefs now fail explicitly, and
   cyclic required-decision graphs now fail explicitly
4. the caller must still supply the upstream input object; broader
   `requiredInput` remapping, `knowledgeRequirement`, `authorityRequirement`,
   `decisionService`, import resolution, full schema validation, and broader
   FEEL semantics all remain explicitly deferred

## 18.4 Follow-up After DMN Required-Input Runtime Contract Slice

The parity note needs one more precision update after the latest bounded local
DMN runtime slice.

What changed:

1. parser-owned bundle loading now materializes one bounded package-owned
   `inputData` registry from top-level DMN `inputData` metadata
2. local BPMN `businessRuleTask` DMN execution can now consume direct
   same-source `requiredInput` hrefs when the referenced `inputData.name` and
   nested `variable.name` are both explicit in the source
3. the runtime aliases the caller-supplied value from `inputData.name` into
   the nested `variable.name` only within the current decision-evaluation
   scope; it does not persist a new BPMN instance variable just from the alias
4. missing or non-local required-input hrefs now fail explicitly, while
   broader input-data mapping, broader DRD planning, import resolution, full
   schema validation, and broader FEEL semantics all remain explicitly
   deferred

## 18.5 Follow-up After DMN Required-Knowledge Runtime-Contract Research Slice

The parity note needs one more precision update after the latest bounded DMN
required-knowledge research slice.

What changed:

1. imported DMN 1.3 schema evidence and imported `SpiffWorkflow` test models
   both show that `businessKnowledgeModel` is an invocable shape with its own
   `variable` and `encapsulatedLogic` contract rather than a plain upstream
   data node
2. the current crate therefore needed a bounded parser/model slice before any
   honest local `requiredKnowledge` runtime work could resume
3. `requiredKnowledge`, broader BKM execution, invocation binding, import
   resolution, full schema validation, and broader FEEL semantics therefore
   remained explicitly deferred at the end of that research slice

## 18.6 Follow-up After DMN BKM Invocable Snapshot Contract Slice

The parity note needs one more precision update after the latest bounded DMN
parser/model slice.

What changed:

1. the crate now preserves one bounded top-level BKM invocable contract:
   optional `variable` metadata plus one bounded `encapsulatedLogic`
   function-definition placeholder with preserved kind, formal parameters, and
   one bounded literal-expression body seam
2. this closes the parser/model prerequisite that blocked honest same-source
   `requiredKnowledge` runtime work
3. runtime still does not execute preserved BKM invocable metadata, so there
   is still no claim of local `requiredKnowledge` execution parity here
4. broader BKM execution, invocation binding, import resolution, full schema
   validation, and broader FEEL semantics all remain explicitly deferred

## 18.7 Follow-up After DMN BKM Package Registry Slice

The parity note needs one more precision update after the latest bounded DMN
package-registry slice.

What changed:

1. parser-owned bundle loading can now materialize one bounded package-owned
   same-source BKM registry from top-level `businessKnowledgeModel` metadata
2. the new registry preserves the BKM id, name, invocable `variable` name and
   type, bounded `encapsulatedLogic` placeholder, and direct body placeholder
   needed by later runtime work without re-reading document snapshots
3. runtime still does not execute preserved BKM callable metadata or consume
   `requiredKnowledge` automatically, so there is still no claim of local
   `requiredKnowledge` execution parity here
4. broader BKM execution, invocation binding, import resolution, full schema
   validation, and broader FEEL semantics all remain explicitly deferred

## 18.8 Follow-up After DMN Invocation Executable-Contract Slice

The parity note needs one more precision update after the latest bounded DMN
invocation-runtime slice.

What changed:

1. the crate can now parse one direct decision-owned `<invocation>` into the
   executable DMN model contract instead of treating it as snapshot-only
   evidence
2. local package-aware DMN evaluation can now execute one bounded invocation
   seam that resolves the invoked text to exactly one same-source top-level
   `businessKnowledgeModel` by id or invocable `variable` name, binds explicit
   named parameters through supported literal-expression arguments, and
   evaluates one supported direct `encapsulatedLogic` literal-expression body
3. standalone public DMN evaluation still does not execute invocation without
   package-owned same-source BKM context, and `qianji lint --dmn` now blocks
   invocation shapes that parse but do not satisfy that bounded local callable
   contract
4. `requiredKnowledge`, broader BKM execution, broader callable recursion,
   imports, decision-service invocation, full schema validation, and broader
   FEEL semantics all remain explicitly deferred

## 19. Follow-up After DMN Item Definition Guidance Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not resolve or execute top-level `itemDefinition`
   declarations, so there is still no claim of DMN type-resolution parity
   here
2. the non-executable DMN root snapshot surface now preserves bounded
   top-level `itemDefinition` metadata plus one direct `itemComponent`
   placeholder layer, which closes a deeper adapter/lint gap exposed by
   metadata-rich `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows not just that a
   metadata-only DMN document exposes top-level item definitions, but also
   which bounded definition metadata is present, which is materially safer
   than the earlier generic missing-decision wording
4. item-definition resolution, DRD execution, DMN dependency resolution, full
   schema validation, and broader FEEL semantics all remain explicitly
   deferred

## 20. Follow-up After DMN Knowledge-Source Snapshot Metadata Slice

The parity note needs one more precision update after the latest bounded DMN
root-artifact metadata slice.

What changed:

1. the crate still does not resolve or execute top-level `knowledgeSource`
   declarations directly, so there is still no claim of governance- or
   authority-resolution parity here
2. the non-executable DMN root snapshot surface now preserves bounded
   top-level `knowledgeSource` metadata, which closes another real
   adapter/lint gap exposed by metadata-only `SpiffWorkflow` DMN research
   fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows not just that a
   metadata-only DMN document exposes top-level knowledge sources, but also
   which bounded knowledge-source metadata is present, which is materially
   safer than the earlier count-only wording
4. authority-reference resolution, DRD execution, DMN dependency
   resolution, full schema validation, and broader FEEL semantics all remain
   explicitly deferred

## 21. Follow-up After DMN Decision-Service Snapshot Metadata Slice

The parity note needs one more precision update after the latest bounded DMN
root-artifact metadata slice.

What changed:

1. the crate still does not resolve or execute top-level `decisionService`
   declarations directly, so there is still no claim of decision-service
   execution parity here
2. the non-executable DMN root snapshot surface now preserves bounded
   top-level `decisionService` metadata, which closes another real
   adapter/lint gap exposed by metadata-rich `SpiffWorkflow` DMN research
   fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows not just that a
   metadata-only DMN document exposes top-level decision services, but also
   which bounded decision-service metadata is present, which is materially
   safer than the earlier count-only wording
4. decision-service output resolution, DRD execution, DMN dependency
   resolution, full schema validation, and broader FEEL semantics all remain
   explicitly deferred

## 21.1 Follow-up After DMN Decision-Service Reference Snapshot Evidence Slice

The parity note needs one more precision update after the latest bounded DMN
decision-service evidence slice.

What changed:

1. the crate still does not resolve or execute top-level `decisionService`
   declarations directly, so there is still no claim of decision-service
   execution parity here
2. the non-executable DMN root snapshot surface now preserves direct
   `outputDecision`, `encapsulatedDecision`, `inputDecision`, and `inputData`
   href placeholders under each top-level decision service
3. `qianji lint --dmn` can now tell LLM-driven repair flows which
   decision-service references are present instead of forcing them to infer a
   service contract from id/name metadata alone
4. decision-service reference resolution, output-decision execution, DRD
   execution, DMN dependency resolution, full schema validation, and broader
   FEEL semantics all remain explicitly deferred

## 21.2 Follow-up After DMN Decision-Service Local Runtime Contract Slice

The parity note needs one more precision update after the latest bounded DMN
decision-service runtime slice.

What changed:

1. the crate no longer treats every top-level `decisionService` as
   categorically non-executable; it can now route one same-source service as a
   thin local alias to one or more direct local `outputDecision` targets
2. parser-owned bundle loading now materializes one bounded package-owned
   decision-service registry from preserved top-level decision-service
   metadata, so BPMN local business-rule runtime no longer has to re-read
   snapshot-only surfaces to consume that seam
3. the new runtime contract is intentionally narrow but no longer ignores the
   other preserved exposure refs completely: same-source
   `encapsulatedDecision` / `inputDecision` / `inputData` hrefs are now
   validated against the local package registries before the alias runs, while
   imported hrefs, zero-output services, broader DRD planning, and general
   decision-service orchestration all remain unsupported
4. `qianji lint --dmn` should still tell repair flows to preserve
   decision-service metadata, but metadata-only decision-service documents
   remain non-executable and should not be rewritten into fabricated
   decision-table logic

## 21.3 Follow-up After DMN Decision-Service Exposure Contract Slice

The earlier local-runtime note also needs one more precision update after the
latest bounded exposure-contract slice.

What changed:

1. one same-source local `decisionService` alias no longer ignores preserved
   `encapsulatedDecision`, `inputDecision`, or `inputData` refs when they are
   present on that service
2. before the local alias executes its one direct `outputDecision`, the
   evaluator now validates those preserved same-source exposure hrefs against
   the package-owned decision and input-data registries and fails explicitly on
   missing or non-local targets
3. this is still not general decision-service orchestration: the non-output
   refs act only as closure validation for the bounded alias seam and do not
   trigger DRD planning, input mediation, or hidden-decision execution on
   their own
4. imported services, broader output planning beyond direct same-source local
   decisions, broader exposure semantics, full schema validation, and broader
   FEEL semantics remain explicitly deferred

## 21.4 Follow-up After DMN Decision-Service Multi-Output Slice

The OMG DMN 1.5 decision-service execution semantics distinguish single-output
and multiple-output decision services. The local runtime now owns the bounded
same-source part of that behavior.

What changed:

1. a same-source local `decisionService` may now expose multiple direct local
   `outputDecision` targets instead of failing on output count greater than one
2. single-output services preserve the existing local output shape, while
   multiple-output services evaluate direct output decisions in source order
   and merge their outputs into one object-shaped context for BPMN variable
   merging
3. every output target still must be a local executable decision in the same
   source, and preserved same-source `encapsulatedDecision`, `inputDecision`,
   and `inputData` refs are still validated before execution
4. imported output decisions, hidden output planning, FEEL function invocation
   of a service from outside the BPMN business-rule path, full schema
   validation, and broader decision-service orchestration remain explicitly
   deferred

## 21.5 Follow-up After DMN Import Metadata Boundary-Proof Slice

The import-boundary proof confirmed that current package registries cannot
honestly resolve imported decision-service targets yet.

What changed:

1. top-level DMN `<import>` declarations are now preserved in the document
   snapshot with bounded `name`, `namespace`, `locationURI`, and `importType`
   metadata
2. `qianji lint --dmn` now exposes that import metadata through
   `document_root.imports`, so LLM repair flows can identify the external
   dependency before deciding whether to vendor it locally or keep the model
   non-executable
3. executable DMN package loading still rejects top-level imports; it does not
   silently treat imported hrefs as local hrefs
4. the missing prerequisite for imported decision-service execution is an
   explicit package-owned import-resolution registry that maps import
   metadata to parsed DMN sources and qualified references

## 22. Follow-up After DMN Business-Knowledge-Model Snapshot Metadata Slice

The parity note needs one more precision update after the latest bounded DMN
root-artifact metadata slice.

What changed:

1. the crate still does not resolve or execute top-level
   `businessKnowledgeModel` declarations directly, so there is still no
   claim of business-knowledge-model execution parity here
2. the non-executable DMN root snapshot surface now preserves bounded
   top-level `businessKnowledgeModel` metadata, which closes another real
   adapter/lint gap exposed by metadata-rich `SpiffWorkflow` DMN research
   fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows not just that a
   metadata-only DMN document exposes top-level business-knowledge models,
   but also which bounded business-knowledge-model metadata is present,
   which is materially safer than the earlier count-only wording
4. business-knowledge-model body execution, DRD execution, DMN dependency
   resolution, full schema validation, and broader FEEL semantics all remain
   explicitly deferred

## 22.1 Follow-up After DMN Business-Knowledge-Model Body Snapshot Evidence Slice

The parity note needs one more precision update after the latest bounded DMN
business-knowledge-model evidence slice.

What changed:

1. the crate still does not execute top-level `businessKnowledgeModel`
   declarations directly, so there is still no claim of
   business-knowledge-model execution parity here
2. the non-executable DMN root snapshot surface now preserves one direct BKM
   body `literalExpression` placeholder with expression id, optional typeRef,
   and direct text payload
3. `qianji lint --dmn` can now tell LLM-driven repair flows to preserve the
   body evidence under `document_root.business_knowledge_models` instead of
   inventing guessed decision-table rules
4. business-knowledge-model body evaluation, DRD execution, DMN dependency
   resolution, full schema validation, and broader FEEL semantics all remain
   explicitly deferred

## 23. Follow-up After DMN Business-Context Snapshot Metadata Slice

The parity note needs one more precision update after the latest bounded DMN
business-context metadata slice.

What changed:

1. the crate still does not resolve or execute top-level `organizationUnit`
   or `performanceIndicator` business-context elements, so there is still no
   claim of business-context execution parity here
2. the non-executable DMN root snapshot surface now preserves bounded
   top-level `organizationUnit` and `performanceIndicator` metadata, which
   closes another real adapter/lint gap exposed by metadata-rich
   `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows not just that a
   metadata-only DMN document exposes business-context elements, but also
   which bounded organization-unit and performance-indicator metadata is
   present, which is materially safer than the earlier count-only wording
4. business-context execution, threshold evaluation, organization
   hierarchies, DRD execution, DMN dependency resolution, full schema
   validation, and broader FEEL semantics all remain explicitly deferred

## 24. Follow-up After DMN Business Context Guidance Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not resolve or execute top-level `organizationUnit`
   or `performanceIndicator` business-context elements, so there is still no
   claim of business-context parity here
2. the non-executable DMN root snapshot surface can now classify those
   top-level governance counts explicitly, which closes another real
   adapter/lint gap exposed by metadata-rich `SpiffWorkflow` DMN research
   fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows whether a
   metadata-only DMN document is exposing top-level organization units or
   performance indicators, which is materially safer than the earlier
   generic missing-decision wording
4. business-context resolution, item-definition resolution, DRD execution,
   DMN dependency resolution, full schema validation, and broader FEEL
   semantics all remain explicitly deferred

## 25. Follow-up After DMN Text-Annotation Snapshot Metadata Slice

The parity note needs one more precision update after the latest bounded DMN
text-annotation metadata slice.

What changed:

1. the crate still does not resolve or execute top-level `textAnnotation`
   elements directly, so there is still no claim of text-annotation
   execution parity here
2. the non-executable DMN root snapshot surface now preserves bounded
   top-level `textAnnotation` metadata plus one direct nested text payload,
   which closes another real adapter/lint gap exposed by metadata-rich
   `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows not just that a
   metadata-only DMN document exposes text annotations, but also which
   bounded text-annotation metadata is present, which is materially safer
   than the earlier count-only wording
4. annotation execution, association resolution, broader XML text capture,
   DRD execution, DMN dependency resolution, full schema validation, and
   broader FEEL semantics all remain explicitly deferred

## 26. Follow-up After DMN Document-Structure Snapshot Metadata Slice

The parity note needs one more precision update after the latest bounded DMN
document-structure metadata slice.

What changed:

1. the crate still does not resolve or execute top-level `association`,
   `elementCollection`, or `group` artifacts directly, so there is still no
   claim of document-structure or artifact parity here
2. the non-executable DMN root snapshot surface now preserves bounded
   top-level `association` metadata with direct
   `associationDirection` / `sourceRef` / `targetRef` placeholders, plus
   bounded top-level `elementCollection` and `group` metadata, which closes
   another real adapter/lint gap exposed by metadata-rich
   `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows not just that a
   metadata-only DMN document exposes document-structure artifacts, but also
   which bounded `association`, `elementCollection`, and `group` metadata is
   present, which is materially safer than the earlier count-only wording
4. association resolution, element-collection membership parsing,
   group-to-DMNDI relationships, broader DRD execution, DMN dependency
   resolution, full schema validation, and broader FEEL semantics all
   remain explicitly deferred

## 27. Follow-up After DMN Text Annotation Guidance Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not resolve or execute top-level `textAnnotation`
   elements, so there is still no claim of annotation or artifact parity
   here
2. the non-executable DMN root snapshot surface can now classify top-level
   `textAnnotation` counts explicitly, which closes another real adapter/lint
   gap exposed by metadata-rich `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows whether a
   metadata-only DMN document is exposing top-level text annotations, which
   is materially safer than the earlier generic missing-decision wording
4. annotation resolution, `association`, `elementCollection`, DMNDI
   relationships, business-context resolution, item-definition resolution,
   DRD execution, DMN dependency resolution, full schema validation, and
   broader FEEL semantics all remain explicitly deferred

## 28. Follow-up After DMN Association and Element Collection Guidance Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not resolve or execute top-level `association` or
   `elementCollection` structures, so there is still no claim of
   document-structure or artifact-graph parity here
2. the non-executable DMN root snapshot surface can now classify those
   top-level document-structure counts explicitly, which closes another real
   adapter/lint gap exposed by the imported `SpiffWorkflow` DMN schema lane
3. `qianji lint --dmn` can now tell LLM-driven repair flows whether a
   metadata-only DMN document is exposing top-level associations or element
   collections, which is materially safer than the earlier generic
   missing-decision wording
4. association resolution, element-collection membership parsing, DMNDI
   relationships, annotation resolution, business-context resolution,
   item-definition resolution, DRD execution, DMN dependency resolution,
   full schema validation, and broader FEEL semantics all remain explicitly
   deferred

## 29. Follow-up After DMN DMNDI Snapshot Metadata Slice

The parity note needs one more precision update after the latest bounded DMN
DMNDI metadata slice.

What changed:

1. the crate still does not resolve or execute top-level `dmndi:DMNDI`
   structures directly, so there is still no claim of DMN
   diagram-interchange parity here
2. the non-executable DMN root snapshot surface now preserves bounded
   top-level `dmndi:DMNDI` metadata plus one direct `DMNDiagram`
   placeholder layer carrying diagram ids and direct shape/edge counts,
   which closes another real adapter/lint gap exposed by metadata-rich
   `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows not just that a
   metadata-only DMN document exposes DMNDI content, but also which bounded
   DMNDI placeholder fields are present, which is materially safer than the
   earlier count-only wording
4. `DMNShape` / `DMNEdge` metadata beyond direct counts, geometry
   interpretation, DMNDI relationship parsing, broader DRD execution, DMN
   dependency resolution, full schema validation, and broader FEEL
   semantics all remain explicitly deferred

## 30. Follow-up After DMN DMNDI Diagram Element Metadata Slice

The parity note needs one more precision update after the latest bounded DMN
diagram-element slice.

What changed:

1. the crate still does not resolve or execute top-level `dmndi:DMNDI`
   structures directly, so there is still no claim of DMN
   diagram-interchange parity here
2. the non-executable DMN root snapshot surface now preserves direct
   `DMNShape` and `DMNEdge` placeholder metadata under one direct
   `DMNDiagram`, bounded to optional `id` plus `dmnElementRef`, which
   closes another real adapter/lint gap exposed by metadata-rich
   `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows not just that a
   metadata-only DMN document exposes DMNDI content, but which direct
   shape and edge placeholders are actually present, which is materially
   safer than direct-count wording alone
4. bounds, waypoints, labels, DMNDI relationship parsing, broader DRD
   execution, DMN dependency resolution, full schema validation, and
   broader FEEL semantics all remain explicitly deferred

## 31. Follow-up After DMN Listed Input Data Shape Metadata Slice

The parity note needs one more precision update after the latest bounded DMN
direct-shape extension-attribute slice.

What changed:

1. the crate still does not resolve or execute top-level `dmndi:DMNDI`
   structures directly, so there is still no claim of DMN
   diagram-interchange parity here
2. the non-executable DMN root snapshot surface now preserves one optional
   direct `DMNShape.isListedInputData` boolean under the existing bounded
   `DMNDiagram` direct-shape placeholder contract, which closes another real
   adapter/lint gap exposed by metadata-rich `SpiffWorkflow` DMN research
   fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows when a direct
   listed-input-data shape marker is actually present, which is materially
   safer than exposing only id/ref shape placeholders
4. `DMNLabel`, bounds, waypoints, other richer DMNDI extension attributes,
   DMNDI relationship parsing, broader DRD execution, DMN dependency
   resolution, full schema validation, and broader FEEL semantics all
   remain explicitly deferred

## 32. Follow-up After DMN Label Placeholder Metadata Slice

The parity note needs one more precision update after the latest bounded DMN
direct-label placeholder slice.

What changed:

1. the crate still does not resolve or execute top-level `dmndi:DMNDI`
   structures directly, so there is still no claim of DMN
   diagram-interchange parity here
2. the non-executable DMN root snapshot surface now preserves one optional
   direct `DMNLabel` placeholder under bounded `DMNShape` and `DMNEdge`
   placeholders, which closes another real adapter/lint gap exposed by
   metadata-rich `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows when direct
   label placeholders are actually present on shapes or edges, which is
   materially safer than exposing only id/ref placeholders
4. `DMNLabel` text payloads, bounds, waypoints, `DMNDecisionServiceDividerLine`,
   DMNDI relationship parsing, broader DRD execution, DMN dependency
   resolution, full schema validation, and broader FEEL semantics all
   remain explicitly deferred

## 33. Follow-up After DMN Label Text Payload Slice

The parity note needs one more precision update after the latest bounded DMN
direct-label text slice.

What changed:

1. the crate still does not resolve or execute top-level `dmndi:DMNDI`
   structures directly, so there is still no claim of DMN
   diagram-interchange parity here
2. the non-executable DMN root snapshot surface now preserves one optional
   direct `DMNLabel/Text` payload under bounded `DMNShape` and `DMNEdge`
   placeholders, which closes another real adapter/lint gap exposed by
   metadata-rich `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows when direct
   label text is actually present on shapes or edges, which is materially
   safer than exposing only label placeholder ids
4. bounds, waypoints, `DMNDecisionServiceDividerLine`, DMNDI relationship
   parsing, broader DRD execution, DMN dependency resolution, full schema
   validation, and broader FEEL semantics all remain explicitly deferred

## 34. Follow-up After DMN Shape Bounds Placeholder Slice

The parity note needs one more precision update after the latest bounded DMN
direct-shape bounds slice.

What changed:

1. the crate still does not resolve or execute top-level `dmndi:DMNDI`
   structures directly, so there is still no claim of DMN
   diagram-interchange parity here
2. the non-executable DMN root snapshot surface now preserves one optional
   direct `dc:Bounds` placeholder under bounded `DMNShape` placeholders,
   which closes another real adapter/lint gap exposed by metadata-rich
   `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows when direct
   shape bounds are actually present on diagram shapes, which is materially
   safer than exposing only id/ref or listed-input markers
4. label bounds, edge waypoints, `DMNDecisionServiceDividerLine`, DMNDI
   relationship parsing, broader DRD execution, DMN dependency resolution,
   full schema validation, and broader FEEL semantics all remain explicitly
   deferred

## 35. Follow-up After DMN Label Bounds Placeholder Slice

The parity note needs one more precision update after the latest bounded DMN
direct-label bounds slice.

What changed:

1. the crate still does not resolve or execute top-level `dmndi:DMNDI`
   structures directly, so there is still no claim of DMN
   diagram-interchange parity here
2. the non-executable DMN root snapshot surface now preserves one optional
   direct `dc:Bounds` placeholder under bounded `DMNLabel` placeholders,
   which closes another real adapter/lint gap exposed by metadata-rich
   `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows when direct
   label bounds are actually present on shape or edge labels, which is
   materially safer than exposing only label ids or label text
4. edge waypoints, `DMNDecisionServiceDividerLine`, DMNDI relationship
   parsing, broader DRD execution, DMN dependency resolution, full schema
   validation, and broader FEEL semantics all remain explicitly deferred

## 36. Follow-up After DMN Edge Waypoint Placeholder Slice

The parity note needs one more precision update after the latest bounded DMN
direct-edge waypoint slice.

What changed:

1. the crate still does not resolve or execute top-level `dmndi:DMNDI`
   structures directly, so there is still no claim of DMN
   diagram-interchange parity here
2. the non-executable DMN root snapshot surface now preserves one repeated
   direct `di:waypoint` placeholder list under bounded `DMNEdge`
   placeholders, which closes another real adapter/lint gap exposed by
   metadata-rich `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows when direct
   edge waypoints are actually present on diagram edges, which is
   materially safer than exposing only edge ids, refs, or label metadata
4. `DMNDecisionServiceDividerLine`, DMNDI relationship parsing, broader
   DRD execution, DMN dependency resolution, full schema validation, and
   broader FEEL semantics all remain explicitly deferred

## 37. Follow-up After DMN Decision-Service Divider-Line Placeholder Slice

The parity note needs one more precision update after the latest bounded DMN
decision-service divider-line slice.

What changed:

1. the crate still does not resolve or execute top-level `dmndi:DMNDI`
   structures directly, so there is still no claim of DMN
   diagram-interchange parity here
2. the non-executable DMN root snapshot surface now preserves one optional
   direct `DMNDecisionServiceDividerLine` placeholder under bounded
   `DMNShape` placeholders, bounded to one repeated direct `di:waypoint`
   placeholder list with optional x/y pairs, which closes another real
   adapter/lint gap exposed by metadata-rich `SpiffWorkflow` decision-service
   fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows when direct
   divider-line waypoints are actually present on decision-service shapes,
   which is materially safer than exposing only shape ids, refs, bounds, or
   label metadata
4. DMNDI relationship parsing, broader DRD execution, DMN dependency
   resolution, full schema validation, and broader FEEL semantics all remain
   explicitly deferred

## 38. Follow-up After DMN Shape isCollapsed Placeholder Slice

The parity note needs one more precision update after the latest bounded DMN
direct-shape `isCollapsed` slice.

What changed:

1. the crate still does not resolve or execute top-level `dmndi:DMNDI`
   structures directly, so there is still no claim of DMN
   diagram-interchange parity here
2. the non-executable DMN root snapshot surface now preserves one optional
   direct `DMNShape.isCollapsed` boolean under bounded `DMNShape`
   placeholders, which closes another real adapter/lint gap exposed by
   metadata-rich `SpiffWorkflow` DMN fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows when direct
   collapsed-state metadata is actually present on shapes, which is
   materially safer than exposing only shape ids, refs, bounds, or
   divider-line metadata
4. `sharedStyle`, top-level `DMNStyle`, DMNDI relationship parsing,
   broader DRD execution, DMN dependency resolution, full schema
   validation, and broader FEEL semantics all remain explicitly deferred

## 39. Follow-up After DMN DMNDI Guidance Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not resolve or execute top-level `dmndi:DMNDI`
   structures, so there is still no claim of DMN diagram-interchange parity
   here
2. the non-executable DMN root snapshot surface can now classify top-level
   `dmndi:DMNDI` counts explicitly, which closes another real adapter/lint
   gap exposed by imported `SpiffWorkflow` DMN fixtures that carry diagram
   metadata alongside or apart from executable decisions
3. `qianji lint --dmn` can now tell LLM-driven repair flows whether a
   metadata-only DMN document is exposing top-level DMNDI metadata, which is
   materially safer than the earlier generic missing-decision wording
4. DMNDI relationship parsing, `DMNDiagram` / `DMNShape` / `DMNEdge`
   resolution, association resolution, element-collection membership
   parsing, annotation resolution, business-context resolution,
   item-definition resolution, DRD execution, DMN dependency resolution,
   full schema validation, and broader FEEL semantics all remain explicitly
   deferred

## 40. Follow-up After DMN Group Guidance Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not resolve or execute top-level `group`
   artifacts, so there is still no claim of group or broader artifact
   parity here
2. the non-executable DMN root snapshot surface can now classify top-level
   `group` counts explicitly, which closes another real adapter/lint gap
   exposed by the imported `SpiffWorkflow` DMN schema lane
3. `qianji lint --dmn` can now tell LLM-driven repair flows whether a
   metadata-only DMN document is exposing top-level groups, which is
   materially safer than the earlier generic missing-decision wording
4. group resolution, DMNDI relationships, association resolution,
   element-collection membership parsing, annotation resolution,
   business-context resolution, item-definition resolution, DRD execution,
   DMN dependency resolution, full schema validation, and broader FEEL
   semantics all remain explicitly deferred

## 41. Follow-up After DMN Allowed Answers Guidance Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not interpret decision-owned `allowedAnswers` as
   executable decision logic, so there is still no claim of decision-
   metadata or richer decision-table parity here
2. the non-executable DMN decision snapshot surface can now classify direct
   `allowedAnswers` counts explicitly, which closes another real adapter/lint
   gap exposed by the imported `SpiffWorkflow` DMN schema lane
3. `qianji lint --dmn` can now tell LLM-driven repair flows when a decision
   only exposes `allowedAnswers` metadata without any local executable
   `<decisionTable>`, which is materially safer than the earlier generic
   missing-decision-table wording
4. FEEL evaluation, output coercion, broader decision-table metadata
   support, decision-owner metadata, decision-service member resolution,
   DMN dependency resolution, full schema validation, and broader FEEL
   semantics all remain explicitly deferred

## 42. Follow-up After DMN Decision Governance Guidance Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not interpret decision-owned `decisionMaker` or
   `decisionOwner` metadata as executable decision logic, so there is still
   no claim of governance-metadata or richer decision-table parity here
2. the non-executable DMN decision snapshot surface can now classify direct
   `decisionMaker` and `decisionOwner` counts explicitly, which closes
   another real adapter/lint gap exposed by the imported `SpiffWorkflow`
   DMN schema lane
3. `qianji lint --dmn` can now tell LLM-driven repair flows when a decision
   only exposes maker-only or owner-only governance metadata without any
   local executable `<decisionTable>`, which is materially safer than the
   earlier generic missing-decision-table wording
4. identity resolution, mixed maker-plus-owner governance classification,
   FEEL evaluation, broader decision-table metadata support,
   decision-service member resolution, DMN dependency resolution, full
   schema validation, and broader FEEL semantics all remain explicitly
   deferred

## 43. Follow-up After DMN Mixed Decision Governance Guidance Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not interpret combined `decisionMaker` plus
   `decisionOwner` metadata as executable decision logic, so there is still
   no claim of governance-execution or richer decision-table parity here
2. the existing non-executable DMN decision snapshot surface can now be used
   to classify mixed maker-plus-owner governance counts explicitly, which
   closes another real adapter/lint gap exposed by the imported
   `SpiffWorkflow` DMN schema lane
3. `qianji lint --dmn` can now tell LLM-driven repair flows when one
   decision combines maker-plus-owner governance metadata without any local
   executable `<decisionTable>`, which is materially safer than the earlier
   generic missing-decision-table wording
4. identity resolution, FEEL evaluation, broader decision-table metadata
   support, decision-service member resolution, DMN dependency resolution,
   full schema validation, and broader FEEL semantics all remain explicitly
   deferred
