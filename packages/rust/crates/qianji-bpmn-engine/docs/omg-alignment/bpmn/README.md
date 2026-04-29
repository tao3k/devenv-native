# BPMN Alignment Index

This module tracks how `qianji-bpmn-engine` aligns with the official
[BPMN 2.0.2 specification](https://www.omg.org/spec/BPMN/2.0.2).

The current engine is intentionally bounded. Alignment means:

- the parser accepts one explicit BPMN shape
- the runtime executes that same shape deterministically
- the lint surface explains unsupported shapes in an LLM-friendly way

The source-backed clause registry for these notes lives in
[BPMN Official Source Map](spec-source-map.md).

## Current Modules

- [Official Source Map](spec-source-map.md)
- [Collaboration, Lanes, and Data](collaboration-lanes-and-data.md)
- [Events and Boundaries](events-and-boundaries.md)
- [Full Conformance Coverage](full-conformance-coverage.md)
- [Gateways and Concurrency](gateways-and-concurrency.md)
- [Human Interaction Spiff/OMG Audit](human-interaction-spiff-omg-audit.md)
- [Human Interaction Milestone Plan](human-interaction-milestone-plan.md)
- [Host Request ABI Ledger](host-request-abi-ledger.md)
- [Loops and Multi-Instance](loops-and-multi-instance.md)
- [Subprocesses, Transactions, and Compensation](subprocesses-transactions-and-compensation.md)
- [Tasks and Host Dispatch](tasks-and-host-dispatch.md)

## Current Package Boundary

The current package owns bounded support for:

- linear flows and bounded gateway routing
- bounded start/intermediate waits and boundary events
- bounded loop and multi-instance task execution
- bounded host-dispatched task families including `sendTask` and `scriptTask`
- bounded subprocess, transaction, and same-package call-activity slices
- bounded transaction-owned compensation slices
- bounded task-local Data/IO through native `ioSpecification`,
  `dataInputAssociation`, and `dataOutputAssociation` mappings
- non-executable BPMN document snapshots for collaboration, choreography,
  artifact, lane, data-object, data-store, import, extension, relationship,
  BPMN DI, conversation, catalog, and category metadata

The current package still defers:

- collaboration and lane semantics
- full BPMN data-object/data-store execution and broader IO execution coverage
- unbounded event families and event subprocesses
- broader FEEL or script-backed flow semantics

Deferred collaboration, choreography, artifact, lane, data-object, data-store,
import, extension, relationship, BPMN DI, category, and unsupported IO
surfaces are reported by the linter with explicit repair guidance instead of
being treated as executable runtime semantics. Those lint reports also include
bounded snapshot-derived evidence for the deferred family, such as
participant/message-flow counts, conversation node/link/association counts,
choreography activity counts, artifact association/group/text-annotation
counts, lane flow-node refs, data-object and
data-association references, or diagram element counts.
