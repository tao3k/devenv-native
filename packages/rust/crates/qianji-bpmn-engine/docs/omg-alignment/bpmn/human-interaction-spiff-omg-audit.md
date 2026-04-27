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
- Bounded `qianji:interaction` metadata is parsed into Rust form metadata and
  surfaced through requests, stream JSON, HTTP snapshots, and CLI text output.
- Completion data for form-backed user/manual tasks is validated before
  variable merge.
- Standard BPMN `humanPerformer` and `potentialOwner` metadata is parsed and
  surfaced as routing metadata.
- The linter now reports native BPMN `rendering` and assignment semantics
  beyond routing metadata as explicit deferred human-task semantics.
- Pending user/manual work can carry checkpointed `claim` metadata, and the
  qianji control service can derive a bounded worklist from persisted pending
  host work.
- Claimed pending user/manual work must be completed by the same claimant on
  the typed completion surface before qianji resumes the workflow.
- The same claimant can release claimed pending user/manual work, returning it
  to unclaimed worklist behavior.

This matches SpiffWorkflow's strongest practice: the engine owns state
progression and exposes human work as a task boundary; the application only
renders, collects data, updates the task payload, and asks the engine to
continue.

## Explicitly Diagnosed Gaps

### 1. Native BPMN Rendering Element

OMG BPMN has a `renderings` hook for `userTask`. Qianji currently supports
`qianji:interaction` as the bounded form contract but does not parse native
`rendering` elements. This is acceptable for current execution, but the audit
should record it as a deliberate extension-first choice.

Current status: the linter reports native `rendering` elements as deferred
rendering semantics and directs executable form metadata to `qianji:interaction`.

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

### 3. Manual Task Runtime Semantics

OMG distinguishes `manualTask` as unmanaged by a process runtime. Qianji
intentionally exposes manual tasks as pending host work because this engine is
used as an operator interaction coordinator. The important constraint is to
avoid pretending that a manual task has runtime-managed assignment semantics.

Current status: manual tasks may carry Qianji runtime claim metadata because
they are operator-visible pending host work in this engine. Treat that claim as
host coordination over an acknowledgement/result boundary, not as BPMN-managed
manual-task assignment.

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

Recommended next step: defer nested output envelopes until a real form needs
them. Do not weaken the current declared-field validation.

## Good Practices To Adopt

1. Treat human-task rendering as host responsibility over engine-owned pending
   work.
2. Keep task identity stable and explicit across checkpoint resume.
3. Advance automatically only through non-human engine tasks.
4. After human completion, immediately continue the runtime loop until the next
   human boundary, wait, completion, suspension, or failure.
5. Preserve standard BPMN resource roles even before full worklist semantics
   exist.
6. Make unsupported standard surfaces visible through lint/deferred-semantics
   diagnostics.
7. Keep custom form extensions bounded and typed instead of allowing arbitrary
   UI interpretation to become runtime authority.

## Implementation Direction

The next implementation slice should still not move to UI. Native rendering and
full assignment semantics are explicit diagnostics, and bounded claim/worklist
state plus completion-time claimant enforcement and same-claimant release are
Rust-owned. The next Rust-side candidate is nested form-output contracting when
a concrete form requirement appears, or downstream adapter simplification once
the Rust surfaces are accepted.
