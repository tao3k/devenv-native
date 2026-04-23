# OMG Alignment Index

This module maps the bounded `qianji-bpmn-engine` feature set to the official
OMG BPMN and DMN specifications.

It exists for three reasons:

1. keep the package-local support contract readable without forcing every
   reader through one long research note
2. separate current bounded support from intentionally deferred OMG surface
3. give future slices one stable place to update when parser, runtime, lint,
   or checkpoint behavior widens

## BPMN Modules

- [BPMN Alignment Index](bpmn/README.md)
- [BPMN Events and Boundaries](bpmn/events-and-boundaries.md)
- [BPMN Gateways and Concurrency](bpmn/gateways-and-concurrency.md)
- [BPMN Loops and Multi-Instance](bpmn/loops-and-multi-instance.md)
- [BPMN Subprocesses, Transactions, and Compensation](bpmn/subprocesses-transactions-and-compensation.md)
- [BPMN Tasks and Host Dispatch](bpmn/tasks-and-host-dispatch.md)

## DMN Modules

- [DMN Alignment Index](dmn/README.md)
- [DMN Decision Tables](dmn/decision-tables.md)
- [DMN Literal Expressions](dmn/literal-expressions.md)
- [DMN List Expressions](dmn/list-expressions.md)
- [DMN Context Expressions](dmn/context-expressions.md)
- [DMN Relation Expressions](dmn/relation-expressions.md)
- [DMN FEEL Subset](dmn/feel-subset.md)

## Scope Rule

These documents describe the current bounded engine contract. They are not a
claim of full OMG conformance.
