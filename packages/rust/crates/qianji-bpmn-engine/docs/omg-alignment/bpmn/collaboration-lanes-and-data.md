# BPMN Collaboration, Lanes, and Data

This module records the current snapshot- and lint-owned alignment against the
official [BPMN 2.0.2 specification](https://www.omg.org/spec/BPMN/2.0.2).

## Current Support

The current engine preserves non-executable BPMN metadata for these families:

- collaboration-level participant and message-flow structure
- top-level item-definition catalog metadata used by message and data
  references
- top-level message and correlation-property catalogs, including correlation
  retrieval `messageRef` and `messagePath` metadata, used by collaboration
  evidence
- lane-set, lane, and lane-owned flow-node references
- data-object, data-store, and item-definition metadata
- lint evidence that reports those preserved structures back to `qianji lint`
  in an LLM-friendly way

The current engine executes a bounded task-local Data/IO subset for supported
host-dispatched tasks. `ioSpecification`, `dataInputAssociation`, and
`dataOutputAssociation` are executable only when they are attached to a
supported task and fit the bounded source/target mapping rules. Those task IO
bindings resolve host request inputs and validate/map host completion outputs.

The runtime may still move executable data through workflow variables,
host-work payloads, waits, and DMN inputs, but it does not treat those BPMN
collaboration, lane, item-definition, data-object, or data-store families as
engine-owned execution semantics.

The first collaboration-alignment slices are metadata-only: Rust snapshots
preserve standard BPMN `message`, `correlationProperty`, and nested
`correlationPropertyRetrievalExpression` declarations and surface that catalog
through collaboration lint evidence. That makes participant/message-flow and
correlation references auditable without requiring adapters to re-scan XML,
but it does not dispatch message flows or evaluate correlation subscriptions
or retrieval expressions.

## Runtime Boundary

These BPMN surfaces remain deferred:

- collaboration-aware message routing across pools or participants
- executable correlation keys, correlation subscriptions, and retrieval
  expression evaluation
- executable item-definition schema validation or payload coercion
- lane-driven assignment, authorization, or execution ownership semantics
- executable `dataObject` or `dataStore` persistence semantics
- transformations, multiple-source data associations, and data-store-backed
  task IO
- runtime behavior inferred from BPMN DI layout artifacts

## Repair Guidance

Preserve collaboration, lane, item-definition, and data structures in the
source BPMN document. When executable behavior is required in the current
bounded slice, route task-local operational payloads through native BPMN task
IO and route broader state through workflow variables, host dispatch, waits,
or DMN inputs instead of fabricating partial collaboration, type-validation,
or data-object execution rules.

For collaboration documents, keep `participant`, `messageFlow`, `message`, and
`correlationProperty` definitions standard and explicit, including retrieval
expressions when a correlation property depends on a specific message path.
The linter can report that metadata as evidence, but runtime behavior must
still be modeled through one supported process graph, host work, or event
waits until collaboration routing lands.
