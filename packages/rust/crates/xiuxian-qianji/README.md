# xiuxian-qianji (千机)

> **"The Dao of logic is like a thousand interlocking gears; only through extreme precision can one ascend from computational chaos."**

`xiuxian-qianji` (千机 - Thousand Mechanisms) is the high-performance, probabilistic execution heart of the **Quadrilateral Cognitive Architecture**. It serves as the "Divine Artifact" that orchestrates the flow of reasoning, transforming fragmented agent actions into a seamless, clockwork artifact of pure logic.

---

## 1. Philosophy & Culture: The Qianji Box (千机匣)

In the lore of **CyberXiuXian**, a "Qianji Box" is a legendary mechanical device of immense complexity and infinite adaptability. It represents the pinnacle of craftsmanship, where a thousand hidden mechanisms work in perfect unison to achieve a singular, transcendent purpose.

### 1.1 From Entropy to Ascension

Standard AI workflows (like LangGraph) often suffer from **"Computational Entropy"**—loose Python scripts that become unmanageable as complexity scales. `xiuxian-qianji` rejects this chaos. We treat every agentic workflow as a **Refined Artifact**.

- **The Iron Frame:** Like the tempered steel of a cultivation blade, our graph kernel is unyielding and formally verified.
- **The Divine Logic:** Like the flow of Qi through meridians, our scheduling is dynamic, probabilistic, and self-aware.

### 1.2 The Artisan's Way

We believe that an Agent should not just "execute code"—it should **"Cultivate Reasoning."** By moving the entire orchestration logic into this Rust-native engine, we achieve a state of **Intelligence-Knowledge Decoupling**, allowing the system to outlive the foundational models it employs.

---

## 2. Core Architecture: The Triple Mechanisms

### 2.1 The Iron Frame (Kernel)

Based on `petgraph::StableGraph`, the Iron Frame provides the physical structure. It supports millions of nodes with near-zero traversal overhead and utilizes **LTL (Linear Temporal Logic)** guards to ensure that no Agent falls into an "Infinite Loop" (the Zen of Termination).

### 2.1.1 Rust-Native Workflow Kernel

Qianji also exposes a Rust-native workflow kernel for hot-path package
pipelines that need typed stage execution rather than generic host-work
dispatch. The kernel records stage order, typed or Arrow-backed edge facts,
latency, item counts, and failure traces while keeping execution close to
direct Rust function calls.

Both `petgraph` DAGs and a bounded BPMN subset can be native front ends for
this kernel. The architectural boundary is not graph notation versus code; the
boundary is whether the chosen notation compiles into strongly typed Rust
stages and edge contracts. Visual, BPMN, or manifest layers may describe the
control flow, but shard hot paths should still execute over Rust-owned types
and Arrow-compatible row contracts.

The front-end-neutral `WorkflowTopology` contract is the shared handoff point:
BPMN and `petgraph` adapters declare required stages and dependency edges, and
`WorkflowRun` can bind that topology before typed execution. A checked finish
rejects missing required stages, undeclared stages, duplicate successful
stages, and edge-order violations, giving promotion gates a deterministic
structure guard before any higher-level adapter reports success.

The same kernel now supports in-process memory checkpoints for stage edges.
Checkpoint metadata is stored in Qianji traces, while the retained payload is a
typed Rust handle owned by the producer. This lets Arrow-backed consumers keep
same-process `RecordBatch` buffers available for retry, fan-out, and precision
rechecks without making Qianji depend on Arrow or replacing durable BPMN
checkpoint stores.

For homogeneous shard work, `WorkflowRun` provides bounded fan-out/fan-in with
order-preserving output collection. The helper enforces a caller-supplied
concurrency cap, records one workflow stage trace, reports failed item indices,
and still honors topology declarations. This is the lightweight scheduling
primitive for later audio, OCR, and graph-search shard execution; domain crates
still own the shard payloads, cache rules, and precision gates.

Workflow traces can also be projected into the independent Qianji control
plane through the workflow-kernel control adapter. The adapter maps
`WorkflowTrace` rows into workflow-neutral projection records for replay,
evidence, cost, lease, and recovery management. This is a one-way boundary:
Qianji still owns workflow/BPMN/Flowhub semantics and trace-to-projection
mapping, while
[`xiuxian-qianji-control`](../xiuxian-qianji-control/README.md) owns generic
run and step control state, durable event construction, and append batching.
Explicit recording returns the normal workflow report, the appended control
records, and the ledger-replayed run view so callers can inspect recoverable
management state without making recording the default finish path.
Topology-bound callers can use checked control recording to validate required
stages and dependency order before any control events are appended. Callers
that must retain typed workflow output after a control ledger failure can use
the recoverable recording path, which returns the completed workflow report
with the control error for retry or manual recovery. The
checked recoverable path combines both protections for topology-bound runs:
validate structure first, then preserve the report if only control persistence
fails. Recoverable control failures are directly retryable from the retained
report, so retrying persistence does not require rerunning workflow stages.
Retry callers can also opt into reusing an already-recorded control run, which
returns the existing replay view with zero appended events instead of writing a
duplicate event sequence. Workflow recovery controllers can append
run-scoped or step-scoped recovery attempts into the same control ledger,
making retry state visible in replayed run views without changing trace
recording defaults. Controllers can also attach step-scoped evidence
references after recording, allowing deterministic gates and audits to inspect
the exact artifacts or routes that justified a workflow step. Run-scoped and
step-scoped cost observations can be recorded through the same adapter, keeping
cost monitoring auditable without adding provider-specific cost policy to the
workflow kernel. Step-scoped gate results can also be recorded after evidence
or policy evaluation, keeping promotion and recovery decisions replayable
without making the workflow kernel execute the gate itself. For callers that
need to persist a complete stage decision, the managed decision helper records
evidence, gate results, and cost observations in a stable order and returns the
replayed control view for inspection. The managed recovery decision helper
extends that order with a final recovery attempt and requires a failed gate
result, keeping lifecycle mutation explicit and gate-driven.
Those projection, observation, and decision helpers now live in
[`xiuxian-qianji-control`](../xiuxian-qianji-control/README.md) because they
only operate on workflow-neutral run, step, stage projection, evidence, gate,
cost, and recovery contracts. The local workflow-kernel control adapter keeps
only the semantic mapping from Qianji's concrete `WorkflowTrace` type into
that neutral projection record.
Workflow callers can also opt into required-evidence projection when recording
a trace. The default projection keeps `StepCreated.required_evidence` empty,
while the opt-in contract lets a caller attach deterministic stage evidence
keys before later evidence attachment or gate evaluation.

### 2.2 The Divine Logic (Scheduling)

- **Probabilistic MDP Routing:** Decisions are not binary. Edges carry weights influenced by **Omega's Confidence**, allowing the system to explore multiple paths based on probability.
- **Adversarial Loops:** Natively supports the **Synapse-Audit** pattern, where nodes actively challenge and verify each other’s evidence links.

### 2.3 The Mirror Face (Qianhuan Integration)

Qianji is a **High-Performance Annotator**. In the milliseconds before a node executes, it calls upon `xiuxian-qianhuan` to transmute raw data into persona-aligned context, ensuring the Agent always wears the correct "Face" for the task.

The formal-audit advisory bridge can opt into `advisory-prompt-pack-cache`.
When that feature is enabled, callers may supply a
`xiuxian-db-store::artifact_cache::ArtifactBlobCache` to
`build_plan_with_prompt_context_pack_cache(...)` or attach it to
`QianjiLlmAdvisoryAuditExecutor` with
`with_prompt_context_pack_cache(...)`. Contract-feedback callers that use
`QianjiLiveContractFeedbackRuntime` can attach the same cache to the runtime
bundle before running live advisory feedback; concrete feedback routes then
propagate the injected cache to the live advisory executor without constructing
route-local cache backends. Qianji still owns only the workflow/advisory plan
boundary; Qianhuan owns prompt-context pack identity and serialization; db-store
owns the artifact cache backend. Plans and live findings report per-role
prompt-context pack cache hits and byte counts without changing the default
advisory planning path. When the cache is enabled, Qianji calls Qianhuan's
owned fetch-through helper for prompt-context packs; db-store and Foyer perform
the same-key miss coalescing, while Qianji remains a cache consumer and never
constructs a concrete cache backend.

---

## 3. Declarative Orchestration (The TOML Manifest)

True to the **"Rust-Hard, Host-Thin"** mandate, the "Thousand Mechanisms" are
defined through a declarative TOML manifest. The primary operator surface is
the `qianji` CLI, while the Rust API remains available for embedding and
testing.
The BPMN host-integration path now also follows that same split explicitly:
`QianjiBpmnWorkflowControlService` is the lib-owned workflow start/control seam
for bundle loading, checkpoint backend resolution, and execution-facade
dispatch, while `qianji bpmn start` is the explicit control-plane CLI adapter
that parses local fixtures and renders the resulting report. The older
`qianji bpmn run` command remains as a compatibility alias over that same
start path. The same checkpoint-first control surface now also backs
`qianji bpmn status`, which inspects persisted workflow state without
re-running the BPMN driver, `qianji bpmn resume`, which resolves
checkpoint-backed process identity explicitly before continuing a waiting or
host-blocked workflow instance, `qianji bpmn events poll`, which gives
external-event ingress an explicit operator action above the same checkpointed
resume path, `qianji bpmn tasks complete`, which gives pending host work an
explicit operator action above the same checkpointed resume path, and
`qianji bpmn cancel`, which deletes one persisted workflow checkpoint through
an explicit operator path while keeping the Valkey runtime backend under
strict scheduler-agent lease ownership. The lib service mirrors those
operator intents with `poll_workflow_events(...)`,
`complete_workflow_task(...)`, `claim_workflow_task(...)`,
`release_workflow_task(...)`, and `list_workflow_worklist(...)`, so later
HTTP/API adapters can call the BPMN workflow-control actions directly instead
of routing through CLI-specific names. These are concrete workflow-control
service actions in this crate, not workflow-neutral
`xiuxian-qianji-control` ledger APIs.
Concrete event-poll, task-complete, and batch-complete request types derive
their matching resume requests through `workflow_resume_request(...)`, keeping
service, HTTP, and CLI adapters thin while checkpoint backend resolution stays
inside this crate.
The pending-host stream exposes Rust-owned `process_id`, BPMN `activity_id`,
optional parsed `form` metadata, and optional standard BPMN `assignment`
metadata for `userTask` and `manualTask` entries, so UI adapters can render,
route, and complete work by stable workflow identity instead of inferring
identity or interaction fields from display labels. Assignment metadata is a
routing hint only; the bounded claim state exposed by
`qianji bpmn tasks claim`, `release`, and `worklist` is runtime coordination
metadata, not BPMN resource-role authorization. Form-backed human-task
completion is validated by the Rust runtime before variable merge: declared
required outputs must be present and undeclared fields are rejected.
The `qianji bpmn start`, `start-at`, `resume`, and `status` text outputs also
render pending host-work `activity`, `form`, and `assignment` summaries for
operator visibility; the JSON stream and HTTP snapshot remain the canonical
machine-readable contract.
`qianji bpmn tasks complete` requires a typed completion payload
(`--token-id <id> --process-id <id> --activity-id <id> --kind user|manual
--data-json <json>`) and no longer uses host fixtures as the canonical
task-completion path. The runtime rejects completion attempts whose process or
activity identity does not match the checkpointed pending host work.
`qianji bpmn tasks claim` and `release` use the same instance, token, process,
activity, and claimant identity tuple against checkpointed user/manual work.
`qianji bpmn tasks worklist` lists checkpointed user/manual work, optionally
including only unclaimed work and work already claimed by the supplied claimant.
`qianji_bpmn_workflow_router(...)` is the first embeddable HTTP JSON surface
over that same service layer. Its workflow snapshot responses include
`pending_host_work` entries with Rust-owned identity plus optional `form` and
standard BPMN `assignment` metadata, matching the stream contract for
non-stream clients. HTTP snapshots also expose host-dispatch details that
non-stream clients need to execute the work without falling back to the CLI
stream: resolved `node_id`, workflow `variables`, materialized task `inputs`,
declared `output_bindings`, and repeat metadata for bounded multi-instance
tasks.
The HTTP `POST /workflows/start` request accepts the optional
`start_at_node_id` field and forwards it to the same fresh-instance start-at
control path used by `qianji bpmn start-at`.
The HTTP task-completion surface also supports
`POST /workflows/{instance_id}/tasks/complete-batch` for multiple completions
from the same blocked host boundary. The batch path loads one checkpoint,
validates all pending host-work identities, applies the completion set inside
one BPMN session, and advances once to the next host boundary or terminal
outcome. Single-task completion remains available at
`/workflows/{instance_id}/tasks/complete`.
`qianji-server --bind 127.0.0.1:38130 --valkey-url redis://127.0.0.1:6379/0`
starts the minimal daemon shell over that router. HTTP defaults are
Valkey-only: omitted checkpoint backend fields resolve to the service-owned
Valkey URL when supplied, or to `[checkpoint].valkey_url` from `qianji.toml`
otherwise. The service bind address resolves from `[server].bind_addr` unless
`--bind` is supplied. `[server].require_valkey_ready` or
`--require-valkey-ready` can make startup fail before socket bind when Valkey
does not answer `PING`; `--no-require-valkey-ready` explicitly disables that
gate. `GET /healthz` reports service liveness, `GET /readyz` verifies
that the effective Valkey checkpoint backend responds to `PING`, and
`GET /capabilities` reports the stable server capability ids exposed by the
running process. Workflow-control clients should use `/capabilities` to reject
stale qianji-server processes before relying on recently added routes such as
`bpmn.workflow.task.complete-batch`, `bpmn.workflow.task.fail`, or
`bpmn.workflow.activity-evidence`,
`qianji.control.workflow-source.admit`,
`qianji.control.bpmn-source.admit`, `qianji.control.bpmn-source`,
`qianji.control.history`, `qianji.control.recovery.apply`, or
`qianji.control.worker.openai-compatible-llm.run`.
`--control-ledger <path>` enables an optional DuckDB-backed append-only control
ledger for server-owned BPMN execution trace and host-work ActivityTask
evidence. With that ledger configured, `GET /control/runs/{run_id}/history`
returns the in-process append-only event timeline for one control run, and
`GET /control/runs/{run_id}/bpmn-source` returns the BPMN XML read by
qianji-server from the source reference recorded on the run-created event. This
source route is for UI canvases and inspectors that need true BPMN element ids;
old clients may still render their own projection, but promotion-grade BPMN
markers should bind to the server-owned XML.
Skill.md and natural-language workflow authoring follow the same server-owned
boundary in product deployments. qianji-server owns the legal BPMN source
admission path. `POST /control/workflow-source/admit` accepts authoring source
such as `text/markdown`, compiles it inside qianji-server, runs qianji lint and
parse checks on the derived BPMN XML, verifies the requested process id, and
returns a server-owned `bpmn_path`/source ref for `/workflows/start`.
The first deterministic Markdown compiler admits explicit `## Step N: Title`
sections only. Free-form Markdown returns `workflow_source_repair_required`
until the server-owned Skill.md/pi-agent repair compiler is enabled; it is not
silently downgraded to a one-node BPMN fallback.
The request accepts `compiler_mode = "deterministic_markdown_step"` by default;
`compiler_mode = "server_repair"` enters the server-owned repair path. The
route requires the qianji-server durable control ledger and worker hot state;
without those substrates it returns a substrate-specific service-unavailable
error before writing any final admitted BPMN source. With durable substrates
installed, qianji-server starts the embedded
`qianji.workflow_source_repair.v1` BPMN repair workflow from
`resources/workflows/workflow_source_repair_v1.bpmn` and returns a
`repair_started` admission response. That workflow separates deterministic
source intake, qianji lint evidence, and final BPMN source admission from the
LLM draft, repair, and reasoning lint judge nodes. The server completes
`source_intake`, `run_qianji_lint`, and `admit_bpmn_source` itself and excludes
those deterministic nodes from generic LLM scheduling; LLM workers only receive
declared model-owned nodes such as `draft_bpmn`, `reason_lint_diagnostics`, and
`repair_bpmn`. Structured output bindings such as `candidateBpmn`,
`repairPlan`, and `admittedBpmnSourceRef` carry the repair state back through
the existing BPMN host-work completion contract.
`POST /control/bpmn-source/admit` remains the lower-level BPMN XML admission
route for already-generated candidates. Model-assisted draft generation through
pi-agent-capable worker surfaces, qianji lint/repair, source ref persistence,
control-run creation, and durable execution stay server-side. pi-wendao may
expose the same capability through a CLI for fast local tests, but server-side
wendao.ai must call qianji-server and render the server-owned BPMN source
instead of compiling or repairing BPMN in the UI process.
`GET /control/runs/{run_id}/summary` returns the replay-derived
`RunOperatorSummary` projection for operators that need activity, timer,
signal, cost, and recovery counters without parsing raw events.
`GET /control/runs/{run_id}/recovery` returns the replay-derived
`RunRecoverySnapshot`, including ordered recovery actions, without applying
those actions. `GET /control/runs/{run_id}/diagnostics` packages the same
summary and recovery projection from one durable event replay. When the server
is built with the Valkey feature and started with a recovery hot-state store,
`POST /control/runs/{run_id}/recovery/apply` applies the replay-derived plan
through the bounded `xiuxian-qianji-control` applier and returns the application
trace plus diagnostics. Omitted ledgers or hot-state stores keep existing HTTP
behavior unchanged and make these control routes return service-unavailable.
The `qianji-server` test suite is mounted through the library test tree because
it exercises private server adapters under `src/qianji_server_cli/`. Use
`direnv exec . rtk --ultra-compact cargo test -p xiuxian-qianji --lib
qianji_server` for the full server surface, and use focused `--lib` filters
such as `qianji_server_http_completion_records_host_work_activity_evidence`
when validating durable host-work evidence. The `unit_test` target covers the
external unit aggregate and intentionally does not run these private server
adapter tests.
The checkpoint backend remains Valkey-only for qianji-server workflow state.
Local no-server CLI/control workflow state defaults to
`$PRJ_DATA_HOME/xiuxian-qianji/duckdb/workflow-state.duckdb` unless
`qianji.toml` or `QIANJI_WORKFLOW_STATE_DUCKDB_PATH` provides an explicit
override.
The generic Qianji control ledger has its own operator surfaces.
`qianji control run-create --ledger <path> --run-id <id>
--occurred-at-ms <ms> --intent <text> [--json]` appends the explicit
`RunCreated` fact required before later run-scoped control events can be
admitted. The concrete CLI parses operator input and delegates the durable
append to `xiuxian-qianji-control`'s workflow-neutral run journal helper. It
does not schedule activities, create steps, mirror hot state, or execute
workflow logic. `qianji control history --ledger <path> --run-id <id> [--json]` renders the
append-only event timeline for one run. `qianji control heartbeat --ledger
<path> --run-id <id> --worker-id <id> --observed-at-ms <ms> --expires-at-ms
<ms> [--valkey-url <url>] [--namespace <ns>] [--metadata <json>] [--json]`
records a durable Worker liveness audit fact. Without `--valkey-url` it stays
ledger-only; with `--valkey-url` it mirrors the heartbeat to Valkey hot state
before appending the durable event, so a durable heartbeat fact represents a
successful live-state mirror. `qianji control view --ledger <path> --run-id
<id> [--json]` renders the deterministic replayed run state. `qianji control
query --ledger <path> --run-id <id> --state --now-ms <ms> [--json]`
returns a compact read-only state package with event count, replayed run view,
and recovery snapshot. `qianji control summary --ledger <path> --run-id <id>
--now-ms <ms> [--json]` renders a compact operator summary across event count,
run status, steps, active leases, activity lifecycle counts, timer counts,
signal counts, cost totals, and recovery counters without executing recovery
or mutating hot state. `qianji
control step --ledger <path> --run-id <id> --step-id <id> [--json]` renders
one replayed step view for step-local evidence, gate, Agent, activity, timer,
and signal inspection. `qianji control lease --ledger <path> --run-id <id>
--step-id <id> [--json]` renders the current replayed active lease for one
step without acquiring, renewing, releasing, reclaiming, or mutating hot
state. `qianji control leases --ledger <path> --run-id <id> [--json]` renders
the replayed active lease inventory for a run, including empty inventories,
without mutating hot state. `qianji control activity --ledger <path> --run-id
<id> --activity-id <id> [--step-id <id>] [--json]` renders one replayed activity
view for lifecycle, task, attempt, worker, result, and failure inspection.
`qianji control activity-queue --ledger <path> --run-id <id>
[--task-queue <queue>] [--json]` renders scheduled-but-not-started activity
tasks from durable replay, optionally filtered by task queue, without claiming
work or mutating hot scheduler state. The same projection includes lifecycle
summary counts for scheduled, in-flight, completed, and failed activities. JSON
output also includes `worker_tasks`, a worker-facing envelope derived from the
same durable schedule events with run, optional step, activity, queue,
idempotency, timeout, retry policy, scheduled timestamp, and next-attempt
fields.
`qianji control llm-activities --ledger <path> --run-id <id>
[--require-request-audit] [--json]` renders the replay-derived LLM activity
inventory for one run. It lists all `llm.*` activities and LLM-annotated tasks
with lifecycle status, attempts, model ids when the admitted request audit
metadata is present, and a missing request-audit count. The optional request
audit gate turns missing admitted-request metadata into a deterministic command
failure for CI or operator checks. The command is read-only: it does not call
providers, claim work, append events, or mutate hot state.
`qianji control activity-mirror --ledger <path> --valkey-url <url> --run-id
<id> [--namespace <ns>] [--task-queue <queue>] [--priority <n>]
[--not-before-ms <ms>] [--metadata <json>] [--json]` mirrors those
replay-derived worker task envelopes into the Valkey hot-state polling surface.
The command requires `duckdb` and `valkey`; it does not claim work, start
activities, or append ledger events.
`qianji control activity-claim --valkey-url <url> --worker-id <id> --now-ms
<ms> --lease-ttl-ms <ms> [--namespace <ns>] [--task-queue <queue>] [--json]`
claims one `WorkerActivityTask` from the Valkey hot-state mirror and returns a
leased worker payload. The command requires the `valkey` feature and does not
append ledger events; workers should record durable lifecycle facts with
`activity-start`, `activity-complete`, or `activity-fail`, preferably by
passing the returned task through `--worker-task-json`.
`qianji control activity-take --ledger <path> --valkey-url <url> --worker-id
<id> --now-ms <ms> --lease-ttl-ms <ms> [--namespace <ns>]
[--task-queue <queue>] [--json]` composes the polling and durable lifecycle
boundary for worker adapters: it claims one hot-state activity lease and, only
when a lease is claimed, appends the idempotent durable `ActivityStarted` fact
for the replay-derived worker task. The command requires both `duckdb` and
`valkey`. It does not execute the provider, complete or fail the activity, or
auto-release the hot-state lease if the durable start write fails; recovery
should use the lease TTL and `activity-reclaim`.
`qianji control activity-worker-once --ledger <path> --valkey-url <url>
--worker-id <id> --now-ms <ms> --lease-ttl-ms <ms>
--executor fixture|openai-compatible-llm|flowhub-service
--outcome complete|fail --settled-at-ms <ms> [--namespace <ns>]
[--task-queue <queue>] [--output-ref-json <artifact-ref-json>]
[--output-hash <hash>]
[--output-artifact-path <path> --output-artifact-content <text>
[--output-artifact-id <id>] [--output-artifact-kind <kind>]]
[--openai-compatible-base-url <url>] [--openai-compatible-api-key <key>]
[--openai-compatible-timeout-ms <ms>]
[--error-code <code>] [--message <text>]
[--retryable <true|false>] [--metadata <json>] [--json]`
is a bounded single-activity worker adapter. It claims one hot-state activity
task, records the durable start, applies the selected executor result,
records the durable terminal result, and releases the hot-state lease only
after the terminal ledger write succeeds. Completed worker results may carry
an explicit output `ArtifactRef` claim-check through `--output-ref-json`; failed
activities reject output refs because provider payloads must not be embedded in
failure records. Completed worker results may alternatively write a local
claim-check artifact with `--output-artifact-path` and
`--output-artifact-content`; the worker derives the output hash and
`ArtifactRef`, rejects conflicting existing file content, and records the
derived reference only after the artifact write succeeds. Empty polls do not
append ledger events. Registry execution is anchored to the claimed
`WorkerActivityTask` envelope, so provider adapters receive the durable
activity type, task queue, input ref, retry policy, timeout, and metadata
before they can produce a terminal result.
The registry also validates executor route ownership before execution. A
claimed task whose activity type or task queue is outside the selected
executor contract is rejected before any fixture or future provider adapter can
produce durable terminal output. Worker adapters run this route validation as
a preflight gate immediately after claiming hot-state work and before writing
`ActivityStarted`, so a mismatched executor cannot create misleading durable
lifecycle history. Successful claimed-task worker output includes the selected
executor contract snapshot for operator inspection; empty polls expose no
contract because no activity task was authorized. The initial fixture contract
recognizes the governed LLM roles `llm.plan`, `llm.tool_select`, and
`llm.repair` across local inspection queues for OpenAI, Anthropic,
OpenRouter, and local model workers. It also recognizes the deterministic
Episteme review route `episteme.ontology.reasoning_fill` on
`episteme.ontology.reasoning`; fixture completions on that route are
review-artifact outputs only and do not promote RDF or mutate Episteme source
truth. The worker-once adapter also carries an
OpenAI-compatible LLM executor for admitted `llm.openai`, `llm.openrouter`,
`llm.local`, and `episteme.ontology.reasoning` tasks when the claimed activity
type is explicitly admitted. Episteme reasoning tasks use local prompt and
context artifacts as request-audit inputs; their provider outputs remain
review artifacts and do not promote RDF. That executor requires a claimed task
input reference, admitted request-audit metadata, a local-file prompt reference
that matches the task input reference, `--openai-compatible-base-url`, and
`--output-artifact-path`. For `episteme.ontology.reasoning_fill`, the
request-audit metadata must also carry a local-file context reference with kind
`episteme.reasoning_fill_context`, the expected context schema, and non-empty
`contextEvidence` text. The context must also include the Episteme
object-model `targetContract` schema so the provider receives a deterministic
review-only `ObjectType` or `LinkType` candidate contract. The contract must
declare object-model compatibility, RDF source authority, disabled runtime
mutation, disabled RDF mutation, and allowed object-model patch kinds. It
delegates raw OpenAI-compatible `/chat/completions` transport to
`xiuxian-llm`, writes successful provider responses to the output artifact,
and stores a canonical `episteme_review` JSON object after accepting either raw
JSON or fenced JSON provider content. Qianji still owns the worker claim,
request-audit validation, durable terminal event, and retry/failure policy;
`xiuxian-llm` owns provider HTTP transport and wire-level response validation.
Episteme review content must match the expected schema, fill item id, target
ledger field group, allowed patch kind, `candidatePatchCount`, candidate
evidence, and `rdfMutation=false` contract before completion. The durable
completion event records only the derived claim-check, while HTTP, malformed
response, contract, or input materialization failures are recorded as durable
activity failures.
BPMN host-work that should become an LLM activity is configured through
Qianji-owned workflow/task profiles under
`packages/rust/crates/xiuxian-qianji/resources/config/workflows/`; user
overlays live under `workflows/` next to the user `qianji.toml`. The default
profile is `bpmn-host-work-llm`, which maps stable BPMN pending-host identity
to an admitted `LlmActivityTask` with `llm.plan`, `llm.openrouter`, retry,
timeout, and prompt claim-check requirements. `qianji.toml` remains the
server/global runtime default surface; task-level workflow routing is not
stored in `xiuxian-llm` model-routing config.
For server-backed execution, `qianji-server` exposes a bounded
`run_qianji_server_openai_compatible_llm_worker_loop` facade. The facade
mirrors replay-derived `llm.*` activities into hot state, claims leases,
records durable start and terminal events, delegates raw provider transport to
`xiuxian-llm`, writes deterministic response artifacts, and releases leases.
It is intentionally explicit and finite; always-on background supervision is a
separate server process concern rather than a task-profile or `xiuxian-llm`
model-routing concern. When the server is built with the full Valkey worker
features and has both a control ledger and hot-state store, the same bounded
worker is available over
`POST /control/runs/{run_id}/workers/openai-compatible-llm/run`. Provider
base URL, API key, model, and wire API resolve from `qianji.toml` and process
environment through the Qianji runtime config; the HTTP request supplies only
worker bounds, queue selection, timestamps, timeout, and the local artifact
output directory.
The `flowhub-service` executor is the deterministic worker path for BPMN
service tasks materialized from Flowhub Org+BPMN scenarios. It admits only
`flowhub.service` tasks on `flowhub.*` queues, requires the replay-derived
pending-host-work input reference, validates the Flowhub service-task metadata,
and derives completion metadata from the BPMN `requiredOutputs` contract. It
does not accept fixture output refs, output artifacts, ad-hoc metadata, or
failure arguments; retries and failures remain activity lifecycle facts, while
successful execution records the exact completion data later used by the
qianji-server BPMN task-completion route.
Workflow authoring workers that convert Skill.md or natural language into BPMN
must use the same server-side discipline: they are workers behind
qianji-server, not frontend compilers. They can use pi-agent tools for file
inspection and repair assistance, but qianji lint/repair remains the admission
gate and the accepted BPMN source is recorded by the server before execution.
`qianji control activity-worker-loop --ledger <path> --valkey-url <url>
--worker-id <id> --now-ms <ms> --lease-ttl-ms <ms> --poll-limit <n>
--executor fixture|openai-compatible-llm|flowhub-service --outcome complete|fail
--settled-at-ms <ms>
[--namespace <ns>] [--task-queue <queue>] [--now-step-ms <ms>]
[--heartbeat-ttl-ms <ms>] [--settled-step-ms <ms>] [--empty-limit <n>]
[--output-hash <hash>] [--output-artifact-dir <dir>]
[--output-artifact-kind <kind>] [--openai-compatible-base-url <url>]
[--openai-compatible-api-key <key>] [--openai-compatible-timeout-ms <ms>]
[--error-code <code>] [--message <text>]
[--retryable <true|false>] [--metadata <json>] [--json]` is a bounded finite
worker-loop adapter. It reuses `activity-worker-once` semantics for each poll,
stops at `--poll-limit` or after the configured empty-poll streak, and does
not sleep, spawn a daemon, or append ledger events for empty polls. When
`--heartbeat-ttl-ms` is supplied, each claimed task also records a Worker
heartbeat through the same Valkey hot-state and durable ledger path before the
fixture result is applied. Empty polls still do not write heartbeats because
there is no run-scoped activity task to anchor the durable event. With
`--executor openai-compatible-llm`, the loop requires `--outcome complete`,
`--openai-compatible-base-url`, and `--output-artifact-dir`; each claimed LLM
task derives a deterministic provider-response artifact path from the activity
id and attempt number, then reuses the worker-once OpenAI-compatible executor
so provider failures remain durable activity failures after start.
With `--executor flowhub-service`, the loop uses the same deterministic
contract-completion behavior as `activity-worker-once` for each claimed
Flowhub service task and rejects output artifact or ad-hoc metadata flags.
`qianji control activity-reclaim --valkey-url <url> --lease-json <json>
--now-ms <ms> [--namespace <ns>] [--json]` reclaims a claimed hot-state
activity lease only after it is expired at the supplied observation time and
returns the activity task to the hot-state queue. It does not append ledger
events and is intended for recovery loops, not normal successful worker
completion.
`qianji control activity-release --valkey-url <url> --lease-json <json>
[--namespace <ns>] [--json]` releases a claimed hot-state activity lease after
the worker has recorded the durable terminal lifecycle fact. It does not append
ledger events, requeue work, or replace `activity-complete` / `activity-fail`.
`qianji control activity-schedule-llm --ledger <path> --run-id <id>
--occurred-at-ms <ms> --llm-activity-json <json> [--step-id <id>] [--json]`
validates a serialized `LlmActivityTask` through the control crate's LLM
admission contract, then records an idempotent durable `ActivityScheduled`
fact. It does not call a model provider, enqueue Valkey work, acquire a lease,
or start a worker.
`qianji control activity-admit-plan --ledger <path> --run-id <id>
--occurred-at-ms <ms> --schedule-plan-json <path> [--step-id <id>] [--json]`
admits a precompiled Qianji activity schedule plan. Each plan row must carry
the supported schedule contract, the same run id, safe non-execution flags,
pending status, and a valid workflow-neutral `ActivityTask` with an input
claim-check. The command appends or reuses durable `ActivityScheduled` facts
only. It does not execute workers, mirror Valkey hot state, call models, read
private source text, or mutate ontology data.
`qianji control activity-settle --ledger <path> --valkey-url <url>
--leased-task-json <json> --outcome complete|fail --settled-at-ms <ms>
[--namespace <ns>] [--output-hash <hash>] [--error-code <code>]
[--message <text>] [--retryable <true|false>] [--metadata <json>] [--json]`
is the worker adapter terminal helper. It expects the `claimed` payload
returned by `activity-take`, records the idempotent durable complete or fail
event first, and releases the hot-state activity lease only after that durable
write succeeds. The command requires both `duckdb` and `valkey`. It does not
execute provider code, schedule retries, or reclaim expired leases; failed
durable writes leave the active hot-state lease for TTL-based recovery.
`qianji control costs --ledger <path> --run-id <id> [--json]` renders durable
run and step cost observations with event sequence, observed timestamp, token
counts, latency, and USD micros totals, without appending observations or
mutating hot state.
`qianji control activity-start --ledger <path> --run-id <id> --activity-id
<id> --worker-id <id> --started-at-ms <ms> --attempt <n> [--step-id <id>]
[--json]` records an idempotent durable `ActivityStarted` fact through the
control crate's replay guards. It does not complete, fail, execute, or lease
the activity. Worker adapters can instead pass a replay-derived task envelope
from `activity-queue` with `--worker-task-json <json> --worker-id <id>
--started-at-ms <ms>` so the CLI derives run or step scope, activity id, and
attempt from durable history rather than operator-supplied duplicate fields.
`qianji control activity-complete --ledger <path> --run-id <id> --activity-id
<id> --completed-at-ms <ms> [--step-id <id>] [--output-hash <hash>]
[--metadata <json>] [--json]` and `qianji control activity-fail --ledger
<path> --run-id <id> --activity-id <id> --failed-at-ms <ms> --error-code
<code> --message <text> --retryable <true|false> --attempt <n> [--step-id
<id>] [--metadata <json>] [--json]` record idempotent durable terminal
activity lifecycle facts after replay verifies that the target activity is in a
started state. Both terminal commands also accept the same
`--worker-task-json <json>` envelope mode; completion still requires the
terminal timestamp and optional output metadata, while failure still requires
error code, message, retryable flag, and optional metadata. In envelope mode
the failure attempt comes from the task's replay-derived `next_attempt`.
`qianji control decision --ledger <path> --run-id <id> --decision-id <id>
[--step-id <id>] [--json]` renders one replayed agent decision for proposal,
outcome, reason, scheduled activity, checkpoint, and gate inspection.
`qianji control timer --ledger <path> --run-id <id> --timer-id <id>
[--step-id <id>] [--json]` renders one replayed durable timer for scheduled
fire time, fired time, and status inspection. `qianji control timers --ledger
<path> --run-id <id> [--json]` renders the replayed run and step timer
inventory with pending, scheduled, and fired counts, without firing timers,
enqueueing work, or mutating hot scheduler state.
`qianji control signal --ledger <path> --run-id <id> --signal-name <name>
--payload <json> --received-at-ms <ms> [--step-id <id>] [--json]` appends one
durable external signal event. The payload JSON is stored in signal metadata so
the existing control event schema remains unchanged; the CLI delegates the
append to `xiuxian-qianji-control`'s workflow-neutral signal journal helper
instead of constructing the durable event locally. `qianji control signals
--ledger <path> --run-id <id> [--json]` renders the replayed run and step
signal inventory with event sequence, received timestamp, and scope counts,
without appending signals or mutating hot state.
`qianji control recovery-snapshot --ledger <path> --run-id <id> --now-ms <ms>
[--json]` reads the same `xiuxian-qianji-control` DuckDB event ledger and
returns the replay-derived recovery view, ordered recovery plan, and compact
summary without executing recovery actions or touching hot scheduler state.
`qianji control apply-recovery-plan --ledger <path> --valkey-url <url>
--run-id <id> --now-ms <ms> --attempt <n> --reason <text> --max-attempts <n>
[--namespace <ns>] [--backoff-ms <ms>] [--require-human-approval]
[--priority <n>] [--json]` records a recovery-start fact and applies the
current bounded recovery plan through `xiuxian-qianji-control` against the
Valkey hot-state mirror. The command can requeue run-scoped activity retries
into the hot activity queue, enqueue step-scoped retries, fire ready timers,
and reclaim expired leases. Unsupported actions are reported as skipped
results.
`qianji control hot-state --valkey-url <url> --now-ms <ms> [--namespace <ns>]
[--json]` reads the Valkey hot scheduling state directly and renders pending
steps, leased steps, lease expiry state, and worker heartbeat visibility. This
is an operator snapshot for live queue debugging; it does not mutate the
append-only control ledger and requires the `valkey` feature.

```toml
name = "artifact_refining_pipeline"

[[nodes]]
id = "Seeker"
task_type = "knowledge"
weight = 1.0
params = {}

[[nodes]]
id = "Auditor"
task_type = "calibration"
weight = 1.0
params = {}

[[edges]]
from = "Seeker"
to = "Auditor"
weight = 1.0
label = "Verify"
```

The Flowhub graph-contract direction for reusable Flowhub flows, materialized plan
work surfaces, Codex operational workdirs, and validation-first
materialization is tracked in
[RFC: Qianji Flowhub Graph Contract Model](docs/rfcs/2026-04-07-qianji-flowhub-graph-contract-model-rfc.md).
Scheduler preflight also consumes Wendao semantic-scope metadata from run
context. When `semanticScopeMetadata` is present, Qianji injects
`semanticScopeGuardTrace` and `semanticScopeGuardRoute` before mechanism
execution. The trace exposes semantic status and evidence; the route exposes
the configured `semanticScopeGuardPolicy`, the current guard status, whether
execution is continuing, and the recommended semantic action. The default
policy remains advisory, and Qianji still consumes semantic truth without
owning it.
Router nodes may opt into that route with `semantic_guard_route = true`, or
with `semantic_guard_route_key` for a custom context key. When enabled,
`semanticScopeGuardRoute.recommendedAction` selects a matching branch such as
`continue`, `review_required`, or `blocked` before probabilistic fallback.
`resources/tests/semantic_guard_route_branch.toml` is the checked-in resource
fixture for that workflow shape and is covered by the
`semantic_guard_route` integration test filter. Operators can render the same
shape with `qianji template --semantic-guard-route`.
That RFC now also treats scenarios as guard graphs over the bounded work
surface, with explicit done-gate semantics and blocked-vs-failed diagnostics
for `qianji check`.
The retrieval-facing SQL surface for bounded plan work is tracked in
[RFC 0003: Wendao SQL Minimal Retrieval Surface for Bounded Plan Work](docs/rfcs/2026-04-07-wendao-sql-minimal-retrieval-surface-rfc.md).
The compact validation and flowchart-alignment contract for bounded plan work
is tracked in
[RFC 0004: Compact Validation and Flowchart Alignment](docs/rfcs/2026-04-08-compact-validation-flowchart-alignment-rfc.md).
The markdown `skeleton` contract for bounded plan work is tracked in
[RFC 0005: Markdown Skeleton Minimal Rules for Bounded Plan Work](docs/rfcs/2026-04-08-markdown-skeleton-minimal-rules-rfc.md).
The stable row-segmentation contract for the `markdown` retrieval surface is
tracked in
[RFC 0006: Markdown Row Segmentation Minimal Rules for Bounded Plan Work](docs/rfcs/2026-04-08-markdown-row-segmentation-minimal-rules-rfc.md).
The minimum visible `flowchart.mmd` backbone contract for bounded plan work is
tracked in
[RFC 0007: Flowchart Backbone Minimal Rules for Bounded Plan Work](docs/rfcs/2026-04-08-flowchart-backbone-minimal-rules-rfc.md).
The stable external `heading_path` convention for the `markdown` retrieval
surface is tracked in
[RFC 0008: Heading Path Minimal Conventions for Bounded Plan Work](docs/rfcs/2026-04-08-heading-path-minimal-conventions-rfc.md).
The research-layered workspace split for localized runs, persistent
single-paper packages, and cross-paper topic synthesis is tracked in
[RFC: Qianji Research Workspace Layering](docs/rfcs/2026-04-18-qianji-research-workspace-layering-rfc.md).
The current implementation-status matrix for this active RFC cluster is
tracked in
[Audit: Qianji RFC Implementation Coverage](docs/rfcs/2026-04-08-qianji-rfc-implementation-coverage-audit.md).
In that model, directory shape is the first structural signal, optional
`tree` is only a bounded probe for deciding whether deeper inspection is
necessary, and exact fragment retrieval is performed through Wendao SQL rather
than through a Qianji-specific query DSL.
The crate now also exposes bounded `workdir` helpers for compact root-manifest
parse/load/validate, first-order surface rendering, and structural
`flowchart.mmd` checks, plus a Flowhub runtime bridge for real
`qianji-flowhub` roots and module directories. The `qianji` binary now accepts
`show --dir <path>` plus `check --dir <path>` and auto-detects whether the
target is a bounded work surface or a Flowhub root/module, with direct binary
coverage for rendered output and blocking invalid-check status.
The `qianji-server` HTTP surface also serves `/flowhub/scenarios` as the
Gateway-backed Org+BPMN scenario registry. It reuses the
`xiuxian-qianji-client` source-pair contract and returns the same
`sourcePairs` JSON fields consumed by pi-wendao's Flowhub scenario provider.
The server resolves its Flowhub root
from `--flowhub-root`, then `QIANJI_FLOWHUB_ROOT`, then
`PRJ_ROOT/qianji-flowhub`, then a local `qianji-flowhub` directory.
Flowhub BPMN `serviceTask` boundaries can now be converted into the generic
Qianji control-plane `ActivityTask` schedule contract through
`build_flowhub_service_activity_schedule_record`, which is owned by
[`xiuxian-qianji-runtime`](../xiuxian-qianji-runtime/README.md). That runtime
adapter preserves BPMN instance, process, activity, token, source path, and
declared task-output metadata in the scheduled task and routes work to
`flowhub.<scenario-id>`, while
[`xiuxian-qianji-control`](../xiuxian-qianji-control/README.md) owns the
workflow-neutral durable activity, queue, lease, retry, and replay semantics
behind the scheduled task. The same runtime crate owns the deterministic
completion contract helpers that validate required task-output fields and
derive bounded worker output. `build_flowhub_service_task_completion_payload`
wraps that runtime-neutral completion contract into the typed BPMN service
completion payload after a worker has produced output. The pure schedule and
completion adapters do not execute the service task, append worker lifecycle
events, mutate hot state, or complete the BPMN task. The companion
`build_flowhub_service_task_complete_http_request` helper builds the bounded
HTTP completion request from the same replay-derived `WorkerActivityTask`
metadata, so worker bridges can post through the existing qianji-server
`/workflows/{instance_id}/tasks/complete` route instead of mutating BPMN state
directly. For server-backed execution,
`run_qianji_server_flowhub_service_worker_completion_loop` is now a thin
wrapper over the runtime-owned bounded loop. The runtime loop reads the BPMN
checkpoint frontier through `QianjiRuntimeWorkflowControlPort`, records the
Flowhub service `ActivityTask` in the control ledger, mirrors replay-derived
work into hot state, claims and executes the deterministic `flowhub-service`
contract worker, records the durable terminal activity event, completes the
BPMN service task through the same port, and releases the lease. qianji-server
still owns concrete HTTP/checkpoint state assembly and passes its service/host
pair into the runtime loop.
Worker bridges that only have qianji-server HTTP snapshots can still use
`build_flowhub_service_activity_schedule_record_from_http_pending_work` to turn
pending service-work DTOs back into the same durable schedule contract.
The current server proof completes the full linear `agent-coding` service
chain through the reusable server worker completion loop.
Generic BPMN host work now uses the runtime-owned BPMN-to-ActivityTask
evidence adapter in
[`xiuxian-qianji-runtime`](../xiuxian-qianji-runtime/README.md) for native
host-tool completions that are not Flowhub service contracts. `xiuxian-qianji`
keeps HTTP request parsing and maps typed server payloads into the
runtime-neutral completion and failure facts. Durable ledger semantics are
still owned by
[`xiuxian-qianji-control`](../xiuxian-qianji-control/README.md): qianji-server
supplies the run id, pending work, worker id, timestamps, and HTTP-derived
facts, while the runtime adapter composes control-owned run creation, activity
schedule, worker start, and terminal completion or failure helpers. When
qianji-server starts with
`--control-ledger <path>`, workflow start, resume, poll, and task-completion
routes project BPMN node status into the durable control run
`bpmn.workflow.{instance_id}`. The trace projection records run creation,
admission, plan summary, BPMN element-id step events, and terminal blocked,
completed, or failed run status from the checkpointed workflow session.
Successful single-task and batch task-completion routes additionally match the
submitted completion ids against the checkpoint's pending host work through the
runtime-owned `BpmnHostWorkIdentity` matcher and append the replayable
`RunCreated` -> `ActivityScheduled` -> `ActivityStarted` -> `ActivityCompleted`
chain to the control ledger. The adapter records BPMN instance, process,
activity, token, host-work kind, input reference, and completion hash metadata
without changing the HTTP task-completion payloads, the BPMN checkpoint schema,
or pi-wendao's `Agent` / `get_subagent_result` host-tool contract.
`POST /workflows/{instance_id}/tasks/fail` uses the same checkpoint-backed host
work identity guard to append `RunCreated` -> `ActivityScheduled` ->
`ActivityStarted` -> `ActivityFailed` for native host-work failures. The failure
route is evidence-only: it does not complete the BPMN task, advance tokens,
or mutate checkpoint state. `GET /control/runs/{run_id}/history` exposes that
same durable event chain through qianji-server while the process owns the
DuckDB ledger connection, avoiding a separate external ledger reader.
`GET /control/runs/{run_id}/bpmn-source` exposes the BPMN XML from the
server-recorded source reference so browser canvases can align markers with
real BPMN ids instead of a registry projection.
`GET /control/runs/{run_id}/summary` projects the same ledger stream into an
operator-safe summary, including failed activity counters for recovery UI and
subagent-runner status checks. `GET /control/runs/{run_id}/recovery` exposes
the read-only recovery plan derived from the same event stream, such as retry
review or terminal escalation actions, but does not enqueue work, retry
activities, fire timers, reclaim leases, mutate hot state, delete checkpoints,
or change the completion payload schemas. The diagnostics route
`GET /control/runs/{run_id}/diagnostics` returns the combined operator summary
and recovery package from one replay. The recovery apply route
`POST /control/runs/{run_id}/recovery/apply` is the opt-in mutation surface for
the bounded recovery applier and requires both the control ledger and a
configured recovery hot-state store.
The same `workdir` surface now also exposes a thin bounded markdown query
wrapper over Wendao SQL, so library callers can execute exact SQL retrieval
against `blueprint/` plus `plan/` without changing the `qianji` CLI surface.
Failing workdir checks can now also derive one default follow-up skeleton query
from their current diagnostics, so repair-oriented callers can fetch only the
bounded markdown surfaces implicated by the current `qianji check` failure
without widening the command surface.
That same bounded guidance now appears directly in failing `qianji check --dir`
workdir output as a `## Follow-up Query` section, while success output and the
Flowhub / scenario check surfaces remain unchanged.
The Flowhub materialize lane now also consumes that follow-up surface on
generated-workdir validation failure, so invalid materialized outputs return
the blocking markdown diagnostics together with one bounded SQL repair query
instead of only the raw failure report.
The crate now also exposes early parser helpers for hierarchical Flowhub
module refs and composite module manifests (`rust`,
module-root `[template]` / `template.link`), plus file-backed manifest
loaders and bounded nested-module resolution, while full operational
work-surface materialization and deeper bounded execution semantics remain
later slices.
The Flowhub library surface now owns that whole lane under one namespace:
module/scenario manifest parsing, hierarchical resolution, and scenario
preview/check over real `qianji-flowhub` node graphs. The real Flowhub tree is
now qianji.toml-only at each node, so checked-in `template/` and `validation/`
surfaces are no longer part of the live library contract. The early
library-only materialize core remains covered through test-only template
fixtures rather than the real Flowhub root. The Flowhub root is now anchored
by its own `qianji.toml` with `[contract].register` plus
`[contract].required`, so top-level node ownership is explicit (`coding`,
`rust`, `blueprint`, `plan`) and undeclared child directories now count as
Flowhub structural drift. The main RFC now also documents `[contract]` as the
primary structure contract and keeps `[[validation]]` as an optional secondary
rule surface, including the current grammar limits for `register`, `required`,
`*/...` expansion, and the minimum markdown diagnostic shape for contract
failures. The crate now also routes Flowhub, scenario, and workdir check
rendering through one shared internal markdown diagnostics surface so the
`qianji check` output shape stays aligned across targets. The crate now also
routes Flowhub root/module previews, Flowhub scenario previews, and bounded
work-surface previews through one shared internal markdown show surface so
`qianji show` keeps the same H1-plus-metadata-plus-H2-sections contract
across targets. Live Flowhub nodes may now also own immediate Mermaid
scenario-case graphs such as `qianji-flowhub/plan/codex-plan.mmd`, contracted
through `[contract].required`. `qianji check` now parses those `.mmd` files
through the mature `merman-core` render-model parser, classifies labels
matching live Flowhub module names as graph-module nodes, and rejects
malformed or uncontracted scenario-case graphs. A valid scenario-case graph
must resolve its `merimind_graph_name` from module-owned `[[graph]].name`
when declared, otherwise from the owning filename stem rather than from the
Mermaid direction token, must cover every registered Flowhub module node
required by the current root contract, and must keep one connected module
backbone across those module nodes. Undeclared graph-node labels such as
stale semantic-node names now fail validation explicitly, and
`qianji show --dir .../plan` now surfaces each immediate Mermaid case through
explicit `Graph name: <merimind_graph_name>` and `Path: ./plan/<file>.mmd`
fields in the markdown preview surface. The
control-plane markdown renderer path is now also deduplicated through one
shared embedded `qianhuan` template catalog exported by
`xiuxian-qianhuan`, so `show`, `check`, Flowhub-root/module blocks, and
Flowhub-scenario preview blocks no longer each own a separate local
`OnceLock` plus embedded-template bootstrap path inside `xiuxian-qianji`.
Those control-plane templates now also live as checked-in `.md.j2` files
under `resources/templates/control_plane/`, so the Rust side only keeps
payload assembly plus `include_str!` bindings rather than large inline
template strings.
The same CLI surface now also splits graph understanding from localized
contract evaluation explicitly: `qianji show --graph <scenario.mmd>` renders
the graph contract surface in five bounded sections only: graph metadata, raw
Mermaid, node semantics, expected work surface, and the minimal local
`qianji.toml` template that Codex or any other agent executor should
materialize. That graph surface still parses node/edge structure through the
mature Mermaid parser, resolves `merimind_graph_name` from `[[graph]].name`
or the filename-stem fallback, renders `Path` as the owning Mermaid file with
repo-root-relative display when the graph lives under the active checkout,
and aligns registered module nodes back to the Flowhub root contract plus
module exports, while
`qianji check --dir <workdir>` continues to evaluate the localized workdir
contract materialized for the current bounded slice.
The current execution model is now explicit in the docs: Codex is the
execution layer, `qianji-flowhub` is the constraint layer, and
`qianji check` is the evaluation layer. The localized workdir contract stays
intentionally small, with only `[plan].name`, `[plan].surface`,
`[check].require`, and `[check].flowchart`. The main RFC now also freezes the
`show --graph` output contract itself: graph metadata, raw Mermaid, node
semantics, expected work surface, and the localized `qianji.toml` template.
The new research-layered RFC narrows that rule further for research lanes:
the localized run surface remains the execution plane only, while persistent
paper packages and topic syntheses live outside it under `runs/`, `papers/`,
and `topics/`. In that model, user-visible answers are materialized previews,
not the authority source for research state.
The same RFC now also freezes the v0 node taxonomy and label-normalization
rules for `show --graph`, plus the v0 `Next` edge semantics for backbone,
fail, and repair-loop edges. The same `Nodes` contract now also fixes the
wording boundary: `Role` stays descriptive and `Agent action` stays
imperative. The same RFC now also fixes `unknown` node failure semantics:
visible in `show --graph`, blocking in `qianji check`, and excluded from
localized contract materialization guidance. Module alignment in the same RFC
is now also explicit: module nodes are anchored by root `contract.register`,
and export alignment stays bounded to `entry` and `ready`. The same RFC now
also freezes the graph path and naming contract: `Name` is the resolved
`merimind_graph_name` from `[[graph]].name` or the filename-stem fallback,
and `Path` is the owning Mermaid file rendered repo-root-relative when the
graph lives under the active checkout. The same RFC now also freezes the
Mermaid consumption boundary for `show --graph`: the
raw Mermaid block stays verbatim, while graph-contract semantics consume only
first-order node labels and directed adjacency rather than Mermaid
presentation directives such as direction, styling, or click metadata. The
current parser path now code-backs that boundary directly by delegating
flowchart syntax acceptance to `merman-core`, including repeated labels,
subgraphs, directives, and expanded node-shape syntax, while the Qianji
projection still only keeps direction plus first-order node and edge
semantics and the rendered `## Mermaid` block stays verbatim.
The same Flowhub graph-contract surface now also carries explicit topology
semantics. Flowhub-owned scenario-case graphs may declare whether they are
`dag`, `bounded_loop`, or `open_loop` through module-owned `[[graph]]`
entries, and `qianji check` / `qianji show --graph` now evaluate that declared
topology through a petgraph-backed analysis layer instead of relying only on
first-order backbone checks.
The crate now also exposes one separate LLM-facing contract snapshot surface:
`qianji show --contract wendao.docs.navigation` or
`qianji show --contract wendao.docs.retrieval_context`. That bounded display
renders the checked-in Wendao invocation snapshot as raw `contract.toml` plus
`schema.json`, keeping the stable HTTP method/path, matching `wendao docs ...`
CLI form, and tool-input schema separate from the frozen `show --graph`
output contract. The same slice also lets `qianji.toml` author real
`http_call` and `cli_call` nodes directly, with the authored invocation fields
validated against the referenced snapshot contract instead of against
Wendao-internal structs. This slice does not add a new CLI verb yet.
The touched CLI and
integration-test coverage now anchors repo/workspace resolution through the
shared `PRJ_ROOT`-aware resolver in `xiuxian-config-core` rather than through
crate-local ancestor guessing.
The crate source root now also mounts the shared crate-test-policy source
harness, and the previously inline source test modules in
`src/bin/qianji.rs` and `src/contract_feedback/rest_docs.rs` are now
externalized under `tests/unit/`. Follow-up bounded slices also externalized
the remaining inline source test modules under `src/executors/` and
`src/sovereign/` into `tests/unit/executors/` plus `tests/unit/sovereign/`.
The shared crate-test-policy harness for
`xiuxian-qianji` now passes end-to-end again, without changing the `show` /
`check` behavior of the Flowhub lane.
The same shared gate now also curates post-harness test leaves: large consumer
suites should move into folder-first roots such as
`tests/integration/test_compiler_dispatch_routes/{mod.rs,core_dispatch.rs,...}`,
`tests/integration/runtime_config/{mod.rs,llm_config.rs,...}`, or
`tests/unit/bin/qianji/{mod.rs,dir_runtime.rs,...}` instead of regressing into
one monolithic `tests/unit/*.rs` or `tests/integration/*.rs` file.

---

## 4. Performance Baselines

| Metric           | Result           | Philosophy                       |
| :--------------- | :--------------- | :------------------------------- |
| **Compilation**  | **< 1ms**        | Swift as a Thought.              |
| **Node Jump**    | **< 100ns**      | Precision at the Speed of Light. |
| **Safety Audit** | **Pre-verified** | No Demon (Loop) shall pass.      |

---

## 5. Quick Start

```sh
direnv exec "$PRJ_ROOT" cargo run -p xiuxian-qianji -- \
  /path/to/repo \
  /path/to/qianji.toml \
  '{"seed":"artifact_refining_pipeline"}'

direnv exec "$PRJ_ROOT" cargo run -p xiuxian-qianji -- \
  graph \
  /path/to/qianji.toml \
  /path/to/workflow.bpmn

direnv exec "$PRJ_ROOT" cargo run -p xiuxian-qianji --bin qianji -- \
  show \
  --dir "$PRJ_ROOT/qianji-flowhub"

direnv exec "$PRJ_ROOT" cargo run -p xiuxian-qianji --bin qianji -- \
  show \
  --graph "$PRJ_ROOT/qianji-flowhub/plan/codex-plan.mmd"

direnv exec "$PRJ_ROOT" cargo run -p xiuxian-qianji --bin qianji -- \
  show \
  --contract wendao.docs.navigation

direnv exec "$PRJ_ROOT" cargo run -p xiuxian-qianji --bin qianji -- \
  check \
  --dir "$PRJ_ROOT/qianji-flowhub"
```

---

## License

Apache-2.0 - Developed with artisan precision by **CyberXiuXian Artisan workshop**.
