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
   host-dispatch shell, plus one interrupting timer `boundaryEvent` on one
   host-blocking task, plus one bounded embedded `subProcess` body with
   exactly one nested `startEvent`
   and at least one nested `endEvent`, plus one bounded `<transaction>` shell
   with exactly one nested `startEvent` and at least one nested `endEvent`,
   plus one bounded transaction cancel path composed of one interrupting
   cancel `boundaryEvent` attached to that transaction shell and one nested
   cancel end that restores the parent frame and rolls back
   transaction-local variable mutations, plus one bounded transaction owner
   that may expose one interrupting cancel `boundaryEvent` plus one or more
   interrupting error `boundaryEvent` nodes, where one nested error end may
   restore the parent frame while preserving transaction-local variable
   mutations and route through every matching parent error boundary including
   one catch-all boundary, while normal completion and cancel routing cancel
   the non-selected sibling boundaries, plus one bounded transaction cancel
   compensation subset where compensable activities may bind one explicit
   compensation handler and cancel routing replays those handlers in reverse
   completion order before the parent cancel boundary fires, plus one
   synchronous throw-compensation `endEvent` subset that either uses explicit
   `activityRef` targeting or omits `activityRef` for default reverse replay,
   plus one synchronous targeted throw-compensation
   `intermediateThrowEvent` subset with explicit `activityRef` inside that
   same transaction shell, plus one bounded same-package `callActivity`, plus bounded
   `standardLoopCharacteristics`, plus bounded sequential and bounded
   parallel `multiInstanceLoopCharacteristics` with integer
   `loopCardinality` on one host-blocking task family are supported, and those
   same multi-instance shapes may now also carry one bounded
   `completionCondition` using either one simple boolean variable path or one
   bounded counter comparison, plus one bounded collection-backed data-binding
   subset using `loopDataInputRef`, `inputDataItem`, optional
   `loopDataOutputRef`, and `outputDataItem`, while broader unstructured
   inclusive gateways, compensation event subprocesses, asynchronous or
   default throw-compensation intermediate forms, asynchronous
   throw-compensation end-event forms, more than one cancel boundary on one
   transaction owner, broader transaction error
   propagation beyond that bounded shell,
   non-interrupting boundaries, richer BPMN orchestration, broader
   multi-BPMN import/dependency handling, and broader FEEL or script-backed
   temporal behavior remain deferred.
2. `packages/rust/crates/qianji-bpmn-engine/src/parser/import.rs` now accepts
   `parallelGateway`, `exclusiveGateway`, `intermediateCatchEvent`, one
   bounded `receiveTask`/`sendTask` message-task family, one bounded timer
   `boundaryEvent` family, one bounded embedded `subProcess` body family, one
   bounded `<transaction>` shell family, one bounded
   `callActivity` family, bounded `standardLoopCharacteristics`, and bounded
   sequential or bounded
   parallel `multiInstanceLoopCharacteristics`, with message, signal, and
   timer event definitions inside the currently supported wait shapes and
   message-task validation surface, plus bounded cancel and error event
   definitions for one transaction owner that may expose one cancel boundary
   plus one or more error boundaries, plus one synchronous
   throw-compensation `endEvent` subset that either uses explicit
   `activityRef` targeting or omits `activityRef` for default reverse replay,
   plus one synchronous targeted throw-compensation
   `intermediateThrowEvent` subset with explicit `activityRef` inside that
   same transaction shell.
3. `packages/rust/crates/qianji-bpmn-engine/src/runtime/lifecycle.rs` now
   supports bounded multi-token routing for parallel split/join, bounded
   exclusive condition-driven routing using simple boolean-path or
   numeric-comparison outgoing `sequenceFlow` `conditionExpression` values
   plus one optional `default` flow, plus one bounded structured inclusive
   split/join subset with the same condition/default routing rules and one
   matching linear join fragment, plus deterministic wait registration for intermediate
   message/signal/timer catch events, one bounded `receiveTask` message wait
   shell, one bounded `sendTask` host-dispatch shell, one interrupting timer
   boundary path, and parent-frame enter/return semantics for one bounded
   same-package `callActivity` plus one bounded embedded `subProcess` body,
   plus one bounded transaction cancel path that restores the parent frame,
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
   activity in reverse completion order, plus one synchronous targeted
   throw-compensation `intermediateThrowEvent` path that replays one
   referenced compensable activity inside that same transaction shell, plus
   bounded standard-loop re-entry and skip semantics, plus bounded
   sequential and bounded parallel multi-instance
   re-entry, repeat-context propagation, zero-cardinality skip, one bounded
   early-completion path via `completionCondition`, plus bounded collection
   input and output bindings with checkpoint-safe per-iteration overlays and
   deterministic output aggregation, and interrupting-boundary cleanup on one
   host-blocking task family.
   `businessRuleTask` can now also execute locally when the package carries a
   matching engine-owned DMN decision definition, while still deferring
   broader unstructured inclusive semantics, non-interrupting boundary,
   asynchronous or default throw-compensation intermediate forms,
   asynchronous throw-compensation end-event forms, more than one cancel
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

| BPMN family | SpiffWorkflow evidence | Current engine parse status | Current engine runtime status | Severity | Recommended next slice |
| --- | --- | --- | --- | --- | --- |
| Core linear flow: `startEvent`, `endEvent`, `serviceTask`, `userTask`, `manualTask`, `businessRuleTask`, `sequenceFlow` | Covered by `SpiffWorkflow/spiff/parser/process.py`; also stated in ReadTheDocs tasks list | Supported | Supported for linear single-frontier routing; `businessRuleTask` can now execute locally when the parser-owned bundle snapshot or later callers register one matching engine-owned DMN definition, but adapter wiring remains incomplete | baseline | keep stable while widening richer shapes |
| Gateways: exclusive, inclusive, parallel, event-based | Declared in ReadTheDocs; serializer config includes `ExclusiveGateway`, `InclusiveGateway`, `ParallelGateway`, `EventBasedGateway`; tests cover gateway families | `parallelGateway`, `exclusiveGateway`, one structured `inclusiveGateway` subset, and one bounded exclusive `eventBasedGateway` shape are supported | bounded `parallelGateway` split/join, bounded `exclusiveGateway` routing with simple boolean-path or numeric-comparison branch conditions plus one optional `default` flow, one structured `inclusiveGateway` split/join subset with the same condition/default rules plus one matching linear join fragment, and one bounded event-based winner-takes-all wait race are supported; broader unstructured inclusive joins and broader FEEL/script-backed gateway conditions remain unsupported | P1 | broader inclusive-gateway reachability and broader gateway-condition semantics |
| Intermediate, boundary, timer, message, signal, escalation, error, cancel events | Parser registrations and serializer config include boundary and intermediate events; tests cover timer, message, boundary, escalation, cancel, event-based gateways | Bounded `intermediateCatchEvent` support is now present for `messageEventDefinition`, `signalEventDefinition`, and `timerEventDefinition`; one interrupting timer `boundaryEvent` attached to one host-blocking task is also supported; one bounded transaction owner may now expose one cancel boundary plus one or more error boundaries; one bounded transaction cancel compensation subset with one explicit compensation-handler marker is also supported; and one synchronous throw-compensation subset is now supported for nested `endEvent` and `intermediateThrowEvent` shapes inside that same transaction shell, with targeted or default replay allowed on `endEvent` and explicit `activityRef` still required on `intermediateThrowEvent` | Intermediate message/signal/timer waits now register and resume through the engine-owned wait shell; one interrupting timer boundary path can cancel blocked host work; one bounded `eventBasedGateway` can race those waits and cancel the losing siblings; one bounded transaction cancel path can restore the parent frame, roll back transaction-local variable mutations, and route through the parent cancel boundary; one bounded transaction cancel compensation path can replay explicit compensation handlers in reverse completion order before that parent cancel boundary fires; one bounded transaction error path can restore the parent frame, preserve transaction-local variable mutations, route through every matching parent error boundary including one catch-all boundary, and cancel non-selected sibling boundaries; and one synchronous throw-compensation path can now replay the referenced compensable activity either before transaction-shell completion from a nested `endEvent` or before normal downstream sequence-flow routing resumes from a nested `intermediateThrowEvent`, with default end-event replay draining every already compensable activity in reverse completion order | P1 | broader event families plus default intermediate or asynchronous throw-compensation forms |
| Script, send, and receive tasks | Registered in `SpiffWorkflow/spiff/parser/process.py`; tests cover script and event-driven workflows | Bounded `sendTask` and `receiveTask` are now supported when they carry exactly one message binding through task-level `messageRef` or one nested `messageEventDefinition`; `scriptTask` remains unsupported at parse time | `receiveTask` now reuses the engine-owned message-wait shell, `sendTask` now reuses the host-dispatch shell with preserved message metadata, `xiuxian-qianji` now wires that `sendTask` host work through the callback bridge and `qianji bpmn run --host-fixture` `send_tasks.<node_id>` contract, and `scriptTask` remains unsupported | P1 | `scriptTask`, correlations, and broader collaboration-aware message routing |
| Subprocess, call activity, transaction subprocess | Registered in parser and serializer config; tests cover call activity and nested processes | One bounded embedded `subProcess` body with exactly one nested `startEvent` and at least one nested `endEvent`, one bounded `<transaction>` shell with exactly one nested `startEvent` and at least one nested `endEvent`, one bounded transaction owner with one cancel boundary plus one or more error boundaries, one bounded transaction cancel compensation subset with one explicit compensation-handler marker per compensable activity, one synchronous throw-compensation subset with targeted or default replay on nested `endEvent` plus explicit `activityRef` targeting on nested `intermediateThrowEvent`, and one bounded `callActivity` that targets another executable process in the same BPMN package are supported; compensation event subprocesses and broader transaction error propagation remain unsupported | The runtime can enter any of those bounded nested shapes through the same child-process frame model, suspend there, restore the parent frame on child completion, execute one bounded transaction cancel path with rollback through the parent boundary, execute one bounded transaction cancel compensation path that replays explicit compensation handlers in reverse completion order before the parent cancel boundary, execute one bounded transaction error path without rollback through every matching parent error boundary, cancel non-selected sibling boundaries on the same transaction owner, and execute one synchronous throw-compensation path that replays the referenced compensable activity either before normal completion from a nested `endEvent` or before normal downstream sequence-flow routing resumes from a nested `intermediateThrowEvent`, with default end-event replay draining every already compensable activity in reverse completion order, but recursive call graphs, default or asynchronous intermediate throw compensation, compensation event subprocesses, and broader nested subprocess families remain unsupported | P1 | default or asynchronous intermediate throw-compensation forms and broader transaction error propagation |
| Standard loop and multi-instance tasks | ReadTheDocs lists loop, parallel multi-instance, sequential multi-instance; tests cover both loop and multi-instance | Bounded `standardLoopCharacteristics`, bounded sequential `multiInstanceLoopCharacteristics isSequential="true"`, and bounded parallel `multiInstanceLoopCharacteristics` with omitted or `isSequential="false"` plus integer `loopCardinality` are now supported on one service/user/manual/business-rule task family; one bounded `completionCondition` subset is now also supported on those same multi-instance shapes, and one bounded collection-backed data-binding subset using `loopDataInputRef`, `inputDataItem`, optional `loopDataOutputRef`, and `outputDataItem` is now supported | Standard loop now supports `testBefore` skip, loop-maximum re-entry, and simple boolean conditions such as `done` or `not done`; sequential multi-instance now supports checkpoint-safe sequential re-entry, repeat-context propagation, zero-cardinality skip, bounded `completionCondition` early-stop, interrupting-boundary cleanup, and collection-backed iteration overlays/output aggregation; bounded parallel multi-instance now supports single-writer token fan-out, per-iteration repeat-context propagation, zero-cardinality skip, bounded `completionCondition` sibling cancellation, interrupting-boundary cleanup, and collection-backed iteration overlays/output aggregation | P1 | transaction subprocess and richer nested orchestration |
| Collaboration, pools, lanes, messages, correlations | README mentions pools and lanes; tests cover collaboration, correlations, and swimlanes | Unsupported document family | Unsupported | P1 | collaboration and lane metadata slice after core execution parity |
| Data object, data store, IO specification | ReadTheDocs lists data object and data store; tests cover data object, data store reference, and IO spec | Unsupported document family | Unsupported workflow-data binding model | P1 | data binding slice after core control flow |
| Schema validation and broader source-bundle handling | Spiff has BPMN and DMN validators and dependency discovery; tests cover invalid workflows and dependency detection | `parse_bpmn_package(...)` still rejects schema validation and multi-BPMN bundles, but one bounded `BpmnBundleSnapshot` with exactly one BPMN source plus optional DMN sources is now supported | not applicable | P2 | parser completeness and import/dependency slice |

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

1. broader transaction error propagation, default compensation beyond the
   landed synchronous throw-compensation end-event subset,
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

| DMN capability | Current engine status | SpiffWorkflow status | Nuance | Severity |
| --- | --- | --- | --- | --- |
| One DMN source with one decision | Supported | Supported | Both implementations handle the simple case | baseline |
| Multiple decisions per source | Supported within the bounded parser subset | Broader file/version handling exists, but the parser remains bounded in structure | Current engine now materializes multiple decision definitions from one source while both implementations still remain narrower than full DMN schema/version breadth | baseline |
| One decision with one decision table | Supported | Supported | This is the strongest shared surface | baseline |
| Multiple decision tables inside one decision | Unsupported | DMN parser comment explicitly says it assumes one decision table within a decision | Not a parity gap worth prioritizing because upstream is also bounded here | P3 |
| Hit policies `UNIQUE` and `COLLECT` | Supported | Supported | This is the strongest overlapping execution surface | baseline |
| Other hit policies such as `FIRST`, `PRIORITY`, `ANY`, `RULE ORDER`, `OUTPUT ORDER` | Unsupported | Not implemented in the active `HitPolicy` enum even though schema files mention them | Do not overstate upstream support | P3 |
| Literal equality with strings, numbers, booleans, `null`, wildcard `-` | Supported | Supported indirectly through script evaluation | Current engine is stricter and easier to reason about | baseline |
| FEEL-like expressions, comparison operators, range syntax, and script-backed predicates | Partially supported: numeric unary comparisons, bounded numeric ranges, ISO date literals, ISO date comparisons, bounded ISO date ranges, ISO local datetime literals, ISO local datetime comparisons, bounded ISO local datetime ranges, RFC3339 offset-aware datetime literals, comparisons, and bounded ranges, signed ISO 8601 day-time duration literals, comparisons, and bounded ranges, signed ISO 8601 year-month duration literals, comparisons, and bounded ranges, plus ISO time literals, ISO time comparisons, and bounded ISO time ranges are now supported, but broader FEEL and script-backed predicates remain unsupported | Supported through the script engine in `DMNEngine.evaluate(...)`; tests cover ranges, comparisons, and dates | The gap narrowed further, but upstream still proves materially broader evaluator semantics | P1 |
| Date decisions and richer temporal predicates | Partially supported: ISO date-only equality, comparisons, and bounded ranges plus ISO local datetime equality, comparisons, and bounded ranges plus RFC3339 offset-aware datetime equality, comparisons, and bounded ranges plus one bounded mixed local-vs-offset UTC normalization rule that now also covers datetime literal equality in addition to comparisons and ranges, plus signed ISO 8601 day-time duration equality, comparisons, and bounded ranges including bounded fractional day-time forms such as `duration("P1.5D")`, `duration("P1,5D")`, `duration("PT1.5H")`, `duration("PT1,5H")`, `duration("PT1.5M")`, `duration("PT1,5M")`, `duration("PT1.5S")`, and `duration("PT1,5S")`, plus signed ISO 8601 year-month duration equality, comparisons, and bounded ranges, plus ISO time-only equality, comparisons, and bounded ranges are now supported | Covered by python-engine tests with broader datetime semantics | Local support is still intentionally narrower than upstream and currently excludes trailing-lower-unit fractional forms such as `duration("PT1.5H30S")`, fractional year-month duration literals, mixed year-month/day-time duration forms, and script-backed temporal functions | P1 |
| DMN schema/version parsing | Partially supported: one non-executable document snapshot can now scan namespaced/versioned DMN roots plus decision headers, model-version hints, top-level `import`, `itemDefinition`, `inputData`, `knowledgeSource`, `businessKnowledgeModel`, `decisionService`, `organizationUnit`, `performanceIndicator`, `textAnnotation`, `association`, `elementCollection`, `dmndi:DMNDI`, and `group` counts, decision-owned `allowedAnswers`, `decisionMaker`, and `decisionOwner` counts, direct decision-owned requirement counts for `informationRequirement`, `knowledgeRequirement`, and `authorityRequirement`, nested requirement-target counts for `requiredInput`, `requiredDecision`, `requiredKnowledge`, and `requiredAuthority`, and direct decision-logic counts for `literalExpression`, `context`, `invocation`, `relation`, `functionDefinition`, and `list`; executable parsing now also rejects non-`definitions` roots, missing or unsupported DMN model namespaces, and top-level `<import>` declarations, but it still performs no full schema validation and still requires the bounded decision-table subset | DMN 1.0, 1.2, and 1.3 schema/version handling exists in `BpmnDmnParser` and version tests | The crate now has one real placeholder surface for later adapter work and for construct-aware lint diagnostics across the main unsupported type-model, business-context, annotation, document-structure, diagram-interchange, group-artifact, decision-metadata, governance-metadata, decision-logic, metadata-only DRD artifact, and decision-dependency shapes seen in the current research lane, while the latest business-context, item-definition, text-annotation, association/element-collection, DMNDI, group, allowed-answers, decision-governance, root-artifact, requirement-target, requirement-edge, document-root, and import validation cuts close real parser/lint safety gaps before dependency resolution or broader schema parity, but full parser/schema parity is still materially incomplete | P2 |
| BPMN `businessRuleTask` to DMN execution integration | Partially supported: engine-owned package registries can execute locally, parser-owned bundle snapshots can now populate those registries, and `xiuxian-qianji` now owns a bounded host adapter for missing-definition fallback | Integrated parser-to-engine binding exists | The unconditional host-only gap is closed, parser-owned registration now exists, and the host adapter now exists in bounded form, but full BPMN scheduler/CLI orchestration is still missing | P1 |

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
   host-dispatch shell, one interrupting timer boundary path, one bounded
   embedded `subProcess` body, one bounded `<transaction>` shell, one bounded
   transaction cancel path, one bounded same-package `callActivity`, one bounded
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

## 11. Follow-up After DMN Context and Invocation Classification Slice

The parity note needs one more precision update after the latest bounded DMN
placeholder slice.

What changed:

1. the crate still does not execute direct `context` or direct `invocation`
   decisions, so there is still no claim of broader executable parity here
2. the non-executable DMN snapshot surface can now classify those constructs
   explicitly, which closes another real adapter/lint gap against the broader
   versioned DMN documents present in the imported `SpiffWorkflow` research
   lane
3. `qianji lint --dmn` can now tell LLM-driven repair flows not to flatten
   context entries or fabricate invocation rewrites into guessed
   decision-table logic, which is materially safer than the earlier generic
   missing-table guidance
4. executable schema validation, broader FEEL semantics, `context`
   execution, and `invocation` execution all remain explicitly deferred

## 12. Follow-up After DMN Relation Classification Slice

The parity note needs one more precision update after the latest bounded DMN
placeholder slice.

What changed:

1. the crate still does not execute direct `relation` decisions, so there is
   still no claim of broader executable parity here
2. the non-executable DMN snapshot surface can now classify that construct
   explicitly, which closes another real adapter/lint gap against the
   broader versioned DMN documents present in the imported `SpiffWorkflow`
   research lane
3. `qianji lint --dmn` can now tell LLM-driven repair flows not to flatten
   direct `relation` rows into guessed decision-table logic, which is
   materially safer than the earlier generic missing-table guidance
4. executable schema validation, broader FEEL semantics, and `relation`
   execution all remain explicitly deferred

## 13. Follow-up After DMN Function and List Classification Slice

The parity note needs one more precision update after the latest bounded DMN
placeholder slice.

What changed:

1. the crate still does not execute direct `functionDefinition` or direct
   `list` decisions, so there is still no claim of broader executable parity
   here
2. the non-executable DMN snapshot surface can now classify those constructs
   explicitly, which closes the remaining direct-expression adapter/lint gap
   exposed by the official DMN 1.3 schema after the earlier
   `literalExpression`, `context`, `invocation`, and `relation` cuts
3. `qianji lint --dmn` can now tell LLM-driven repair flows not to inline
   function bodies or flatten direct list items into guessed decision-table
   logic, which is materially safer than the earlier generic missing-table
   guidance
4. executable schema validation, broader FEEL semantics,
   `functionDefinition` execution, and `list` execution all remain
   explicitly deferred

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
   counts explicitly, and the executable parser now rejects top-level
   `<import>` declarations before decision-table execution begins, which
   closes one real document-dependency safety gap exposed by imported
   `SpiffWorkflow` DMN research fixtures
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
   top-level artifact counts explicitly, which closes another real
   adapter/lint gap exposed by metadata-only `SpiffWorkflow` DMN research
   fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows whether a
   metadata-only DMN document is exposing top-level input data, knowledge
   sources, or business knowledge models, which is materially safer than the
   earlier generic missing-decision wording
4. DRD execution, DMN dependency resolution, full schema validation, broader
   FEEL semantics, and item-definition execution all remain explicitly
   deferred

## 19. Follow-up After DMN Item Definition Guidance Slice

The parity note needs one more precision update after the latest bounded DMN
lint-precision slice.

What changed:

1. the crate still does not resolve or execute top-level `itemDefinition`
   declarations, so there is still no claim of DMN type-resolution parity
   here
2. the non-executable DMN root snapshot surface can now classify top-level
   `itemDefinition` counts explicitly, which closes another real adapter/lint
   gap exposed by metadata-rich `SpiffWorkflow` DMN research fixtures
3. `qianji lint --dmn` can now tell LLM-driven repair flows whether a
   metadata-only DMN document is exposing top-level item definitions, which
   is materially safer than the earlier generic missing-decision wording
4. item-definition resolution, DRD execution, DMN dependency resolution, full
   schema validation, and broader FEEL semantics all remain explicitly
   deferred

## 20. Follow-up After DMN Business Context Guidance Slice

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

## 21. Follow-up After DMN Text Annotation Guidance Slice

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

## 22. Follow-up After DMN Association and Element Collection Guidance Slice

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

## 23. Follow-up After DMN DMNDI Guidance Slice

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

## 24. Follow-up After DMN Group Guidance Slice

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

## 25. Follow-up After DMN Allowed Answers Guidance Slice

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

## 26. Follow-up After DMN Decision Governance Guidance Slice

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

## 27. Follow-up After DMN Mixed Decision Governance Guidance Slice

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
