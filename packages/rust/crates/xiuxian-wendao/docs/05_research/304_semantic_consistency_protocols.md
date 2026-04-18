# Agentic Workflow & Context Engineering (OpenDev 2026)

:PROPERTIES:
:ID: paper-2026-opendev-workflow
:PARENT: [[301_research_papers|Research Index: Map of Content]]
:TAGS: research, context-engineering, agent-harness, adaptive-compaction
:STATUS: HARDENING
:END:

## 🧠 Core Theory: The Agent Harness

Research (OpenDev 2026) defines the "Harness" as a deterministic layer surrounding the LLM reasoning loop. It manages context as a finite resource and enforces safety through "Plan-Execute" separation.

## 🔬 Technical Breakdown (OpenDev Implementation)

### 1. Adaptive Context Compaction (5-Stage)

Instead of simple sliding windows, context is managed through progressively aggressive stages:

1. **Masking (80%)**: Replaces verbose tool outputs with semantic IDs.
2. **Pruning (85%)**: Deletes low-saliency nodes.
3. **Full Compaction (99%)**: Hierarchical summarization of the entire session.

### 2. Subagent-Based Planning

Separates "Reasoning about the Change" from "Executing the Change" at the **Tool Schema** level.

- **Planner**: Read-only access, generates a 7-point structured plan.
- **Executor**: Full access, implements the approved plan using Cas-like transactions.

### 3. Doom-Loop & Stale-Read Detection

The harness monitors the **State Delta Trajectory**. If the agent repeats an action without changing the state, or attempts to edit based on an outdated `content_hash`, the harness injects a "Recovery Nudge".

## 🛠️ Wendao Implementation Blueprint

- [ ] **Feature**: Implement `Context Compactor` in `LinkGraph` using the 5-stage logic.
- [ ] **Feature**: Add `Stale-Read Detection` to `semantic_edit` by checking the input `content_hash` against the live AST.
- [ ] **Feature**: Create `wendao.plan` sub-command to allow Agents to generate structural modification previews before mutation.

---

:RELATIONS:
:LINKS: [[06_roadmap/401_project_sentinel|Project Sentinel: Semantic Consistency]], [[addressing/mod.rs|descriptive label]]
:ASSETS: OpenDev 2026 workflow paper
:END:
