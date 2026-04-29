# BPMN Gateways and Concurrency

This module records the current bounded gateway and in-instance concurrency
alignment against the official
[BPMN 2.0.2 specification](https://www.omg.org/spec/BPMN/2.0.2).

## Supported Gateway Families

The current engine supports these gateway families:

- `parallelGateway` as a split that creates multiple active runtime tokens and
  as a join that waits for one arrival on every incoming sequence flow.
- `exclusiveGateway` with one optional `default` outgoing flow and bounded
  sequence-flow conditions over simple boolean paths or numeric comparisons.
- structured `inclusiveGateway` with the same bounded condition/default subset
  plus one linear matching join fragment.
- exclusive `eventBasedGateway` whose outgoing targets are
  `intermediateCatchEvent` nodes with exactly one message, signal, timer, or
  conditional event definition.

## Runtime Guarantees

Within that bounded slice, the runtime guarantees:

- deterministic active-token order under the single-writer checkpoint model
- explicit frontier snapshots and runnable proposal collection for every
  active token
- conflict-aware same-node parallel-join merge for deterministic join
  arrivals
- edge-aware join buffering so duplicate arrivals on the same incoming flow do
  not fire a parallel join early
- preservation of excess parallel-join arrivals instead of deleting them when
  one activation fires
- deterministic event-competition ownership where one event-based gateway
  winner cancels the losing waits
- conditional event-based gateway wait targets are selected by the runtime
  when poll data satisfies the bounded condition expression
- optional Rayon-backed immutable frontier inspection without parallel mutable
  state writes

## Deferred Gateway Semantics

These gateway shapes remain outside the bounded surface:

- `complexGateway`
- unstructured inclusive joins
- parallel event-based gateway instantiation semantics
- broader FEEL or script-backed gateway conditions
- collaboration-aware message correlation for event-based gateway routing
- multi-writer checkpoint execution for one workflow instance

## Alignment Notes

Gateway concurrency is semantic concurrency inside one workflow instance. It
does not imply multiple distributed checkpoint writers. `qianji-bpmn-engine`
keeps one writer owner for checkpoint truth while allowing that writer to
materialize multiple active tokens, waits, and join states in one deterministic
instance state.
