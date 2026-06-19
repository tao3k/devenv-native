---
type: feature
metadata:
  title: "Wendao: ContextSnap (Stateful Context Governance)"
  status: "Landed"
  last_updated: "2026-03-12"
---

# Wendao: ContextSnap (Context Engineering)

## 1. Overview

ContextSnap provides $O(1)$ state snapshots and zero-cost rollback
capabilities for quantum-fusion retrieval contexts. It implements
**ContextSnap (2025)** principles, allowing Qianji Agents to checkpoint their
knowledge state.

## 2. Core Mechanisms

### 2.1 Deterministic Snapshot Hash (SnapID)

Every unique combination of query text and semantic anchors produces a stable 16-hex SnapID.

- **Implementation**: Defined in [[packages/rust/crates/xiuxian-wendao/src/link_graph/context_snapshot.rs|context_snapshot.rs]].
- **Traceability**: Injected into [[packages/rust/crates/xiuxian-wendao/src/link_graph/models/records/quantum_fusion.rs|QuantumContext]] as `trace_label`.

### 2.2 Valkey-Backed State Persistence

Full retrieval states are serialized and stored in Valkey, enabling the rollback of complex reasoning chains.

### 2.3 Zero-Cost Rollback

The engine provides a direct `rollback(snap_id)` interface, physically implemented in [[packages/rust/crates/xiuxian-wendao/src/link_graph/index/search/quantum_fusion/orchestrate.rs|orchestrate.rs]].

## 3. Physical Architecture

- **Sovereign Hub**: `xiuxian-wendao/src/link_graph/context_snapshot.rs`
- **Integration**: Hooked into the Quantum Fusion search pipeline.

## 4. Related Features

- [[docs/03_features/wendao-agentic-retrieval.md|Agentic Retrieval]]
- [[docs/03_features/wendao-living-brain.md|Living Brain]]

---

_Time is but a dimension of Knowledge._
