# OMG Alignment Index

This module maps the bounded `xiuxian-qianji-bpmn-engine` feature set to the official
OMG BPMN and DMN specifications.

It exists for three reasons:

1. keep the package-local support contract readable without forcing every
   reader through one long research note
2. separate current bounded support from intentionally deferred OMG surface
3. give future slices one stable place to update when parser, runtime, lint,
   or checkpoint behavior widens

The current index is source-backed. Each family now includes an explicit
official-source map anchored to the OMG inventory page, the normative PDF, and
the machine-readable artifacts that define the wire format or diagram
interchange surface.

## BPMN Modules

- [BPMN Alignment Index](bpmn/README.md)
- [BPMN Official Source Map](bpmn/spec-source-map.md)
- [BPMN Collaboration, Lanes, and Data](bpmn/collaboration-lanes-and-data.md)
- [BPMN Events and Boundaries](bpmn/events-and-boundaries.md)
- [BPMN Gateways and Concurrency](bpmn/gateways-and-concurrency.md)
- [BPMN Loops and Multi-Instance](bpmn/loops-and-multi-instance.md)
- [BPMN Subprocesses, Transactions, and Compensation](bpmn/subprocesses-transactions-and-compensation.md)
- [BPMN Tasks and Host Dispatch](bpmn/tasks-and-host-dispatch.md)

## DMN Modules

- [DMN Alignment Index](dmn/README.md)
- [DMN Official Source Map](dmn/spec-source-map.md)
- [DMN Invocations](dmn/invocations.md)
- [DMN Decision Tables](dmn/decision-tables.md)
- [DMN Literal Expressions](dmn/literal-expressions.md)
- [DMN List Expressions](dmn/list-expressions.md)
- [DMN Context Expressions](dmn/context-expressions.md)
- [DMN Relation Expressions](dmn/relation-expressions.md)
- [DMN FEEL Subset](dmn/feel-subset.md)

## Scope Rule

These documents describe the current bounded engine contract. They are not a
claim of full OMG conformance.
