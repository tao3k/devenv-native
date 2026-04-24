---
type: knowledge
title: "Xiuxian-Agent: The Thin Orchestrator & Execution Container"
category: "core"
tags:
  - agent
  - container
  - runtime
metadata:
  title: "Xiuxian-Agent: The Thin Orchestrator (赛博修仙主脑)"
---

# Xiuxian-Agent: The Thin Orchestrator (赛博修仙主脑)

`xiuxian-agent` is the primary high-performance execution container for the Xiuxian OS. It follows the **"Ignorant Host"** paradigm, where the core remains stateless and delegates all domain-specific logic to Synaptic Flows.

## 1. Radical Thinning Architecture (ADR-010)

The Agent core is a stateless orchestrator. All domain-specific knowledge and internal states have been physically offloaded to specialized crates:

- **LLM Infrastructure**: Cooldowns, dimension repair, and provider routing are offloaded to `xiuxian-llm`.
- **Cognitive Memory**: Ranking, filtering, and Q-reranking reside in `xiuxian-memory-engine`.
- **Manifestation**: Persona registries and prompt injection management are governed by `xiuxian-qianhuan`.

## 2. The Validated Feeding Artery (ADR-011)

To ensure zero-trust data security and structural integrity, the Agent utilizes the **Zhenfa Transmuter** as its primary data artery.

- **Workflow**: `Extract (VFS) -> Wash (Zhenfa) -> Validate (Predicate) -> Feed (LLM)`.
- **Benefit**: Prevents prompt injection and formatting hallucinations by ensuring only "Sealed" data enters the model context.

## 3. Workflow-Driven Execution

The Agent's primary lifecycle is driven by declarative TOML manifests:

- **Late Binding**: Resources (personas, templates) are resolved on-demand using the **`$` placeholder** mapping to `wendao://` URIs.
- **Bootcamp Integration**: Agent utilizes the `Qianji` Bootcamp API for deterministic scenario validation, ensuring that business logic is verified in a controlled laboratory environment before live deployment.

## 4. Performance Standards

- **Parallel Pipeline (ADR-006)**: Non-blocking fan-out for history, recall, and validation.
- **Zero-Copy Loading**: Integrated with the `SkillRuntimeResolver` for microsecond-level pointer-based resource access.
