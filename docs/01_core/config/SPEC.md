---
type: knowledge
title: "Xiuxian-Config: The Universal Configuration Engine"
category: "core"
tags:
  - config
  - macro
  - ssot
  - performance
metadata:
  title: "Xiuxian-Config: The Universal Configuration Engine"
---

# Xiuxian-Config: The Universal Configuration Engine

`xiuxian-config` is the centralized state orchestration layer for the Xiuxian OS ecosystem. It enforces a **Single Source of Truth (SSoT)** and provides high-performance, macro-driven configuration loading.

## 1. Core Paradigm: #[xiuxian_config]

All domain configurations are defined as Rust structs decorated with the `#[xiuxian_config]` attribute macro (provided by `xiuxian-macros`).

Canonical declaration pattern:

```rust
#[xiuxian_macros::xiuxian_config(
    namespace = "skills",
    internal_path = "resources/config/skills.toml",
    orphan_file = "",
    array_merge = "append"
)]
```

- `namespace`: table projected from user `xiuxian.toml` (supports dotted projection like `skills.validation`).
- `internal_path`: explicit crate-local default TOML path (required by project convention).
- `orphan_file`: set to `""` when orphan compatibility is intentionally disabled.

### 1.1 Cascading Loading Logic

The engine resolves configuration through a strictly ordered 3-layer stack:

1.  **Level 0: Embedded Base**: Default values are embedded into the binary at compile-time via `include_str!` from the crate's `resources/config/` directory (e.g., `packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml`).
2.  **Level 1: Unified Override**: User-defined settings are loaded from the hierarchical path: `$PRJ_CONFIG_HOME/xiuxian-artisan-workshop/xiuxian.toml`.
    - In local development, `$PRJ_CONFIG_HOME` typically defaults to the project-local `.config/` directory.
3.  **Level 2: Environment**: Runtime overrides via environment variables (e.g., `XIUXIAN_SKILLS__VALIDATION__STRICT_MODE`).

### 1.2 User Consolidation Rule

User overrides must live only in:

`$PRJ_CONFIG_HOME/xiuxian-artisan-workshop/xiuxian.toml`

If `xiuxian.toml` exists, legacy orphan files (for example `skills.toml`) are treated as invalid redundant sources and rejected.

## 2. Performance: High-Speed Read-Through Cache

To support high-frequency configuration access without IO bottlenecks:

- **Zero-IO Reads**: A thread-safe memory cache (utilizing `OnceLock` or `RwLock`) ensures that subsequent calls to `Config::load()` return in microseconds.
- **Atomic Reliability**: Synchronized access across hundreds of concurrent agent co-routines.

## 3. Governance: Conflict Enforcement

The engine acts as a "Legal Enforcement" layer to prevent config drift:

- **Orphan File Rejection**: If a global `xiuxian.toml` is present, any standalone legacy files (e.g., `skills.toml`) are strictly rejected with a fatal error.
- **Namespace Isolation**: Each crate resides in its own TOML table (e.g., `[skills]`, `[wendao]`), preventing key collisions.

## 4. Integration Status

- **xiuxian-wendao-parsers**: Owns Markdown and SKILL.md frontmatter parsing
  contracts.
- **xiuxian-wendao**: Owns runtime skill VFS and internal alias schema
  resources.
- **xiuxian-daochang**: Consumes Wendao-owned native alias specs during
  bootstrap.
