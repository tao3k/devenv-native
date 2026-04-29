# BPMN Events and Boundaries

This module records the current bounded event and boundary alignment against
the official [BPMN 2.0.2 specification](https://www.omg.org/spec/BPMN/2.0.2).

## Supported Event Families

The current engine supports these bounded families:

- `intermediateCatchEvent` with exactly one `messageEventDefinition`,
  `signalEventDefinition`, `timerEventDefinition`, or
  `conditionalEventDefinition` with one bounded `condition`
- one process `startEvent` with exactly one `messageEventDefinition`,
  `signalEventDefinition`, `timerEventDefinition`, or
  `conditionalEventDefinition` with one bounded `condition`; runtime treats
  this as created-instance gating, not as a collaboration subscription
  registry
- one interrupting timer, message, signal, or conditional `boundaryEvent`
  attached to one host-blocking task owner
- one interrupting timer, message, signal, or conditional `boundaryEvent`
  attached to one bounded embedded subprocess owner, either alone or paired
  with one or more interrupting error boundaries on that same owner
- one interrupting timer, message, signal, or conditional `boundaryEvent`
  attached to one bounded same-package `callActivity` owner, either alone or
  paired with one or more interrupting error boundaries on that same owner
- one interrupting timer, message, signal, or conditional `boundaryEvent`
  attached to one bounded transaction shell owner, either on its own, paired
  with one or more interrupting error boundaries on that same owner, paired
  with one
  interrupting cancel boundary on that same owner, or paired with one
  interrupting cancel boundary plus one or more interrupting error
  boundaries on that same owner, while still permitting only one
  timer/message/signal/conditional boundary and one cancel boundary on that
  owner
- one non-interrupting timer, message, signal, or conditional `boundaryEvent`
  attached to one non-repeating task owner
- one non-interrupting timer, message, signal, or conditional `boundaryEvent`
  attached to one bounded standard-loop task owner
- one non-interrupting timer, message, signal, or conditional `boundaryEvent`
  attached to one bounded sequential or parallel multi-instance task owner
- one bounded transaction cancel boundary plus one or more bounded transaction
  error boundaries
- one or more bounded interrupting error boundaries on one embedded
  subprocess owner, including the bounded mixed-owner shape with one
  interrupting timer/message/signal/conditional boundary on that same owner
- one or more bounded interrupting error boundaries on one same-package
  `callActivity` owner, including the bounded mixed-owner shape with one
  interrupting timer/message/signal/conditional boundary on that same owner
- one or more bounded interrupting escalation boundaries on one embedded
  subprocess, same-package `callActivity`, or transaction owner, routed from a
  matching escalation end event or intermediate escalation throw event inside
  that child scope
- one bounded top-level `endEvent` with `errorEventDefinition`

## Repeating Owner Note

The current bounded repeating-owner slice keeps the owner-level
non-interrupting boundary armed across both standard-loop re-entry and
sequential multi-instance iteration handoff, while still letting the next
bounded owner execution start immediately.

That matches three BPMN-side expectations:

- standard-loop execution may re-enter the same activity after one completed
  iteration
- sequential multi-instance execution generates the next inner instance only
  after the previous instance completes
- non-interrupting boundary handling exists to open concurrent flow without
  cancelling the enclosing activity

The BPMN issue tracker also records that the multi-instance design was created
with non-interrupting catching boundaries as an intended use case; see
[OMG issue BPMN2-295](https://issues.omg.org/issues/BPMN2-295).

## Deferred Event Families

These shapes remain outside the bounded surface:

- multiple and parallel-multiple event families
- root-level escalation ends or throws, non-interrupting escalation
  boundaries, escalation start events, and escalation event subprocess
  triggers
- non-interrupting conditional boundaries on subprocess-like owners and
  conditional event subprocess triggers
- event subprocesses
- broader message, signal, timer, or conditional boundary families on
  subprocess-like owners beyond one interrupting embedded subprocess owner that may
  optionally pair that boundary with one or more interrupting error
  boundaries, or beyond one interrupting same-package `callActivity` owner
  that may optionally pair that boundary with one or more interrupting error
  boundaries, or beyond one interrupting transaction shell owner that may
  optionally pair that boundary with one interrupting cancel boundary, one
  or more interrupting error boundaries, or both on that same owner, while
  still rejecting more than one timer/message/signal/conditional boundary or
  more than one cancel boundary on that same owner
- broader non-interrupting boundaries on subprocess-like owners
- broader message correlation, collaboration-aware messaging, and pooling
- full timer semantics beyond the current bounded wait shell
