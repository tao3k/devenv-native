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
`WorkflowTrace` rows into `xiuxian-qianji-control` events for replay,
evidence, cost, lease, and recovery management. This is a one-way boundary:
Qianji still owns workflow/BPMN/Flowhub semantics, while
[`xiuxian-qianji-control`](../xiuxian-qianji-control/README.md) owns generic
run and step control state. Explicit recording returns the normal workflow
report, the appended control records, and the ledger-replayed run view so
callers can inspect recoverable management state without making recording the
default finish path. Topology-bound callers can use checked control recording
to validate required stages and dependency order before any control events are
appended. Callers that must retain typed workflow output after a control ledger
failure can use the recoverable recording path, which returns the completed
workflow report with the control error for retry or manual recovery. The
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
Internally, the adapter is split into recording, projection, and decision
modules so the public control surface can grow without flattening unrelated
responsibilities.
Workflow callers can also opt into required-evidence projection when recording
a trace. The default projection keeps `StepCreated.required_evidence` empty,
while the opt-in contract lets a caller attach deterministic stage evidence
keys before later evidence attachment or gate evaluation.

### 2.2 The Divine Logic (Scheduling)

- **Probabilistic MDP Routing:** Decisions are not binary. Edges carry weights influenced by **Omega's Confidence**, allowing the system to explore multiple paths based on probability.
- **Adversarial Loops:** Natively supports the **Synapse-Audit** pattern, where nodes actively challenge and verify each other’s evidence links.

### 2.3 The Mirror Face (Qianhuan Integration)

Qianji is a **High-Performance Annotator**. In the milliseconds before a node executes, it calls upon `xiuxian-qianhuan` to transmute raw data into persona-aligned context, ensuring the Agent always wears the correct "Face" for the task.

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
HTTP/API adapters can call the control-plane actions directly instead of
routing through CLI-specific names.
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
non-stream clients.
`qianji-server --bind 127.0.0.1:38130 --valkey-url redis://127.0.0.1:6379/0`
starts the minimal daemon shell over that router. HTTP defaults are
Valkey-only: omitted checkpoint backend fields resolve to the service-owned
Valkey URL when supplied, or to `[checkpoint].valkey_url` from `qianji.toml`
otherwise. The service bind address resolves from `[server].bind_addr` unless
`--bind` is supplied. `[server].require_valkey_ready` or
`--require-valkey-ready` can make startup fail before socket bind when Valkey
does not answer `PING`; `--no-require-valkey-ready` explicitly disables that
gate. `GET /healthz` reports service liveness, and `GET /readyz` verifies
that the effective Valkey checkpoint backend responds to `PING`. Local
no-server CLI/control workflow state uses the configured DuckDB path by
default; HTTP remains Valkey-only.
The generic Qianji control ledger has its own read-only operator surfaces.
`qianji control history --ledger <path> --run-id <id> [--json]` renders the
append-only event timeline for one run. `qianji control heartbeat --ledger
<path> --run-id <id> --worker-id <id> --observed-at-ms <ms> --expires-at-ms
<ms> [--metadata <json>] [--json]` records a durable Worker liveness audit
fact without mutating hot queues or leases. `qianji control view --ledger
<path> --run-id <id> [--json]` renders the deterministic replayed run state. `qianji
control query --ledger <path> --run-id <id> --state --now-ms <ms> [--json]`
returns a compact read-only state package with event count, replayed run view,
and recovery snapshot. `qianji
control step --ledger <path> --run-id <id> --step-id <id> [--json]` renders
one replayed step view for step-local evidence, gate, Agent, activity, timer,
and signal inspection. `qianji control activity --ledger <path> --run-id <id>
--activity-id <id> [--step-id <id>] [--json]` renders one replayed activity
view for lifecycle, task, attempt, worker, result, and failure inspection.
`qianji control activity-queue --ledger <path> --run-id <id>
[--task-queue <queue>] [--json]` renders scheduled-but-not-started activity
tasks from durable replay, optionally filtered by task queue, without claiming
work or mutating hot scheduler state.
`qianji control activity-start --ledger <path> --run-id <id> --activity-id
<id> --worker-id <id> --started-at-ms <ms> --attempt <n> [--step-id <id>]
[--json]` records an idempotent durable `ActivityStarted` fact through the
control crate's replay guards. It does not complete, fail, execute, or lease
the activity.
`qianji control decision --ledger <path> --run-id <id> --decision-id <id>
[--step-id <id>] [--json]` renders one replayed agent decision for proposal,
outcome, reason, scheduled activity, checkpoint, and gate inspection.
`qianji control timer --ledger <path> --run-id <id> --timer-id <id>
[--step-id <id>] [--json]` renders one replayed durable timer for scheduled
fire time, fired time, and status inspection.
`qianji control signal --ledger <path> --run-id <id> --signal-name <name>
--payload <json> --received-at-ms <ms> [--step-id <id>] [--json]` appends one
durable external signal event. The payload JSON is stored in signal metadata so
the existing control event schema remains unchanged.
`qianji control recovery-snapshot --ledger <path> --run-id <id> --now-ms <ms>
[--json]` reads the same `xiuxian-qianji-control` DuckDB event ledger and
returns the replay-derived recovery view, ordered recovery plan, and compact
summary without executing recovery actions or touching hot scheduler state.
`qianji control apply-recovery-plan --ledger <path> --valkey-url <url>
--run-id <id> --now-ms <ms> --attempt <n> --reason <text> --max-attempts <n>
[--namespace <ns>] [--backoff-ms <ms>] [--require-human-approval]
[--priority <n>] [--json]` records a recovery-start fact and applies the
current bounded recovery plan through `xiuxian-qianji-control` against the
Valkey hot-state mirror. The command only executes recovery action kinds that
the control crate already supports; unsupported actions are reported as skipped
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
