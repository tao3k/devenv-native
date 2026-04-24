# BPMN Collaboration, Lanes, and Data

This module records the current snapshot- and lint-owned alignment against the
official [BPMN 2.0.2 specification](https://www.omg.org/spec/BPMN/2.0.2).

## Current Support

The current engine preserves non-executable BPMN metadata for these families:

- collaboration-level participant and message-flow structure
- lane-set, lane, and lane-owned flow-node references
- data-object, data-store, and item-definition metadata
- IO-specification, `dataInput`, `dataOutput`, and data-association metadata
- lint evidence that reports those preserved structures back to `qianji lint`
  in an LLM-friendly way

The runtime may still move executable data through workflow variables,
host-work payloads, waits, and DMN inputs, but it does not treat those BPMN
families as engine-owned execution semantics.

## Runtime Boundary

These BPMN surfaces remain deferred:

- collaboration-aware message routing across pools or participants
- lane-driven assignment, authorization, or execution ownership semantics
- executable `dataObject`, `dataStore`, `ioSpecification`, `dataInput`, or
  `dataOutput` semantics
- runtime behavior inferred from BPMN DI layout artifacts

## Repair Guidance

Preserve collaboration, lane, and data structures in the source BPMN document.
When executable behavior is required in the current bounded slice, route
operational payloads through workflow variables, host dispatch, waits, or DMN
inputs instead of fabricating partial collaboration or data-object execution
rules.
