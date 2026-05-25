# xiuxian-qianji-bpmn-engine Docs

This directory holds package-level documentation for the bounded
`xiuxian-qianji-bpmn-engine` surface.

The current documentation baseline is organized around official OMG standards:

- BPMN alignment tracks the formal
  [BPMN 2.0.2 specification](https://www.omg.org/spec/BPMN/2.0.2).
- DMN alignment tracks the current formal
  [DMN 1.5 specification](https://www.omg.org/spec/DMN/1.5).

Use these modules as the package-local source of truth for what the crate
currently supports, what is still intentionally deferred, and where new slices
should extend the bounded surface.

The BPMN parser surface also exposes a non-executable document snapshot API for
metadata-oriented tooling. Snapshot coverage preserves collaboration, lane, and
data/IO metadata for linting and adapters, but does not make those constructs
runtime-executable.

## Modules

- [OMG Alignment Index](omg-alignment/README.md)
- [BPMN Alignment Index](omg-alignment/bpmn/README.md)
- [BPMN Events and Boundaries](omg-alignment/bpmn/events-and-boundaries.md)
- [BPMN Gateways and Concurrency](omg-alignment/bpmn/gateways-and-concurrency.md)
- [BPMN Loops and Multi-Instance](omg-alignment/bpmn/loops-and-multi-instance.md)
- [BPMN Subprocesses, Transactions, and Compensation](omg-alignment/bpmn/subprocesses-transactions-and-compensation.md)
- [BPMN Tasks and Host Dispatch](omg-alignment/bpmn/tasks-and-host-dispatch.md)
- [DMN Alignment Index](omg-alignment/dmn/README.md)
- [DMN Decision Tables](omg-alignment/dmn/decision-tables.md)
- [DMN Literal Expressions](omg-alignment/dmn/literal-expressions.md)
- [DMN List Expressions](omg-alignment/dmn/list-expressions.md)
- [DMN Context Expressions](omg-alignment/dmn/context-expressions.md)
- [DMN Relation Expressions](omg-alignment/dmn/relation-expressions.md)
- [DMN FEEL Subset](omg-alignment/dmn/feel-subset.md)
