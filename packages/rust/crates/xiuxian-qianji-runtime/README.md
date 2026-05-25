# xiuxian-qianji-runtime

`xiuxian-qianji-runtime` owns dependency-safe runtime adapters for durable
Qianji workflow execution.

The crate sits between workflow semantics and the workflow-neutral control
plane:

- `xiuxian-qianji-bpmn-engine` owns BPMN host-work facts and frontier semantics.
- `xiuxian-qianji-control` owns durable activity tasks, ledger events, hot
  queues, leases, recovery views, and worker lifecycle records.
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
server-loop migration.

The crate deliberately does not own:

- CLI parsing
- HTTP routes
- BPMN parser internals
- LLM provider clients
- Wendao data-plane access
- qianji-server worker loops themselves

The next runtime slices should move bounded worker-loop execution after the
remaining `xiuxian-qianji` service implementation dependencies are split or
moved downward.
