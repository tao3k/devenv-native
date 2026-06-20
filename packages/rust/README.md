---
type: knowledge
metadata:
  title: "Rust Crates for xiuxian-artisan-workshop"
---

# Rust Crates for xiuxian-artisan-workshop

> Rust workspace managed from the project-root `Cargo.toml`.

This directory contains the Rust crates for `xiuxian-artisan-workshop`.
Crate membership, shared dependency versions, and build profiles are owned by
the repository root `Cargo.toml`.

## Quick Start

```bash
# Build all crates from the project root.
cargo build

# Run a focused package test.
cargo test -p xiuxian-wendao --lib

# Build Wendao Python bindings from the project root.
uv sync --reinstall-package xiuxian-core-rs
```

## Boundary Notes

- `xiuxian-config-core` owns project-local configuration and `PRJ_*` directory
  resolution.
- `xiuxian-db-store` owns reusable storage primitives such as Arrow codecs,
  DuckDB, and DuckLake support.
- `xiuxian-git-repo` owns managed repository materialization and sync paths.
- Wendao crates own knowledge graph, parser, runtime, query, and Studio
  behavior.
- Qianji crates own BPMN/control-plane/client/runtime behavior.
- Julia and polyglot crates own Julia runtime contracts and cross-language
  orchestration.

## Crates

| Crate                           | Purpose                                                           |
| ------------------------------- | ----------------------------------------------------------------- |
| `xiuxian-config-core`           | Cascading config, env lookup, and project directory resolution    |
| `xiuxian-db-store`              | Arrow/DuckDB/DuckLake storage primitives                          |
| `xiuxian-event`                 | Event contracts                                                   |
| `xiuxian-git-repo`              | Managed repository checkout, mirror, and sync substrate           |
| `xiuxian-graph-core`            | Shared graph projection and optional rendering/algorithm adapters |
| `xiuxian-julia-core`            | Julia-facing core contracts                                       |
| `xiuxian-julia-runtime`         | Julia runtime integration                                         |
| `xiuxian-lance`                 | Lance integration boundary                                        |
| `xiuxian-logging`               | Logging setup                                                     |
| `xiuxian-macros`                | Project macros                                                    |
| `xiuxian-memory-engine`         | Memory engine algorithms and persistence boundary                 |
| `xiuxian-polyglot-orchestrator` | Cross-language scheduling and contract bridge                     |
| `xiuxian-qianji`                | Qianji aggregate crate                                            |
| `xiuxian-qianji-bpmn-engine`    | BPMN/DMN engine                                                   |
| `xiuxian-qianji-client`         | Qianji client surface                                             |
| `xiuxian-qianji-control`        | Qianji control-plane contracts                                    |
| `xiuxian-qianji-runtime`        | Qianji runtime integration                                        |
| `xiuxian-security`              | Security and sanitization utilities                               |
| `xiuxian-types`                 | Common type definitions                                           |
| `xiuxian-vector`                | Retiring vector-storage compatibility shell                       |
| `xiuxian-wendao`                | Wendao knowledge graph, query/search, and DocOS contracts         |
| `xiuxian-wendao-attachments`    | Attachment extraction and artifact boundaries                     |
| `xiuxian-wendao-builtin`        | Built-in Wendao plugin set                                        |
| `xiuxian-wendao-client`         | Wendao CLI/client surfaces                                        |
| `xiuxian-wendao-core`           | Wendao shared core contracts                                      |
| `xiuxian-wendao-episteme`       | Episteme-specific Wendao surfaces                                 |
| `xiuxian-wendao-parsers`        | Wendao document and skill parser substrate                        |
| `xiuxian-wendao-runtime`        | Wendao runtime and transport support                              |
| `xiuxian-wendao-server`         | Wendao server transport adapters                                  |
| `xiuxian-wendao-sql`            | Wendao SQL support                                                |
| `xiuxian-wendao-studio`         | Studio HTTP and gateway adapter boundary                          |
| `xiuxian-window`                | Window/runtime utility boundary                                   |
| `xiuxian-zhenfa`                | Zhenfa route and native adapter boundary                          |
| `xiuxian-zhixing`               | Zhixing integration boundary                                      |
| `xiuxian-core-rs`               | Wendao Python bindings via PyO3                                   |

## Directory Structure

```text
packages/rust/
├── bindings/
│   └── python/                  # PyO3 bindings
└── crates/
    ├── xiuxian-config-core/     # Config and PRJ_* directory resolution
    ├── xiuxian-db-store/        # Storage primitives
    ├── xiuxian-git-repo/        # Managed repository substrate
    ├── xiuxian-polyglot-orchestrator/
    ├── xiuxian-qianji*/
    ├── xiuxian-wendao*/
    └── xiuxian-*/
```

## Python Binding Usage

```python
from xiuxian_core_rs import get_schema

schema = get_schema("xiuxian_wendao.link_graph.record.v1")
print(schema[:80])
```
