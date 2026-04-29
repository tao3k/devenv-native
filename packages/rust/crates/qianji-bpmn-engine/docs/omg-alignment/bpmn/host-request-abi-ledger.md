# Host Request ABI Ledger

This ledger defines the current Qianji BPMN human-work ABI. It is the M2
contract for user and manual task interaction surfaces.

## Scope

The ledger covers pending human work produced by `userTask` and `manualTask`.
It applies after the engine has advanced to a host boundary and before an
external host submits a typed completion payload.

The ABI has one authority chain:

1. `qianji-bpmn-engine` owns pending work identity, form metadata, assignment
   metadata, lane metadata, claim state, and completion validation.
2. `xiuxian-qianji` transports those fields through stream JSON, HTTP
   snapshots, CLI text, and worklist views.
3. Downstream adapters render the transported fields and submit typed
   completion requests. They must not infer BPMN task identity, form output
   mapping, assignment semantics, or lane routing from XML, display labels, or
   graph shape.

## Field Ledger

| Field         | Rust owner                                                | Meaning                                                                       | Required for user/manual host work                 | Transport rule                                                                                                         |
| ------------- | --------------------------------------------------------- | ----------------------------------------------------------------------------- | -------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `kind`        | `PendingHostWorkKind` and `PendingHostWorkRequest`        | Host-work family. Human interaction uses `user` or `manual`.                  | yes                                                | Preserve as an enum/string label; do not infer from BPMN XML downstream.                                               |
| `instance_id` | `UserTaskRequest` and `ManualTaskRequest`                 | Workflow instance that owns the blocked task.                                 | yes on typed requests and completion commands      | Preserve on stream items and worklist items; HTTP snapshots carry it through the surrounding workflow object.          |
| `process_id`  | `PendingHostWork`, `UserTaskRequest`, `ManualTaskRequest` | BPMN process that owns the blocked activity.                                  | yes                                                | Preserve as the completion target process.                                                                             |
| `activity_id` | `PendingHostWork`, `UserTaskRequest`, `ManualTaskRequest` | Stable BPMN activity identifier for the blocked user/manual task.             | yes                                                | Preserve as the completion target activity. Do not substitute UI labels or node display text.                          |
| `token_id`    | `PendingHostWork`, `UserTaskRequest`, `ManualTaskRequest` | Runtime token identifier for this blocked work item.                          | yes                                                | Preserve as the primary runtime completion target.                                                                     |
| `node_index`  | `PendingHostWork`, `UserTaskRequest`, `ManualTaskRequest` | Dense runtime node index.                                                     | yes                                                | Preserve for runtime diagnostics and graph correlation; do not use as the only human-facing identity.                  |
| `variables`   | `UserTaskRequest`, `ManualTaskRequest`                    | Current workflow data snapshot, or iteration-local data for repeat work.      | yes on host requests and stream JSON               | Hosts may render from these values but must submit only declared completion fields when form metadata exists.          |
| `repeat`      | `UserTaskRequest`, `ManualTaskRequest`                    | Optional multi-instance iteration context.                                    | no                                                 | Preserve when present; absence means ordinary single-instance work.                                                    |
| `form`        | `BpmnHumanTaskFormSpec`                                   | Bounded native BPMN IO metadata metadata for rendering and output validation. | no, but required for generated interactive prompts | Preserve `interaction_type`, question source, choices source or inline choices, free-text fields, and `result_output`. |
| `assignment`  | `BpmnHumanTaskAssignmentSpec`                             | Standard BPMN `humanPerformer` and `potentialOwner` routing hints.            | no                                                 | Preserve as passive routing metadata only. It is not authorization, delegation, escalation, or reassignment.           |
| `lane`        | `BpmnLaneMembershipSpec`                                  | BPMN lane membership for passive display and worklist filtering.              | no                                                 | Preserve as passive routing metadata only. It is not scheduling, authorization, participant resolution, or escalation. |
| `claim`       | `PendingHostWorkClaim`                                    | Checkpointed allocation state for one claimant.                               | no                                                 | Preserve when present. A claimed task requires matching claimant completion or release.                                |
| `work_id`     | `PendingHostWork`                                         | Optional host-generated work identifier.                                      | no                                                 | Preserve for host diagnostics only. It is not the canonical completion target.                                         |

## Lifecycle Event Ledger

`BpmnInstanceState` also carries `human_task_events`, a durable lifecycle-event
ledger for checkpointed `userTask` and `manualTask` work. This ledger is
separate from the execution `trace`: trace events stay node/flow oriented,
while lifecycle events record human-work coordination milestones. The field is
part of the current checkpoint API and is serialized even when empty;
checkpoints that omit it are rejected rather than backfilled.

Each lifecycle event records:

- `sequence`
- `occurred_at_ms`
- `kind`: `created`, `claimed`, `released`, or `completed`
- `process_id`
- `activity_id`
- `token_id`
- `node_index`
- `work_kind`
- optional `claimant`
- optional `work_id`

Events append only after successful state changes. Failed validation, wrong
claimant completion, duplicate no-op claim, non-human host work, and rejected
release paths do not append human-task lifecycle events. Completion events
store task identity and claimant metadata only; submitted completion payload
data is not stored in the ledger.

## Transport Surfaces

| Surface                   | Canonical field shape                                                                                                                                                                                                                                           | Current coverage                                                                                                                                                                                                                                                                                               | Required adapter behavior                                                                                            |
| ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Rust runtime request      | `UserTaskRequest` and `ManualTaskRequest` carry `instance_id`, `process_id`, `activity_id`, `token_id`, `node_index`, `variables`, `repeat`, `form`, `assignment`, `lane`, and `claim`.                                                                         | Runtime and host-dispatch tests cover identity, form, assignment, lane, claim, and completion behavior.                                                                                                                                                                                                        | Treat this as the source of truth for all host rendering and completion.                                             |
| Stream JSON               | `@@QIANJI_HOST_WORK` emits `kind`, `instance_id`, `process_id`, `activity_id`, `node_id`, `node_index`, `token_id`, `variables`, `repeat`, `form`, `assignment`, `lane`, and `claim` when present.                                                              | Stream tests cover runtime identity plus form, assignment, and lane metadata.                                                                                                                                                                                                                                  | Render directly from the stream payload. If required form metadata is absent, fail before asking the user for input. |
| HTTP snapshot             | `pending_host_work[]` carries `token_id`, `process_id`, `node_index`, `activity_id`, `kind`, `work_id`, `form`, `assignment`, `lane`, and `claim`; the surrounding workflow snapshot carries `instance_id`; `human_task_events[]` carries the lifecycle ledger. | HTTP snapshot tests cover identity, form, assignment, lane, claim, lifecycle events, and serialized wire fields.                                                                                                                                                                                               | HTTP clients must use the snapshot values as the completion target and may use the ledger for audit/status display.  |
| CLI execution/status text | Pending host-work text includes token, kind, process/activity identity, form summary, assignment summary, lane summary, and claim when present; status and task-complete output also include compact lifecycle-event summaries.                                 | CLI start-at and status tests cover form and assignment summaries; lane coverage is handled by runtime and worklist tests; task lifecycle tests cover compact event summaries.                                                                                                                                 | CLI text is operator-facing. JSON stream or HTTP snapshot remains the machine contract.                              |
| CLI worklist text         | Worklist items include instance, token, process, activity, kind, checkpoint sequence, state sequence, claim, form summary, assignment summary, and lane summary.                                                                                                | CLI worklist tests cover claimed and unclaimed human work plus focused human-task ABI field parity and passive lane filtering.                                                                                                                                                                                 | Use worklist output for operator triage; submit typed claim/release/complete commands for state changes.             |
| Downstream adapters       | Adapter state must be a direct projection of stream JSON, HTTP snapshots, or typed control-service responses.                                                                                                                                                   | pi-wendao tests reject missing streamed `form` and `result_output` before prompt rendering, forward streamed `assignment` and `claim`, prove missing optional assignment/claim data is not synthesized locally, and run a generated-BPMN smoke where streamed form/result output overrides local XML metadata. | Do not parse BPMN XML during execution to recover form, assignment, claim, identity, or output mapping.              |

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

Form-backed human completion data uses a flat declared-field object. The
required `result_output` field must be present, optional free-text fields may
be omitted, and no undeclared top-level keys are accepted. Nested completion
envelopes are not a compatibility path in the current ABI.

## Form Schema Boundary

Executable form metadata is intentionally bounded. The current supported
native BPMN IO metadata catalog is:

- `input`
- `confirm`
- `choice`
- `choice_input`

Each form must have deterministic host rendering inputs:

1. a question comes from exactly one source: inline question text, a `text`
   attribute, or a dynamic `ref`;
2. choices come from exactly one source family: either one dynamic
   choices data input `sourceRef` or inline choices JSON literal item `value` entries;
3. the primary completion field is declared by answer `dataOutputAssociation targetRef`;
4. the current flat ABI supports at most one supplemental `freeText` data input
   field per interaction;
5. unsupported multi-field schemas must fail through lint or completion
   validation before a host renders them.

Arbitrary JSON schema execution and nested completion envelopes are deferred.
They require a future Rust-owned contract before hosts can render or submit
structured multi-field forms.

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

Worklists may apply passive assignment-resource filters over preserved
`humanPerformer` and `potentialOwner` role names or `resourceRef` values. They
may also apply passive lane filters over preserved BPMN lane id or lane name.
The filters are exact selectors over Rust-owned metadata. They do not
authorize claim, release, completion, participant resolution, or scheduling.

Native BPMN `rendering` is not executable in the current bounded runtime.
Executable form rendering must come from bounded native BPMN IO metadata
metadata until a separate native-rendering design is implemented.

## Evidence Map

| Contract point                                                             | Evidence                                                                                                                                                                                                                                                                                                                                                                                                              |
| -------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Engine advances to human work without adapter graph inference              | `runtime_human_interaction_loop_advances_from_engine_work_to_human_wait_and_completion`                                                                                                                                                                                                                                                                                                                               |
| Stream JSON mirrors runtime pending human-work identity                    | `pending_host_work_stream_preserves_runtime_host_loop_identity_contract`                                                                                                                                                                                                                                                                                                                                              |
| Stream JSON preserves form and assignment metadata                         | `pending_host_work_stream_includes_human_task_form_contract`                                                                                                                                                                                                                                                                                                                                                          |
| HTTP snapshot exposes pending human-work identity and nested metadata      | `bpmn_workflow_http_snapshot_exposes_pending_human_task_contract`                                                                                                                                                                                                                                                                                                                                                     |
| CLI start/status text exposes form and assignment summaries                | `run_bpmn_start_at_and_status_render_human_task_interaction_contract`                                                                                                                                                                                                                                                                                                                                                 |
| CLI claim/worklist/release uses checkpointed control service state         | `run_bpmn_task_claim_worklist_release_commands_use_checkpointed_control_service`                                                                                                                                                                                                                                                                                                                                      |
| CLI worklist text exposes human-work ABI field summaries                   | `run_bpmn_task_worklist_renders_human_task_abi_fields`                                                                                                                                                                                                                                                                                                                                                                |
| pi-wendao rejects missing streamed form/result metadata before interaction | `interaction-contract.test.ts` no-fallback tests and `executor.test.ts` qianji host-work negative tests                                                                                                                                                                                                                                                                                                               |
| Broad BPMN assignment semantics remain outside routing metadata            | `bpmn_linter_reports_generic_performer_assignment_semantics`, `bpmn_linter_reports_generic_resource_role_assignment_semantics`, `bpmn_linter_reports_participant_ref_assignment_semantics`, and `bpmn_linter_reports_resource_parameter_binding_assignment_semantics`                                                                                                                                                 |
| Checkpointed claim lifecycle preserves human-task identity                 | `workflow_control_service_preserves_claim_identity_across_checkpoint_roundtrip`                                                                                                                                                                                                                                                                                                                                       |
| HTTP checkpoint replay preserves human-task claim identity                 | `bpmn_workflow_http_preserves_claim_identity_across_checkpoint_roundtrip`                                                                                                                                                                                                                                                                                                                                             |
| Durable human-task lifecycle ledger records successful state changes       | `human_task_claim_records_checkpointed_owner_metadata`, `human_task_release_clears_checkpointed_owner_metadata`, `host_resume_claimed_user_result_records_claimant_on_completed_event`, `workflow_control_service_preserves_claim_identity_across_checkpoint_roundtrip`, `bpmn_workflow_http_preserves_claim_identity_across_checkpoint_roundtrip`, and `run_bpmn_task_complete_renders_human_task_lifecycle_summary` |
| Assignment-resource worklist routing remains passive                       | `workflow_control_service_worklist_filters_assignment_routing_metadata_without_authorization` and `run_bpmn_task_worklist_filters_assignment_routing_metadata`                                                                                                                                                                                                                                                        |
| Lane worklist routing remains passive                                      | `parsed_bpmn_lane_membership_projects_to_pending_human_work`, `workflow_control_service_worklist_filters_lane_metadata_without_authorization`, and `run_bpmn_task_worklist_filters_assignment_routing_metadata`                                                                                                                                                                                                       |
| Form schema source ambiguity is rejected before host rendering             | `bpmn_linter_rejects_ambiguous_qianji_question_sources` and `bpmn_linter_rejects_choice_input_with_dynamic_and_inline_choices`                                                                                                                                                                                                                                                                                        |
| Unsupported free-text cardinality is rejected before host rendering        | `bpmn_linter_rejects_multiple_qianji_free_text_fields`                                                                                                                                                                                                                                                                                                                                                                |
| Generated BPMN remains adapter-thin at execution time                      | `runs generated BPMN fixture from qianji host-work without local interaction fallback`                                                                                                                                                                                                                                                                                                                                |
| Task-data shape remains a flat declared-field object                       | `workflow_control_service_task_complete_accepts_declared_result_without_optional_free_text`, `workflow_control_service_task_complete_rejects_non_object_form_payload`, and `workflow_control_service_task_complete_rejects_nested_form_output_envelope`                                                                                                                                                               |

## M2 Status

M2 is complete for the bounded host request ABI ledger. Runtime, stream, HTTP,
CLI start/status, and CLI worklist surfaces have focused parity evidence for
core human-work fields, and pi-wendao rejects missing Rust-owned `form` or
`result_output` metadata before user interaction starts. The bounded M10
adapter milestone also proves streamed assignment and claim metadata are
projected directly, missing optional assignment or claim data remains absent
rather than being recovered from local BPMN XML, and generated BPMN artifacts
do not reintroduce local XML interaction fallback during execution. M3 also
locks form-backed human completion data to a flat declared-field object and
keeps nested output envelopes deferred. The bounded lifecycle-event slice also
adds a checkpointed `human_task_events` audit ledger for user/manual created,
claimed, released, and completed milestones without adding Flowable-style
listeners or authorization semantics.
