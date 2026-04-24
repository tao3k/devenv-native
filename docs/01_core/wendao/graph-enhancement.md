---
type: knowledge
title: "Fusion Knowledge Architecture"
category: "architecture"
tags:
  - architecture
  - link
saliency_base: 7.0
decay_rate: 0.03
metadata:
  title: "Fusion Knowledge Architecture"
---

# Fusion Knowledge Architecture

## Two Cores

|                 | Core 1: LinkGraph (Wendao)                                  | Core 2: Lance / Arrow Storage Facade           |
| --------------- | ----------------------------------------------------------- | ---------------------------------------------- |
| **Engine**      | Common LinkGraph backend (`omni.rag.link_graph`)            | `xiuxian-db-store` vector-store facade         |
| **Strength**    | Explicit link graph, structural traversal, metadata filters | Storage-backed embeddings and Arrow batches    |
| **Data**        | Project markdown notes and link graph                       | Knowledge chunks, embeddings, multimodal rows  |
| **Query**       | `knowledge.search(..., mode="link_graph")`                  | Evidence source for Wendao/DuckDB query owners |
| **Enhancement** | xiuxian-wendao Rust (entity extraction, relation inference) | Lance/Arrow compatibility and table lifecycle  |

## Problem: Islands

Before integration, the two cores operated independently:

```
┌─────────────────────────┐     ┌─────────────────────────┐
│   Core 1: LinkGraph     │     │   Core 2: LanceDB       │
│                         │     │                         │
│  link_graph/backend.py          │     │  hybrid_search.py       │
│  link_graph/policy.py   │  ✗  │  recall.py              │
│  link_graph_navigator.py│     │  rust_vector.py         │
│  link_graph_enhancer.py         │     │  indexer.py             │
│  unified_knowledge.py   │     │  Arrow Flight query surface │
└─────────────────────────┘     └─────────────────────────┘
         ✗ No data sharing
         ✗ No cross-ranking
         ✗ No shared entity registry
```

Specific disconnections:

1. **Tool Router** used to be LanceDB-only. LinkGraph graph signal was not consulted.
2. **Knowledge Recall** used to be LanceDB-only. LinkGraph proximity was ignored.
3. **LinkGraph Search** lacked robust vector fallback. Semantic coverage was limited.
4. **Entity Registry** used to be split: LinkGraph had notes, LanceDB had tool schemas.

## Solution: Fusion Architecture

### Principle

**LinkGraph is the primary knowledge graph engine.** Lance-backed tables are a
storage and evidence boundary. Search semantics and result contracts belong to
Wendao/DuckDB-owned query layers, not to the vector-table storage shell.

### Integration Points

```
┌──────────────────────────────────────────────────────────────────┐
│                    FusionSearch                                    │
│                                                                  │
│  Query ──► Rust extract_query_intent() ──► FusionWeights         │
│              │           (computed once, flows to all bridges)    │
│         ┌────┴────────────────────┐                              │
│         ▼                         ▼                              │
│  ┌──────────────┐       ┌──────────────────┐                    │
│  │  Core 1: LinkGraph │  │ Core 2: Storage  │                    │
│  │  FTS + Graph │       │ semantic evidence │                    │
│  │  Traversal   │       │ Arrow batches     │                    │
│  └──────┬───────┘       └────────┬─────────┘                    │
│         │                        │                               │
│         └────────┬───────────────┘                               │
│                  ▼                                                │
│         ┌──────────────────┐                                     │
│         │  Fusion Engine   │                                     │
│         │  - RRF merge     │  (vector_weight, keyword_weight     │
│         │  - Graph boost   │   from FusionWeights → Rust RRF)   │
│         │  - Entity rerank │                                     │
│         └──────┬───────────┘                                     │
│                ▼                                                  │
│         ┌──────────────────┐                                     │
│         │  KnowledgeGraph  │  (Rust, persisted)                  │
│         │  Entity + Relation accumulation                        │
│         └──────────────────┘                                     │
└──────────────────────────────────────────────────────────────────┘
```

### Five Bridges + Intent-Driven Fusion

#### Bridge 1: LinkGraph → Router (Graph Signal for Tool Routing)

When routing a user query to the right tool:

```
User: "help me analyze a git repo"
  1. Router query plane → ranked tool list
  2. LinkGraph check: is "git repo" linked to any skill notes?
     -> If docs/reference/researcher.md links to docs/reference/git.md -> boost researcher
  3. Entity graph: does KnowledgeGraph have DOCUMENTED_IN/USES relations?
     -> Multi-hop boost for connected tools
```

#### Bridge 2: Storage Evidence -> LinkGraph Search

When LinkGraph search finds few results via links:

```
User: "vector search optimization"
  1. compute_fusion_weights() -> link_graph_proximity_scale=1.0 (code target -> neutral)
  2. LinkGraph traversal/search -> direct matches
  3. Storage-backed semantic evidence -> semantically similar notes
  4. Fuse: graph precision boost = 1.0 + 0.5 * link_graph_proximity_scale (dynamic)
     Graph boost scaled by link_graph_entity_scale
  5. Result: LinkGraph precision + semantic coverage, balanced by query intent
```

**Implementation**: `assets/skills/knowledge/scripts/search/hybrid.py::run_hybrid_search` now routes through `omni.rag.link_graph.policy` and merges via `omni.rag.link_graph.search_results`.

#### Bridge 3: LinkGraph Proximity + KG Entity → Knowledge Recall

When recalling knowledge chunks:

```
User: "how does the routing algorithm work?"
  1. compute_fusion_weights() -> shared intent analysis for all bridges
  2. Storage-backed recall -> top-k chunks by semantic similarity
  3. Bridge 1a: link_graph_proximity_boost(fusion_scale=link_graph_proximity_scale)
     -> If chunk A's doc links to chunk B's doc -> boost co-linked chunks
  4. Bridge 1b: apply_kg_recall_boost(fusion_scale=link_graph_entity_scale)
     -> If chunk source matches KG entities for query keywords -> boost
  5. Result: semantically relevant + structurally connected + entity-connected chunks
```

**Implementation**: `recall.py::_apply_fusion_recall_boost` computes fusion weights once and threads them through both LinkGraph proximity (Bridge 1a) and KG entity recall (Bridge 1b).

#### Bridge 4: Shared Entity Registry (`register_skill_entities`)

During `omni sync` / `omni reindex`:

```
omni sync
  1. Scanner parses SKILL.md -> skill metadata
  2. Publish storage-backed semantic evidence through the owning facade
  3. Build skill relationship graph (keyword overlap, same-skill edges)
  4. register_skill_entities() -> Rust-native batch registration
     -> SKILL entities (one per skill)
     -> TOOL entities (one per command)
     -> CONCEPT entities (one per routing keyword)
     -> CONTAINS relations: Skill -> Tool
     -> RELATED_TO relations: Tool -> keyword:*
  5. Graph persisted as a Valkey snapshot under a stable `scope_key`
```

**Implementation**: `fusion/graph_enrichment.py::register_skill_entities` → `KnowledgeGraph::register_skill_entities` (Rust, `graph/skill_registry.rs`)

This bridge ensures Bridge 3 has data: when `enrich_skill_graph_from_link_graph` searches the KnowledgeGraph for shared entities, it finds the entities registered here during sync.

#### Bridge 5: Query-Time KG Rerank (KG → Router at search time)

When routing a user query, after initial query-plane retrieval:

```
User: "help me search for knowledge about async patterns"
  1. Rust extract_query_intent() -> action="search", target="knowledge", keywords=["search", "knowledge", "async", "patterns"]
  2. compute_fusion_weights() -> link_graph_proximity_scale=1.5, kg_rerank_scale=1.3 (knowledge target boosts graph)
  3. KnowledgeGraph.query_tool_relevance(keywords, hops=2) -> multi-hop graph walk
     -> Finds tools connected via keyword:search, keyword:knowledge entities
  4. apply_kg_rerank() -> boosts tool scores by KG relevance * fusion_scale
  5. Result: tools with graph connections to query entities rank higher
```

**Implementation**: `fusion/kg_rerank.py::apply_kg_rerank` + `fusion/fusion_weights.py::compute_fusion_weights` → `KnowledgeGraph::query_tool_relevance` (Rust, `graph/query.rs`)

#### Intent-Driven Fusion Weights

The Rust-native `extract_query_intent` (in `graph/intent.rs`) decomposes queries into:

- **action**: canonical verb (search, create, commit, etc.)
- **target**: domain noun (knowledge, code, git, web, etc.)
- **context**: remaining qualifiers
- **keywords**: all significant tokens (stop-words removed)

`compute_fusion_weights()` uses these signals to dynamically adjust:

- `link_graph_proximity_scale`: graph proximity boost strength (Bridge 1a, 2, 3)
- `link_graph_entity_scale`: graph entity enrichment strength (Bridge 1b, 2, 3)
- `kg_rerank_scale`: KG query-time rerank strength (Bridge 5)
- `vector_weight` / `keyword_weight`: conceptual fusion weights owned by the
  caller's query/routing layer

Rules:
| Query Target | LinkGraph Emphasis | Semantic Evidence Emphasis |
|---|---|---|
| knowledge, docs | 1.5x LinkGraph proximity, 1.3x KG rerank | 0.9x semantic evidence |
| code, database, skill | 0.7x LinkGraph proximity | 1.2x semantic evidence, 1.3x lexical evidence |
| git (via action) | 0.8x LinkGraph proximity | 1.4x lexical evidence |
| (default) | 1.0x balanced | 1.0x balanced |

**Key architectural property**: No bridge computes fusion weights independently.
The `FusionWeights` object flows top-down from a single `compute_fusion_weights()` call,
eliminating redundant Rust calls and ensuring all bridges agree on query classification.

## Responsibility Split

### LinkGraph Backend (Wendao)

- Scan all markdown files
- Detect links (markdown + wiki format)
- FTS, tag filtering, date filtering, graph traversal
- Complex query syntax

### xiuxian-wendao (Rust Crate) — Enhancement Layer

| Capability                                                 | Module                                                   | Status |
| ---------------------------------------------------------- | -------------------------------------------------------- | ------ |
| Extract typed entity refs from `[[Entity#type]]`           | `link_graph_refs.rs`                                     | Done   |
| Parse TOML frontmatter                                     | `enhancer.rs`                                            | Done   |
| Infer relations (DOCUMENTED_IN, CONTAINS, RELATED_TO)      | `enhancer.rs`                                            | Done   |
| Batch enhance notes (Rayon-parallelized)                   | `enhancer.rs`                                            | Done   |
| Entity/Relation graph with multi-hop search                | `graph/query.rs`                                         | Done   |
| Multi-signal entity search (fuzzy + alias + token overlap) | `graph/query.rs`                                         | Done   |
| Bidirectional multi-hop traversal (outgoing + incoming)    | `graph/query.rs`                                         | Done   |
| Query-time tool relevance scoring                          | `graph/query.rs`                                         | Done   |
| Entity deduplication and normalization                     | `graph/dedup.rs`                                         | Done   |
| Graph persistence (JSON save/load)                         | `graph/persistence.rs`                                   | Done   |
| Batch skill entity registration (Bridge 4)                 | `graph/skill_registry.rs`                                | Done   |
| Lightweight query intent extractor (action/target/context) | `graph/intent.rs`                                        | Done   |
| PyO3 bindings for all above                                | `graph_py.rs`, `enhancer_py.rs`, `link_graph_refs_py.rs` | Done   |

### Lance / Arrow Storage Facade

| Capability                                  | Owner or Module                           | Status |
| ------------------------------------------- | ----------------------------------------- | ------ |
| Lance-backed vector-table lifecycle         | `xiuxian-db-store` facade + storage shell | Active |
| Arrow batch compatibility helpers           | vector-store compatibility surface        | Active |
| Retrieval-row RecordBatch helpers           | `query_support.rs`                        | Active |
| Search semantics and routing result shapes  | Wendao/DuckDB or owning router crate      | Active |
| Vector-local keyword/tool/agentic ownership | retired                                   | Done   |

### Python Layer — Thin Orchestration

| Responsibility                                       | Module                                                                   |
| ---------------------------------------------------- | ------------------------------------------------------------------------ |
| LinkGraph backend calls                              | `link_graph/backend.py`, `link_graph/factory.py`                         |
| Fusion search logic (intent-aware merge)             | `assets/skills/knowledge/scripts/search/hybrid.py`                       |
| Rust enhancer delegation                             | `link_graph_enhancer.py`                                                 |
| Storage-backed semantic evidence bridge              | owned by the caller facade                                               |
| **Fusion bridges (modularized package)**             | **`fusion/`**                                                            |
| Bridge 1a: LinkGraph proximity boost (fusion-scaled) | `fusion/link_graph_proximity.py`                                         |
| Bridge 1b: KG entity recall boost (fusion-scaled)    | `fusion/kg_recall.py`                                                    |
| Bridge 2: Storage evidence -> LinkGraph bridge       | `assets/skills/knowledge/scripts/search/hybrid.py::_run_vector_fallback` |
| Bridge 3+4: Graph enrichment + entity registry       | `fusion/graph_enrichment.py`                                             |
| Bridge 5: KG query-time rerank (fusion-scaled)       | `fusion/kg_rerank.py`                                                    |
| Dynamic fusion weights (intent → weights)            | `fusion/fusion_weights.py`                                               |
| Bridge constants and graph path resolution           | `fusion/_config.py`                                                      |
| **Unified intent pipeline: recall**                  | `recall.py::_apply_fusion_recall_boost`                                  |
| **Unified intent pipeline: router**                  | `hybrid_search.py::HybridSearch.search`                                  |
| **Unified intent pipeline: LinkGraph hybrid**        | `assets/skills/knowledge/scripts/search/hybrid.py::run_hybrid_search`    |
| Sync hook (Bridge 4 caller)                          | `indexer.py`, `reindex.py`                                               |

## Data Flow

### Sync Time (Index Building)

```
omni sync / omni reindex
  │
  ├── Scanner → SKILL.md frontmatter
  │     ├── owning indexer publishes storage-backed semantic evidence
  │     ├── build_relationship_graph() → skill_graph.json (keyword overlap edges)
  │     └── register_skill_entities() → knowledge.lance + JSON (Bridge 4, Rust-native)
  │           Creates: SKILL, TOOL, CONCEPT entities + CONTAINS, RELATED_TO relations
  │
  └── LinkGraph indexer → auto-detects links across markdown sources
```

### Query Time (Unified Intent Signal)

All query-time pipelines share a **single intent analysis** via `compute_fusion_weights()`.
This ensures consistent behavior: one Rust extraction drives all bridges.

```
Query → Rust extract_query_intent() → FusionWeights (computed once per pipeline)
  │
  ├── Tool Routing (HybridSearch.search)
  │     ├── router-owned retrieval and rerank ← from FusionWeights
  │     ├── Associative rerank → skill_graph.json
  │     ├── Bridge 3: enrich_skill_graph_from_link_graph()
  │     └── Bridge 5: apply_kg_rerank(fusion_scale=kg_rerank_scale)
  │
  ├── Knowledge Recall (recall skill command)
  │     ├── Core 2: storage-backed semantic evidence → top-k chunks
  │     ├── Bridge 1a: link_graph_proximity_boost(fusion_scale=link_graph_proximity_scale)
  │     └── Bridge 1b: apply_kg_recall_boost(fusion_scale=link_graph_entity_scale)
  │
  └── LinkGraph Hybrid Search (link_graph_hybrid_search skill command)
        ├── Core 1: LinkGraph structural traversal → reasoning-based results
        ├── Bridge 2: storage-backed semantic fallback
        ├── Merge: graph precision boost = 1.0 + 0.5 * link_graph_proximity_scale
        └── Graph boost: base_boost * link_graph_entity_scale
```

## Graph Persistence

### Valkey Snapshot Storage (Primary)

The KnowledgeGraph is persisted as a Valkey snapshot keyed by a stable `scope_key`.
This keeps graph persistence on the DB hot path and avoids filesystem coupling.

### Persistence API

| API                       | Rust                                  | Python (via PyO3)                              |
| ------------------------- | ------------------------------------- | ---------------------------------------------- |
| Save to Valkey snapshot   | `kg.save_to_valkey(scope, dim).await` | `kg.save_to_valkey(scope_key, dimension=1024)` |
| Load from Valkey snapshot | `kg.load_from_valkey(scope).await`    | `kg.load_from_valkey(scope_key)`               |
| Save to JSON (legacy)     | `kg.save_to_file(path)`               | `kg.save_to_file(path)`                        |
| Load from JSON (legacy)   | `kg.load_from_file(path)`             | `kg.load_from_file(path)`                      |

### Bridge Load/Save Resolution

All bridge modules (`kg_recall`, `kg_rerank`, `graph_enrichment`) use `_load_kg` / `_save_kg` from `fusion/_config.py`:

1. **Default runtime** (no explicit scope): resolve stable scope via database config
2. **Explicit `scope_key`**: read/write that snapshot namespace directly
3. **No Valkey configured**: `_load_kg` degrades to `None` and bridge logic becomes a no-op

### Population

- **Sync-time seeding**: Every `omni sync` registers all skills/tools/keywords as entities (Bridge 4, dual-write)
- **Search-time enrichment**: Each query can discover new entity connections
- **Cross-session memory**: The graph persists across restarts, accumulating knowledge

## Test Coverage

| Bridge                         | Test File                                            | Tests                                          |
| ------------------------------ | ---------------------------------------------------- | ---------------------------------------------- |
| Bridge 1a (LinkGraph → Recall) | `test_fusion.py::TestLinkGraphProximityBoost`        | 6                                              |
| Bridge 3 (LinkGraph → Router)  | `test_fusion.py::TestEnrichSkillGraphFromLinkGraph`  | 5                                              |
| Bridge 4 (Sync → KG)           | `test_fusion.py::TestRegisterSkillEntities`          | 5                                              |
| Dynamic Fusion Weights         | `test_fusion.py::TestFusionWeights`                  | 6                                              |
| Skill wiring                   | `test_fusion.py::TestSkillCommandWiring`             | 3                                              |
| Router wiring                  | `test_fusion.py::TestRouterSkillRelationshipsWiring` | 2                                              |
| Rust: Graph CRUD + search      | `test_graph.rs`                                      | 23 (incl. Lance roundtrip, skill registration) |
| Rust: Intent extractor         | `test_intent.rs`                                     | 14 (action/target/context decomposition)       |
| **Total**                      |                                                      | **67**                                         |
