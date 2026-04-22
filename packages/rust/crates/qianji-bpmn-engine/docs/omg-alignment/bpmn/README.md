# BPMN Alignment Index

This module tracks how `qianji-bpmn-engine` aligns with the official
[BPMN 2.0.2 specification](https://www.omg.org/spec/BPMN/2.0.2).

The current engine is intentionally bounded. Alignment means:

- the parser accepts one explicit BPMN shape
- the runtime executes that same shape deterministically
- the lint surface explains unsupported shapes in an LLM-friendly way

## Current Modules

- [Events and Boundaries](events-and-boundaries.md)
- [Loops and Multi-Instance](loops-and-multi-instance.md)
- [Tasks and Host Dispatch](tasks-and-host-dispatch.md)

## Current Package Boundary

The current package owns bounded support for:

- linear flows and bounded gateway routing
- bounded waits and boundary events
- bounded loop and multi-instance task execution
- bounded host-dispatched task families including `sendTask` and `scriptTask`
- bounded subprocess, transaction, and same-package call-activity slices

The current package still defers:

- collaboration and lane semantics
- full BPMN data-object and IO-specification coverage
- unbounded event families and event subprocesses
- broader FEEL or script-backed flow semantics
