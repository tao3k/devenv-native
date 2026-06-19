---
type: knowledge
title: "Omega + Graph + Loop/ReAct: Rust Unification Blueprint"
category: "plans"
tags:
  - plan
  - omega
saliency_base: 7.2
decay_rate: 0.03
metadata:
  title: "Omega + Graph + Loop/ReAct: Rust Unification Blueprint"
---

# Omega + Graph + Loop/ReAct: Rust Unification Blueprint

> Legacy transition blueprint. Historical references to the older external-tool stack in this document describe an earlier migration stage and are no longer the target end-state.

> Goal: converge execution into a single Rust runtime (`xiuxian-daochang`) by fusing Omega reasoning, Graph planning, ReAct tool execution, and authoritative Rust context injection, then progressively remove Python runtime paths.
>
> Detailed companion: [Rust Context Injection + Memory Self-Evolution + Reflection](../memory/injection-evolution.md)
>
> LinkGraph execution companion (primary core track): [LinkGraph PPR Algorithm Spec](../wendao/ppr-algorithm.md)
>
> Execution sequence companion: superseded by the Rust context injection and memory-engine package notes.

## 1. Scope and Boundaries

- In scope:
  - Unify Omega, Graph, and ReAct under one Rust execution kernel.
  - Move session windowing, compression, and memory self-evolution to Rust-first execution path.
- Out of scope:
  - Rewriting every tool/skill from Python to Rust immediately.
  - Reintroducing a Python- or external-tool-centered runtime loop outside Rust ownership.

## 2. Target Architecture

```mermaid
flowchart LR
  U[User / Channel] --> G[xiuxian-daochang gateway/repl]
  G --> R[Unified Rust Runtime Kernel]

  R --> O[Omega Deliberation Engine]
  O --> I[Rust Context Assembler]
  R --> P[Graph Planning Engine]
  R --> X[ReAct Execution Engine]

  X --> T[Tool Integration Layer]
  T --> PY[Legacy Python Tool Adapters (compat only)]
  T --> RS[Rust-native Tool Services]

  R --> W[xiuxian-window]
  R --> MM[xiuxian-memory-engine]
  R --> KG[xiuxian-wendao / link-graph]
  R --> SPI[Session Prompt Injection XML]

  W --> I
  MM --> I
  KG --> I
  SPI --> I
  I --> P
  I --> X
  W --> MM
  MM --> R
  KG --> R
```

## 3. Unified Runtime Workflow

1. Intake:
   - Receive request and resolve `session_id` (channel/chat/thread aware).
   - Load bounded context from `xiuxian-window`.
2. Omega deliberation:
   - Evaluate complexity and choose execution route (`react` direct vs `graph` first).
   - Produce context policy (what to inject, max size, ordering, role-mix profile, injection mode).
3. Rust context assembly (knowledge inject role):
   - Assemble typed context blocks from:
     - session prompt injection XML (operator/session scoped),
     - memory recall context (`xiuxian-memory-engine`, MemRL-style),
     - bounded summaries/window state (`xiuxian-window`),
     - knowledge context (`xiuxian-wendao`, link-graph).
   - Compose scenario-specific mixed-role prompts (for example debug/recovery/architecture reflection packs).
   - Apply deterministic ordering and token budget before execution.
4. Execution routing:
   - Fast/simple request goes to ReAct execution with assembled context.
   - Complex request triggers Graph plan synthesis first, then ReAct/tool execution.
5. Omega quality gating:
   - Evaluate plan quality, risk, and tool ordering.
   - Repair plan before execution when quality checks fail.
6. ReAct execution:
   - Execute tool loop with budget, retries, and structured error taxonomy.
   - Call tools through one Rust-owned tool integration layer only.
7. Self-evolution update:
   - Store episode outcome and feedback in `xiuxian-memory-engine`.
   - Persist session window snapshots and summary segments.
8. Response:
   - Emit user-facing answer plus structured observability events.

## 3.1 Rust Context Injection: Architectural Role

- Ownership:
  - Owned by Rust runtime, policy decided by Omega.
  - Not owned by Python runtime loop.
- Responsibility:
  - Deliver high-signal context to Graph/ReAct without changing model weights.
  - Provide flexible injection modes (`single`, `classified`, `hybrid`) and mixed-role composition.
  - Keep context bounded, session-scoped, and auditable.
- Non-goals:
  - No free-form hidden prompt mutation in random call sites.
  - No bypass of context policy on deterministic execution paths.
- Contract direction:
  - Introduce typed `PromptContextBlock` and `InjectionPolicy` contracts.
  - Keep tool payload contracts stable; pass injected context through explicit fields only when schema supports it.

## 4. Feature-Name Roadmap (Backlog-Aligned)

Project progress must be tracked by feature name (not phase/stage labels). Recommended feature names:

| Feature name                             | Definition of done                                                                        |
| ---------------------------------------- | ----------------------------------------------------------------------------------------- |
| **Unified Rust Execution Kernel**        | One Rust entry for channel/repl/gateway execution; no Python runtime loop on hot path.    |
| **Graph Planning Engine (Rust)**         | Graph planning API runs inside Rust runtime and produces stable, testable plan contracts. |
| **Omega Deliberation Engine (Rust)**     | Quality gates and plan-repair logic run in Rust with structured outputs.                  |
| **ReAct Tool Runtime (Rust)**            | Tool-call loop, retry, budget, and failure policy consolidated in Rust.                   |
| **Session Window Compression (Rust)**    | Predictable context compression and restore strategy backed by `xiuxian-window`.          |
| **Memory Self-Evolution Runtime (Rust)** | Outcome feedback and recall adaptation persisted via DB-backed `xiuxian-memory-engine`.   |
| **Python Runtime Decommissioning**       | Python side is transport/adapter-only; no duplicated runtime loop entrypoints.            |

## 5. Migration Rules

- Single authority:
  - Runtime orchestration authority is Rust.
  - Python authority is thin adapter/transport implementation only when still needed.
- Thin orchestrator rule:
  - `xiuxian-daochang` remains orchestration-only.
  - Memory lifecycle/revalidation/promotion core logic must live in Rust memory package(s), not inside agent runtime modules.
- External interoperability rule:
  - Keep any legacy external tool facade thin and compatibility-only.
  - Memory policy must remain in Rust core without duplicated facade logic.
- Prompt/context authority:
  - Prompt/knowledge injection authority is the Rust context assembler.
  - Python side must not inject hidden runtime prompt context.
- No dual-loop fallback:
  - Do not keep long-term “Rust loop + Python loop” behavior parity mode.
  - Keep one execution contract and migrate callers to it.
- Contract-first evolution:
  - Keep external tool-facing contracts stable while internals move.
  - Version schemas when changing output shape.
- Isolation by default:
  - Session partition key is mandatory (`channel:chat_id:thread_id` when applicable).
  - Window snapshots and memory feedback must be session-scoped.

## 5.1 Boundary Corrections (Roadmap Clarification)

- `memory`:
  - short-term operational runtime memory (Rust core owned)
  - exposed through thin compatibility facades only when required
- `knowledge`:
  - long-term durable knowledge interface
- `xiuxian-daochang`:
  - orchestration only; no embedding of memory lifecycle policy logic

## 5.2 Data Plane Standard (Valkey + LanceDB + Arrow)

- `Valkey`:
  - hot runtime state, dedup/idempotency, stream events, and high-concurrency caches.
- `LanceDB`:
  - durable retrieval state, tool/knowledge indexes, episodic memory persistence, replay analytics.
- `Arrow`:
  - canonical inter-stage schema for ranking/gate traces with zero-copy batch movement.

Rule:

- no hot-path JSON file state source;
- read-through/write-through flows must follow `Valkey -> LanceDB` boundaries with Arrow contracts.

## 5.3 Discover Confidence Contract

`skill.discover` and route selection must preserve calibrated ranking metadata end-to-end:

- `score`
- `final_score`
- `confidence` (`high` | `medium` | `low`)
- `ranking_reason`
- `usage_template`

Policy:

- `high`: direct recommendation allowed
- `medium`: top-k + clarification
- `low`: refine intent before execution

## 6. Quality Gates

- Correctness:
  - Cross-session isolation matrix (multi-group, multi-thread, mixed `/reset` and `/resume` concurrency).
  - Deterministic parser and command routing tests in dedicated `tests/` modules.
  - Prompt injection determinism tests (same inputs => same ordered context blocks).
- Reliability:
  - External tool startup and reconnect resilience under slow-start and transient failures where such integrations still exist.
  - No silent exits; structured startup/shutdown diagnostics.
- Performance:
  - Baseline and regression benchmark for p50/p95 latency, failure rate, and memory peak.
  - Concurrent-session load tests for gateway mode.
- Observability:
  - Structured events for session lifecycle, snapshot operations, memory recall/update, tool call duration, and tool failures.

## 7. Python Runtime Removal End-State

- End-state contract:
  - `xiuxian-daochang` is the only runtime orchestrator.
  - Python process provides transport/adapters only when still required.
- Cleanup targets:
  - Remove Python runtime loop command paths after Rust parity is proven.
  - Keep compatibility wrappers only where they map directly to Rust commands.
- Acceptance rule:
  - Removal is complete only after black-box suites pass on multi-session, multi-channel, and memory self-evolution scenarios.

## 8. Contract Seeds (Next)

- `OmegaDecision`:
  - route (`react` | `graph`)
  - risk level
  - injection policy (enabled blocks, max chars/tokens, ordering strategy)
- `PromptContextBlock`:
  - source (`memory_recall` | `session_xml` | `window_summary` | `knowledge`)
  - priority, size, session scope
  - payload (rendered text/XML)
- `TurnTrace`:
  - selected route, tool chain, retries, latency, failure taxonomy
  - injection stats (`blocks_used`, `chars_injected`, dropped-by-budget)
- `ReflectionRecord`:
  - outcome, failure category, corrective action
  - memory credit update and next-turn strategy hint
- `DiscoverMatch`:
  - tool id, usage template, score/final_score/confidence, ranking_reason, schema digest

## 9. Post-A7 P0 Execution Queue

After A0-A7 closure, the next implementation queue is feature-driven (not gate-driven):

| Priority | Feature                                     | Scope                                                                                              | Exit criteria                                                                              | Primary verification                                                                 |
| -------- | ------------------------------------------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------ |
| P0-1     | Graph Planning Engine (Rust)                | Move planning contract and graph execution entry to Rust runtime with deterministic plan schema.   | Rust graph plan contract is generated and consumed without Python runtime loop dependency. | `cargo test -p xiuxian-daochang --test contracts` + graph planning integration tests |
| P0-2     | Omega Deliberation Engine (Rust)            | Expand policy routing into explicit plan-repair/quality-gate path in Rust.                         | Route policy can enforce repair or fallback with auditable reason fields.                  | `cargo test -p xiuxian-daochang --test agent_injection` + reflection threshold tests |
| P0-3     | Role-Mix Injection Profiles                 | Add `single/classified/hybrid` profile selection with deterministic assembly.                      | Role-mix profile is selected by policy and recorded in injection snapshot traces.          | `cargo test -p xiuxian-daochang --lib injection::tests` + trace reconstruction gate  |
| P0-4     | Python Runtime Decommissioning (Loop paths) | Remove duplicated Python runtime loop entrypoints while preserving only the minimum adapter plane. | Runtime orchestration entry remains Rust-only (`xiuxian-daochang`).                        | `python3 scripts/channel/test_xiuxian_daochang_memory_ci_gate.py --profile nightly`  |
| P0-5     | Adversarial Sub-graph Routing               | Deprecate regex-based triggers for Qianji workflows; elevate to Omega routing policy.              | Omega natively outputs `route: graph` + `workflow_mode: agenda_validation` via LLM JSON.   | `cargo test -p xiuxian-daochang --test agent_omega_routing`                          |

### P0-1 Status Update (2026-02-23)

Completed in current branch:

- Added `GraphExecutionPlan::validate_shortcut_contract()` as the shared deterministic-schema validator for graph shortcut plans.
- Wired graph plan consumption (`agent/graph/executor.rs`) to enforce the shared validator before execution.
- Added planner-side debug assertion so generated shortcut plans are checked against the same validator.
- Added planner-focused tests and contract-focused tests:
  - `tests/agent/graph_planner.rs`
  - `tests/contracts/test_runtime_contracts.rs`
  - `tests/agent/graph_executor.rs` (invalid fallback action / step ordering validation)

Verification commands:

- `cargo test -p xiuxian-daochang --lib graph_ -- --nocapture`
- `cargo test -p xiuxian-daochang --test contracts graph_execution_plan_contract -- --nocapture`

### P0-2 Status Update (2026-02-23)

Completed in current branch:

- Added explicit quality-gate repair path in `agent/omega/decision.rs`:
  - `apply_quality_gate(decision)` enforces high-risk graph safeguards.
  - When `route=graph` and risk is `high|critical`, fallback policy is repaired from
    `switch_to_graph` to `retry_react`.
  - High-risk `tool_trust_class=evidence` is upgraded to `verification`.
- Added auditable reason markers for deterministic traceability:
  - `quality_gate=graph_retry_loop_guard;repair=fallback_policy:retry_react`
  - `quality_gate=graph_high_risk_trust_upgrade;repair=tool_trust_class:verification`
- Wired quality-gate enforcement into runtime execution paths:
  - `agent/turn_execution/shortcut.rs`
  - `agent/turn_execution/react_loop.rs`
- Added unit tests for quality-gate behavior:
  - `tests/agent/omega_decision.rs`

Verification commands:

- `cargo test -p xiuxian-daochang --lib apply_quality_gate_ -- --nocapture`
- `cargo test -p xiuxian-daochang --test agent_injection omega_shortcut_ -- --nocapture`

### P0-3 Status Update (2026-02-23)

Completed in current branch:

- Added deterministic injection policy resolver in Rust runtime:
  - New module: `agent/injection/policy.rs`
  - Adaptive rule from `classified` baseline:
    - single block => `single`
    - multi-domain blocks => `hybrid`
    - otherwise => `classified`
  - `single` mode is compact by construction (`max_blocks <= 1`, priority-first ordering).
- Wired effective policy resolution into both injection paths:
  - `normalize_messages_with_snapshot(...)`
  - `build_snapshot_from_messages(...)`
- Upgraded role-mix profile selection to align with policy mode and always produce auditable profile metadata:
  - `role_mix.single.v1`
  - `role_mix.classified.v1`
  - `role_mix.hybrid.v1`
- Extended injection trace observability:
  - `session.injection.snapshot_created` now logs `injection_mode`
  - shortcut `_omni.session_context` now includes `injection_mode`

Verification commands:

- `cargo test -p xiuxian-daochang --lib injection::tests -- --nocapture`
- `cargo test -p xiuxian-daochang --test agent_injection graph_shortcut_includes_typed_injection_snapshot_metadata -- --nocapture`

### P0-4 Status Update (2026-02-22)

Completed in current branch:

- Python gateway/CLI entrypoints now dispatch to Rust runtime only; Python loop helpers removed.
- Added Rust-orchestrator startup guard in Python CLI (`agent.runtime_orchestrator` must stay `rust`).
- The historical Python workflow and orchestration packages are removed.
- The historical Python runtime loop modules are removed.
- Legacy external tool behavior is preserved only where needed; no compatibility fallback to Python runtime loops was added.

Verification evidence:

- Rust-owned runtime validation now replaces the deleted Python agent package.
- The remaining Python boundary is verified through retained package tests and
  package-removal assertions under `packages/python/core/tests/units/`.

### P0-5 Status Update (Planned: Adversarial Sub-graph Routing)

**Goal:** Eradicate the regex-based `should_run_agenda_validation` placeholder.

**Action Plan:**

1. Extend `OmegaDecision` in `packages/rust/crates/xiuxian-daochang/src/contracts/omega.rs` to support `workflow_mode = "agenda_validation"`.
2. Update the Omega system prompt (or tool schema) so the LLM explicitly selects this mode when asked to schedule tasks.
3. Remove `apply_agenda_validation_if_needed` from `agent/turn_execution/react_loop/mod.rs`.
4. Intercept the request in `agent/turn_execution/shortcut.rs`. When `route == graph` and `workflow_mode == agenda_validation`, execute the Qianji `agenda_validation_pipeline.toml`.
5. Ensure the final result of the Qianji execution is returned directly as the user-facing response, avoiding the secondary ReAct loop entirely.

Execution rule:

- Land each P0 item with tests in the same change.
- Keep `xiuxian-daochang` orchestration-only; do not move memory lifecycle policy into channel/runtime handlers.
