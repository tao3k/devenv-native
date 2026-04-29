# BPMN Collaboration, Lanes, and Data

This module records the current snapshot- and lint-owned alignment against the
official [BPMN 2.0.2 specification](https://www.omg.org/spec/BPMN/2.0.2).

## Current Support

The current engine preserves non-executable BPMN metadata for these families:

- collaboration-level participant and message-flow structure
- top-level partner entity, partner role, and endpoint catalogs, plus
  participant interface references, endpoint references, and multiplicity
  metadata
- collaboration-level conversation nodes, links, associations, correlation
  keys, and choreography references
- choreography roots and choreography activity metadata, including
  participant refs, message-flow refs, nested correlation keys, participant
  associations, and called choreography refs
- BPMN artifact metadata for `association`, `group`, and `textAnnotation`
- top-level item-definition catalog metadata used by message and data
  references
- top-level message and correlation-property catalogs, including correlation
  retrieval `messageRef` and `messagePath` metadata, used by collaboration
  evidence
- IO-set metadata under process and global-task IO specifications, including
  input/output set names and direct reference lists
- process callable metadata, including `processType`, `isClosed`,
  `definitionalCollaborationRef`, `supports`, process `property`, and process
  `correlationSubscription` bindings
- a process correlation-boundary evidence object that marks
  `correlationSubscription`, `correlationPropertyBinding`, `correlationKey`,
  and `dataPath` declarations as preserved metadata while keeping runtime
  matching deferred
- callable IO metadata, including direct process/global-task `ioBinding`
  declarations and direct global-task `ioSpecification` declarations
- direct process and global-task resource-role metadata, including
  `resourceRole`, `performer`, `humanPerformer`, `potentialOwner`,
  `resourceRef`, `resourceParameterBinding`, and
  `resourceAssignmentExpression`
- direct process flow-element common metadata, including `auditing`,
  `monitoring`, and `categoryValueRef`
- lane-set, lane, and lane-owned flow-node references
- data-object, data-store, item-definition, direct `dataState` metadata, and
  data-association `transformation` and `assignment` payloads
- lint evidence that reports those preserved structures back to `qianji lint`
  in an LLM-friendly way
- a Rust-owned collaboration routing-boundary evidence object that marks
  preserved metadata separately from deferred participant dispatch,
  message-flow routing, conversation routing, choreography execution, and
  correlation matching semantics

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
preserve standard BPMN `message`, `correlationProperty`, nested
`correlationPropertyRetrievalExpression`, partner entity, partner role,
endpoint, participant interface reference, participant endpoint reference,
participant multiplicity, conversation node, conversation link, conversation
association, participant association, message-flow association,
correlation-key, choreography-reference, and choreography activity
declarations plus artifact associations, groups, and text annotations and
surface that catalog through collaboration lint evidence. That makes partner,
participant, message-flow, conversation, choreography, artifact, process
callable, callable IO, IO-set, data-state, data-association expression,
resource-role, flow-element metadata, and correlation references auditable
without requiring adapters to re-scan XML, but it does not dispatch message
flows, route conversations, execute
choreography, invoke endpoints, execute groups, interpret annotations,
schedule participant multiplicity, resolve process support, execute process
properties, bind callable operations, invoke callable IO bindings, execute
data-state transitions, execute generic resource assignments, authorize roles,
execute auditing or monitoring declarations, classify category values, or
evaluate correlation subscriptions, keys, or retrieval expressions.

The linter includes a `routing_boundary` evidence object for collaboration
documents. That object declares `metadata_only` status, `deferred` execution
policy, a `single_process_graph` runtime scope, the preserved collaboration
metadata families, and the exact deferred routing and correlation semantics.
This is evidence for future routing work, not an executable routing contract.

Process callable lint evidence also includes a `correlation_boundary` object.
That boundary records the distinction between bounded executable waits that
use explicit event references and deferred BPMN correlation subscriptions.
The runtime can wait on explicit message, signal, timer, or conditional event
references, but it does not evaluate `correlationSubscription`,
`correlationPropertyBinding`, `correlationKey`, or `dataPath` declarations for
matching.

## Runtime Boundary

These BPMN surfaces remain deferred:

- collaboration-aware message routing across pools or participants
- runtime routing inferred from `messageFlow`, `conversation`,
  `choreography`, or correlation metadata
- endpoint invocation, partner routing, or participant multiplicity execution
- executable conversation routing and choreography execution
- executable group semantics or text-annotation interpretation
- executable correlation keys, correlation subscriptions, and retrieval
  expression evaluation
- executable correlation property binding or data-path evaluation
- executable process support resolution, process property semantics, or
  process inheritance
- executable callable-operation binding or callable IO invocation from
  process/global-task metadata
- executable optional or while-executing IO-set lifecycle semantics
- executable generic resource-role assignment, authorization, delegation,
  escalation, scheduling, or reassignment
- executable flow-element auditing, monitoring, category classification, or
  metadata-driven routing
- executable item-definition schema validation or payload coercion
- lane-driven assignment, authorization, or execution ownership semantics
- executable `dataObject` or `dataStore` persistence semantics
- executable data-state transition behavior
- executable data-association transformations, multiple-source data
  associations, and data-store-backed task IO
- runtime behavior inferred from BPMN DI layout artifacts

## Repair Guidance

Preserve collaboration, lane, item-definition, and data structures in the
source BPMN document. When executable behavior is required in the current
bounded slice, route task-local operational payloads through native BPMN task
IO and route broader state through workflow variables, host dispatch, waits,
or DMN inputs instead of fabricating partial collaboration, type-validation,
or data-object execution rules.

For collaboration documents, keep `participant`, `messageFlow`,
`participantMultiplicity`, `partnerEntity`, `partnerRole`, `endPoint`,
`messageFlow`, `conversation`, `conversationLink`, `choreography`,
`choreographyTask`, `subChoreography`, `callChoreography`, `association`,
`group`, `textAnnotation`, `correlationKey`, `message`, and
`correlationProperty` definitions standard and explicit, including retrieval
expressions when a correlation property depends on a specific message path.
The linter can report that metadata as evidence, but runtime behavior must
still be modeled through one supported process graph, host work, or event
waits until collaboration routing and choreography execution land.
