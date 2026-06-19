---
type: knowledge
metadata:
  title: "LLM Reading Guide: Project Context Index (2026)"
---

# LLM Reading Guide: Project Context Index (2026)

> **Role:** Bootstrapping Entry Point for AI Agents.
> **Scope:** High-density architectural mapping and schema resolution.

## 1. Core Reasoning Paths

When tasked with modifying or reasoning about this codebase, refer to these "Ground Truth" documents:

### 1.1 Quadrilateral Engine (The "How")

- **Topological Logic:** `docs/core/wendao/ppr-algorithm.md`
- **Memory Evolution:** `docs/core/memory/memrl-evolution.md`
- **Context Injection:** `docs/01_core/memory/injection-evolution.md`
- **Governance:** `docs/core/omega/trinity-control.md`

### 1.2 System Schemas (The "What")

All structural contracts are defined in Rust owner crates and exposed via Python bindings.

- **Provider:** `packages/python/foundation/src/omni/foundation/api/schema_provider.py`

## 2. Context Injection Protocol (XML Shadow DOM)

AI agents must adhere to the XML isolation protocol defined in `docs/01_core/memory/injection-evolution.md`.

- **L0:** `<genesis_rules>`
- **L1:** `<persona_steering>`
- **L2:** `<narrative_context>`
- **L3:** `<working_history>`

## 3. Knowledge Base Hierarchy

This project treats `.data/research/papers` as the **Primary Belief System**.

- **Foundational Saliency:** 10.0
- **Tag:** `#research/foundational`
