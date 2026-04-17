---
type: knowledge
title: "Wendao Architecture: High-Performance ID Resolution Mechanism"
category: "architecture"
tags:
  - wendao
  - search-engine
  - indexing
  - valkey
  - performance
saliency_base: 9.0
decay_rate: 0.01
metadata:
  title: "Wendao Architecture: High-Performance ID Resolution Mechanism"
---

# Wendao Architecture: High-Performance ID Resolution Mechanism

This document specifies how the **Wendao Search Engine** provides $O(1)$ exact-match resolution for entities (Personas, Templates, Tasks) while maintaining a unified search abstraction.

## 1. The Search vs. Indexing Paradox

In typical systems, "Search" (fuzzy, semantic) and "Index Lookup" (exact, key-based) are treated as two different APIs. To maximize the LLM's efficiency, Wendao unifies these under a single **Search Grammar**, but implements a **Short-Circuit Execution Path** for IDs to ensure maximum performance.

### 1.1 Dual Resolution Paths

Wendao's ID resolution supports two distinct physical paths depending on the deployment context:

1. **Distributed Path (Valkey)**: Resolves IDs across the network via [[docs/01_core/zhenfa/SPEC.md|Zhenfa Gateway]] for external or user-defined configurations.
2. **Embedded Path (Pure AST)**: Used by internal Rust crates to resolve built-in IDs from local `resources/` without database overhead. See [[docs/01_core/qianhuan/architecture/markdown-config-bridge.md|Markdown Configuration Bridge (Section 2.4)]].

## 2. The Unified Query Grammar

The Wendao engine intercepts specific directives within the query string. The most critical directive for system configuration and precise retrieval is the `id:` prefix.

- **Query**: `id:agenda_steward`
- **Logic**: Tells the engine to bypass vector similarity and PPR graph traversal.

## 3. The "Short-Circuit" Execution Pipeline

When a request arrives via the [[docs/01_core/zhenfa/SPEC.md|Zhenfa Gateway]], the Wendao engine follows this optimized internal path:

### Phase 1: Grammar Decomposition

The `QueryParser` (in Rust) identifies the presence of the `id:` directive. It extracts the literal value (e.g., `"agenda_steward"`).

### Phase 2: Short-Circuit Trigger

If an ID is present, the engine sets the `ExecutionStrategy` to `DirectLookup`.
_This prevents the system from spinning up the heavy-weight PPR (Personalized PageRank) algorithm or querying the Vector database, saving significant CPU cycles._

### Phase 3: Valkey Primary-Key Fetch

Wendao constructs the physical Valkey key using the system's namespace convention (e.g., `wendao:entity:id:{id}`).

- It performs a **single asynchronous GET** operation.
- Because Valkey is an in-memory store, this operation is completed in **< 100 microseconds**.

### Phase 4: Entity Materialization & Stripping

The raw bytes from Valkey are deserialized into a `WendaoEntity`.

- For configuration requests (like Persona/Template), the engine extracts the specific metadata field (e.g., the `jinja2` code block).
- It applies the [[docs/01_core/zhenfa/architecture/schema-contract.md|JSON Stripping Layer]] logic at the response boundary to return ONLY the raw content.

## 4. Why This Matters for the "Zero-Export" Bridge

By using this ID-resolution mechanism, the [[docs/01_core/qianhuan/architecture/markdown-config-bridge.md|Qianhuan-Wendao Markdown Configuration Bridge]] becomes practically instantaneous.

When Qianhuan asks for a template by ID:

1. It feels like a local memory read.
2. But it provides the benefits of a global, distributed configuration store (Valkey).
3. The LLM sees a simple, consistent interface for both "Finding information" and "Loading its own personality."

## 5. Implementation Status

- **Grammar Support**: ✅ Implemented in `xiuxian-wendao::link_graph::index::search::plan`.
- **Short-Circuit Path**: ✅ Implemented in `xiuxian-wendao::fusion`.
- **$O(1)$ Indexing**: ✅ Enforced in `MarkdownConfigMemoryIndex`.
