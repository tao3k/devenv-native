---
type: knowledge
title: "Xiuxian-Zhixing-Heyi Feature Plan (2026)"
category: "plans"
tags:
  - xiuxian-zhixing
  - heyi
  - manifestation
  - reminders
  - strict-teacher
saliency_base: 7.8
decay_rate: 0.03
metadata:
  title: "Xiuxian-Zhixing-Heyi Feature Plan (2026)"
---

# Xiuxian-Zhixing-Heyi Feature Plan (2026)

> Status: Active (core runtime online; enforcement and indexing hardening in progress)
> Scope: `packages/rust/crates/xiuxian-zhixing` and host wiring in `packages/rust/crates/xiuxian-daochang`

## 1. Name and Intent

The naming origin is Xiuxian (修仙) and Zhixing-Heyi (知行合一), which in this repository means:

- turn stored knowledge into enforceable action,
- keep action state observable and reviewable,
- close the loop from reflection to future execution quality.

### 1.1 Namespace Alignment (Current)

- `xiuxian-zhixing`: domain runtime for agenda/journal/blockers/reminders and Wendao bridge.
- Zhixing manifestation trait: template assembly boundary used by host bootstrap.
- `xiuxian-qianji`: workflow runtime namespace used by agent workflow execution paths.
- `xiuxian-daochang`: host runtime namespace for native tool registration and notification transport.
- Engineering role mapping:
  - `Action Compiler`: isolated execution backend for deterministic action compilation.
  - `Cognitive Interface`: user-facing dialogue/persona layer for multi-turn interaction.

## 2. Related Documents

- Theory baseline: `docs/99_llm/xiuxian_zhixing_theory.md`
- Host orchestration and injection context: `docs/01_core/omega/trinity-control.md`
- Manifestation spec: `docs/01_core/memory/injection-evolution.md`
- Qianji workflow runtime spec: `docs/01_core/qianji/SPEC.md`
- Wendao execution track: `docs/01_core/wendao/roadmap.md`
- Test and runbook references: `docs/testing/rust-agent-loop-memory-testing.md`

## 3. Runtime Architecture (Implemented Shape)

### 3.1 Plugin Boundary

`xiuxian-zhixing` owns domain logic for agenda, journal, blockers, reminders, and thin-bridge indexing:

- `packages/rust/crates/xiuxian-zhixing/src/lib.rs`
- `packages/rust/crates/xiuxian-zhixing/src/heyi/mod.rs`
- `packages/rust/crates/xiuxian-zhixing/src/heyi/tasks.rs`
- `packages/rust/crates/xiuxian-zhixing/src/heyi/reminders.rs`
- `packages/rust/crates/xiuxian-zhixing/src/wendao/indexer/mod.rs`

### 3.2 Native Tool Surface

The host agent exposes deterministic native tools:

- `journal.record`
- `task.add`
- `agenda.view`

Implementation:

- `packages/rust/crates/xiuxian-daochang/src/agent/native_tools/zhixing.rs`
- `packages/rust/crates/xiuxian-daochang/src/agent/bootstrap/zhixing.rs`

### 3.3 Reminder and Notification Flow

- Reminder polling loop: configurable interval (default 60 seconds), with one-shot reminder marking.
- Dispatch boundary: plugin emits reminders through `mpsc::Sender`; host handles transport providers.

Implementation:

- `packages/rust/crates/xiuxian-zhixing/src/heyi/reminders.rs`
- `packages/rust/crates/xiuxian-daochang/src/agent/bootstrap/zhixing.rs`
- `packages/rust/crates/xiuxian-daochang/src/agent/notification/mod.rs`
- `packages/rust/crates/xiuxian-daochang/src/agent/notification/dispatcher.rs`

### 3.4 Zhixing Template and Runtime Configuration

Host bootstrap resolves Zhixing runtime settings from `xiuxian.toml` (system + user overlay):

- `[wendao.zhixing].notebook_path`: notebook storage root.
- `[wendao.zhixing].time_zone`: IANA timezone used by scheduling and rendering.
- `[wendao.zhixing].template_paths`: template directories for manifestation loading.
- `[wendao.zhixing.reminder_queue]`: optional due-queue backend settings.

Example:

```toml
[wendao.zhixing]
notebook_path = ".data/xiuxian/notebook"
time_zone = "America/Los_Angeles"
template_paths = ["assets/templates", ".omni/templates"]

[wendao.zhixing.reminder_queue]
key_prefix = "xiuxian_zhixing:heyi:reminder"
poll_interval_seconds = 5
poll_batch_size = 128
```

Runtime behavior:

- Relative `template_paths` entries are resolved against project root (`PRJ_ROOT` or detected root).
- Absolute `template_paths` entries are used as-is.
- Empty/invalid template path entries are ignored; if all are empty, host falls back to default template paths.

## 4. Capability Status (Audit Snapshot)

| Capability                                                    | Status | Evidence                                                                                                                                                                                                                            |
| ------------------------------------------------------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Heyi orchestrator construction and timezone validation        | Done   | `packages/rust/crates/xiuxian-zhixing/src/heyi/types.rs`; `packages/rust/crates/xiuxian-zhixing/tests/test_heyi.rs`                                                                                                                 |
| Native tool registration and deterministic invocation         | Done   | `packages/rust/crates/xiuxian-daochang/src/agent/bootstrap/zhixing.rs`; `packages/rust/crates/xiuxian-daochang/src/agent/native_tools/zhixing.rs`                                                                                   |
| Reminder polling and one-shot mark (`timer:reminded`)         | Done   | `packages/rust/crates/xiuxian-zhixing/src/heyi/reminders.rs`; `packages/rust/crates/xiuxian-zhixing/tests/test_heyi.rs`                                                                                                             |
| Strict teacher blocker computation (`journal:carryover >= 3`) | Done   | `packages/rust/crates/xiuxian-zhixing/src/heyi/blockers.rs`; `packages/rust/crates/xiuxian-zhixing/tests/test_strict_teacher.rs`                                                                                                    |
| Strict teacher enforcement in runtime tool path               | Done   | `packages/rust/crates/xiuxian-zhixing/src/heyi/tasks.rs`; `packages/rust/crates/xiuxian-zhixing/src/heyi/agenda_render.rs`                                                                                                          |
| Wendao thin bridge boundary type                              | Done   | `packages/rust/crates/xiuxian-zhixing/src/wendao/indexer/mod.rs`; `packages/rust/crates/xiuxian-zhixing/tests/test_wendao_indexer.rs`                                                                                               |
| Markdown persistence append safety                            | Done   | `packages/rust/crates/xiuxian-zhixing/src/storage/markdown.rs`; `packages/rust/crates/xiuxian-zhixing/tests/test_storage_markdown.rs`                                                                                               |
| Config and path compliance with PRJ directory API             | Done   | `packages/rust/crates/xiuxian-daochang/src/agent/bootstrap/zhixing.rs` and `packages/rust/crates/xiuxian-daochang/src/agent/bootstrap/tests.rs` validate `PRJ_ROOT`, `PRJ_DATA_HOME`, and `XIUXIAN_WENDAO_NOTEBOOK_PATH` resolution |

## 5. Hard Constraints

1. `xiuxian-wendao` remains domain-agnostic; no agenda semantics leak into graph core.
2. Runtime notebook/data path must resolve from project directory APIs or config, not hardcoded literals.
3. Strict teacher mode is considered complete only after runtime enforcement is wired, not only unit-tested.

## 6. Execution Backlog (Next Sprint)

1. ZH-01 Strict Teacher Runtime Enforcement (Done)

- Wire `check_heart_demon_blocker()` into `task.add` and `agenda.view` flow.
- Add integration tests proving block-and-release behavior.
- Gate: `cargo test -p xiuxian-zhixing --test test_strict_teacher`

2. ZH-02 Storage Correctness Hardening (Done)

- Replace overwrite writes with append-safe async writes.
- Add multi-entry persistence tests for journal and agenda.
- Gate: `cargo test -p xiuxian-zhixing --test test_storage_markdown`

3. ZH-03 Wendao Thin Bridge Completion (Done)

- Implement real ingestion from notebook and task state into `Entity` nodes.
- Add assertions for created nodes/metadata, not only no-panic execution.
- Gate: `cargo test -p xiuxian-zhixing --test test_wendao_indexer`

4. ZH-04 PRJ Path Compliance (Done)

- Remove direct `project_root/.data` fallback and move to configured project data path resolution.
- Add config-driven runtime init tests.

5. ZH-05 End-to-End Host Verification (Done)

- Add host-level tests that validate native tools + reminder dispatch + blocker behavior together.
- Evidence: `packages/rust/crates/xiuxian-daochang/tests/agent/native_tools_zhixing.rs` and `packages/rust/crates/xiuxian-daochang/tests/agent_suite.rs`.
- Gate: `cargo test -p xiuxian-daochang --test agent_suite`.

6. ZH-06 Graph Power Evaluation (Done)

- Verify the integration between Wendao's PPR algorithm and Zhixing's domain objects (`journal`, `agenda`).
- Prove that searching for a concept in a journal naturally surfaces related, uncompleted tasks from the agenda due to graph linkage.
- Validate that the `xiuxian-daochang`'s LLM context window successfully incorporates these cross-domain graph references to enforce `journal:carryover` logic naturally.
- Evidence:
  - `packages/rust/crates/xiuxian-wendao/tests/test_link_graph_seed_and_priors/link_graph_related_journal_semantic_pull_surfaces_agenda_tasks.rs`
  - `packages/rust/crates/xiuxian-daochang/tests/agent/native_tools_zhixing_e2e.rs`
- Gate:
  - `cargo test -p xiuxian-wendao --test test_link_graph_seed_and_priors link_graph_related_journal_semantic_pull_surfaces_agenda_tasks -q`
  - `cargo test -p xiuxian-daochang --test agent_suite native_tools_zhixing_e2e::zhixing_e2e_tool_loop_reads_metadata_and_proactively_rejects_malicious_request -q`

7. ZH-07 Zero-Hardcoding via Skill Injection (Planned)

- Deprecate rigid, hardcoded tools like `agenda.view` that attempt to parse dates in Rust.
- Inject the **Wendao Query Grammar** (e.g., `date:this_week`, `status:open`) directly into the LLM's context.
- Empower the LLM to autonomously construct and execute precise graph queries via a unified `wendao.search` endpoint, maximizing its semantic intelligence.

8. ZH-08 Org-Mode Task State Integration (Planned)

- Adopt the Emacs Org-Mode workflow methodology for task states (`TODO`, `NEXT`, `STARTED`, `WAITING`, `DONE`, `CANCELLED`).
- Eliminate hardcoded Rust status enums. Instead, define the state sequence pipeline dynamically inside the `[wendao.zhixing]` TOML configuration.
- Upgrade the `ZhixingWendaoIndexer` to extract these Org-Mode keywords directly from Markdown AST and map them to searchable Wendao `Entity` metadata.

9. ZH-09 Org-Mode Property Drawers (Rich Metadata Extraction) (Planned)

- Implement support for parsing Emacs Org-Mode style `:PROPERTIES:` drawers attached to Markdown headings.
- Enable the `ZhixingWendaoIndexer` to extract arbitrary, schemaless key-value metadata (e.g., `:EFFORT: 5h`, `:DEADLINE: 2026-03-01`) via `comrak` AST traversal.
- Map these property drawers directly into the Wendao `Entity` metadata JSON to support advanced graph querying without requiring database schema migrations.
