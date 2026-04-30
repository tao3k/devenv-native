# BPMN Full Conformance Coverage

This module tracks the conformance ladder for major BPMN families. OMG BPMN
2.0.2 remains the normative source for vocabulary and shape. Runtime alignment
means the parser accepts one explicit shape, the runtime executes that same
shape deterministically, and lint reports unsupported shapes with repair
guidance.

## Registry Source Of Truth

The Rust API `bpmn_conformance_registry()` is the machine-checkable source of
truth for this coverage table. Each registry row records the BPMN family,
overall status, parser coverage, snapshot coverage, lint coverage, runtime
coverage, host-surface coverage, a stable package-doc anchor, and the next
milestone that should maintain or promote that family.

This document remains the human-readable explanation of the registry. Tests
assert that every BPMN family and status below stays aligned with the Rust
registry, so new milestones must update both surfaces together.

## Status Vocabulary

| Status               | Meaning                                                                |
| -------------------- | ---------------------------------------------------------------------- |
| `supported`          | Accepted and stable for the current bounded engine contract.           |
| `bounded executable` | Executed for one documented subset with explicit runtime tests.        |
| `metadata-only`      | Preserved in snapshots or host metadata, not executable runtime logic. |
| `lint-deferred`      | Reported as unsupported executable semantics with repair guidance.     |
| `missing`            | Not yet represented by parser, snapshot, lint, or runtime coverage.    |

## Coverage Matrix

| BPMN family               | Current status     | Current boundary                                                                                                                                                              |
| ------------------------- | ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Linear process flow       | bounded executable | Start/end events, bounded start waits, sequence flow, and deterministic tokens.                                                                                               |
| Host-dispatched tasks     | bounded executable | Service, send, script, business-rule, user, and manual task families.                                                                                                         |
| Human interaction         | bounded executable | Native BPMN documentation and IO metadata feed Rust-owned host-work forms.                                                                                                    |
| Parallel gateway          | bounded executable | Split and join with deterministic active-token and join-buffer behavior.                                                                                                      |
| Exclusive gateway         | bounded executable | Default flow plus simple boolean-path and numeric condition expressions.                                                                                                      |
| Inclusive gateway         | bounded executable | Structured split and one matching linear join fragment.                                                                                                                       |
| Event-based gateway       | bounded executable | Exclusive competition over message, signal, timer, or conditional catches.                                                                                                    |
| Complex gateway           | lint-deferred      | Activation/fan-in/fan-out semantics remain deferred; unsupported usage has a specific repair diagnostic.                                                                      |
| Intermediate catch events | bounded executable | Message, signal, timer, and bounded conditional waits.                                                                                                                        |
| Boundary events           | bounded executable | Bounded task owners plus interrupting subprocess-like external boundaries.                                                                                                    |
| Error and cancel events   | bounded executable | Bounded subprocess, call-activity, transaction, and top-level error paths.                                                                                                    |
| Compensation              | bounded executable | Transaction-owned compensation handlers and throw-compensation paths.                                                                                                         |
| Conditional events        | bounded executable | Start events, catches, task boundaries, and interrupting subprocess-like boundaries.                                                                                          |
| Escalation events         | bounded executable | Child-scope end/throw routes execute; deferred escalation shapes get diagnostics.                                                                                             |
| Import declarations       | metadata-only      | Top-level import declarations are preserved; external dependency resolution is deferred.                                                                                      |
| Extension declarations    | metadata-only      | Top-level extension declarations are preserved; extension behavior remains deferred.                                                                                          |
| Relationship declarations | metadata-only      | Top-level relationships are preserved; endpoint resolution and graph semantics defer.                                                                                         |
| Event definition catalogs | metadata-only      | Message/error/escalation/signal catalogs are preserved, not schema-validated.                                                                                                 |
| Interfaces/operations     | metadata-only      | Callable-operation catalogs are preserved; executable task `operationRef` binding has explicit deferred diagnostics.                                                          |
| Global task catalogs      | metadata-only      | Top-level global task definitions are preserved in the Rust-owned callable registry; `callActivity` bindings to them get explicit deferred diagnostics.                       |
| Process callable metadata | metadata-only      | Process callable attributes, support refs, properties, and correlation subscriptions are preserved in the callable registry.                                                  |
| Callable IO metadata      | metadata-only      | Process/global-task `ioBinding` and global-task `ioSpecification` declarations are preserved in the callable registry.                                                        |
| Resource catalogs         | metadata-only      | Top-level resources and parameters are preserved; assignment binding remains deferred.                                                                                        |
| Resource-role metadata    | metadata-only      | Direct process and global-task resource-role declarations are preserved with explicit deferred assignment diagnostics.                                                        |
| Flow-element metadata     | metadata-only      | Direct process flow-element auditing, monitoring, and category refs are preserved passively.                                                                                  |
| Category catalogs         | metadata-only      | Top-level categories and values are preserved; classification remains passive.                                                                                                |
| Terminate events          | bounded executable | `terminateEventDefinition` end events terminate the current runtime scope.                                                                                                    |
| Multiple events           | lint-deferred      | Multiple and parallel-multiple event definitions have stable parser/lint diagnostics.                                                                                         |
| Embedded subprocess       | bounded executable | One nested start event and at least one nested end event.                                                                                                                     |
| Call activity             | bounded executable | Same-package executable process targets.                                                                                                                                      |
| Transaction               | bounded executable | Bounded shell with cancel/error/compensation behavior.                                                                                                                        |
| Event subprocess          | bounded executable | One interrupting event subprocess per scope with message, signal, timer, or bounded conditional start trigger.                                                                |
| Standard loop             | bounded executable | Supported on selected host-dispatched task families.                                                                                                                          |
| Sequential multi-instance | bounded executable | Cardinality and bounded collection-backed input/output subset.                                                                                                                |
| Parallel multi-instance   | bounded executable | Cardinality and bounded collection-backed input/output subset.                                                                                                                |
| Collaboration and pools   | metadata-only      | Participant, partner, endpoint, message-flow, conversation, choreography, and correlation metadata is preserved with explicit routing-boundary evidence.                      |
| Artifacts                 | metadata-only      | Association, group, and text-annotation metadata is preserved without execution semantics.                                                                                    |
| Lanes                     | metadata-only      | Preserved for passive routing/display; no scheduling or authorization.                                                                                                        |
| Item definitions          | metadata-only      | Top-level item catalogs are preserved; schema validation remains deferred.                                                                                                    |
| Data objects              | bounded executable | Process-level data object/reference ids can be used as bounded task data-association variable bindings.                                                                       |
| Data stores               | lint-deferred      | Data store/reference metadata and direct `dataState` are preserved; data-store-reference bindings have explicit deferred diagnostics.                                         |
| IO specification          | bounded executable | Human-task form IO and bounded host-task Data/IO metadata are executable; IO sets are preserved passively.                                                                    |
| Data associations         | bounded executable | Bounded host-task source/target mapping is executable; transformation and assignment payloads are preserved.                                                                  |
| BPMN DI                   | metadata-only      | Diagram, plane, shape, edge, bounds, waypoint, label, and font metadata is preserved; DI topology, anchor presence/kind, links, ids, and minimum layout payloads are audited. |
| DMN links                 | bounded executable | Business-rule tasks can execute local bounded DMN decisions when available.                                                                                                   |

## Completed M1 Milestone

The first Data/IO milestone promotes bounded task Data/IO execution. Supported
host-dispatched task requests carry resolved task inputs, and task completion
writes outputs through declared BPMN `dataOutputAssociation` targets.

## Active M4.1 Data Object Milestone

The data-object milestone promotes process-level `dataObject` and
`dataObjectReference` declarations to bounded executable copy-in/copy-out
bindings. A task `dataInputAssociation/sourceRef` may point at a data object
or data object reference, and the runtime request reads the referenced
workflow variable. A task `dataOutputAssociation/targetRef` may point at the
same standard BPMN data object/reference surface, and completion writes back
through that canonical variable binding.

`dataStore` and `dataStoreReference` remain lint-deferred because executable
persistence still needs an explicit storage and transaction policy. Direct
standard data associations that bind through `dataStoreReference` ids are
reported as data-store binding diagnostics, not as executable Data/IO support.

## Completed M4.2 Event Subprocess Milestone

The event-subprocess milestone promotes one interrupting
`subProcess triggeredByEvent="true"` shape to bounded executable behavior.
The supported trigger start event must use exactly one standard message,
signal, timer, or bounded conditional event definition. Runtime will expose the
trigger as a passive scope-level wait; when the wait wins, the parent scope is
cancelled and execution enters the event-subprocess body after its start
event.

Non-interrupting event subprocesses, compensation event subprocesses, multiple
event subprocesses in one scope, and BPMN correlation matching remain
deferred.

Data-store persistence, executable transformations, multiple-source
associations, and collaboration-aware routing remain deferred until separate
milestones define their execution contracts.

The collaboration, data, callable-operation, and event metadata slices
preserve top-level `import`, `extension`, `relationship`, `BPMNDiagram`,
`itemDefinition`, `message`, `interface`, `operation`, `resource`,
`resourceParameter`, `category`, `categoryValue`, `error`, `escalation`,
`signal`, `globalTask`, `globalBusinessRuleTask`, `globalManualTask`,
`globalScriptTask`, `globalUserTask`, `correlationProperty`, nested
`correlationPropertyRetrievalExpression` metadata alongside collaboration
participants, message flows, conversation nodes, conversation associations,
participant associations, message-flow associations, conversation links,
correlation keys, choreography references, choreography activity metadata, and
artifact metadata, partner entities, partner roles, endpoints, participant
interface refs, participant endpoint refs, participant multiplicity, process
callable attributes, process support refs, process properties, process
correlation subscriptions, direct process resource roles, direct global-task
resource roles, direct process/global-task callable IO bindings, direct
global-task IO specifications, direct `dataState` metadata on standard BPMN
data owners, IO-set metadata, data-association expression metadata, direct
flow-element auditing/monitoring/category metadata, and data references. This
gives Rust-owned evidence for future routing and
type-alignment work while keeping
pool routing, message dispatch, conversation routing, choreography execution,
endpoint invocation, participant multiplicity execution, global task
execution, process support resolution, process property execution,
callable-operation binding, callable IO operation invocation, optional and
while-executing IO-set lifecycle semantics, data-association transformation
execution, generic resource assignment execution, resource authorization,
delegation, escalation, scheduling, data-state transition execution, audit
execution, monitoring execution, category classification, group execution,
annotation interpretation, import resolution, extension behavior, extension
payload parsing, diagram rendering, layout validation, relationship endpoint
resolution, event subscription registries, correlation
matching, correlation subscription matching, correlation-key evaluation,
retrieval expression evaluation, and schema validation deferred.

For BPMN DI specifically, metadata-only preservation includes Rust-owned
audits for direct `BPMNDiagram` to `BPMNPlane` topology, required
`bpmnElement` anchors on planes, shapes, and edges, conservative anchor-kind
checks for obvious plane/shape/edge semantic mismatches, semantic
`bpmnElement` references, DI-local edge endpoint and label-style references,
duplicate DI identifiers, direct `dc:Bounds` on `BPMNShape`, and at least two
direct `di:waypoint` entries on `BPMNEdge`. Coordinate values, geometry
quality, diagram rendering, and runtime routing from layout coordinates remain
out of scope.

## Active M2 Event Milestone

The first event-family slice promotes `terminateEventDefinition` end events
from missing to bounded executable. A terminate end cancels sibling active
tokens, waits, and pending host work in the current runtime scope. At the root
process it completes the instance; inside a bounded called or embedded scope it
completes the parent activity route.

The second event-family slice promotes native `conditionalEventDefinition` on
`intermediateCatchEvent`. The runtime evaluates the condition with the bounded
boolean-path or numeric-comparison subset used by gateway conditions. If the
condition is already true, the catch event routes immediately; otherwise it
registers a conditional wait and re-evaluates after poll data is merged.
At that slice boundary, conditional start events and conditional event
subprocesses were deferred.

The third event-family slice promotes native `escalationEventDefinition` for
bounded subprocess-scope end events. The runtime routes a thrown escalation to
matching interrupting escalation boundary events on the parent embedded
subprocess, same-package call activity, or transaction owner. At that slice
boundary, root-level escalation ends, non-interrupting escalation boundaries,
intermediate throw escalation, escalation start events, and escalation event
subprocess triggers remained deferred.

The fourth event-family slice promotes native `conditionalEventDefinition` on
task-attached `boundaryEvent` nodes. The same bounded boolean-path and
numeric-comparison expression subset is used for interrupting and
non-interrupting task boundaries.

The fifth event-family slice promotes native `conditionalEventDefinition` on
interrupting `boundaryEvent` nodes attached to bounded embedded subprocess,
same-package call-activity, and transaction owners. The runtime arms the same
parent-frame wait path used by bounded timer, message, and signal external
boundaries, then re-evaluates the bounded condition after event-poll data is
merged. Non-interrupting conditional boundaries on subprocess-like owners
remained deferred at that slice boundary.

The sixth event-family slice promotes native `conditionalEventDefinition` as
an exclusive `eventBasedGateway` `intermediateCatchEvent` wait target. Runtime
event competition still uses a single winning branch, and non-ready poll data
can select the first conditional wait whose bounded expression becomes true.
Parallel event-based gateways and collaboration-aware correlation remain
deferred.

The seventh event-family slice promotes native event definitions on the single
process `startEvent` to bounded executable start waits. The parser accepts one
`messageEventDefinition`, `signalEventDefinition`, `timerEventDefinition`, or
`conditionalEventDefinition`; the runtime creates the instance, blocks at the
start event, and routes after a matching poll outcome or already-satisfied
bounded condition. Multiple start events, collaboration-aware subscription
registries, and multiple event definitions remain deferred.

The eighth event-family slice promotes native `escalationEventDefinition` on
`intermediateThrowEvent` inside a bounded embedded subprocess, same-package
call activity, or transaction child scope. The runtime routes the throw
through the existing matching interrupting parent escalation boundary path and
cancels the child scope. Root-level escalation throws, non-interrupting
escalation boundaries, escalation start events, and escalation event
subprocess triggers remain deferred.

The ninth event-family slice formalizes `multipleEventDefinition`,
`parallelMultipleEventDefinition`, and several concrete event definitions on
one event node as lint-deferred standard boundaries. The parser emits stable
diagnostics and the linter provides repair guidance to remodel the behavior
with one supported concrete event definition, an explicit event-based gateway,
or supported boundary-event structures. Executable multiple-event fan-in and
parallel-multiple event semantics remain deferred.

The tenth event-family slice formalizes the remaining deferred escalation
surfaces that are not part of the bounded executable route. Escalation start
events, non-interrupting escalation boundaries, and task-owned interrupting
escalation boundaries now report stable parser/lint diagnostics with repair
guidance. Executable semantics remain limited to escalation end events or
intermediate escalation throws inside bounded subprocess-like child scopes,
routed to matching interrupting escalation boundaries on those parent owners.

## Active M3 Collaboration Boundary Milestone

The first collaboration-boundary slice keeps collaboration and pool semantics
metadata-only, but makes the runtime boundary machine-checkable in Rust-owned
lint evidence. Collaboration diagnostics now report a `routing_boundary`
object that separates preserved metadata from deferred execution semantics.

Preserved metadata includes participants, partner catalogs, endpoints,
messages, message flows, conversations, choreography declarations,
correlation properties, and correlation retrieval expressions. Deferred
execution semantics include participant dispatch, endpoint invocation,
message-flow routing, conversation routing, choreography execution,
correlation matching, correlation subscription matching, correlation-key
evaluation, and retrieval-expression evaluation.

The second collaboration-boundary slice extends the same evidence discipline
to process-level `correlationSubscription` metadata. Process callable
diagnostics now report a `correlation_boundary` object that distinguishes
bounded explicit event-reference waits from deferred BPMN correlation matching.
The engine may wait on an explicit message, signal, timer, or conditional
event reference, but it does not evaluate `correlationSubscription`,
`correlationPropertyBinding`, `correlationKey`, or `dataPath` declarations as
runtime matching policy.

The third collaboration-boundary slice aligns the executable wait ABI with
that terminology: wait metadata exposes an optional `deduplication_key` for
host-side event de-duplication, not a BPMN `correlation_key`. When present,
that key is derived from the explicit event reference and must not be treated
as correlation subscription matching.

Current executable behavior must still be modeled through one supported
process graph, host-dispatched tasks, or supported event waits. This milestone
does not implement cross-pool message dispatch or correlation-aware runtime
routing.

## Completed M4.3 Callable Binding Milestone

The callable-binding slice adds a Rust-owned callable registry to the parsed
package surface. The registry records same-package process callable
definitions, top-level global task definitions, process/global-task callable
IO metadata, and existing process-target `callActivity` bindings.

This milestone does not execute top-level global task definitions, invoke
interface operations, resolve remote imports, dispatch endpoint bindings, or
apply BPMN correlation matching. Existing `callActivity` runtime execution
remains limited to another executable process in the same parsed BPMN package.

## Completed M4.4 Collaboration Host Envelope Milestone

The collaboration host-envelope slice exposes collaboration intent from the
parsed package surface instead of requiring hosts to reread BPMN XML. The
envelope covers collaboration shells, participants, message-flow intent,
correlation properties, correlation keys, and process correlation
subscriptions.

The milestone keeps collaboration execution metadata-only. It does not execute
pool routing, participant dispatch, endpoint invocation, message-flow routing,
conversation routing, choreography execution, BPMN correlation matching,
correlation subscription matching, correlation-key evaluation, or data-path
evaluation. Existing wait metadata may expose a host `deduplication_key`, but
that value is not a BPMN correlation key.

## Completed M4.5 Compatibility Suite Milestone

The compatibility-suite slice adds representative native BPMN interchange
fixtures and tests. The proof combines standard BPMN process XML, task-local
`ioSpecification`, `dataInputAssociation`, `dataOutputAssociation`, and BPMN DI
layout metadata. It verifies that parse, lint evidence, host-work request
materialization, strict output mapping, and runtime completion work without a
custom XML namespace or custom moddle descriptor.

BPMN DI remains metadata-only. The compatibility proof may report the standard
DI metadata lint issue while executable process semantics still parse and run
through the bounded runtime contract.

Generated and imported BPMN DI is also referentially audited. `BPMNPlane`,
`BPMNShape`, and `BPMNEdge` `bpmnElement` values must point at semantic BPMN ids
declared in the same source. `BPMNEdge` `sourceElement` and `targetElement`
values must point at DI element ids in the same plane, and `BPMNLabel`
`labelStyle` values must point at label-style ids in the same diagram. Missing
references report a specific lint diagnostic before the generic metadata-only
DI guidance.

DI structural completeness is also audited for stable interchange. `BPMNShape`
entries should carry direct `dc:Bounds`, and `BPMNEdge` entries should carry at
least two direct `di:waypoint` entries. Missing minimum layout payloads report a
specific lint diagnostic before the generic metadata-only DI guidance. The
audit does not execute layout, validate coordinate geometry, or infer runtime
behavior from diagram coordinates.

## Completed M4.6 Global Task Binding Diagnostics Milestone

The global-task binding diagnostics slice keeps top-level global task
definitions metadata-only, but makes invalid executable bindings precise. A
`callActivity` whose `calledElement` points at a same-package `globalTask`,
`globalBusinessRuleTask`, `globalScriptTask`, `globalUserTask`, or
`globalManualTask` must report a specific deferred-binding diagnostic with
source evidence and repair guidance.

Runtime execution remains limited to `callActivity` targets that resolve to
another executable process in the same BPMN package. Global-task execution,
interface-operation invocation, and host dispatch inferred from top-level
global-task metadata remain deferred.

## Completed M4.7 Data Store Binding Diagnostics Milestone

The data-store binding diagnostics slice keeps `dataStore` and
`dataStoreReference` persistence lint-deferred, but makes executable binding
misuse precise. A standard `dataInputAssociation/sourceRef` or
`dataOutputAssociation/targetRef` that points at a process-level
`dataStoreReference` reports `bpmn.unsupported_data_store_binding` with
snapshot evidence for the process id, association kind, association id, usage
site, data-store-reference id, and referenced data-store id.

Runtime semantics remain unchanged. Persistent data-store reads and writes
require a future storage and transaction policy; current executable BPMN should
use workflow variables, bounded `dataObjectReference` mappings, or explicit
host-dispatched task payloads instead.

## Completed M4.8 Complex Gateway Diagnostics Milestone

The complex-gateway diagnostics slice keeps `complexGateway` lint-deferred, but
replaces the generic unsupported-element fallback with
`bpmn.unsupported_complex_gateway`. The diagnostic states that activation,
fan-in, and fan-out semantics remain deferred, and guides BPMN authors toward
bounded `exclusiveGateway`, `inclusiveGateway`, `parallelGateway`, or
`eventBasedGateway` rewrites.

Runtime semantics remain unchanged. Complex gateway activation conditions and
unstructured synchronization still require a future advanced-control-flow
policy.

## Completed M4.9 Operation Binding Diagnostics Milestone

The operation-binding diagnostics slice keeps top-level BPMN `interface` and
`operation` catalogs metadata-only, but makes executable task-level
`operationRef` usage explicit. A `serviceTask`, `sendTask`, or `receiveTask`
whose runtime behavior would otherwise appear to bind through an operation
reference must report a deferred operation-binding diagnostic.

Runtime semantics remain unchanged. Host-dispatched task execution still comes
from the explicit task node, bounded task Data/IO, message metadata where
supported, and host-work request metadata. Interface-operation invocation,
endpoint binding, and external callable contract validation remain deferred.

## Completed M4.10 Resource Role Diagnostics Milestone

The resource-role diagnostics slice keeps direct process and global-task
`resourceRole`, `performer`, `humanPerformer`, and `potentialOwner`
declarations metadata-only, but makes the deferred assignment boundary explicit
instead of reporting the surface through generic collaboration diagnostics.

Runtime semantics remain unchanged. Human-task local `humanPerformer` and
`potentialOwner` declarations continue to provide bounded routing hints, but
generic process/global-task assignment, scheduling, authorization, delegation,
escalation, and resource-parameter binding execution remain deferred.

## Completed M4.11 Flow Element Diagnostics Milestone

The flow-element diagnostics slice keeps direct BPMN `auditing`, `monitoring`,
and `categoryValueRef` declarations on process flow elements metadata-only,
but makes the deferred audit, monitoring, and classification boundary explicit
instead of reporting the surface through generic collaboration diagnostics.

Runtime semantics remain unchanged. Executable behavior still comes from
supported process flow, events, tasks, gateways, and bounded data mappings;
audit execution, monitoring telemetry, category classification, scheduling,
authorization, and policy enforcement remain deferred.
