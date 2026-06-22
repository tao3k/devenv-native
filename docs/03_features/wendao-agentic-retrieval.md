---
type: feature
metadata:
  title: "Wendao: Agentic Retrieval (Autonomous Query Planning)"
  status: "Landed"
  last_updated: "2026-03-12"
---

# Wendao: Agentic Retrieval (Agent-G Integration)

## 1. Overview

Agentic Retrieval evolves Wendao from a passive retriever into an autonomous
search engine. It implements the principles of **Agent-G (2025)**, allowing
the engine to "think" before execution by generating dynamic expansion plans.

## 2. Core Mechanisms

### 2.1 Saliency-Aware Priority Scoring

The expansion planner integrates real-time signals from the [[docs/03_features/wendao-living-brain.md|Living Brain]].

- **Priority Logic**: $\text{Priority} = \text{Semantic_Score} \times \text{Saliency_Factor}$
- **Implementation**: Physically defined in [[packages/rust/crates/xiuxian-wendao/src/link_graph/index/agentic_expansion/plan/candidates.rs|agentic_expansion/plan/candidates.rs]].

### 2.2 Idempotent Knowledge Discovery

To ensure the LLM does not hallucinate redundant links, the system maintains a "Discovery Hash" in [[packages/rust/crates/xiuxian-wendao/src/link_graph/agentic/idempotency.rs|Valkey Idempotency Layer]].

### 2.3 Parallel Worker Orchestration

Large search tasks are partitioned into discrete `WorkerPlans`, executing in parallel within the [[packages/rust/crates/xiuxian-wendao/src/link_graph/index/agentic_expansion/execute.rs|Agentic Execution Engine]].

## 3. Physical Architecture

- **Planning Hub**: `xiuxian-wendao/src/link_graph/index/agentic_expansion/plan.rs`
- **Transport Direction**: autonomous dispatch is moving away from legacy
  external-tool transport seams and should not treat transport adapters as a
  permanent architecture dependency.

## 4. Related Features

- [[docs/03_features/wendao-living-brain.md|Living Brain (Saliency Provider)]]
- [[docs/03_features/wendao-context-snapshot.md|ContextSnap (State Recovery)]]

---

_Wisdom is the alignment of Action and Intent._
