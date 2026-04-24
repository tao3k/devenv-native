---
type: knowledge
metadata:
  title: "Rust Crates for xiuxian-artisan-workshop"
---

# Rust Crates for xiuxian-artisan-workshop

> Rust Workspace - Managed from project root `Cargo.toml`

This directory contains Rust crates for the `xiuxian-artisan-workshop` repository. The workspace is managed from the **project root** (`xiuxian-artisan-workshop/Cargo.toml`).

## Quick Start

```bash
# Build all crates from project root
cd xiuxian-artisan-workshop
cargo build

# Run a focused package test
cargo test -p xiuxian-wendao --lib

# Build Wendao Python bindings (from project root)
uv sync --reinstall-package xiuxian-core-rs
```

## Crates

| Crate                    | Purpose                                                                  | Type    |
| ------------------------ | ------------------------------------------------------------------------ | ------- |
| **Core Types**           |
| `xiuxian-types`          | Common type definitions, error types                                     | Library |
| **Code Analysis**        |
| `xiuxian-ast`            | AST parsing and analysis                                                 | Library |
| `xiuxian-tags`           | Tag extraction and management                                            | Library |
| **Editor & Tools**       |
| `xiuxian-edit`           | Code editing and batch operations (The Surgeon)                          | Library |
| `xiuxian-tokenizer`      | BPE tokenization                                                         | Library |
| **Storage & Data**       |
| `xiuxian-db-store`       | Storage compatibility facade for Lance/Arrow-backed data surfaces        | Library |
| `xiuxian-vector`         | Retiring Lance/Arrow storage shell; not a search or skill owner          | Library |
| `xiuxian-lance`          | LanceDB integration                                                      | Library |
| **Wendao**               |
| `xiuxian-wendao`         | Knowledge graph, DuckDB-backed query/search, and DocOS runtime contracts | Library |
| `xiuxian-wendao-parsers` | Markdown, frontmatter, link, and skill document parser substrate         | Library |
| `xiuxian-wendao-client`  | Client-side Wendao CLI surfaces such as linting                          | Library |
| **Security & I/O**       |
| `xiuxian-security`       | Security and sanitization (Hyper-Immune System)                          | Library |
| `xiuxian-io`             | Safe file I/O operations, context assembly                               | Library |
| **Bindings**             |
| `xiuxian-core-rs`        | Wendao Python bindings via PyO3                                          | cdylib  |

## Directory Structure

```
packages/rust/
├── crates/
│   ├── xiuxian-ast/           # AST parsing
│   ├── xiuxian-edit/          # Code editing (The Surgeon)
│   ├── xiuxian-io/            # Safe I/O, context assembly
│   ├── xiuxian-db-store/      # storage compatibility facade
│   ├── xiuxian-lance/         # LanceDB integration
│   ├── xiuxian-security/      # Security (Hyper-Immune)
│   ├── xiuxian-tags/          # Tag extraction
│   ├── xiuxian-tokenizer/     # BPE tokenization
│   ├── xiuxian-types/         # Type definitions
│   ├── xiuxian-vector/        # retiring Lance/Arrow storage shell
│   ├── xiuxian-wendao/        # Wendao knowledge graph and query/search owner
│   ├── xiuxian-wendao-client/ # Wendao CLI/client surfaces
│   └── xiuxian-wendao-parsers/# Wendao parser substrate
└── bindings/
    └── python/             # PyO3 bindings (Wendao surface)
```

## Trinity Architecture

These crates power the current Wendao-centered kernel boundaries:

- **Wendao query/search owner** (`xiuxian-wendao`): Knowledge graph, DuckDB
  query execution, and DocOS search contracts
- **Storage shell** (`xiuxian-db-store`, retiring `xiuxian-vector`): Lance/Arrow
  storage compatibility while search semantics move out of the vector crate
- **The Surgeon** (`xiuxian-edit`): AST-based code editing
- **Hyper-Immune System** (`xiuxian-security`): Security and sanitization

## Python Binding Usage

```python
from xiuxian_core_rs import get_schema

schema = get_schema("xiuxian_wendao.link_graph.record.v1")
print(schema[:80])
```
