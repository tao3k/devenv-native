---
type: knowledge
metadata:
  title: "Guide: LinkGraph Precision Code Adaptation (2026)"
---

# Guide: LinkGraph Precision Code Adaptation (2026)

> **Authority:** CyberXiuXian Artisan workshop
> **Goal:** Document the exact code changes required to align the `link_graph` package with HippoRAG, GRAG, and HMAS research.
> **Key Research Context:** Agent-G (2025), ContextSnap (2025), and HippoRAG
> (2025).

---

## 1. Schema Extensions (Resource Localization)

### 1.1 Owner Crates

Schemas are now moved from `shared/schemas` to their respective owner crates' `resources/` directory.

- **LinkGraph Schemas:** `packages/rust/crates/xiuxian-wendao/resources/`
- **Agent Schemas:** `packages/rust/crates/xiuxian-wendao/resources/agent/`
- **Memory Schemas:** `packages/rust/crates/xiuxian-memory-engine/resources/`

### 1.2 Rust Binding Access

Python code must access these schemas via the centralized `schema_provider` which calls Rust bindings.

- **Provider:** `packages/python/foundation/src/omni/foundation/api/schema_provider.py`
- **Rust Hook:** `xiuxian_wendao::schema_py::get_schema(name)`

---

## 2. Model Adaptations (`packages/python/foundation/src/omni/rag/link_graph/models.py`)

### 2.1 `LinkGraphHit` Data Class

**Proposed Refinement:**

```python
@dataclass(frozen=True)
class LinkGraphHit:
    # Existing fields...
    source_claims: list[str] = field(default_factory=list) # For GRAG
    triples: list[list[str]] = field(default_factory=list) # For HippoRAG
    saliency: float | None = None                         # For PPR weighting
```

---

## 3. Algorithm Refinement (`packages/python/foundation/src/omni/rag/link_graph/policy.py`)

### 3.1 Dynamic Damping Logic (HippoRAG Alignment)

**Target:** Replace hardcoded `alpha` (damping) with a query-aware controller.
**Proposed Algorithm:**

- **Baseline:** $d = 0.5$ (Paper p.4 recommendation for ZK graphs).
- **Rule A (Focus):** If `confidence > 0.8`, increase $d$ to $0.7$ (Focus on the specific node).
- **Rule B (Explore):** If `query_length > 100` or `confidence < 0.4`, decrease $d$ to $0.3$ (Deep topological exploration).

### 3.2 Saliency-Weighted Confidence

**Proposed Formula Change:**
Include **"Distribution Parity"** in the confidence score. If scores are too evenly spread, confidence in a specific structural path is lowered.

---

## 4. Narrative Topology (`packages/python/foundation/src/omni/rag/link_graph/narrator.py`)

**New Component Requirement:**
Implement a `SubGraphNarrator` that:

1. Iterates through `LinkGraphHit.source_claims`.
2. Assembles a "Hierarchical Narrative" hard-prompt.
3. **Format:** `[Concept A] links to [Concept B] because of [Claim X]`.
   _Purpose: Prevents LLM reasoning drift during long-context retrieval (GRAG Core Theory)._

---

## 5. Storage Architecture: Dual-Drive Memory Network

This project implements a hybrid storage strategy to balance semantic precision (LanceDB) with low-latency associative reasoning (Valkey).

### 5.1 LanceDB (Cortex Trigger)

- **Role:** High-cost semantic entry point lookup.
- **Data:** `[note_id, semantic_anchor_vector]`.
- **Constraint:** Do NOT vectorize full body. Only vectorize 1-sentence summaries.

### 5.2 Valkey (Hippocampal Engine)

- **Role:** Low-cost structural walk and weight updates.
- **Naming Standard:** Use domain prefixes such as `xw` (Wendao) with versioning; downstream packages should define their own prefixes at the package boundary.

| Domain           | Key Template                  | Type       | Artisan Rationale                                                          |
| :--------------- | :---------------------------- | :--------- | :------------------------------------------------------------------------- |
| **KG Kernel**    | `xw:kg:v1:node:{id}:core`     | Hash       | SSOT for node grounding data (Wendao ownership).                           |
| **Persona Face** | `xq:face:v1:{p_id}:node:{id}` | Hash       | Stores persona-specific activation/boost (The "Thousand Faces" dimension). |
| **KG Topology**  | `xw:kg:v1:edge:out:{id}`      | **ZSET**   | Stores linked nodes with global saliency scores.                           |
| **Working BB**   | `xq:bb:v1:session:{s_id}`     | Hash       | TTL-guarded orchestrator blackboard.                                       |
| **Audit Stream** | `xq:stream:v1:trace`          | **Stream** | Behavioral evidence log for HMAS.                                          |

---

## 6. Implementation: Self-Evolving Saliency (Valkey Logic)

### 6.1 Manager System Prompt (Strategy Layer)

- **Role:** Supervisor.
- **Instruction:** "Post tasks to the blackboard using [TASK] tags. You must validate the [DIGITAL THREAD] JSON of workers against the original requirement."

### 6.2 Worker System Prompt (Tactical Layer)

- **Role:** Executor.
- **Instruction:** "Read [TASK] from the blackboard. Output observations in [EVIDENCE] and a final JSON [DIGITAL THREAD] mapping your conclusion to source nodes."

---

## 9. Dynamic Research Linking (Meta-Reasoning)

The system must treat the `.data/research/papers` directory as its **Core Belief System**.

### 9.1 Automatic Meta-Tagging

- During indexing, any node originating from the research directory must be tagged with `#research/foundational`.
- **Saliency Override:** These nodes receive a mandatory $S_{base} = 10.0$.

### 9.2 Grounded Reflection

- When Qianji local annotation or advisory prompt-context planning generates a snapshot,
  it must check if any `#research/foundational` nodes are in the Top-K.
- If yes, the snapshot planner must prioritize their technical claims as the
  "Root of Truth" for the current reasoning turn.

---

## 10. Rust Core Injection Adaptation

Context injection is owned by the consuming Rust crate instead of a separate
prompt-persona crate. Qianji owns workflow annotation snapshots and advisory
prompt-context records directly.

### 10.1 XML-Based Context Tagging

To ensure strict isolation between system rules and RAG data (Basis: _Contextual Snapshots 2025_), the engine uses XML tags:

- `<persona_id>` and `<persona_name>`: Pin the active role identity.
- `<narrative_context>`: Inject L2 LinkGraph subgraphs.
- `<working_history>`: Carry bounded appended-mode context when the workflow
  explicitly requests history reuse.
- **Validation:** Snapshot builders must escape XML text before insertion to
  prevent tag-escape injections.

### 10.2 Local Annotation Contracts

Qianji annotation uses local profile data and node-level manifest bindings to
assemble deterministic snapshots. It does not depend on an external persona
runtime crate.

### 10.3 Core Modules

- **Qianji annotation executor**: Assembles workflow-local XML snapshots.
- **Qianji advisory prompt-context planner**: Builds typed role-mix snapshots for
  contract feedback.
- **Zhixing manifestation trait**: Provides a lightweight template rendering
  boundary without depending on a separate prompt-persona crate.

## 11. Architectural Principle: Rust-Hard, Python-Thin

To maintain maximum performance, safety, and cognitive integrity, this project enforces a strict boundary between Rust and Python.

### 11.1 Logic Enclosure (Rust)

- **Calculation & State:** All algorithms ($\phi$ evolution, PPR random walks, XML assembly) must reside in Rust crates.
- **Resources:** Configuration files (Personas, Schemas) must be compiled into the binary via `include_str!`.
- **Validation:** Integrity checks (XML balance, Schema validation) are performed at the Rust boundary.

### 11.2 Interface Exposure (PyO3)

- **Thin Slices:** Python modules should only expose high-level orchestration methods.
- **Zero-Logic Python:** Python-side "backends" should be simple wrappers that delegate 100% of the work to Rust calls.

---

## 12. Final Implementation Status (Audit Pass)

- **LinkGraph Evolution:** ✅ Integrated in `xiuxian-wendao`.
- **Context Injection:** Implemented locally in consuming Rust crates.
- **Research Grounding:** ✅ Configured via `.data/research/papers` indexing.
- **Schema Purity:** ✅ Resource localization pattern established.
