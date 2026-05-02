---
type: knowledge
metadata:
  title: "Omni Vector"
---

# Omni Vector

> Lance-backed storage and query placeholder for the Xiuxian workspace.

## Overview

Omni Vector is the Lance-backed vector-table storage shell for the Xiuxian
workspace. It keeps Arrow/Lance ownership, generic vector-table mutation, and
low-level storage query primitives behind higher-level owners such as
`xiuxian-db-store` and Wendao. It does not own `SKILL.md` discovery, tool
routing, agentic orchestration contracts, or DuckDB search semantics.

## Features

- Disk-based Lance storage (no server required)
- Low-level Lance vector similarity lookup via `search_optimized`
- CRUD + merge-insert (upsert) operations
- Versioning / snapshot (time travel) APIs
- Schema evolution helpers
- Generic Arrow IPC codec and Lance-facing Arrow re-exports

## Usage

```rust
use xiuxian_vector::{SearchOptions, VectorStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = VectorStore::new("./vectors", Some(3)).await?;

    store
        .add_documents(
            "documents",
            vec!["doc1".to_string()],
            vec![vec![0.1, 0.2, 0.3]],
            vec!["example document".to_string()],
            vec![serde_json::json!({"source":"docs/readme.md"}).to_string()],
        )
        .await?;

    let results = store
        .search_optimized(
            "documents",
            vec![0.1, 0.2, 0.3],
            5,
            SearchOptions {
                where_filter: Some(r#"{"source":"docs/readme.md"}"#.to_string()),
                ..SearchOptions::default()
            },
        )
        .await?;

    println!("results={}", results.len());

    Ok(())
}
```

## Architecture

```
xiuxian-vector/
├── src/lib.rs                # Main exports / module wiring
├── src/store.rs              # VectorStore state and method-family includes
├── src/arrow_codec.rs        # Generic Arrow IPC codec + metadata helpers
├── src/ops/                  # Core CRUD + admin + writer operations
├── src/search/               # generic vector search helpers
└── tests/                    # snapshots + data-layer + perf guard
```

## Project Harness Boundary

`xiuxian-vector` uses `rust-lang-project-harness` for project-policy gates.
The source and test gate roots run without disabled rules. The current layout
keeps `lib.rs` as the public facade, `store.rs` as the `VectorStore` owner, and
search/admin/writer implementation files under explicit feature owners so the
parser-to-reasoning-tree harness can report low-noise module facts.

## Out Of Scope

`xiuxian-vector` does not own:

- `SKILL.md` or frontmatter scanning
- tool catalog indexing
- tool routing / agentic search policy
- workspace-specific skill manifests
- keyword / FTS / hybrid search semantics

## Arrow Ownership Boundary

`xiuxian-vector` no longer exposes an Arrow-over-HTTP transport client. The
crate keeps only generic Arrow batch helpers on the public surface:

- `encode_record_batch_ipc` / `encode_record_batches_ipc`
- `decode_record_batches_ipc`
- `attach_record_batch_metadata`
- `attach_record_batch_trace_id`

`xiuxian-vector` intentionally has two Arrow surfaces:

- Lance-facing storage, mutation, and repo-hydration paths must use Lance's Arrow-57 types re-exported from `lance::deps`.
- residual DataFusion and other workspace Arrow-native compute paths continue
  to use the workspace Arrow surface.

Do not pass workspace Arrow arrays into `LanceRecordBatch` construction or downcast Lance batches using workspace Arrow collection types. Use the Lance-prefixed re-exports from `xiuxian-vector` for any code that touches Lance-owned schemas or arrays.

## See Also

- [Wendao vector boundary split](../xiuxian-wendao/docs/03_features/215_vector_boundary_split.md)

## License

Apache-2.0
