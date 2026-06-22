# xiuxian-qianji-runtime

`xiuxian-qianji-runtime` owns dependency-safe workflow adapters and bounded
worker-loop runners for Qianji execution.

The crate sits between workflow semantics and the workflow-neutral control
plane. It does not own the durable truth source; it translates workflow facts
into contracts owned by `xiuxian-qianji-control`.

- `xiuxian-qianji-bpmn-engine` owns BPMN host-work facts and frontier semantics.
- `xiuxian-qianji-control` owns durable activity tasks, ledger events, replay
  views, hot queues, leases, recovery views, and worker lifecycle records.
- `xiuxian-qianji-runtime` converts execution-facing workflow boundaries into
  control-plane activity contracts without depending on the CLI/server crate.
- `xiuxian-qianji` owns command, HTTP, and package-level orchestration surfaces
  that can consume this crate.

## Current Boundary

The first runtime slices own Flowhub BPMN `serviceTask` scheduling through
`build_flowhub_service_activity_schedule_record`. The adapter validates stable
BPMN identity, preserves scenario, instance, process, activity, token, source
path, work id, and declared output metadata, and returns an admitted
`ActivityTask` schedule record for the control ledger.

The runtime crate also owns deterministic Flowhub service completion contract
helpers. `build_flowhub_service_task_completion` validates replay-derived
worker metadata and required BPMN outputs, while
`build_flowhub_service_task_contract_completion_data` and
`build_flowhub_service_task_contract_activity_result` derive the bounded
contract-worker output. Package-specific BPMN and HTTP request wrappers remain
in `xiuxian-qianji`.

The crate now also owns the workflow-control port used by bounded worker
loops. `QianjiRuntimeWorkflowControlPort` models status loading, resume
preparation, and prepared task completion through runtime-owned request
shapes. `xiuxian-qianji` still implements the concrete BPMN control service,
but qianji-server worker code can call through this port before any larger
server-loop migration. `QianjiRuntimeWorkflowStatusView` owns the pending
host-work count and first-by-kind frontier helpers used by worker loops.
`FlowhubServiceWorkerLoopRequest` owns the status, resume, and service-task
completion request builders that feed the workflow-control port, keeping
checkpoint backend selection opaque to the worker-loop internals.
`QianjiRuntimeWorkflowTaskCompleteRequest` derives the resume request needed
for its own completion, so completion flows do not duplicate the checkpointed
workflow identity shape.

The crate also owns the generic BPMN host-work activity-evidence contract.
`build_bpmn_host_work_activity_schedule_record` turns replay-derived
`PendingHostWork` facts into durable `bpmn.host_work` activity schedules for
send, service, script, user, manual, and business-rule work. The companion
`build_bpmn_host_work_activity_result` converts runtime-neutral completion
facts into stable `ActivityResult` metadata and hashes. `xiuxian-qianji`
continues to own HTTP request parsing and maps server-specific payloads into
these runtime types.
The runtime crate also owns the BPMN-specific evidence adapter for those facts:
`record_bpmn_host_work_completion_activity_evidence` and
`record_bpmn_host_work_failure_activity_evidence` create or reuse the evidence
run, record the activity schedule, replay the worker task, record worker
start, and append the terminal completion or failure event by composing
workflow-neutral `xiuxian-qianji-control` ledger helpers. The durable event
stream, replay rules, idempotency guards, retry policy, queues, leases, and
worker lifecycle semantics remain owned by `xiuxian-qianji-control`. Server
crates still choose their run id and classify local bad-request errors before
calling the runtime adapter.
The same boundary now owns `BpmnHostWorkIdentity` plus
`find_matching_bpmn_host_work`, so server and CLI adapters can validate
token/process/activity/kind identity against checkpoint-derived
`PendingHostWork` without duplicating matching rules.

The Flowhub service worker-loop kernel is now runtime-owned as a BPMN/Flowhub
adapter and runner.
`run_flowhub_service_worker_completion_loop` is generic over a
`QianjiRuntimeWorkflowControlPort`, host bridge, control ledger, and hot-state
store. It first creates the durable Flowhub worker control run when the ledger
has no events for the supplied run id, then reads the workflow frontier,
records and mirrors the service `ActivityTask`, claims it, derives
deterministic contract output, records the durable terminal event through
`xiuxian-qianji-control`, completes the BPMN service task through the runtime
port, and releases the lease. The control crate remains the authority for
durable history and hot-state contracts, while `xiuxian-qianji` still owns
concrete server state assembly and passes its service/host pair into the
runtime loop.

The crate deliberately does not own:

- CLI parsing
- HTTP routes
- BPMN parser internals
- LLM provider clients
- Wendao data-plane access
- qianji-server process or route orchestration
- workflow-neutral durable ledger, replay, retry, queue, lease, gate, cost, or
  recovery semantics

The next slices should move workflow-neutral control semantics into
`xiuxian-qianji-control`, keep BPMN/Flowhub translation in
`xiuxian-qianji-runtime`, and leave concrete checkpoint storage plus HTTP/CLI
routing in `xiuxian-qianji`.
