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

| BPMN family               | Current status     | Current boundary                                                                        |
| ------------------------- | ------------------ | --------------------------------------------------------------------------------------- |
| Linear process flow       | bounded executable | Start/end events, sequence flow, and deterministic token advancement.                   |
| Host-dispatched tasks     | bounded executable | Service, send, script, business-rule, user, and manual task families.                   |
| Human interaction         | bounded executable | Native BPMN documentation and IO metadata feed Rust-owned host-work forms.              |
| Parallel gateway          | bounded executable | Split and join with deterministic active-token and join-buffer behavior.                |
| Exclusive gateway         | bounded executable | Default flow plus simple boolean-path and numeric condition expressions.                |
| Inclusive gateway         | bounded executable | Structured split and one matching linear join fragment.                                 |
| Event-based gateway       | bounded executable | Exclusive competition over message, signal, or timer intermediate catches.              |
| Complex gateway           | lint-deferred      | No executable semantics yet.                                                            |
| Intermediate catch events | bounded executable | Message, signal, timer, and bounded conditional waits.                                  |
| Boundary events           | bounded executable | Bounded interrupting and non-interrupting timer/message/signal/conditional task owners. |
| Error and cancel events   | bounded executable | Bounded subprocess, call-activity, transaction, and top-level error paths.              |
| Compensation              | bounded executable | Transaction-owned compensation handlers and throw-compensation paths.                   |
| Conditional events        | bounded executable | Intermediate catches and task boundaries with bounded boolean or numeric conditions.    |
| Escalation events         | bounded executable | End-to-interrupting-boundary routing on bounded subprocess-like owners.                 |
| Terminate events          | bounded executable | `terminateEventDefinition` end events terminate the current runtime scope.              |
| Multiple events           | lint-deferred      | Multiple and parallel-multiple event families are deferred.                             |
| Embedded subprocess       | bounded executable | One nested start event and at least one nested end event.                               |
| Call activity             | bounded executable | Same-package executable process targets.                                                |
| Transaction               | bounded executable | Bounded shell with cancel/error/compensation behavior.                                  |
| Event subprocess          | lint-deferred      | Deferred, including compensation event subprocesses.                                    |
| Standard loop             | bounded executable | Supported on selected host-dispatched task families.                                    |
| Sequential multi-instance | bounded executable | Cardinality and bounded collection-backed input/output subset.                          |
| Parallel multi-instance   | bounded executable | Cardinality and bounded collection-backed input/output subset.                          |
| Collaboration and pools   | metadata-only      | Participant and message-flow structures are preserved, not routed.                      |
| Lanes                     | metadata-only      | Preserved for passive routing/display; no scheduling or authorization.                  |
| Data objects              | metadata-only      | Preserved in snapshots; bounded task IO execution is handled separately.                |
| Data stores               | lint-deferred      | Persistence semantics require a separate storage policy.                                |
| IO specification          | bounded executable | Human-task form IO and bounded host-task Data/IO metadata are executable.               |
| Data associations         | bounded executable | Bounded host-task input resolution and output target mapping are executable.            |
| BPMN DI                   | metadata-only      | Diagram metadata is not runtime behavior.                                               |
| DMN links                 | bounded executable | Business-rule tasks can execute local bounded DMN decisions when available.             |

## Completed M1 Milestone

The first Data/IO milestone promotes bounded task Data/IO execution. Supported
host-dispatched task requests carry resolved task inputs, and task completion
writes outputs through declared BPMN `dataOutputAssociation` targets.

Data-store persistence, transformations, multiple-source associations, and
collaboration-aware routing remain deferred until separate milestones define
their execution contracts.

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
Conditional start events and conditional event subprocesses remain deferred.

The third event-family slice promotes native `escalationEventDefinition` for
bounded subprocess-scope end events. The runtime routes a thrown escalation to
matching interrupting escalation boundary events on the parent embedded
subprocess, same-package call activity, or transaction owner. Root-level
escalation ends, non-interrupting escalation boundaries, intermediate throw
escalation, escalation start events, and escalation event subprocess triggers
remain deferred.

The fourth event-family slice promotes native `conditionalEventDefinition` on
task-attached `boundaryEvent` nodes. The same bounded boolean-path and
numeric-comparison expression subset is used for interrupting and
non-interrupting task boundaries. Conditional boundaries on subprocess-like
owners, conditional start events, and conditional event subprocess triggers
remain deferred.
