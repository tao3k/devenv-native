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

# Run tests
cargo test -p xiuxian-vector

# Build Wendao Python bindings (from project root)
uv sync --reinstall-package xiuxian-core-rs
```

## Crates

| Crate                  | Purpose                                                | Type    |
| ---------------------- | ------------------------------------------------------ | ------- |
| **Core Types**         |
| `xiuxian-types`        | Common type definitions, error types                   | Library |
| **Code Analysis**      |
| `xiuxian-ast`          | AST parsing and analysis                               | Library |
| `xiuxian-tags`         | Tag extraction and management                          | Library |
| **Editor & Tools**     |
| `xiuxian-edit`         | Code editing and batch operations (The Surgeon)        | Library |
| `xiuxian-tokenizer`    | BPE tokenization                                       | Library |
| **Storage & Vector**   |
| `xiuxian-vector`       | Vector store operations, tool indexing (The Librarian) | Library |
| `xiuxian-lance`        | LanceDB integration                                    | Library |
| **Security & I/O**     |
| `xiuxian-security`     | Security and sanitization (Hyper-Immune System)        | Library |
| `xiuxian-io`           | Safe file I/O operations, context assembly             | Library |
| **Skills & Discovery** |
| `skills-scanner`       | Skill discovery and metadata scanning                  | Library |
| **Bindings**           |
| `xiuxian-core-rs`      | Wendao Python bindings via PyO3                        | cdylib  |

## Directory Structure

```
packages/rust/
├── crates/
│   ├── xiuxian-ast/           # AST parsing
│   ├── xiuxian-edit/          # Code editing (The Surgeon)
│   ├── xiuxian-io/            # Safe I/O, context assembly
│   ├── xiuxian-lance/         # LanceDB integration
│   ├── xiuxian-security/      # Security (Hyper-Immune)
│   ├── xiuxian-tags/          # Tag extraction
│   ├── xiuxian-tokenizer/     # BPE tokenization
│   ├── xiuxian-types/         # Type definitions
│   ├── xiuxian-vector/        # Vector store (The Librarian)
│   └── skills-scanner/     # Skill discovery
└── bindings/
    └── python/             # PyO3 bindings (Wendao surface)
```

## Trinity Architecture

These crates power the **Trinity Architecture**:

- **The Librarian** (`xiuxian-vector`): Vector store for semantic memory
- **The Surgeon** (`xiuxian-edit`): AST-based code editing
- **Hyper-Immune System** (`xiuxian-security`): Security and sanitization

## Python Binding Usage

```python
from xiuxian_core_rs import get_schema

schema = get_schema("xiuxian_wendao.link_graph.record.v1")
print(schema[:80])
```
