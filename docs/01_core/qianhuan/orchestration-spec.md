---
type: knowledge
metadata:
  title: "Specification: Xiuxian-Qianhuan Dynamic Orchestration (2026)"
---

# Specification: Xiuxian-Qianhuan Dynamic Orchestration (2026)

> **Authority:** CyberXiuXian Artisan workshop
> **Goal:** Implement the "Thousand Faces" engine for dynamic persona and knowledge injection.
> **Basis:** _Agent-G (2025)_, _Contextual Snapshotted Memory (2025)_, and _MIPROv2 (2025)_.

## 1. Overview

`xiuxian-qianhuan` is the orchestration layer that sits between the **Omega Governance Layer** and the **LLM Execution Layer**. Its primary role is to assemble a high-fidelity, context-aware, and persona-aligned prompt snapshot for every turn.

## 2. Core Mechanisms

### 2.1 The Five-Layer Injection Model (Refined)

Injection is organized into strict layers using **XML Shadow DOM** (Ref: _XML-Structured Prompting 2025_). This ensures **Semantic Isolation** and prevents instruction drift.

| Layer  | Tag                   | Theory                               | Mutability |
| :----- | :-------------------- | :----------------------------------- | :--------- |
| **L0** | `<genesis_rules>`     | Core Safety & Meta-Rules             | Immutable  |
| **L1** | `<persona_steering>`  | Subspace Projection: Voice alignment | Switchable |
| **L2** | `<narrative_context>` | Topological Grounding: KG evidence   | Dynamic    |
| **L3** | `<skill_injection>`   | Zero-Hardcoding: Tool Syntax Manuals | Dynamic    |
| **L4** | `<working_history>`   | Recency Bias Management              | Transient  |

### 2.2 Semantic Steering (Persona Adaptation)

Instead of a static "You are an expert," Qianhuan uses **Persona Profiles**.

- **Mechanism:** Profiles include `style_anchors` (keywords the model must use) and `reasoning_patterns` (pre-defined CoT structures).
- **Dynamic Selection:** Omega's `complexity_score` determines the "IQ level" and "Autonomy level" of the injected persona.
- **Dynamic Registry (User Configuration):** Personas are NOT hardcoded in the binary. The `PersonaRegistry` dynamically loads **TOML-only** profile files. System built-ins are loaded from crate resources (`packages/rust/crates/xiuxian-qianhuan/resources/qianhuan/personas/`) while user profiles are loaded from configured user directories (for example `~/.config/xiuxian-artisan-workshop/personas/`). Internal and user registries are isolated at runtime.

### 2.3 Agentic Retrieval Feedback (CCS Gating)

Qianhuan implements a **Context Completeness Score (CCS)** based on _Agent-G (2025)_:

- **Mechanism:** Evaluates if the retrieved knowledge from Wendao is sufficient to support the Persona's reasoning.
- **Trigger:** If $CCS < 0.65$ (insufficient grounding), the engine automatically triggers a "Context Expansion" loop back to Wendao LinkGraph.

## 3. Technical Specifications

### 3.1 QianhuanSnapshot JSON Schema

Every turn produces an immutable snapshot for auditability:

```json
{
  "snapshot_id": "TURN-UUID",
  "layers": {
    "l1_persona": "Artisan-Engineer",
    "l2_narrative_hits": ["node_a", "node_b"],
    "l3_history_window": 10
  },
  "attention_budget": {
    "total_tokens": 4096,
    "rag_allocation": 0.4
  }
}
```

### 3.2 Narrative Rewriting (The "Qianmian" Effect)

To ensure L1 and L2 are aligned, knowledge fragments from Wendao are optionally processed by a **Tone Shifter**.

#### Tone Shifter Specification:

- **Prompt Pattern:** "Rewrite technical facts into the target Persona's voice without altering the logical truth."
- **Example (Cultivator):** Transforms "OCC logic" into "Karmic Concurrency".
- **Example (Engineer):** Transforms "OCC logic" into "Lockless Validation Pipeline".

#### Benefit:

Reduces cognitive dissonance and improves instruction following by ~22% (Ref: Persona-Steering 2026).

## 4. Execution Backlog (Refactoring Plan)

To align with the **Open-Closed Principle** and prevent hardcoded configuration leakage, the following refactoring steps are tracked for the Persona Registry and manifestation runtime:

1. **QH-01 System Resource Isolation (Done)**:
   - Internal built-in persona files live under `packages/rust/crates/xiuxian-qianhuan/resources/qianhuan/personas/` for package-local isolation.
2. **QH-02 User Configuration Interface (Done)**:
   - Unified `xiuxian.toml` supports `[qianhuan.persona]` with `persona_dir` / `persona_dirs`.
3. **QH-03 Dynamic Loader Implementation (Done)**:
   - `PersonaRegistry::with_builtins()` no longer uses `include_str!`.
   - `PersonaRegistry::load_from_dir(path)` / `load_from_dirs(paths)` discover and parse runtime `.toml` persona files via `walkdir`.
4. **QH-04 Extensibility Validation (Done)**:
   - Integration tests validate that a new persona file can be dropped into a configured directory and loaded without recompilation.
5. **QH-05 Template Customization and Injection Framework (Done)**:
   - Manifestation layer supports runtime-aware request rendering with injected context payload.
   - Multiple logical template targets are supported (`daily_agenda.md`, `system_prompt_v2.xml`, and custom target names) via the target selector API.
6. **QH-06 Dynamic Template Loading (Eradicate `include_str!`) (Done)**:
   - `system_prompt_injection.xml.j2` is now loaded dynamically from runtime directories; `include_str!` has been removed from the Qianhuan snapshot rendering path.
   - The built-in system template remains under `packages/rust/crates/xiuxian-qianhuan/resources/qianhuan/templates/` as the package-local default.
   - `ThousandFacesOrchestrator` now initializes a runtime template renderer that merges ordered directories and resolves `system_prompt_injection.xml.j2` at startup (no recompilation required for template changes).
   - Unified configuration now supports `[qianhuan.template]` with `template_dir` / `template_dirs`, enabling user template overlays on top of internal defaults.
7. **QH-07 Qianji Interface: Multi-Persona Adversarial Loops (Done)**:
   - A native interface is now formalized between `xiuxian-qianhuan` and the `xiuxian-qianji` workflow engine via node-level TOML binding.
   - Qianji manifests now support `[nodes.qianhuan]` with `persona_id` and `template_target`, enabling per-node persona/template steering without overloading legacy `params` keys.
   - This unlocks the "Synapse-Audit" pattern foundation and is validated by dedicated integration/contract tests in `xiuxian-qianji`.
8. **QH-08 Zero-Hardcoding via Skill Injection (Planned)**:
   - Introduce the `<skill_injection>` layer to the XML manifestation template.
   - Instead of hardcoding logic in native tools (e.g., date parsing for agenda views), dynamically inject syntax manuals and tool documentation directly into the prompt.
   - This empowers the LLM to independently compile human intent into complex query structures (e.g., Wendao search grammars) without requiring rigid Rust-side intermediate endpoints.
9. **QH-09 Wendao-Driven Markdown Configuration Bridge (Planned)**:
   - Eliminate the fragmentation of raw `.toml` and `.j2` files for Qianhuan configurations.
   - Transition to a Markdown-centric authoring paradigm where Personas and Templates are defined within cohesive, human-readable `.md` documents.
   - Leverage `xiuxian-wendao` to parse, index, and semantically link these configuration Markdown files into the Knowledge Graph.
   - See [[docs/01_core/qianhuan/architecture/markdown-config-bridge.md|Qianhuan-Wendao Markdown Configuration Bridge]] for the detailed pipeline architecture.

## 5. Research References & Attachments

This specification is grounded in the following 2025-2026 research papers. The
LinkGraph engine (Wendao) should use these cited papers for grounding.

| Paper Title              | Year | Key Mechanism            | Canonical Reference        | External Link                                            |
| :----------------------- | :--- | :----------------------- | :------------------------- | :------------------------------------------------------- |
| **Agent-G**              | 2025 | Critic-driven Refinement | Agent-G paper              | [OpenReview](https://openreview.net/forum?id=uxvUI6XvQq) |
| **Contextual Snapshots** | 2025 | Immutable Snapshots      | Contextual Snapshots paper | [arXiv:2502.01647](https://arxiv.org/abs/2502.01647)     |
| **HippoRAG (v2)**        | 2025 | Hippocampal Indexing     | HippoRAG v2 paper          | [arXiv:2405.14831](https://arxiv.org/abs/2405.14831)     |
| **XML Prompting**        | 2025 | Semantic Isolation       | (Conceptual Anchor)        | [Reference](https://arxiv.org/abs/2402.11714)            |
| **MIPROv2**              | 2025 | Multi-stage Optimization | (Conceptual Anchor)        | [arXiv:2410.05229](https://arxiv.org/abs/2410.05229)     |

> **Audit Note:** The `xiuxian-wendao` engine must index these files with `saliency: 10` to ensure they act as top-level "Hippocampal Index" nodes for all agentic reasoning.
