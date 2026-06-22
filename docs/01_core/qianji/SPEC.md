---
type: knowledge
metadata:
  title: "Specification: xiuxian-qianji (千机)"
---

# Specification: xiuxian-qianji (千机)

> **Authority:** CyberXiuXian Artisan workshop
> **Mission:** Building a High-Performance, Probabilistic, and Formally Verified Workflow Engine in Rust.
> **Status:** Full-Spectrum Logic Enclosure / TOML-Driven.

---

## 1. Research Foundations (Belief System)

The **Qianji** engine is derived from the synthesis of four foundational research papers (2024-2026).

| Reference                   | Key Theory                              | Local Evidence Path                                |
| :-------------------------- | :-------------------------------------- | :------------------------------------------------- |
| **Synaptic-Flow (2026)**    | Asynchronous Dependency-Aware Inference | `.data/research/papers/qianji_foundation_2026.txt` |
| **Agent-Prob-Route (2025)** | Probabilistic Graph Routing (MDP)       | `.data/research/papers/qianji_foundation_2026.txt` |
| **LTL-Agents (2024)**       | Formal Logic Verification of Loops      | `.data/research/papers/qianji_foundation_2026.txt` |
| **Synapse-Audit (2025)**    | Iterative Calibration Loops             | `.data/research/papers/synapse_audit_2025.txt`     |
| **Reflexion (2023/2024)**   | Verbal Reinforcement Learning           | `docs/research/reflexion_self_refine.md` (Stub)    |
| **Self-Refine (2024)**      | Iterative Refinement with Self-Feedback | `docs/research/reflexion_self_refine.md` (Stub)    |

---

## 2. Architectural Design: The "Iron Frame & Divine Logic"

### 2.1 The "Iron Frame" (Kernel)

- **Engine:** Based on Rust's `petgraph` library using `StableGraph`.
- **Topology:** Supports DAGs, Cycles (with LTL guards), and Sub-graphs (nested Qianji boxes).
- **Performance:** Aiming for < 100ns topological traversals and zero-overhead node scheduling via `tokio` parallel tasks.

### 2.2 The "Divine Logic" (Scheduling & Orchestration)

- **TOML-Driven Orchestration:** The engine is entirely governed by a declarative `QianjiManifest` (TOML).
  - **Logic Enclosure:** Graph construction, node dependency resolution, and probabilistic weights are defined in TOML and compiled by the Rust `QianjiCompiler`.
- **Probabilistic Routing:** Every edge has a weight $W = f(\text{Omega_Confidence})$. The path is not binary but probability-weighted (MDP-based).
- **Adversarial 回路:** Implements the **Synapse-Audit** skeptic-prospector-calibrator loop as a native graph pattern.
- **State Machine:** Implements a strict state machine for each node: `Idle -> Queued -> Annotating -> Executing -> Calibrating -> Finalized`.

---

## 3. The "Rust-Hard, Host-Thin" Philosophy

In the Qianji architecture, host integrations are reduced to a thin glue layer.
The `xiuxian-qianji` crate no longer exposes an in-crate PyO3 module surface.

### 3.1 Responsibilities

- **Rust (The Brain):**
  - Parses `qianji.toml` via `toml` (serde).
  - Compiles the `petgraph` execution DAG.
- Manages parallel `tokio` execution of Knowledge (Wendao) and Annotation nodes.
  - Performs LTL Safety Audits to prevent deadlocks and infinite loops.
- **Host Integrations (The Glue):**
  - Launch the native Qianji runtime through the CLI or higher-level host
    adapters.
  - Handle final UI presentation of the results.

---

## 4. Performance Baselines (Artisan Verified)

- **TOML Compilation:** < 1ms for 50-node graphs.
- **Topological Traversal:** < 100ns per node jump.
- **Concurrent Execution:** Zero-overhead scheduling via `tokio` task spawning.
- **Memory Efficiency:** < 10MB overhead for the engine core.

## 5. The "MemRL Promotion" Workflow (Implemented)

To bridge the gap between `xiuxian-memory-engine` (short-term reflection) and `xiuxian-wendao` (long-term persistent knowledge), Qianji must implement the **"3-in-1 Revalidation Loop"** (defined in the Omega specs) as a native TOML workflow.

### 5.1 The Promotion DAG

When a short-term memory (e.g., a repeated bug workaround) reaches the `revalidate_pending` state, the Omega router triggers a `QianjiManifest` named `memory_promotion_pipeline.toml`.

This workflow will contain the following probabilistic nodes:

1. **[Node: ReAct_Evidence_Gatherer]**: Scans recent execution logs to confirm if the workaround is still being actively used and is successful.
2. **[Node: Graph_Structure_Validator]**: Queries Wendao to check if the underlying code/infrastructure related to the memory has changed.
3. **[Node: Omega_Arbiter]** (Adversarial Loop):
   - Weighs the ReAct and Graph evidence.
   - Outputs a probabilistic route: `Promote (0.8)`, `Retain (0.15)`, or `Obsolete (0.05)`.
4. **[Node: Wendao_Ingester]** (Terminal Node): If `Promote` is selected, this node compiles the reflection into a formal `Entity` and permanently writes it into the `xiuxian-wendao` graph.

This workflow ensures that only battle-tested, structurally valid reflections are persisted into the hyperscale knowledge graph.

Current implementation path:

- `packages/rust/crates/xiuxian-qianji/resources/memory_promotion_pipeline.toml`
- `xiuxian_qianji::MEMORY_PROMOTION_PIPELINE_TOML`
- `QianjiApp::create_memory_promotion_pipeline(...)`
- Native terminal task type: `task_type = "wendao_ingester"` (with best-effort persistence controls)

## 6. Qianji Annotation Interface: Multi-Persona Adversarial Loops

To achieve extreme precision in reasoning (especially during reflection and
memory promotion), Qianji owns a local annotation interface for deterministic
persona-aligned context snapshots.

### 6.1 Node-Level Persona & Template Binding

Instead of a single persona governing the entire workflow, the Qianji TOML
manifest allows each node to independently bind annotation controls through a
dedicated TOML table.

See [[docs/01_core/qianji/architecture/annotation-node-binding-interface.md|Annotation Node Binding Interface]] for the exact TOML schema (`[nodes.annotation]`) and the Rust execution contract that drives node-level persona injection.

This creates an elegant "Role-Play Graph":

- A node can declare `persona: "student_proposer"` and `template: "draft_reflection.md"`.
- The subsequent node can declare `persona: "strict_professor"` and `template: "critique_report.md"`.

### 6.2 The Adversarial Validation Sub-Graph (Synapse-Audit)

This interface powers the **Synapse-Audit** pattern, allowing any scenario requiring rigorous validation to be modeled as an **Adversarial Sub-Graph**. By injecting distinct personas, the system forces LLMs into a multi-agent debate format (e.g., _Proposer vs. Critic_).

**Example Scenario 1: The Agenda Validation Loop**
When the system schedules a task or creates an agenda:

1. **[Node: Proposer]**: Injects the `Agenda Steward` persona. Proposes a structured schedule based on user input and historical context.
2. **[Node: Critic]**: Injects the `Strict Teacher` (Agenda Critic) persona. Takes the Steward's proposed schedule and aggressively critiques it for over-commitment, missing context, or high failure probability based on past procrastinations (`journal:carryover`).
3. **[Edge: Probabilistic Feedback]**: If the Teacher's critique score is low (e.g., `< 0.8`), the graph loops back to the Steward with the strict critique attached, forcing a revision. If it passes, the schedule is finalized.

**Example Scenario 2: The Memory Promotion Loop**
When deciding whether to permanently etch a short-term workaround into Wendao:

1. **[Node: Proposer]**: Injects an `Optimistic Engineer` persona to summarize the workaround.
2. **[Node: Critic]**: Injects the `Strict Architecture Auditor` persona. It attacks the summary: "Is this a hack? Does it break Dependency Inversion?"
3. **[Edge: Probabilistic Feedback]**: Only workarounds that survive the Auditor's critique are allowed to pass to the `Wendao_Ingester` node.

By embedding template and persona annotation directly into the Qianji node execution state machine (`Annotating` phase), we achieve highly structured, self-correcting cognitive loops that can be plugged into _any_ part of the `xiuxian-artisan-workshop` system as an adversarial sub-graph.

### 6.3 Context Window Management

See [[docs/01_core/qianji/architecture/context-window-management.md|Context Window Management]] for a deep dive into Isolated vs. Appended scenarios and the Economic XML Debate Protocol.

### 6.4 Implementation Audit (2026-02-26)

| Capability                                                          | Status               | Evidence                                                                                                                                                                                                         |
| :------------------------------------------------------------------ | :------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Node-level annotation binding in TOML                               | ✅ Implemented       | `packages/rust/crates/xiuxian-qianji/src/contracts/mod.rs`                                                                                                                                                       |
| Isolated vs appended execution mode                                 | ✅ Implemented       | `packages/rust/crates/xiuxian-qianji/src/contracts/mod.rs`, `packages/rust/crates/xiuxian-qianji/src/executors/annotation.rs`                                                                                    |
| Structured handoff via whitelisted keys                             | ✅ Implemented       | `packages/rust/crates/xiuxian-qianji/src/executors/annotation.rs`, `packages/rust/crates/xiuxian-qianji/src/engine/compiler.rs`                                                                                  |
| Concurrent critics + terminal gather                                | ✅ Implemented       | `packages/rust/crates/xiuxian-qianji/src/scheduler/core.rs`, `packages/rust/crates/xiuxian-qianji/src/scheduler/state.rs`, `packages/rust/crates/xiuxian-qianji/tests/test_context_isolation_and_concurrency.rs` |
| Deterministic merge-before-ready scheduling for concurrent branches | ✅ Implemented       | `packages/rust/crates/xiuxian-qianji/src/scheduler/core.rs`                                                                                                                                                      |
| Host-provided read-only context bridge into isolated mode           | ⚠️ Runtime-dependent | Qianji supports `history_key`; caller must provide sanitized history context                                                                                                                                     |

### 6.5 Phase F Audit: LLM Client Multi-Tenancy (2026-02-26)

Qianji now supports node-scoped LLM tenancy controls directly in manifest TOML:

```toml
[[nodes]]
id = "Analyzer"
task_type = "llm"
weight = 1.0
params = { output_key = "analysis" }
[nodes.llm]
provider = "openai-compatible"
model = "deepseek-chat"
base_url = "https://api.deepseek.com/v1" # optional
api_key_env = "TENANT_A_API_KEY"        # optional
```

Compatibility note:

- Preferred table is `[nodes.llm]`.
- Legacy alias `[nodes.llm_config]` is still accepted for backward compatibility.

Model resolution order:

1. `context.llm_model` (explicit runtime override)
2. node-level model from `[nodes.llm].model` (or legacy `params.model`)
3. `context.llm_model_fallback` (global runtime fallback, injected by launcher when needed)

Node transport resolution:

- If node-level `base_url` or `api_key_env` is provided, Qianji builds a node-dedicated client.
- If transport fields are omitted, Qianji reuses the global shared LLM client while still honoring node-level model overrides.
- Current dedicated transport backend is OpenAI-compatible HTTP; `provider = "litellm_rs"` remains a guarded error path in Qianji until native LiteLLM transport is introduced in `xiuxian-llm`.

Built-in workflow convention:

- Built-in manifests should avoid hardcoding provider/model unless tenant isolation explicitly requires it.
- Default model/provider comes from user/runtime configuration (for example MiniMax via `xiuxian.toml` / runtime env).

---

## 7. Implementation Roadmap: "The Silent Takeover"

1.  **Phase A (Done):** Rust Core + `petgraph` Kernel.
2.  **Phase B (Done):** TOML Manifest Compiler.
3.  **Phase C (Done):** Adversarial Loop & Probabilistic Routing.
4.  **Phase D (Done):** Integration Testing on native Rust runtime (Python orchestration shadow path retired).
5.  **Phase E (Done):** Formalized the Qianji annotation node binding interface in TOML (`[nodes.annotation]`).
6.  **Phase F (Done):** Implemented **LLM Client Multi-Tenancy (via `xiuxian-llm`)** with node-scoped LLM bindings in TOML (`[nodes.llm]`, legacy alias `[nodes.llm_config]`). Node-level model/provider selection is supported, and nodes without explicit model settings seamlessly fall back to the global unified runtime model (`llm_model_fallback`) resolved from `xiuxian.toml`.

---

## 10. Distributed Consensus Mechanism (Multi-Agent Voting)

To achieve Byzantine Fault Tolerance (BFT) in complex reasoning tasks, Qianji supports **Distributed Consensus**. This allows multiple independent agent instances to collaborate on a single node's outcome.

### 10.1 The Consensus Gate

When a node is marked for consensus, the scheduler intercepts its completion. Instead of immediate state merging, the node enters a **Pending Consensus** state.

- **Valkey Synchronization**: Each participating agent publishes its proposed output hash and a signature to a shared Valkey "Voting Pool."
- **Quorum Enforcement**: The final state transition only occurs when a pre-defined threshold (e.g., majority or weight-based quorum) is reached.

### 10.2 Consensus Policies (TOML Manifest)

Nodes can opt-in to specific consensus strategies:

- `mode = "majority"`: Requires $> 50\%$ of active agents to agree on the output hash.
- `mode = "unanimous"`: Requires $100\%$ agreement.
- `mode = "weighted"`: Success is determined by the cumulative weight of voting agents.

### 10.3 Failure Handling

If consensus is not reached within a defined `consensus_timeout_ms`, the node is marked as `Failed(ConsensusTimeout)`, triggering the standard `RetryNodes` logic or workflow abortion.
