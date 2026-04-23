# Tasks and Host Dispatch

This module tracks the bounded BPMN task families that
`qianji-bpmn-engine` currently accepts and how they map onto the runtime
host seam.

## Accepted Task Shapes

- `serviceTask`, `userTask`, `manualTask`, and `businessRuleTask` remain
  host-blocking task owners in the bounded runtime slice.
- `sendTask` is accepted when it carries exactly one message binding through
  task-level `messageRef` or one nested `messageEventDefinition`.
- `receiveTask` is accepted when it carries exactly one message binding
  through task-level `messageRef` or one nested `messageEventDefinition`.
- `scriptTask` is accepted as one host-dispatched task family that preserves
  one optional `scriptFormat` attribute and one optional nested
  `<bpmn:script>` body.

## Runtime Contract

- `receiveTask` reuses the engine-owned wait shell.
- `sendTask` reuses the engine-owned host-dispatch shell and preserves
  message metadata in the pending request.
- `scriptTask` reuses that same host-dispatch shell family and preserves
  bounded script metadata in the pending request.
- `businessRuleTask` may execute locally when one matching DMN decision is
  already available inside the parsed package; otherwise it also falls back
  to the host seam.

## Deferred Scope

- in-engine script execution
- correlations and broader collaboration-aware message routing
- signal or timer task-event execution on `sendTask` or `receiveTask`
- broader data-object, IO-specification, and full task-assignment semantics
