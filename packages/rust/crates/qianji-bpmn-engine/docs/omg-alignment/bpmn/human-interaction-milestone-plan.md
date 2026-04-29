# Human Interaction Milestone Plan

This plan turns the SpiffWorkflow and OMG BPMN human-interaction audit into a
milestone-driven implementation roadmap. It is the canonical table for future
Qianji human-task alignment work.

## Source Basis

### OMG BPMN 2.0.2

OMG BPMN 2.0.2 is the normative source. The official specification inventory
publishes both the formal PDF and machine-readable schemas, including
`Semantic.xsd`.

The standard model facts relevant to this plan are:

1. `userTask` and `manualTask` are distinct human-involvement task families.
2. `userTask` and `globalUserTask` can own `rendering`.
3. `manualTask` and `globalManualTask` do not own `rendering` in the schema.
4. `resourceRole` is the broad standard assignment family; `performer`,
   `humanPerformer`, and `potentialOwner` are narrower role surfaces.
5. `resourceAssignmentExpression`, `resourceRef`, `resourceParameterBinding`,
   and `participantRef` imply broader assignment/resource semantics than
   Qianji's current routing-metadata subset.

### SpiffWorkflow

SpiffWorkflow is the implementation reference for host-loop discipline, not a
normative standard. The useful practice is:

1. run engine-owned `READY` tasks that do not require manual input;
2. expose `READY` human tasks to the application;
3. let the application render prompts/forms and update task data;
4. run the human task after data is supplied;
5. refresh waiting/event tasks and continue until the next boundary.

SpiffWorkflow marks both BPMN user and manual task specs as manual host work.
Its documentation treats form rendering and instruction rendering as
application responsibilities, while workflow state progression remains engine
owned.

## Qianji Alignment Principle

Qianji should follow this split:

- OMG defines the standard shape and vocabulary.
- SpiffWorkflow demonstrates the engine/host boundary discipline.
- `qianji-bpmn-engine` owns parsing, linting, runtime state, pending host-work
  identity, completion validation, claim state, and worklist derivation.
- `xiuxian-qianji` owns control-service, HTTP, stream, and CLI transport over
  that Rust engine state.
- pi-wendao and other adapters render Rust-provided pending work and submit
  typed completion payloads. They must not infer BPMN scheduling, task identity,
  output mapping, or standard assignment semantics locally.

## Alignment Matrix

| Source point                                                                        | Current Qianji status                                                                                                                                                                                                                                                                                                                                                                                | Remaining gap                                                                           | Governing milestone              | Evidence gate                                                                                              |
| ----------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | -------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| SpiffWorkflow runs non-human engine tasks until a human or wait boundary            | Implemented through Rust advance/session behavior plus a qianji stream smoke proving pending host-work JSON mirrors runtime state after automatic engine work                                                                                                                                                                                                                                        | None for M1; broader transport field parity moves to M2                                 | M1: Host Loop Conformance        | Runtime integration test and stream smoke with no adapter graph inference                                  |
| SpiffWorkflow exposes human tasks as application-rendered work                      | Implemented: `UserTaskRequest` and `ManualTaskRequest` expose process, activity, token, node, variables, optional form, assignment, lane, and claim; a host request ABI ledger records the field contract                                                                                                                                                                                            | Need remaining parity tests for every host-facing transport and adapter negative checks | M2: Host Request ABI Ledger      | Parser/runtime/HTTP/stream/CLI contract table and focused regression tests                                 |
| SpiffWorkflow lets applications update task data before running the task            | Implemented for M3: form-backed completion data is a flat declared-field object; optional free text may be omitted; non-object payloads and nested envelopes are rejected before workflow advancement                                                                                                                                                                                                | Nested form-output envelope remains deferred                                            | M3: Task Data Shape              | Control-service task-complete tests for flat-only and nested-envelope rejection                            |
| OMG `userTask/globalUserTask` may carry `rendering`                                 | Implemented: both local and global user-task rendering report deferred native rendering; executable rendering uses native BPMN IO metadata                                                                                                                                                                                                                                                           | None for M4                                                                             | M4: Native Rendering Boundary    | Linter tests for userTask and globalUserTask, plus repair guidance                                         |
| OMG `manualTask/globalManualTask` do not own `rendering`                            | Implemented: both local and global manual-task rendering misuse report explicit invalid-standard-surface diagnostics                                                                                                                                                                                                                                                                                 | None for M4                                                                             | M4: Native Rendering Boundary    | Linter tests for manualTask and globalManualTask rendering                                                 |
| OMG resource roles include broad assignment semantics                               | Implemented for the bounded routing slice: `humanPerformer` and `potentialOwner` are preserved as routing metadata; `performer`, generic `resourceRole`, `participantRef`, and `resourceParameterBinding` are linted as deferred standard assignment semantics                                                                                                                                       | No standard role authorization, escalation, delegation, or reassignment                 | M5: Assignment Boundary          | Parser/host tests preserve routing metadata; linter tests reject broad assignment semantics                |
| SpiffWorkflow task identity uses unique task instances plus task specs              | Implemented for M6: pending work and completion carry `token_id`, `node_index`, `process_id`, and `activity_id`; control-service and HTTP RuntimeValkey checkpoint replay prove the same identity survives claim, release, status/worklist, wrong-claimant rejection, and same-claimant completion; checkpointed `human_task_events` now record created, claimed, released, and completed milestones | None for M6; routing policy moves to M7                                                 | M6: Identity and Claim Lifecycle | Checkpoint round-trip tests over claim, release, completion, HTTP snapshots, and lifecycle-event summaries |
| SpiffWorkflow supports lane-based ready-task filtering in host code                 | Implemented for M7 policy: Qianji worklists keep claimant filtering and add Rust-owned passive assignment-resource and lane filtering over preserved `humanPerformer`, `potentialOwner`, and BPMN `lane` metadata; lane scheduling and authorization remain deferred                                                                                                                                 | None for M7; executable lane scheduling and authorization remain deferred               | M7: Worklist Routing Policy      | Control-service and CLI tests for claimant, passive assignment-resource filters, and passive lane filters  |
| SpiffWorkflow forms are application-specific and may use JSON schemas               | Implemented for the bounded M8 slice: Qianji supports a fixed native BPMN IO metadata catalog, one question source, one choices source family, and at most one supplemental free-text field                                                                                                                                                                                                          | Arbitrary JSON schema execution and nested completion envelopes remain deferred         | M8: Form Schema Boundary         | Linter tests for supported interaction types and rejected ambiguous schema shapes                          |
| OMG global human tasks are callable/root definitions, not ordinary process children | Implemented for the bounded M9 policy: top-level `globalUserTask` and `globalManualTask` remain non-executable metadata, and `callActivity calledElement` bindings to those ids lint as unsupported global human-task runtime dependencies                                                                                                                                                           | Future executable global human-task reuse needs a Rust-owned callable binding           | M9: Global Human Task Policy     | Linter tests for callActivity-to-globalUserTask/globalManualTask rejection                                 |
| Adapters should not replay or reinterpret BPMN graphs                               | Implemented for the bounded M10 adapter milestone: pi-wendao renders from Rust-streamed form metadata, forwards streamed assignment and claim metadata, keeps missing optional assignment/claim absent, and runs a generated-BPMN smoke without local XML fallback                                                                                                                                   | None for bounded M10                                                                    | M10: Adapter Conformance         | skillsc/pi-wendao tests using only Rust-streamed form/assignment/claim data                                |

## Milestone Roadmap

### M1: Host Loop Conformance

Goal: prove Qianji follows the same engine/host loop discipline that makes
SpiffWorkflow robust.

Deliverables:

1. one runtime fixture with automatic work, a human boundary, a wait/event
   refresh, and a second boundary or completion;
2. focused runtime tests for `advance -> pending human -> complete -> advance`;
3. a CLI or stream smoke that shows adapters receive only pending host-work
   payloads and never recompute graph state.

Current status: implemented. The Rust runtime fixture proves
engine-owned advance, human completion, wait refresh, and final completion. The
qianji stream smoke proves `@@QIANJI_HOST_WORK` mirrors Rust pending
human-work identity and form metadata after automatic gateway work.

Exit criteria:

- a single command proves the loop on Rust state;
- no UI or adapter fallback is required for activity identity, prompt schema,
  or output mapping.

### M2: Host Request ABI Ledger

Goal: make every user/manual host-work field auditable across Rust, HTTP,
stream, CLI text, and downstream adapter usage.

Deliverables:

1. package doc table for `process_id`, `activity_id`, `token_id`,
   `node_index`, `variables`, `form`, `assignment`, `lane`, `claim`, and repeat
   context;
2. focused tests for each transport preserving the same fields;
3. adapter checks that reject missing form/result metadata instead of falling
   back to local XML inference.

Current status: the canonical
[Host Request ABI Ledger](host-request-abi-ledger.md) is present, and HTTP
snapshot, stream, CLI start/status, and CLI worklist wire or text fields are
covered by focused tests. pi-wendao now has contract and executor tests that
reject missing streamed `form`, `result_output`, or output-binding metadata
before user interaction starts. M2 is complete for the bounded ABI ledger; adapter
assignment, claim, and generated BPMN conformance are covered by M10.

Exit criteria:

- field parity is proven by tests or snapshots;
- missing host ABI fields fail before user interaction starts.

### M3: Task Data Shape

Goal: decide and lock how submitted form data becomes workflow variables.

Current direction: keep the flat declared-field merge because it is already
validated and operationally simple. Introduce nested output envelopes only when
a concrete form requirement needs it.

Current status: implemented for the bounded M3 task-data shape slice.
Form-backed user/manual completion data must be a JSON object. Its top-level
keys must be declared by Rust-owned output bindings. The required BPMN
`dataOutput` name, usually `answer`, must be present; optional `freeText` data
input fields may be omitted, undeclared keys are rejected, and nested
form-output envelopes are not a compatibility path.

Deliverables:

1. decision record inside the alignment docs;
2. tests that preserve current flat-field validation;
3. if nested envelopes are approved later, an additive contract with explicit
   linter and completion validation.

Exit criteria:

- downstream adapters cannot invent output keys;
- every completion output is declared in Rust-owned form metadata.

### M4: Native Rendering Boundary

Goal: keep OMG native `rendering` visible without making it executable until
Qianji has a deliberate native-rendering design.

Deliverables:

1. linter coverage for `userTask` and `globalUserTask` native rendering as
   deferred;
2. linter coverage for `manualTask` and `globalManualTask` rendering misuse as
   invalid for the executable slice;
3. repair guidance that moves executable UI metadata to native BPMN IO metadata.

Current status: implemented for the bounded executable slice. `userTask` and
`globalUserTask` native `rendering` report deferred native rendering;
`manualTask` and `globalManualTask` rendering report invalid standard-surface
usage. Executable prompts remain bound to native BPMN IO metadata.

Exit criteria:

- no BPMN source can rely on native `rendering` and still appear executable in
  the current Qianji bounded runtime.

### M5: Assignment Boundary

Goal: preserve standard assignment hints without pretending to implement full
standard assignment semantics.

Deliverables:

1. parser preservation for `humanPerformer` and `potentialOwner` routing hints;
2. linter diagnostics for `performer`, generic `resourceRole`,
   `participantRef`, and `resourceParameterBinding`;
3. worklist docs that keep claim/worklist separate from BPMN authorization.

Current status: implemented for the bounded routing-metadata slice. Qianji
preserves `humanPerformer` and `potentialOwner` with simple `resourceRef` or
`resourceAssignmentExpression/formalExpression` as passive routing hints.
Generic `performer`, generic `resourceRole`, `participantRef`, and
`resourceParameterBinding` are reported as deferred standard assignment
semantics. Authorization, delegation, reassignment, escalation, and participant
resolution remain deferred Rust-owned contracts.

Exit criteria:

- routing metadata survives to host requests;
- authorization, reassignment, delegation, and participant resolution stay
  explicitly deferred unless implemented in Rust.

### M6: Identity and Claim Lifecycle

Goal: make human work durable and unambiguous across checkpointed execution.

Deliverables:

1. checkpoint round-trip tests for pending human work identity;
2. claim, release, and same-claimant completion tests across control service
   and HTTP;
3. worklist tests for unclaimed work and same-claimant visibility.

Current status: implemented for the bounded M6 slice. Control-service
checkpoint replay proves that claim, release, claimant-filtered worklist,
wrong-claimant completion rejection, and same-claimant completion preserve the
original `instance_id`, `token_id`, `process_id`, and `activity_id` tuple
across fresh service loads. HTTP RuntimeValkey checkpoint replay now proves
the same identity lifecycle through host-facing status, claim, release, wrong
claimant rejection, and same-claimant completion. The runtime also stores a
checkpointed `human_task_events` ledger for user/manual created, claimed,
released, and completed milestones. That ledger is exposed through workflow
snapshots and compact CLI summaries while the execution trace remains
node/flow oriented. The current checkpoint API requires the ledger field and
rejects missing-field checkpoints instead of backfilling legacy state.

Exit criteria:

- a claimed pending human task cannot be completed or released by the wrong
  claimant;
- unclaimed work remains completable through typed identity.

### M7: Worklist Routing Policy

Goal: define deterministic Rust-owned worklist filtering without turning
standard BPMN assignment or lane metadata into authorization.

Deliverables:

1. policy decision for claimant, assignment, and lane filters;
2. tests for the selected filter semantics;
3. clear docs that these filters do not equal authorization.

Current status: implemented for the bounded M7 policy slice. Worklists support
the existing claimant filter, a passive `assignment_resource` filter that
matches preserved `humanPerformer` and `potentialOwner` role names or
`resourceRef` values exactly after trimming, and a passive `lane` filter that
matches preserved BPMN lane id or lane name exactly after trimming. These
filters compose by intersection and do not authorize claim, release, or
completion. Lane and lane-set metadata remain non-executable metadata because
the bounded runtime does not bind lanes into scheduling or authorization.

Exit criteria:

- worklist filtering is deterministic and Rust-owned;
- UI cannot hide or expose work by inventing assignment or lane logic.

### M8: Form Schema Boundary

Goal: keep interaction rendering bounded while allowing future structured form
growth.

Deliverables:

1. linter catalog for supported native BPMN IO metadata types and attributes;
2. rejection tests for ambiguous question/choices references and unsupported
   free-text cardinality;
3. optional future nested-envelope design before implementation.

Current status: implemented for the bounded M8 lint slice. The executable
interaction catalog is `input`, `confirm`, `choice`, and `choice_input`. A
question has exactly one source: inline text, a `text` attribute, or a dynamic
`ref`. Choice interactions use either one dynamic choices data input `sourceRef` or inline
choices JSON literal item `value` entries, not both. The current flat completion ABI supports
at most one supplemental `freeText` data input field per interaction. Richer
multi-field forms require a future Rust-owned nested envelope before execution.

Exit criteria:

- every supported form shape has deterministic host rendering inputs;
- unsupported schemas fail through lint or completion validation.

### M9: Global Human Task Policy

Goal: decide how Qianji treats OMG global human task definitions.

Deliverables:

1. parser/linter policy for `globalUserTask` and `globalManualTask`;
2. callActivity/global task binding decision if executable global task reuse is
   needed;
3. diagnostics for unsupported global-task runtime dependencies.

Exit criteria:

- global human tasks are either explicitly non-executable metadata or have a
  Rust-owned callable binding.

Current status: implemented for the bounded M9 lint slice. Qianji treats
top-level `globalUserTask` and `globalManualTask` definitions as
non-executable metadata when they are not used as runtime bindings.
`callActivity calledElement` remains process-only: it must target another
executable process in the same BPMN package, not a global human task id. If
model authors need reusable executable human work now, they should wrap the
human task in an executable process or keep it as a process-local
`userTask`/`manualTask` with the bounded native BPMN IO metadata contract. A direct
global human-task callable binding remains deferred until Rust owns that
binding explicitly.

### M10: Adapter Conformance

Goal: make downstream interactive adapters thin and testable.

Deliverables:

1. adapter tests that consume only `@@QIANJI_HOST_WORK`, HTTP snapshots, or
   typed control-service responses;
2. negative tests for missing form, result output, assignment, and claim data;
3. a real generated BPMN smoke that proves no local graph/rendering fallback.

Exit criteria:

- pi-wendao and skillsc do not parse BPMN XML for interaction semantics during
  execution;
- all user-visible prompts are backed by Rust-owned pending host work.

Current status: complete for the bounded M10 adapter-conformance milestone.
pi-wendao rejects missing Rust-streamed `form`, `result_output`, and output
bindings before rendering. It exposes streamed `claim` metadata to the human-task handler,
forwards the same claimant on typed completion, and proves that missing
optional `assignment` or `claim` metadata remains absent rather than being
synthesized from local BPMN XML or stale config. A generated-BPMN smoke also
proves that local native BPMN IO metadata is not an execution-time
fallback when qianji streams a different form, result output, assignment, and
claim.

## Execution Order

| Order | Milestone                       | Why first                                                    | Primary owner                          |
| ----- | ------------------------------- | ------------------------------------------------------------ | -------------------------------------- |
| 1     | M1 Host Loop Conformance        | Establishes the runtime shape before adding more metadata    | `qianji-bpmn-engine`                   |
| 2     | M2 Host Request ABI Ledger      | Prevents field drift across stream, HTTP, CLI, and adapters  | `qianji-bpmn-engine`, `xiuxian-qianji` |
| 3     | M4 Native Rendering Boundary    | Closes standard rendering ambiguity before UI work resumes   | `qianji-bpmn-engine` linter            |
| 4     | M5 Assignment Boundary          | Keeps routing metadata and authorization semantics separated | `qianji-bpmn-engine`, `xiuxian-qianji` |
| 5     | M6 Identity and Claim Lifecycle | Hardens durable operator coordination                        | `qianji-bpmn-engine`, `xiuxian-qianji` |
| 6     | M3 Task Data Shape              | Requires stable ABI and completion evidence first            | `qianji-bpmn-engine`                   |
| 7     | M7 Worklist Routing Policy      | Depends on assignment and claim boundaries                   | `xiuxian-qianji`                       |
| 8     | M8 Form Schema Boundary         | Expands only after current form ABI is locked                | `qianji-bpmn-engine` linter/runtime    |
| 9     | M9 Global Human Task Policy     | Lower frequency; should follow core task stability           | `qianji-bpmn-engine` parser/linter     |
| 10    | M10 Adapter Conformance         | Final proof that adapters remain thin                        | skill runtime and pi-wendao adapters   |

## Next Slice Recommendation

M10 is complete for the bounded adapter-conformance milestone, and the M6
human-task lifecycle ledger is now in place. The current interaction path is
locked to Rust-owned host-work form, assignment, lane, claim, completion
identity, and checkpointed lifecycle events, including a generated-BPMN smoke that
prevents adapter regression back to local XML fallback. Future work should
start only from a concrete new form schema, native rendering, global
human-task callable binding, or standard assignment requirement.
