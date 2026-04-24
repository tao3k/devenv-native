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
- [Gateways and Concurrency](gateways-and-concurrency.md)
- [Loops and Multi-Instance](loops-and-multi-instance.md)
- [Subprocesses, Transactions, and Compensation](subprocesses-transactions-and-compensation.md)
- [Tasks and Host Dispatch](tasks-and-host-dispatch.md)

## Current Package Boundary

The current package owns bounded support for:

- linear flows and bounded gateway routing
- bounded waits and boundary events
- bounded loop and multi-instance task execution
- bounded host-dispatched task families including `sendTask` and `scriptTask`
- bounded subprocess, transaction, and same-package call-activity slices
- bounded transaction-owned compensation slices
- non-executable BPMN document snapshots for collaboration, lane, data-object,
  data-store, IO-specification, and data-association metadata

The current package still defers:

- collaboration and lane semantics
- full BPMN data-object and IO-specification coverage
- unbounded event families and event subprocesses
- broader FEEL or script-backed flow semantics

Deferred collaboration, lane, data-object, data-store, and IO-specification
surfaces are reported by the linter with explicit repair guidance instead of
being treated as executable runtime semantics.
Those lint reports also include bounded snapshot-derived evidence for the
deferred family, such as participant/message-flow counts, lane flow-node refs,
or data-object and data-association references.
