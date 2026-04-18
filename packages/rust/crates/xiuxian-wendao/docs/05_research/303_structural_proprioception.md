# Structural Proprioception: Agent Self-Awareness of Topology

:PROPERTIES:
:ID: paper-2025-proprioception
:PARENT: [[301_research_papers|Research Index: Map of Content]]
:TAGS: research, cognitive-rag, topology, proprioception
:STATUS: HARDENING
:END:

## 🧠 Core Theory: Structural Self-Awareness (Refined)

Research (LongRefiner 2025) demonstrates that modeling documents as **hierarchical trees** rather than strings increases signal-to-noise ratio by 10x. Proprioception is achieved via a "Simplified XML" envelope that encodes logical ancestry without context bloat.

## 🔬 Technical Breakdown (LongRefiner / HiSem-RAG)

### 1. Simplified XML Logic

Instead of raw text, the system serves a structured view:

```xml
<doc id="id">
  <sec title="Core">
    <p>Block content...</p>
  </sec>
</doc>
```

- **Result**: 10x reduction in latency $\tau$ and computational cost compared to perplexity-based methods.

### 2. Dual-Level Query Analysis

- **Local Level**: Knowledge scope limited to single snippets.
- **Global Level**: Background context across multiple sections.
- **Wendao Match**: PageIndex handles Local; LinkGraph handles Global.

### 3. Adaptive Document Refinement

The refiner dynamically adjusts the compression ratio $\gamma = |D|/|D'|$ based on query complexity.

## 🛠️ Wendao Implementation Blueprint

- [x] **Data Model**: `PageIndexNode` hierarchy (Implemented).
- [ ] **Feature**: `Simplified XML Output` in `semantic_read.rs` to match the LongRefiner standard.
- [ ] **Feature**: `Adaptive Refinement Controller` to prune context based on Agent intent (Local vs. Global).

---

:RELATIONS:
:LINKS: [[03_features/202_block_addressing|descriptive label]], [[01_core/101_triple_a_protocol|Triple-A Addressing Protocol]]
:ASSETS: LongRefiner 2025 paper
:END:
