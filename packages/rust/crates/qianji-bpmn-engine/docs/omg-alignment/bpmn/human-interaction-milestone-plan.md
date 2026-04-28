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

| Source point                                                                        | Current Qianji status                                                                                                                                                                                   | Remaining gap                                                                                | Governing milestone              | Evidence gate                                                                          |
| ----------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | -------------------------------- | -------------------------------------------------------------------------------------- |
| SpiffWorkflow runs non-human engine tasks until a human or wait boundary            | Implemented through Rust advance/session behavior plus a qianji stream smoke proving pending host-work JSON mirrors runtime state after automatic engine work                                           | None for M1; broader transport field parity moves to M2                                      | M1: Host Loop Conformance        | Runtime integration test and stream smoke with no adapter graph inference              |
| SpiffWorkflow exposes human tasks as application-rendered work                      | Implemented: `UserTaskRequest` and `ManualTaskRequest` expose process, activity, token, node, variables, optional form, assignment, and claim; a host request ABI ledger now records the field contract | Need remaining parity tests for every host-facing transport and adapter negative checks      | M2: Host Request ABI Ledger      | Parser/runtime/HTTP/stream/CLI contract table and focused regression tests             |
| SpiffWorkflow lets applications update task data before running the task            | Implemented for typed completion with form-backed declared-field validation and result-output enforcement                                                                                               | Nested form-output envelope remains deferred                                                 | M3: Task Data Shape              | Decision record plus tests for flat-only or nested-envelope behavior before expansion  |
| OMG `userTask/globalUserTask` may carry `rendering`                                 | Diagnosed as deferred native rendering; executable rendering uses `qianji:interaction`                                                                                                                  | Need canonical fixture for global user-task rendering diagnostics                            | M4: Native Rendering Boundary    | Linter tests for userTask and globalUserTask, plus repair guidance snapshots           |
| OMG `manualTask/globalManualTask` do not own `rendering`                            | Implemented: manual-task rendering misuse reports an explicit diagnostic                                                                                                                                | Need globalManualTask coverage if global tasks enter the accepted parser surface             | M4: Native Rendering Boundary    | Linter test or parser-boundary test for globalManualTask rendering                     |
| OMG resource roles include broad assignment semantics                               | Partially implemented: `humanPerformer` and `potentialOwner` are preserved as routing metadata only                                                                                                     | No standard role authorization, escalation, delegation, or reassignment                      | M5: Assignment Boundary          | Tests prove broad roles are linted and routing metadata remains passive                |
| SpiffWorkflow task identity uses unique task instances plus task specs              | Implemented with `token_id`, `node_index`, `process_id`, and `activity_id` on pending work and completion                                                                                               | Need cross-checkpoint replay proof for identity stability through claim/release/complete     | M6: Identity and Claim Lifecycle | Checkpoint round-trip tests over claim, release, completion, and HTTP snapshots        |
| SpiffWorkflow supports lane-based ready-task filtering in host code                 | Qianji currently treats lanes as non-executable metadata                                                                                                                                                | Lane-based worklist filtering is deferred                                                    | M7: Worklist Routing Policy      | Explicit decision: keep claimant-only filtering or add passive lane/assignment filters |
| SpiffWorkflow forms are application-specific and may use JSON schemas               | Qianji intentionally uses bounded `qianji:interaction` form metadata                                                                                                                                    | Dynamic schema references remain bounded; arbitrary JSON schema execution is deferred        | M8: Form Schema Boundary         | Linter tests for supported interaction types and rejected ambiguous schema shapes      |
| OMG global human tasks are callable/root definitions, not ordinary process children | Qianji lints some global task standard surfaces but runtime does not execute global task definitions directly                                                                                           | Need explicit callActivity/globalTask policy if model authors start using global human tasks | M9: Global Human Task Policy     | Parser/linter tests and docs for globalUserTask/globalManualTask handling              |
| Adapters should not replay or reinterpret BPMN graphs                               | pi-wendao no-fallback work removed local interaction inference                                                                                                                                          | Need end-to-end adapter conformance suite tied to the Rust ABI ledger                        | M10: Adapter Conformance         | skillsc/pi-wendao tests using only Rust-streamed form/assignment/claim data            |

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
   `node_index`, `variables`, `form`, `assignment`, `claim`, and repeat
   context;
2. focused tests for each transport preserving the same fields;
3. adapter checks that reject missing form/result metadata instead of falling
   back to local XML inference.

Current status: the canonical
[Host Request ABI Ledger](host-request-abi-ledger.md) is present, and HTTP
snapshot, stream, CLI start/status, and CLI worklist wire or text fields are
covered by focused tests. pi-wendao now has contract and executor tests that
reject missing streamed `form` or `result_output` metadata before user
interaction starts. M2 is complete for the bounded ABI ledger; broader adapter
assignment, claim, and generated BPMN conformance remains under M10.

Exit criteria:

- field parity is proven by tests or snapshots;
- missing host ABI fields fail before user interaction starts.

### M3: Task Data Shape

Goal: decide and lock how submitted form data becomes workflow variables.

Current direction: keep the flat declared-field merge because it is already
validated and operationally simple. Introduce nested output envelopes only when
a concrete form requirement needs it.

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
3. repair guidance that moves executable UI metadata to `qianji:interaction`.

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

Exit criteria:

- a claimed pending human task cannot be completed or released by the wrong
  claimant;
- unclaimed work remains completable through typed identity.

### M7: Worklist Routing Policy

Goal: decide whether Qianji worklist filtering stays claimant-only or adds
passive routing filters from assignment/lane metadata.

Deliverables:

1. policy decision for claimant, assignment, and lane filters;
2. tests for the selected filter semantics;
3. clear docs that these filters do not equal authorization.

Exit criteria:

- worklist filtering is deterministic and Rust-owned;
- UI cannot hide or expose work by inventing assignment logic.

### M8: Form Schema Boundary

Goal: keep interaction rendering bounded while allowing future structured form
growth.

Deliverables:

1. linter catalog for supported `qianji:interaction` types and attributes;
2. rejection tests for ambiguous question/choices references and unsupported
   free-text cardinality;
3. optional future nested-envelope design before implementation.

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

Start M2. The next implementation slice should add the host request ABI ledger
that records each human-work field across Rust runtime requests, stream JSON,
HTTP snapshots, CLI text, and downstream adapter consumption.

The ledger should prove field parity for:

1. `process_id`, `activity_id`, `token_id`, and `node_index`;
2. `variables` and repeat context;
3. `form` metadata, including `interaction_type`, inputs, and result output;
4. `assignment` metadata as routing-only standard BPMN hints;
5. `claim` metadata as checkpointed allocation state, not authorization.

This is the next milestone because M1 has established the execution loop and
M2 prevents field drift across every interactive host surface.
