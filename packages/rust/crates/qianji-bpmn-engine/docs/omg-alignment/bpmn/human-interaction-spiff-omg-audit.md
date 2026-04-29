# Human Interaction Alignment Audit

This note records the current Qianji alignment position against OMG BPMN 2.0.2
human interaction semantics and the SpiffWorkflow reference implementation
pattern.

## Source Anchors

- OMG BPMN 2.0.2 is the normative specification source. Its published
  artifacts include the formal PDF and machine-readable `Semantic.xsd`.
- BPMN defines two human-involvement task families: `userTask` and
  `manualTask`.
- `userTask` is runtime-managed and can carry UI rendering hooks, resource
  roles, and task instance attributes such as actual owner and priority.
- The BPMN `Semantic.xsd` model places `rendering` under `userTask` and
  `globalUserTask`, not under `manualTask` or `globalManualTask`.
- `manualTask` represents work outside runtime management; in an executable
  engine, Qianji still exposes it as host-visible pending work so operators can
  acknowledge or record the external action.
- SpiffWorkflow's useful implementation pattern is the execution loop: run
  READY non-manual engine tasks, expose READY manual tasks to an application,
  let the application update task data, run the human task, then refresh
  WAITING tasks.

## Alignment Points Already Covered

Qianji now covers the core runtime contract needed for stable human
interaction:

- `userTask` and `manualTask` block as Rust-owned pending host work.
- Pending host work carries stable `process_id`, `activity_id`, `token_id`,
  and `node_index`.
- Typed completion targets the pending task by process/activity/token/kind
  instead of by display label or list position.
- Bounded native BPMN IO metadata metadata is parsed into Rust form metadata and
  surfaced through requests, stream JSON, HTTP snapshots, and CLI text output.
- Completion data for form-backed user/manual tasks is validated before
  variable merge.
- Standard BPMN `humanPerformer` and `potentialOwner` metadata is parsed and
  surfaced as routing metadata.
- BPMN `laneSet/lane/flowNodeRef` membership is parsed and surfaced as
  passive lane metadata for display and worklist filtering.
- The linter now reports native BPMN `rendering` and assignment semantics
  beyond routing metadata as explicit deferred human-task semantics.
- Pending user/manual work can carry checkpointed `claim` metadata, and the
  qianji control service can derive a bounded worklist from persisted pending
  host work.
- Claimed pending user/manual work must be completed by the same claimant on
  the typed completion surface before qianji resumes the workflow.
- The same claimant can release claimed pending user/manual work, returning it
  to unclaimed worklist behavior.
- Checkpointed `human_task_events` record user/manual created, claimed,
  released, and completed milestones after successful state changes. The
  current checkpoint API requires this ledger field and does not backfill
  checkpoints that omit it. Events do not store submitted completion payload
  data.

This matches SpiffWorkflow's strongest practice: the engine owns state
progression and exposes human work as a task boundary; the application only
renders, collects data, updates the task payload, and asks the engine to
continue.

## Explicitly Diagnosed Gaps

### 1. Native BPMN Rendering Element

OMG BPMN has a `renderings` hook for `userTask`. Qianji currently supports
native BPMN IO metadata as the bounded form contract but does not parse native
`rendering` elements. This is acceptable for current execution, but the audit
should record it as a deliberate extension-first choice.

Current status: the linter reports native `rendering` elements under
`userTask` or `globalUserTask` as deferred rendering semantics and directs
executable form metadata to native BPMN IO metadata. It also reports `rendering`
under `manualTask` or `globalManualTask` as a non-standard, non-executable
manual-task interaction surface instead of letting a downstream UI infer a
form contract.

### 2. Assignment Semantics Beyond Routing Metadata

Qianji preserves `humanPerformer` and `potentialOwner`, including
`resourceRef` and `resourceAssignmentExpression/formalExpression`, but does
not resolve those expressions into authorization. Runtime claim is now a
separate checkpointed allocation surface over pending host work; it does not
make standard BPMN assignment metadata executable.

Current status: the linter reports generic `performer` or `resourceRole`,
`participantRef`, and `resourceParameterBinding` usage as deferred assignment
semantics. Bounded claim/worklist state, completion-time claimant enforcement,
and same-claimant release are modeled as separate runtime surfaces, not as
UI-only filters and not as full WSHumanTask authorization.

### 2.1 Lane Metadata and Filtering

SpiffWorkflow exposes lane metadata on task specs and lets host code filter
ready tasks by lane. Qianji now preserves BPMN lane membership as passive
metadata on user/manual pending host work and allows worklists to filter by
lane id or lane name. This is host routing/display metadata only: it does not
schedule work, authorize claim or completion, resolve participants, or model
escalation, delegation, or reassignment.

### 3. Manual Task Runtime Semantics

OMG distinguishes `manualTask` as unmanaged by a process runtime. Qianji
intentionally exposes manual tasks as pending host work because this engine is
used as an operator interaction coordinator. The important constraint is to
avoid pretending that a manual task has runtime-managed assignment semantics.

Current status: manual tasks may carry Qianji runtime claim metadata because
they are operator-visible pending host work in this engine. Treat that claim as
host coordination over an acknowledgement/result boundary, not as BPMN-managed
manual-task assignment.

Manual-task lifecycle events have the same checkpointed audit shape as
user-task lifecycle events. They record that the operator-visible work was
created, claimed, released, or completed inside Qianji's coordination layer;
they do not turn manual tasks into fully runtime-managed BPMN work.

Manual tasks must not use BPMN `rendering` as a runtime form contract. If the
activity is runtime-managed human input, model it as a `userTask`; if it is
external manual work, keep the executable acknowledgement schema in
native BPMN IO metadata.

### 4. Host Loop Discipline

SpiffWorkflow keeps a clear host loop: engine advances until human input is
required, the host updates task data, then the engine continues. Qianji has the
pieces, but downstream adapters must not re-run graph logic or infer output
mapping locally.

Recommended next step: use the Qianji pending-host stream or HTTP snapshot as
the only adapter input for human-task rendering. Submit completion only through
the typed task-complete surface.

### 5. Task Data Shape

SpiffWorkflow lets the application choose whether form data becomes individual
task-data keys or one nested object. Qianji currently validates declared fields
and merges them into workflow variables. That is simple and operationally
useful, but nested form-output support may be needed later for richer forms.

Current status: Qianji keeps a flat declared-field merge for form-backed
user/manual task completion. Completion data must be a JSON object, required
`result_output` fields must be present, optional free-text fields may be
omitted, undeclared fields are rejected, and nested envelopes are rejected
instead of treated as a compatibility path.

Recommended next step: defer nested output envelopes until a real form needs
them. Do not weaken the current declared-field validation.

## Good Practices To Adopt

1. Treat human-task rendering as host responsibility over engine-owned pending
   work.
2. Keep task identity stable and explicit across checkpoint resume.
3. Advance automatically only through non-human engine tasks.
4. After human completion, immediately continue the runtime loop until the next
   human boundary, wait, completion, suspension, or failure.
5. Preserve standard BPMN resource roles as passive routing hints without
   treating them as authorization.
6. Preserve BPMN lane membership as passive routing/display metadata without
   treating it as scheduling or authorization.
7. Make unsupported standard surfaces visible through lint/deferred-semantics
   diagnostics.
8. Keep custom form extensions bounded and typed instead of allowing arbitrary
   UI interpretation to become runtime authority.

## Implementation Direction

The next implementation slice should still not move to UI. Native rendering and
full assignment semantics are explicit diagnostics, and bounded claim/worklist
state, completion-time claimant enforcement, same-claimant release, passive
assignment-resource and lane worklist routing, and lifecycle-event audit are
Rust-owned. The next Rust-side candidate is nested form-output contracting when
a concrete form requirement appears.
