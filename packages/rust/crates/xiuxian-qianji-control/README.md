# xiuxian-qianji-control

`xiuxian-qianji-control` is the workflow-neutral control-plane kernel for
Qianji-managed Agent execution.

Qianji workflow crates describe what should happen. This crate records and
manages what is happening:

- run and step lifecycle events
- deterministic replay views
- durable execution journal events for activities, signals, timers, and
  version pins
- Agent proposal and decision journal facts
- evidence and artifact references
- budget and cost observations
- gate results
- recovery attempts
- hot scheduling queues, leases, activity tasks, and worker heartbeats

The core slice ships Rust contracts plus in-memory stores. The `duckdb`
feature adds the durable append-only ledger adapter. The `valkey` feature adds
the hot-state adapter for step queues, worker activity task queues, leases,
and worker heartbeats.

## Boundary

This crate must stay independent from workflow implementations. It does not
depend on `xiuxian-qianji`, `qianji-bpmn-engine`, `xiuxian-wendao`,
`xiuxian-llm`, or `xiuxian-qianhuan`.

The intended split is:

- DuckDB: durable append-only control ledger and replayable run views
- Valkey: hot queues, leases, heartbeats, rate limits, and live progress
- Qianji workflow/BPMN/Flowhub: domain execution semantics
- Agent workers: leased executors that attach evidence and observations

Qianji workflow traces are projected into control events by
[`xiuxian-qianji`](../xiuxian-qianji/README.md). This crate does not depend on
Qianji workflow types.

Execution history is represented by the same append-only `ControlLedger`.
Checkpoint or workflow-specific state should be treated as a materialized view
or replay accelerator; the durable audit authority remains the ledger event
stream.

`RunView::recovery_view` derives a read-only recovery summary from replayed
history. It classifies scheduled and in-flight activities, retryable and
terminal activity failures, pending and fireable timers, approval-required
Agent decisions, human waits, blocked steps, and active or expired step
leases. The view is an inspection and planner input surface only: it does not
append events, mutate Valkey hot state, enqueue work, execute retries, or
materialize workflow-specific approval objects.
`RunRecoveryView::recovery_plan` projects those facts into ordered recovery
actions such as reclaiming expired leases, firing ready timers, retrying
eligible activities, escalating terminal failures, waiting for human approval,
and preserving active leases. The plan is still declarative and side-effect
free; a workflow runtime may consume it, but this crate does not execute it.
`ControlLedger::load_recovery_plan` exposes the same replay, recovery view,
and recovery plan chain as one durable query helper for runtime callers.
`RunRecoveryPlan::summary` derives compact management counters over the
ordered plan actions so gateway, CLI, or UI surfaces can show recovery state
without reimplementing action classification.
`ControlLedger::load_recovery_snapshot` packages the replay-derived recovery
view, ordered plan, and summary into one read-only management response.

Activity journal fields use typed identities for activity ids, activity types,
task queues, idempotency keys, and error codes while preserving string-shaped
serialization.

Activity retry policies expose deterministic constructors and validation so
zero attempts, zero backoff multipliers, inverted retry intervals, and zero
activity timeouts fail before a Worker adapter executes them.
They also provide a pure retry decision contract over `ActivityFailure`, so the
control plane can decide stop reasons, next attempts, and capped backoff
without executing retry work itself.

LLM calls are represented as governed activity payloads. `LlmActivityRequest`
records model, prompt, context, tool-schema, response-schema, token, and budget
metadata through claim-check references, while `LlmActivityTask` binds that
payload to an `llm.*` activity type and task queue. Provider adapters should
disable provider-client retries and let `ActivityRetryPolicy` control retry
behavior.
LLM activity admission is represented by `LlmActivityAdmission`. It validates
the complete `LlmActivityTask` and requires the generic `ActivityTask`
`input_ref` to match the request `prompt_ref`, so model providers can only
execute claim-check prompts that the deterministic controller admitted.
`record_admitted_llm_activity_schedule` records an admitted LLM activity as a
durable `ActivityScheduled` fact. It does not call providers, enqueue
hot-state work, acquire leases, or start workers.
The stored `ActivityTask.metadata.qianji_llm_activity_request` value carries a
compact audit summary of the admitted model, prompt/context references,
tool-schema hash, response-schema reference, token limits, and budget. It keeps
LLM replay and operator history tied to claim-check references without storing
provider prompt or context payloads in the ledger.

Agent planner output is represented as `AgentProposal`; Qianji reducer output
is represented as `AgentDecision`. Accepted decisions must name the scheduled
activity, while rejected or approval-required decisions cannot smuggle an
activity into execution.

Tool access is represented by `ToolActivityContract`, which maps an
agent-visible tool name to an activity type, task queue, risk level, permission
mode, permission scope, and schema hashes. `ToolPermissionDecision` returns
allowed, approval-required, or denied outcomes without executing the tool.

Human approval waits are represented by `HumanApprovalRequest`. It binds an
approval-required tool decision to an expected signal name, optional payload
claim-check, optional expected payload hash, and optional timeout timer.
Signal and timer matching returns deterministic `HumanApprovalResolution`
values; this crate does not wait, notify, schedule, or execute approvals.
Matched approval signals can be interpreted as `HumanApprovalDecision` values
when compact metadata contains `decision = approved` or `decision = rejected`.
Those decisions are audit facts only; a later reducer must still decide
whether any activity may be scheduled.

Tool authorization is represented by `ToolAuthorizationDecision`. It resolves
tool permissions, approval decisions, and timeout resolutions into authorized,
waiting-approval, rejected, denied, or timed-out facts without creating an
`ActivityTask` or `AgentDecision`.

Authorized tool activity admission is represented by `ToolActivityAdmission`.
It validates that a caller-supplied `ActivityTask` matches the authorized
activity type, task queue, and proposal input reference before a later
scheduler may enqueue it. An admitted tool activity can also construct a
validated accepted `AgentDecision` that names the admitted activity id without
appending ledger events or enqueueing work.
`ToolPolicyReductionRequest` is the first built-in deterministic policy
reducer. It composes an `AgentProposal`, a `ToolAuthorizationDecision`, an
optional `ActivityTask`, and an optional `GateResult` into one
`AgentDecision`. Authorized tools require an admitted activity; approval,
denial, rejection, timeout, and failed-gate outcomes cannot carry scheduled
activity ids. The reducer is side-effect free: it does not append ledger
events, enqueue hot-state work, acquire leases, run retries, or execute
Workers.
`record_admitted_activity_schedule` records an already admitted tool activity
as an `ActivityScheduled` journal fact. This creates durable scheduled
activity state on replay, but it does not enqueue hot-state work, acquire
leases, start workers, or execute providers.
`record_admitted_activity_schedule_idempotent` is the checked Worker-facing
variant. It returns the already stored event for exact duplicate schedules and
rejects conflicting schedules for the same activity id.
Generic precompiled schedule plans use `ActivityScheduleAdmissionPlanItem`
rows with workflow-neutral `ActivityTask` payloads. `admit_activity_schedule_plan`
validates the plan contract, matching Qianji run id, safe non-execution flags,
pending status, and claim-check input reference before recording idempotent
`ActivityScheduled` facts through `AdmittedActivityTaskScheduleRecord`. This
surface is for durable control-plane admission only: it does not depend on the
plan producer, enqueue hot-state work, acquire leases, start workers, call
models, read source text, or mutate ontology data.
`record_activity_started`, `record_activity_completed`, and
`record_activity_failed` record activity lifecycle facts after scheduling.
They append durable journal events only; retry decisions, worker execution,
and queue mutation remain separate control-plane seams.
`record_activity_started_idempotent`,
`record_activity_completed_idempotent`, and
`record_activity_failed_idempotent` add replay-backed guards for live Worker
integration. Exact duplicate lifecycle facts return the original record;
completion must follow a started activity, failures cannot rewrite completed
activities, and retry starts must advance beyond the failed attempt.
`ActivityQueueProjection` derives scheduled-but-not-started activity tasks
from replayed durable history, optionally filtered by task queue. It is a
read-only worker management view and does not claim work, acquire leases,
append lifecycle events, or mutate Valkey hot state. The projection also
reports replayed activity lifecycle counts so operators can see scheduled,
in-flight, completed, and failed activity state without loading the full run
view. Each projection also exposes `WorkerActivityTask` envelopes with durable
run, optional step, activity, queue, input, idempotency, timeout, retry policy,
scheduled timestamp, and next-attempt fields. These envelopes are derived from
the same replayed schedule events and are the worker-facing task contract; hot
state may mirror them for polling, but it is not their source of truth.
Worker task envelopes copy scheduled task metadata, so the LLM request audit
summary remains visible in replay-derived queue inspection and hot-state mirror
payloads.
Activity completion results may carry an `ActivityResult.output_ref`
claim-check alongside an `output_hash`; worker adapters should write large
provider responses through referenced artifacts instead of embedding payloads
inside terminal event metadata.
`LlmActivityInventoryProjection` derives all replayed `llm.*` activity rows
from the same run view. It reports lifecycle counts, missing request-audit
coverage, extracted model ids, input references, and the stored request audit
metadata without appending events, mutating hot state, or calling providers.
The Qianji CLI can use this projection as an opt-in request-audit gate so
operator checks fail deterministically when any replayed LLM activity lacks its
admitted request audit metadata.
`RunnableActivityTask`, `ActivityTaskLease`, and
`HotStateLeasedActivityTask` are the hot-state mirror payloads for that
worker-facing contract. `HotStateStore::enqueue_activity_task` and
`HotStateStore::claim_activity_task` provide task-queue filtered worker
delivery with lease ownership while preserving the ledger as the durable
authority. `HotStateStore::release_activity_task_lease` removes a completed or
failed hot-state activity lease after durable lifecycle recording, and
`HotStateStore::reclaim_expired_activity_task_lease` returns only expired
leases to the runnable activity queue.
`mirror_worker_activity_tasks_to_hot_state` is the bounded bridge from durable
replay to that hot-state mirror: it loads `WorkerActivityTask` envelopes from
`ControlLedger::load_worker_activity_tasks` and enqueues `RunnableActivityTask`
entries for worker polling without appending new ledger events. Hot-state
enqueue is idempotent for pending activity tasks and will not re-enqueue a task
that is already protected by an activity lease.
The Qianji CLI exposes this bridge as an explicit mirror step before worker
claiming, keeping replay-derived task identity separate from hot-state lease
ownership.
Worker-facing CLIs and adapters may claim one mirrored task from hot state, but
the lease is only a polling and ownership guard. Durable lifecycle truth still
comes from the append-only ledger through the worker activity start, complete,
and fail helpers.
`WorkerActivityStartRecord`, `WorkerActivityCompletedRecord`, and
`WorkerActivityFailedRecord` let Worker adapters record lifecycle outcomes
directly from a `WorkerActivityTask` envelope. The helpers recover run or step
scope and attempt from replay-derived task facts, then delegate to the
idempotent activity journal guards. Workers therefore do not reconstruct
scope, activity id, or retry attempt by hand, and hot-state queue delivery
cannot become the durable authority.
`record_worker_heartbeat` records a durable Worker liveness audit fact after
validating heartbeat TTL. `record_worker_heartbeat_with_hot_state` first
mirrors the heartbeat into a `HotStateStore` and then appends the durable
ledger event, so Worker integration can update Valkey liveness and durable
audit state through one governed helper.
`record_step_queued` records a durable `StepQueued` fact.
`record_step_queued_with_hot_state` first enqueues the `RunnableStep` in a
`HotStateStore` and then appends durable history, so scheduling and later
recovery appliers can share the same queue mirror contract without executing
Workers.
`HotStateStore::load_snapshot` is a read-only operator query over hot step
queues, activity task queues, leases, and worker heartbeats. It reports
`HotStateSnapshot` facts without reclaiming leases, reordering queues,
renewing heartbeats, appending ledger events, or executing Workers.
Replay-derived `StepView` records the current active `StepLease`, which lets
read-only operator surfaces inspect lease ownership without touching hot state.
Run-level replay also supports lease inventory views by collecting active
`StepLease` records from the deterministic step map.
`TimerInventoryProjection` derives run-scoped and step-scoped durable timer
state from replayed history. It is a read-only wait-state management view and
does not fire timers, enqueue steps, append events, or mutate Valkey hot state.
The projection reports pending, scheduled, and fired timer counts so operators
can audit durable waits without loading the full run view.
`SignalInventoryProjection` derives run-scoped and step-scoped durable signal
state directly from event records so event sequence and received timestamp stay
visible. It is a read-only external-event management view and does not append
signals, resolve approvals, enqueue work, or mutate hot state.
`CostInventoryProjection` derives run-scoped and step-scoped durable cost
observations directly from event records so sequence, observed timestamp,
provider/model, token counts, latency, and USD micros remain auditable. It is a
read-only budget management view and does not append observations or mutate hot
state.
`RunOperatorSummary` combines durable event count, replayed run status, step
count, active lease count, activity lifecycle counters, timer counters, signal
counters, cost totals, and recovery counters into one compact management view.
It is assembled from the append-only ledger and remains a read-only projection;
it does not execute recovery, fire timers, claim activities, append signals, or
mutate hot state.
`record_timer_fired` records a durable `TimerFired` fact for a run-scoped or
step-scoped timer. It does not poll timers, wait, notify, or enqueue work.
`record_step_lease_released` records a durable `StepLeaseReleased` fact after
hot-state lease ownership has been handled by the caller.
`record_recovery_started` records a durable `RecoveryStarted` fact for a run
or step before a recovery loop applies, inspects, or escalates recovery work.
`apply_recovery_plan` records one run-scoped recovery attempt and applies the
supplied plan's actions in order, returning a per-action trace. It delegates
executable actions to `apply_recovery_action`; non-executable management
actions remain explicit `NotApplicable` results.
`apply_recovery_action` is the first bounded recovery applier. It applies only
step-scoped `RetryActivity` actions by queueing the owning step after the
retry backoff and recording `StepQueued`; run-scoped `RetryActivity` actions
requeue the failed activity task into the hot activity queue after the same
backoff without appending a synthetic schedule event. It applies `FireTimer`
actions by recording `TimerFired` and applies `ReclaimExpiredLease` by
validating the replayed lease, reclaiming the expired hot lease back into the
runnable queue, and recording `StepLeaseReleased`; other action kinds return
`NotApplicable` without side effects.

Agent proposals and deterministic Agent decisions can be recorded as control
journal events and replay into run or step views. Recording an Agent decision
does not create activity lifecycle state; `ActivityScheduled` remains the
separate scheduling fact.

`record_agent_proposal` and `record_agent_decision` are explicit journal
helpers for persisting those Agent facts to a `ControlLedger`.
`AgentProposalJournalRecord` and `AgentDecisionJournalRecord` carry the named
request fields, including run scope or step scope. The helpers only append
durable control events; they do not admit tools, schedule activities, enqueue
hot-state work, lease steps, or execute workers.

## Current Surface

- `ControlEvent` and `ControlEventRecord`
- `AgentProposal` and `AgentDecision`
- `AgentJournalScope`, `AgentProposalJournalRecord`,
  `AgentDecisionJournalRecord`, `record_agent_proposal`, and
  `record_agent_decision`
- `ToolActivityContract` and `ToolPermissionDecision`
- `ToolAuthorizationDecision`
- `ToolActivityAdmission`
- `ToolPolicyReductionRequest`, `ToolPolicyReduction`, and
  `AgentPolicyReason`
- `AdmittedActivityScheduleRecord` and `record_admitted_activity_schedule`
- `record_admitted_activity_schedule_idempotent`
- `ActivityJournalScope`, `ActivityStartedJournalRecord`,
  `ActivityCompletedJournalRecord`, `ActivityFailedJournalRecord`,
  `record_activity_started`, `record_activity_completed`, and
  `record_activity_failed`
- `ActivityJournalWriteOutcome`, `ActivityJournalWriteStatus`,
  `record_activity_started_idempotent`,
  `record_activity_completed_idempotent`, and
  `record_activity_failed_idempotent`
- `ActivityQueueProjection`, `ActivityQueueItem`, and `WorkerActivityTask`
- `WorkerActivityHotStateMirrorRequest`,
  `WorkerActivityHotStateMirrorOutcome`, and
  `mirror_worker_activity_tasks_to_hot_state`
- `WorkerActivityStartRecord`, `WorkerActivityCompletedRecord`,
  `WorkerActivityFailedRecord`, `record_worker_activity_started_idempotent`,
  `record_worker_activity_completed_idempotent`, and
  `record_worker_activity_failed_idempotent`
- `WorkerHeartbeatJournalRecord`, `record_worker_heartbeat`, and
  `record_worker_heartbeat_with_hot_state`
- `StepLeaseReleaseJournalRecord` and `record_step_lease_released`
- `StepQueueJournalRecord`, `record_step_queued`, and
  `record_step_queued_with_hot_state`
- `SignalInventoryProjection`, `SignalInventoryItem`, and
  `SignalInventorySummary`
- `TimerInventoryProjection`, `TimerInventoryItem`, and
  `TimerInventorySummary`
- `TimerFireJournalRecord` and `record_timer_fired`
- `RecoveryStartedJournalRecord` and `record_recovery_started`
- `AdmittedLlmActivityScheduleRecord` and
  `record_admitted_llm_activity_schedule`
- `LlmActivityInventoryProjection`, `LlmActivityInventoryItem`, and
  `LlmActivityInventorySummary`
- `RecoveryLoopApplicationRequest`, `RecoveryLoopApplication`,
  `RecoveryLoopActionApplication`, and `apply_recovery_plan`
- `RecoveryActionApplicationRequest`, `RecoveryActionApplication`,
  `RecoveryActionApplicationReason`, and `apply_recovery_action`
- `HumanApprovalRequest`, `HumanApprovalResolution`, and
  `HumanApprovalDecision`
- `CostInventoryProjection`, `CostInventoryItem`, and
  `CostInventorySummary`
- `LlmActivityAdmission` and `ToolActivityAdmission`
- `ActivityTask`, `ActivityRetryDecision`, `ActivityTaskLease`,
  `LlmActivityRequest`, `LlmActivityTask`, `RunnableActivityTask`,
  `SignalRecord`, `TimerRecord`, and `VersionPin`
- `HotStateSnapshot`, `HotStateLeasedStep`, and
  `HotStateLeasedActivityTask`
- `ControlLedger`
- `ControlLedger::load_run_view`
- `ControlLedger::load_activity_queue_projection`
- `ControlLedger::load_worker_activity_tasks`
- `ControlLedger::load_cost_inventory_projection`
- `ControlLedger::load_signal_inventory_projection`
- `ControlLedger::load_timer_inventory_projection`
- `ControlLedger::load_recovery_plan`
- `RunRecoveryView`, `RecoveryItemScope`, `ActivityRecoveryItem`,
  `FailedActivityRecoveryItem`, `TimerRecoveryItem`,
  `AgentDecisionRecoveryItem`, `StepRecoveryItem`, and `LeaseRecoveryItem`
- `RunRecoveryPlan` and `RecoveryPlanAction`
- `RunRecoveryPlanSummary`
- `RunRecoverySnapshot`
- `HotStateStore`
- `InMemoryControlLedger`
- `InMemoryHotStateStore`
- `RunView` and `StepView`
- `RequiredEvidenceGate`
- `DuckDbControlLedger` behind the `duckdb` feature
- `ValkeyHotStateStore` behind the `valkey` feature
