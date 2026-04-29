# Tasks and Host Dispatch

This module tracks the bounded BPMN task families that
`qianji-bpmn-engine` currently accepts and how they map onto the runtime
host seam.

The field-level user/manual task contract is tracked in the
[Host Request ABI Ledger](host-request-abi-ledger.md).

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
- Supported host-dispatched tasks may carry bounded native BPMN task Data/IO:
  `dataInputAssociation` resolves request `inputs`, and
  `dataOutputAssociation` declares completion fields plus target workflow
  variable mappings.
- `businessRuleTask` may execute locally when one matching DMN decision is
  already available inside the parsed package; otherwise it also falls back
  to the host seam.
- Top-level BPMN `interface` and nested `operation` declarations are preserved
  in document snapshots as callable-operation metadata. The current runtime
  does not resolve service bindings from those catalogs; supported task
  dispatch remains driven by the explicit task node and host-work metadata.
- `userTask` and `manualTask` host requests carry the engine-owned
  `process_id`, BPMN `activity_id`, `token_id`, `node_index`, and workflow
  variables. UI and CLI adapters must treat those fields as the canonical
  waiting-work identity instead of reconstructing activity identity from
  labels, list position, or display text.
- `userTask` and `manualTask` requests may also carry optional `form` metadata
  parsed from bounded native BPMN IO metadata. The current form contract
  preserves interaction type, question references or inline prompt text,
  dynamic or inline choices, free-text fields, and the primary result output
  variable for host rendering.
- `userTask` and `manualTask` requests may also carry optional `assignment`
  metadata parsed from standard BPMN `humanPerformer` and `potentialOwner`
  resource roles. The current contract preserves role names, `resourceRef`
  text, and `resourceAssignmentExpression/formalExpression` text as host
  routing metadata only.
- `userTask` and `manualTask` requests may also carry optional BPMN `lane`
  membership metadata parsed from `laneSet/lane/flowNodeRef`. The current
  contract preserves lane-set id/name and lane id/name as passive host routing
  and display metadata only.
- `userTask` and `manualTask` requests may also carry optional `claim`
  metadata when checkpointed pending host work has been allocated to one
  claimant. This is runtime allocation metadata for host coordination, not
  BPMN assignment authorization.
- When a pending user/manual task is claimed, typed completion requests must
  carry the same claimant before qianji resumes the workflow. Unclaimed human
  work remains completable without a claimant because there is no checkpointed
  owner to validate.
- A claimed user/manual task can be released by the same claimant so it
  returns to unclaimed worklist behavior. Different-claimant release and
  release of unclaimed work fail explicitly.
- User/manual task creation, claim, release, and successful completion append
  checkpointed `human_task_events` after the state change succeeds. The ledger
  stores task identity, work kind, optional claimant, and optional work id; it
  is a required field in the current checkpoint API, does not store submitted
  completion payload data, and does not implement task-listener or interceptor
  callbacks.
- Completion data for host-dispatched work must be a JSON object whose fields
  are declared by Rust-owned `output_bindings`. The primary human answer field
  is usually the BPMN `dataOutput` name `answer`; the
  `dataOutputAssociation targetRef` maps that answer into workflow variables.
  Undeclared completion fields are rejected before variable merge.
- Control-service worklist output is derived from checkpointed user/manual
  `PendingHostWork` entries. Optional claimant filtering returns unclaimed
  work plus work already claimed by the same claimant. Optional
  assignment-resource filtering matches preserved `humanPerformer` and
  `potentialOwner` role names or `resourceRef` values exactly. Optional lane
  filtering matches preserved BPMN lane id or lane name exactly. These filters
  are passive routing selectors, not authorization, scheduling, participant
  resolution, or full BPMN resource-role expression evaluation.
- `PendingHostWork` persists the same BPMN `activity_id` for newly blocked
  work. Legacy checkpoints without that field remain readable, but fresh
  engine output is activity-identity complete.

## Deferred Scope

- in-engine script execution
- correlations and broader collaboration-aware message routing
- service interface binding, operation invocation resolution, and external
  callable contract validation
- signal or timer task-event execution on `sendTask` or `receiveTask`
- broader data-object/data-store execution, IO transformations,
  multiple-source data associations, authorization, lane scheduling, full
  task-assignment semantics, delegation, administrative reassignment, and BPMN
  resource-role expression resolution
- native BPMN `rendering` execution for `userTask`; use bounded
  native BPMN IO metadata for executable form rendering in the current
  slice

The linter reports BPMN data-object, data-store, IO transformations, and
unsupported data association shapes as explicit deferred execution semantics.
Use bounded task-local BPMN Data/IO, JSON workflow variables, host-work payload
fields, or DMN inputs for executable data exchange in the current bounded
slice.
The linter also reports native BPMN `rendering`, generic `performer` or
`resourceRole`, `participantRef`, and `resourceParameterBinding` usage as
explicit deferred human-task interaction semantics. Keep current human-task
assignment metadata to `humanPerformer` or `potentialOwner` routing hints with
simple `resourceRef` or `resourceAssignmentExpression/formalExpression` text.
