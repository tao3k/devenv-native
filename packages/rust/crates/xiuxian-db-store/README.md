# xiuxian-db-store

`xiuxian-db-store` is the shared storage boundary for database-backed local
helpers that should not be owned by Wendao search internals or Qianji workflow
runtime code.

## Feature Surfaces

- `engine`: exposes Arrow/DataFusion engine record batches, IPC helpers,
  retrieval result schemas, and Parquet write helpers without compiling
  `xiuxian-vector` or LanceDB.
- `vector-store`: re-exports the Lance/vector storage surface from
  `xiuxian-vector` for explicit vector-store consumers.
- `duckdb-types`: exposes generic DuckDB runtime config and SQL helper types
  without compiling the DuckDB runtime.
- `duckdb`: enables real DuckDB connection opening and initialization.
- `qianji-bpmn-workflow-state`: enables the Qianji BPMN workflow-state
  DuckDB adapter, checkpoint-envelope storage helpers, append-only durable
  checkpoint events, and Arrow-appender batch snapshot storage for
  audit/replay paths.

## Ownership Boundary

Generic DuckDB storage primitives belong here. Wendao may still own
search-specific runtime resolution, Flight-facing behavior, and query routing,
but it should consume the generic connection and SQL helper surface from this
crate.

Arrow and DataFusion query surfaces also belong here through the lightweight
`engine` feature. Wendao default builds should use that surface for SQL,
Flight, and document parsing contracts. LanceDB and `xiuxian-vector` remain
opt-in through `vector-store` and should be used only for explicit vector or
retrieval-storage paths.

Qianji BPMN workflow-state storage also belongs here behind the
`qianji-bpmn-workflow-state` feature. `xiuxian-qianji` should re-export and
compose that adapter rather than owning DuckDB table layout, JSON checkpoint
encoding, or local workflow-state persistence directly.

The Qianji adapter intentionally separates single-row checkpoint durability
from batch snapshot ingestion. The local runtime facade uses append-only
durable checkpoint events plus its own same-process latest cache for hot
save/load loops. Cold recovery can rebuild a compacted latest-checkpoint table
from the append log and hydrate the same-process cache in one batch. Batch
replay and audit flows use DuckDB's Arrow appender.
