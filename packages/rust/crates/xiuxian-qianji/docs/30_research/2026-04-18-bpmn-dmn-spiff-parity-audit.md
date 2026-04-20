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
   timer, boundary, and nested call-activity slices have now landed.
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
   `exclusiveGateway` pass-through routing plus bounded
   `intermediateCatchEvent` waits backed by `messageEventDefinition`,
   `signalEventDefinition`, and snapshot-style `timerEventDefinition`, plus
   one interrupting timer `boundaryEvent` on one host-blocking task, plus one
   bounded same-package `callActivity`, plus bounded
   `standardLoopCharacteristics`, plus bounded sequential
   `multiInstanceLoopCharacteristics isSequential="true"` with integer
   `loopCardinality` on one host-blocking task family are supported, while
   inclusive gateways, embedded `subProcess` bodies, non-interrupting
   boundaries, parallel multi-instance expansion, richer BPMN orchestration,
   broader multi-BPMN import/dependency handling, and full FEEL coverage
   remain deferred.
2. `packages/rust/crates/qianji-bpmn-engine/src/parser/import.rs` now accepts
   `parallelGateway`, `exclusiveGateway`, `intermediateCatchEvent`, one
   bounded timer `boundaryEvent` family, one bounded `callActivity` family,
   bounded `standardLoopCharacteristics`, and bounded sequential
   `multiInstanceLoopCharacteristics`, with message, signal, and timer event
   definitions inside the currently supported wait shapes.
3. `packages/rust/crates/qianji-bpmn-engine/src/runtime/lifecycle.rs` now
   supports bounded multi-token routing for parallel split/join and bounded
   exclusive single-route pass-through, plus deterministic wait registration
   for intermediate message/signal/timer catch events, one interrupting timer
   boundary path, and parent-frame enter/return semantics for one bounded
   same-package `callActivity`, plus bounded standard-loop re-entry and skip
   semantics, plus bounded sequential multi-instance re-entry, repeat-context
   propagation, zero-cardinality skip, and interrupting-boundary cleanup on
   one host-blocking task family. `businessRuleTask` can now also execute
   locally when the package carries a matching engine-owned DMN decision
   definition, while still deferring inclusive, parallel or data-bound
   multi-instance, non-interrupting boundary, embedded `subProcess`, broader
   multi-BPMN import/dependency handling, and broader condition-driven semantics.
4. `packages/rust/crates/qianji-bpmn-engine/src/parser/package.rs` now exposes
   one bounded parser-owned `BpmnBundleSnapshot` contract plus
   `parse_bpmn_bundle(...)`, allowing one BPMN source plus optional DMN
   sources to populate the package registry deterministically.
5. `packages/rust/crates/qianji-bpmn-engine/src/dmn/parse.rs` accepts one
   decision and one decision table only.
6. `packages/rust/crates/qianji-bpmn-engine/src/dmn/evaluate.rs` supports
   `UNIQUE` and `COLLECT` only, with wildcard matching, literal equality,
   numeric unary comparisons, and bounded numeric ranges.
7. `packages/rust/crates/qianji-bpmn-engine/src/lint/bpmn.rs` and
   `packages/rust/crates/qianji-bpmn-engine/src/lint/dmn.rs` explicitly guide
   callers toward this bounded subset.

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
| Gateways: exclusive, inclusive, parallel, event-based | Declared in ReadTheDocs; serializer config includes `ExclusiveGateway`, `InclusiveGateway`, `ParallelGateway`, `EventBasedGateway`; tests cover gateway families | `parallelGateway`, `exclusiveGateway`, and one bounded exclusive `eventBasedGateway` shape are supported; inclusive gateways remain unsupported | bounded `parallelGateway` split/join, deterministic single-route `exclusiveGateway` pass-through, and one bounded event-based winner-takes-all wait race are supported; inclusive gateways and condition-driven exclusive branching remain unsupported | P1 | loop and multi-instance runtime slice |
| Intermediate, boundary, timer, message, signal, escalation, error, cancel events | Parser registrations and serializer config include boundary and intermediate events; tests cover timer, message, boundary, escalation, cancel, event-based gateways | Bounded `intermediateCatchEvent` support is now present for `messageEventDefinition`, `signalEventDefinition`, and `timerEventDefinition`; one interrupting timer `boundaryEvent` attached to one host-blocking task is also supported | Intermediate message/signal/timer waits now register and resume through the engine-owned wait shell; one interrupting timer boundary path can cancel blocked host work; and one bounded `eventBasedGateway` can race those waits and cancel the losing siblings | P1 | loop and multi-instance runtime slice |
| Script, send, and receive tasks | Registered in `SpiffWorkflow/spiff/parser/process.py`; tests cover script and event-driven workflows | Unsupported element at parse time | Unsupported | P1 | post-gateway parser/runtime slice |
| Subprocess, call activity, transaction subprocess | Registered in parser and serializer config; tests cover call activity and nested processes | One bounded `callActivity` that targets another executable process in the same BPMN package is supported; embedded `subProcess` bodies and transaction subprocesses remain unsupported | The runtime can enter the child process, suspend there, and restore the parent frame on child completion, but it still rejects embedded `subProcess` bodies and recursive call graphs | P1 | embedded subprocess and richer nested orchestration |
| Standard loop and multi-instance tasks | ReadTheDocs lists loop, parallel multi-instance, sequential multi-instance; tests cover both loop and multi-instance | Bounded `standardLoopCharacteristics` and bounded sequential `multiInstanceLoopCharacteristics isSequential="true"` with integer `loopCardinality` are now supported on one service/user/manual/business-rule task family; parallel and richer multi-instance forms remain unsupported | Standard loop now supports `testBefore` skip, loop-maximum re-entry, and simple boolean conditions such as `done` or `not done`; sequential multi-instance now supports checkpoint-safe sequential re-entry, repeat-context propagation, zero-cardinality skip, and interrupting-boundary cleanup; parallel/data-bound multi-instance expansion and aggregation remain unsupported | P0 | richer multi-instance parity slice |
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

1. embedded subprocess bodies and richer nested orchestration
2. multi-instance expansion and richer repeatable-task aggregation
3. broader event families beyond the bounded message/signal/timer race shape
4. lane-aware or collaboration-aware execution surfaces
5. condition-driven gateway branching beyond the bounded deterministic subset

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
| Multiple decisions per source | Unsupported | Broader file/version handling exists, but the parser remains bounded in structure | Current engine fails by contract; Spiff does not justify claiming general multi-decision parity either | P1 |
| One decision with one decision table | Supported | Supported | This is the strongest shared surface | baseline |
| Multiple decision tables inside one decision | Unsupported | DMN parser comment explicitly says it assumes one decision table within a decision | Not a parity gap worth prioritizing because upstream is also bounded here | P3 |
| Hit policies `UNIQUE` and `COLLECT` | Supported | Supported | This is the strongest overlapping execution surface | baseline |
| Other hit policies such as `FIRST`, `PRIORITY`, `ANY`, `RULE ORDER`, `OUTPUT ORDER` | Unsupported | Not implemented in the active `HitPolicy` enum even though schema files mention them | Do not overstate upstream support | P3 |
| Literal equality with strings, numbers, booleans, `null`, wildcard `-` | Supported | Supported indirectly through script evaluation | Current engine is stricter and easier to reason about | baseline |
| FEEL-like expressions, comparison operators, range syntax, and script-backed predicates | Partially supported: numeric unary comparisons, bounded numeric ranges, ISO date literals, ISO date comparisons, and bounded ISO date ranges are now supported, but broader FEEL and script-backed predicates remain unsupported | Supported through the script engine in `DMNEngine.evaluate(...)`; tests cover ranges, comparisons, and dates | The gap narrowed further, but upstream still proves materially broader evaluator semantics | P1 |
| Date decisions and richer temporal predicates | Partially supported: ISO date-only equality, comparisons, and bounded ranges are now supported | Covered by python-engine tests with broader datetime semantics | Local support is now date-only and intentionally excludes `time`, `date and time`, durations, and script-backed temporal functions | P1 |
| DMN schema/version parsing | Unsupported as a crate feature | DMN 1.0, 1.2, and 1.3 schema/version handling exists in `BpmnDmnParser` and version tests | Useful for parser completeness, but lower priority than evaluator breadth | P2 |
| BPMN `businessRuleTask` to DMN execution integration | Partially supported: engine-owned package registries can execute locally, parser-owned bundle snapshots can now populate those registries, and `xiuxian-qianji` now owns a bounded host adapter for missing-definition fallback | Integrated parser-to-engine binding exists | The unconditional host-only gap is closed, parser-owned registration now exists, and the host adapter now exists in bounded form, but full BPMN scheduler/CLI orchestration is still missing | P1 |

## 6. DMN Audit Interpretation

The DMN result needs one important guardrail:

`SpiffWorkflow` is broader than the current engine, but it is not evidence of
full DMN standard coverage.

The audit-proven statement is narrower:

1. `qianji-bpmn-engine` currently implements a deliberately bounded DMN
   contract.
2. `SpiffWorkflow` implements a materially richer DMN parse and evaluation
   surface than that bounded contract, even after the local numeric and
   ISO-date comparison/range widening cuts.
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
   `SpiffWorkflow`-parity validator.
2. `qianji lint --dmn` is currently a bounded decision-table validator, not a
   FEEL-capable DMN validator.
3. `xiuxian-qianji` adapter work should not advertise richer BPMN or DMN
   support until the engine slices land first.

For the later `xiuxian-qianji` adapter lane, the audit implies that parser
parity and runtime parity must move together enough that the CLI does not tell
LLM tooling to "repair" files into shapes the runtime still cannot execute.

## 8. Recommended Post-Audit Slice Order

The recommended order is driven by execution semantics rather than by document
surface breadth.

1. Multi-instance slice
   This slice is now partially landed through bounded sequential-cardinality
   ownership, but richer parallel and data-bound multi-instance parity remains
   open.
2. Frontier concurrency semantics slice
   This slice should align the runtime with OMG BPMN expectations for multiple
   simultaneously runnable nodes under one workflow-instance owner, instead of
   keeping first-token planning and singleton pending host work as an implicit
   limitation.
3. Broader DMN evaluator widening slice
   Bounded numeric and ISO-date comparison/range cuts are now landed;
   remaining work should widen toward richer temporal and other bounded
   FEEL-compatible predicates without claiming full FEEL.
4. `xiuxian-qianji` higher-level BPMN orchestration slice
   The bounded host adapter is now landed, so the next host-side gap is
   scheduler/CLI/runtime ownership of BPMN package loading and execution.
5. Embedded subprocess and richer nested orchestration slice
   This should widen beyond the landed bounded `callActivity` ownership model.
6. Collaboration, lane, and data-binding slice
   This should follow after the core execution model is stable.
7. Schema/version/import-completeness slice
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
   interrupting timer boundary path, one bounded same-package `callActivity`,
   one bounded `standardLoopCharacteristics` shape, and one bounded
   sequential-cardinality `multiInstanceLoopCharacteristics` shape, plus one
   engine-owned local `businessRuleTask` path when the package already carries
   a matching DMN definition, plus one parser-owned bundle snapshot path that
   can register bounded DMN definitions into that package, but still covers
   only a small part of `SpiffWorkflow`.
2. `qianji-bpmn-engine` currently matches only the bounded core of
   `SpiffWorkflow` DMN support: one decision table with `UNIQUE` or `COLLECT`,
   plus wildcard/literal matching, numeric unary comparisons, bounded numeric
   ranges, ISO date literals, ISO date comparisons, and bounded ISO date
   ranges.
3. The next useful implementation target is not "all missing tags". It is the
   runtime semantics needed to unlock multi-instance expansion, richer nested
   orchestration, and richer DMN behavior in an order that preserves
   checkpoint and host-bridge integrity.
