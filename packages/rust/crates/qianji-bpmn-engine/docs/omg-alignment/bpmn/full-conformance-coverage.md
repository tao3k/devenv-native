# BPMN Full Conformance Coverage

This module tracks the conformance ladder for major BPMN families. OMG BPMN
2.0.2 remains the normative source for vocabulary and shape. Runtime alignment
means the parser accepts one explicit shape, the runtime executes that same
shape deterministically, and lint reports unsupported shapes with repair
guidance.

## Status Vocabulary

| Status               | Meaning                                                                |
| -------------------- | ---------------------------------------------------------------------- |
| `supported`          | Accepted and stable for the current bounded engine contract.           |
| `bounded executable` | Executed for one documented subset with explicit runtime tests.        |
| `metadata-only`      | Preserved in snapshots or host metadata, not executable runtime logic. |
| `lint-deferred`      | Reported as unsupported executable semantics with repair guidance.     |
| `missing`            | Not yet represented by parser, snapshot, lint, or runtime coverage.    |

## Coverage Matrix

| BPMN family               | Current status     | Current boundary                                                                                                 |
| ------------------------- | ------------------ | ---------------------------------------------------------------------------------------------------------------- |
| Linear process flow       | bounded executable | Start/end events, bounded start waits, sequence flow, and deterministic tokens.                                  |
| Host-dispatched tasks     | bounded executable | Service, send, script, business-rule, user, and manual task families.                                            |
| Human interaction         | bounded executable | Native BPMN documentation and IO metadata feed Rust-owned host-work forms.                                       |
| Parallel gateway          | bounded executable | Split and join with deterministic active-token and join-buffer behavior.                                         |
| Exclusive gateway         | bounded executable | Default flow plus simple boolean-path and numeric condition expressions.                                         |
| Inclusive gateway         | bounded executable | Structured split and one matching linear join fragment.                                                          |
| Event-based gateway       | bounded executable | Exclusive competition over message, signal, timer, or conditional catches.                                       |
| Complex gateway           | lint-deferred      | No executable semantics yet.                                                                                     |
| Intermediate catch events | bounded executable | Message, signal, timer, and bounded conditional waits.                                                           |
| Boundary events           | bounded executable | Bounded task owners plus interrupting subprocess-like external boundaries.                                       |
| Error and cancel events   | bounded executable | Bounded subprocess, call-activity, transaction, and top-level error paths.                                       |
| Compensation              | bounded executable | Transaction-owned compensation handlers and throw-compensation paths.                                            |
| Conditional events        | bounded executable | Start events, catches, task boundaries, and interrupting subprocess-like boundaries.                             |
| Escalation events         | bounded executable | Child-scope end/throw routes execute; deferred escalation shapes get diagnostics.                                |
| Import declarations       | metadata-only      | Top-level import declarations are preserved; external dependency resolution is deferred.                         |
| Extension declarations    | metadata-only      | Top-level extension declarations are preserved; extension behavior remains deferred.                             |
| Relationship declarations | metadata-only      | Top-level relationships are preserved; endpoint resolution and graph semantics defer.                            |
| Event definition catalogs | metadata-only      | Message/error/escalation/signal catalogs are preserved, not schema-validated.                                    |
| Interfaces/operations     | metadata-only      | Callable-operation catalogs are preserved; host dispatch binding remains explicit.                               |
| Global task catalogs      | metadata-only      | Top-level global task definitions are preserved; call-activity binding remains deferred.                         |
| Process callable metadata | metadata-only      | Process callable attributes, support refs, properties, and correlation subscriptions are preserved passively.    |
| Resource catalogs         | metadata-only      | Top-level resources and parameters are preserved; assignment binding remains deferred.                           |
| Resource-role metadata    | metadata-only      | Direct process and global-task resource-role declarations are preserved; generic assignment execution defers.    |
| Category catalogs         | metadata-only      | Top-level categories and values are preserved; classification remains passive.                                   |
| Terminate events          | bounded executable | `terminateEventDefinition` end events terminate the current runtime scope.                                       |
| Multiple events           | lint-deferred      | Multiple and parallel-multiple event definitions have stable parser/lint diagnostics.                            |
| Embedded subprocess       | bounded executable | One nested start event and at least one nested end event.                                                        |
| Call activity             | bounded executable | Same-package executable process targets.                                                                         |
| Transaction               | bounded executable | Bounded shell with cancel/error/compensation behavior.                                                           |
| Event subprocess          | lint-deferred      | Deferred, including compensation event subprocesses.                                                             |
| Standard loop             | bounded executable | Supported on selected host-dispatched task families.                                                             |
| Sequential multi-instance | bounded executable | Cardinality and bounded collection-backed input/output subset.                                                   |
| Parallel multi-instance   | bounded executable | Cardinality and bounded collection-backed input/output subset.                                                   |
| Collaboration and pools   | metadata-only      | Participant, partner, endpoint, message-flow, conversation, choreography, and correlation metadata is preserved. |
| Artifacts                 | metadata-only      | Association, group, and text-annotation metadata is preserved without execution semantics.                       |
| Lanes                     | metadata-only      | Preserved for passive routing/display; no scheduling or authorization.                                           |
| Item definitions          | metadata-only      | Top-level item catalogs are preserved; schema validation remains deferred.                                       |
| Data objects              | metadata-only      | Preserved in snapshots; bounded task IO execution is handled separately.                                         |
| Data stores               | lint-deferred      | Persistence semantics require a separate storage policy.                                                         |
| IO specification          | bounded executable | Human-task form IO and bounded host-task Data/IO metadata are executable.                                        |
| Data associations         | bounded executable | Bounded host-task input resolution and output target mapping are executable.                                     |
| BPMN DI                   | metadata-only      | Diagram, plane, shape, edge, bounds, waypoint, label, and font metadata is preserved.                            |
| DMN links                 | bounded executable | Business-rule tasks can execute local bounded DMN decisions when available.                                      |

## Completed M1 Milestone

The first Data/IO milestone promotes bounded task Data/IO execution. Supported
host-dispatched task requests carry resolved task inputs, and task completion
writes outputs through declared BPMN `dataOutputAssociation` targets.

Data-store persistence, transformations, multiple-source associations, and
collaboration-aware routing remain deferred until separate milestones define
their execution contracts.

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
resource roles, and data references. This gives Rust-owned evidence for future
routing and type-alignment work while keeping pool routing, message dispatch,
conversation routing, choreography execution, endpoint invocation, participant
multiplicity execution, global task execution, process support resolution,
process property execution, callable-operation binding, generic resource
assignment execution, resource authorization, delegation, escalation,
scheduling, group execution, annotation interpretation, flow-element
classification, import resolution, extension behavior, extension payload
parsing, diagram rendering, layout validation, relationship endpoint
resolution, event subscription registries, correlation matching, correlation
subscription matching, correlation-key evaluation, retrieval expression
evaluation, and schema validation deferred.

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
merged. Non-interrupting conditional boundaries on subprocess-like owners,
and conditional event subprocess triggers remain deferred.

The sixth event-family slice promotes native `conditionalEventDefinition` as
an exclusive `eventBasedGateway` `intermediateCatchEvent` wait target. Runtime
event competition still uses a single winning branch, and non-ready poll data
can select the first conditional wait whose bounded expression becomes true.
Conditional event subprocess triggers, parallel event-based gateways, and
collaboration-aware correlation remain deferred.

The seventh event-family slice promotes native event definitions on the single
process `startEvent` to bounded executable start waits. The parser accepts one
`messageEventDefinition`, `signalEventDefinition`, `timerEventDefinition`, or
`conditionalEventDefinition`; the runtime creates the instance, blocks at the
start event, and routes after a matching poll outcome or already-satisfied
bounded condition. Multiple start events, event subprocess triggers,
collaboration-aware subscription registries, and multiple event definitions
remain deferred.

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
