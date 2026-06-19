---
type: knowledge
title: "ADR-007: Explicit Link-Driven Resource Bus"
status: "Accepted"
date: "2026-02-28"
category: "architecture"
tags:
  - zhenfa
  - wendao
  - graph
  - declarative
  - resources
metadata:
  title: "ADR-007: Explicit Link-Driven Resource Bus"
---

# ADR-007: Explicit Link-Driven Resource Bus

## 1. Context and Problem Statement

To maintain architectural integrity, we must decouple physical storage from logical semantic meaning. Relying on file extensions to "guess" an asset's role leads to ambiguity and "magic" behavior that is hard to debug.

## 2. Decision: The Wendao Semantic Bus

We will transition to a **Graph-Driven Declarative Resource Bus** using a dual-layered mapping strategy.

### 2.1 The wendao:// Protocol

All resources are addressed via a semantic URI that MUST include the file extension.
`wendao://skills/<semantic_name>/references/<file_path>.<ext>`

### 2.2 Internal URI Mapping & Semantic Labels

To maintain compatibility with standard Markdown editors, **source documents MUST use relative WikiLinks** with mandatory semantic type-hints (#tags).

**Authoring Standard (in SKILL.md):**

- **Personas**: `[[references/steward.md#persona]]` -> Relation: `[:MANIFESTS]`
- **Templates**: `[[references/draft.j2#template]]` -> Relation: `[:RENDERS]`
- **Workflows**: `[[references/flow.toml#qianji-flow]]` -> Relation: `[:GOVERNS]`

**Internal Translation:**
The `Wendao` indexer automatically maps these relative paths to their canonical `wendao://` URIs during graph construction.

### 2.3 Tiered Semantic Discovery & Skill Promotion

1. **Layer 1 (Universal)**: Wendao processes all MD files within the mounted runtime resource roots for links and attachments.
2. **Layer 2 (Promotion)**: Encountering `SKILL.md` triggers the parser-owned
   SKILL.md contract for semantic lifting into Wendao runtime inventory.

### 2.4 Functional Naming Policy (The Artisan Standard)

To ensure that the Knowledge Graph remains self-explanatory, we enforce a strict naming policy.

- Prohibited: `agent`, `core`, `misc`.
- Preferred: `brain-kernel`, `agenda-guard`.

### 2.5 Dynamic Discovery & URI Expansion

The system supports resolving resources based on runtime search queries.

1.  **Query**: A node executes a search (e.g., `carryover:>=1`).
2.  **URI Generation**: The engine returns a list of Canonical URIs.
3.  **Expansion**: The executor resolves these URIs and aggregates them into a validated XML-Lite block.

## 3. Technical Design: The Native Runtime Implementation

### 3.1 Binary Embedding via `include_dir!`

Every business crate will bake its `resources/` directory into the binary.

```rust
pub static RESOURCES: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources");
```

### 3.2 Holographic Mounting Protocol

The Zhenfa runtime boundary provides a unified **`SkillRuntimeResolver`** that acts as a mount manager. Physical filesystem calls are prohibited for embedded resources.

### 3.3 Zero-Copy Access & String Interning

The resolver returns `Arc<str>` and uses an interning cache to minimize heap allocations.

## 4. Consequences

### Positive

- **Zero Boilerplate**: Drop a file, link it, and it's live.
- **Deterministic**: Knowledge graph perfectly reflects intent.
- **Portable**: Nix-native.
- **High Efficiency**: Zero-copy access.

## 5. Implementation Plan

1. **Refactor Crates**: Export `pub static RESOURCES`.
2. **Upgrade Wendao**: Implement `Dir`-based mounting and #tag relation extraction.
3. **Migrate Assets**: Use relative WikiLinks with full extensions.
