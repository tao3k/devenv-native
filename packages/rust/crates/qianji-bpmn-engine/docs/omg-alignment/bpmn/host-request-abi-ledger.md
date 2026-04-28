# Host Request ABI Ledger

This ledger defines the current Qianji BPMN human-work ABI. It is the M2
contract for user and manual task interaction surfaces.

## Scope

The ledger covers pending human work produced by `userTask` and `manualTask`.
It applies after the engine has advanced to a host boundary and before an
external host submits a typed completion payload.

The ABI has one authority chain:

1. `qianji-bpmn-engine` owns pending work identity, form metadata, assignment
   metadata, claim state, and completion validation.
2. `xiuxian-qianji` transports those fields through stream JSON, HTTP
   snapshots, CLI text, and worklist views.
3. Downstream adapters render the transported fields and submit typed
   completion requests. They must not infer BPMN task identity, form output
   mapping, or assignment semantics from XML, display labels, or graph shape.

## Field Ledger

| Field         | Rust owner                                                | Meaning                                                                    | Required for user/manual host work                 | Transport rule                                                                                                         |
| ------------- | --------------------------------------------------------- | -------------------------------------------------------------------------- | -------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `kind`        | `PendingHostWorkKind` and `PendingHostWorkRequest`        | Host-work family. Human interaction uses `user` or `manual`.               | yes                                                | Preserve as an enum/string label; do not infer from BPMN XML downstream.                                               |
| `instance_id` | `UserTaskRequest` and `ManualTaskRequest`                 | Workflow instance that owns the blocked task.                              | yes on typed requests and completion commands      | Preserve on stream items and worklist items; HTTP snapshots carry it through the surrounding workflow object.          |
| `process_id`  | `PendingHostWork`, `UserTaskRequest`, `ManualTaskRequest` | BPMN process that owns the blocked activity.                               | yes                                                | Preserve as the completion target process.                                                                             |
| `activity_id` | `PendingHostWork`, `UserTaskRequest`, `ManualTaskRequest` | Stable BPMN activity identifier for the blocked user/manual task.          | yes                                                | Preserve as the completion target activity. Do not substitute UI labels or node display text.                          |
| `token_id`    | `PendingHostWork`, `UserTaskRequest`, `ManualTaskRequest` | Runtime token identifier for this blocked work item.                       | yes                                                | Preserve as the primary runtime completion target.                                                                     |
| `node_index`  | `PendingHostWork`, `UserTaskRequest`, `ManualTaskRequest` | Dense runtime node index.                                                  | yes                                                | Preserve for runtime diagnostics and graph correlation; do not use as the only human-facing identity.                  |
| `variables`   | `UserTaskRequest`, `ManualTaskRequest`                    | Current workflow data snapshot, or iteration-local data for repeat work.   | yes on host requests and stream JSON               | Hosts may render from these values but must submit only declared completion fields when form metadata exists.          |
| `repeat`      | `UserTaskRequest`, `ManualTaskRequest`                    | Optional multi-instance iteration context.                                 | no                                                 | Preserve when present; absence means ordinary single-instance work.                                                    |
| `form`        | `BpmnHumanTaskFormSpec`                                   | Bounded `qianji:interaction` metadata for rendering and output validation. | no, but required for generated interactive prompts | Preserve `interaction_type`, question source, choices source or inline choices, free-text fields, and `result_output`. |
| `assignment`  | `BpmnHumanTaskAssignmentSpec`                             | Standard BPMN `humanPerformer` and `potentialOwner` routing hints.         | no                                                 | Preserve as passive routing metadata only. It is not authorization, delegation, escalation, or reassignment.           |
| `claim`       | `PendingHostWorkClaim`                                    | Checkpointed allocation state for one claimant.                            | no                                                 | Preserve when present. A claimed task requires matching claimant completion or release.                                |
| `work_id`     | `PendingHostWork`                                         | Optional host-generated work identifier.                                   | no                                                 | Preserve for host diagnostics only. It is not the canonical completion target.                                         |

## Transport Surfaces

| Surface                   | Canonical field shape                                                                                                                                                                               | Current coverage                                                                                                                                  | Required adapter behavior                                                                                            |
| ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Rust runtime request      | `UserTaskRequest` and `ManualTaskRequest` carry `instance_id`, `process_id`, `activity_id`, `token_id`, `node_index`, `variables`, `repeat`, `form`, `assignment`, and `claim`.                     | Runtime and host-dispatch tests cover identity, form, assignment, claim, and completion behavior.                                                 | Treat this as the source of truth for all host rendering and completion.                                             |
| Stream JSON               | `@@QIANJI_HOST_WORK` emits `kind`, `instance_id`, `process_id`, `activity_id`, `node_id`, `node_index`, `token_id`, `variables`, `repeat`, `form`, `assignment`, and `claim` when present.          | Stream tests cover runtime identity plus form and assignment metadata.                                                                            | Render directly from the stream payload. If required form metadata is absent, fail before asking the user for input. |
| HTTP snapshot             | `pending_host_work[]` carries `token_id`, `process_id`, `node_index`, `activity_id`, `kind`, `work_id`, `form`, `assignment`, and `claim`; the surrounding workflow snapshot carries `instance_id`. | HTTP snapshot tests cover identity, form, assignment, claim, and serialized wire fields.                                                          | HTTP clients must use the snapshot values as the completion target.                                                  |
| CLI execution/status text | Pending host-work text includes token, kind, process/activity identity, form summary, assignment summary, and claim when present.                                                                   | CLI start-at and status tests cover form and assignment summaries.                                                                                | CLI text is operator-facing. JSON stream or HTTP snapshot remains the machine contract.                              |
| CLI worklist text         | Worklist items include instance, token, process, activity, kind, checkpoint sequence, state sequence, claim, form summary, and assignment summary.                                                  | CLI worklist tests cover claimed and unclaimed human work plus focused human-task ABI field parity.                                               | Use worklist output for operator triage; submit typed claim/release/complete commands for state changes.             |
| Downstream adapters       | Adapter state must be a direct projection of stream JSON, HTTP snapshots, or typed control-service responses.                                                                                       | pi-wendao tests reject missing streamed `form` and `result_output` before prompt rendering; broader adapter conformance remains an M10 milestone. | Do not parse BPMN XML during execution to recover form, assignment, identity, or output mapping.                     |

## Completion Contract

Typed completion must target the Rust-owned identity:

1. `instance_id` selects the checkpoint or active session.
2. `token_id` selects the pending host-work item.
3. `process_id` and `activity_id` must match the pending item.
4. `kind` must match the pending host-work family.
5. `claimant` must match the checkpointed claim when `claim` is present.
6. `data` must be an object whose fields are declared by `form` when form
   metadata exists.

For unclaimed human work, completion may omit `claimant`. For claimed human
work, omission or mismatch fails before workflow advancement.

## Standard Boundary

OMG BPMN provides the `userTask`, `manualTask`, and resource-role vocabulary.
Qianji currently preserves a bounded subset of human-task assignment metadata:

- `humanPerformer`
- `potentialOwner`
- `resourceRef`
- `resourceAssignmentExpression/formalExpression`

These fields are routing hints. They do not authorize completion and do not
implement reassignment, escalation, delegation, participant resolution, or full
WS-HumanTask behavior.

Native BPMN `rendering` is not executable in the current bounded runtime.
Executable form rendering must come from bounded `qianji:interaction`
metadata until a separate native-rendering design is implemented.

## Evidence Map

| Contract point                                                             | Evidence                                                                                                |
| -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| Engine advances to human work without adapter graph inference              | `runtime_human_interaction_loop_advances_from_engine_work_to_human_wait_and_completion`                 |
| Stream JSON mirrors runtime pending human-work identity                    | `pending_host_work_stream_preserves_runtime_host_loop_identity_contract`                                |
| Stream JSON preserves form and assignment metadata                         | `pending_host_work_stream_includes_human_task_form_contract`                                            |
| HTTP snapshot exposes pending human-work identity and nested metadata      | `bpmn_workflow_http_snapshot_exposes_pending_human_task_contract`                                       |
| CLI start/status text exposes form and assignment summaries                | `run_bpmn_start_at_and_status_render_human_task_interaction_contract`                                   |
| CLI claim/worklist/release uses checkpointed control service state         | `run_bpmn_task_claim_worklist_release_commands_use_checkpointed_control_service`                        |
| CLI worklist text exposes human-work ABI field summaries                   | `run_bpmn_task_worklist_renders_human_task_abi_fields`                                                  |
| pi-wendao rejects missing streamed form/result metadata before interaction | `interaction-contract.test.ts` no-fallback tests and `executor.test.ts` qianji host-work negative tests |

## M2 Status

M2 is complete for the bounded host request ABI ledger. Runtime, stream, HTTP,
CLI start/status, and CLI worklist surfaces have focused parity evidence for
core human-work fields, and pi-wendao rejects missing Rust-owned `form` or
`result_output` metadata before user interaction starts. Broader adapter
conformance for assignment, claim, and generated BPMN smoke coverage remains
under M10.
