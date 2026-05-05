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
- `duckdb`: enables real DuckDB connection opening, DuckLake catalog attach
  helpers, and Arrow record-batch append helpers for attached DuckLake tables.
- `qianji-bpmn-workflow-state`: enables the Qianji BPMN workflow-state
  DuckDB adapter, checkpoint-envelope storage helpers, append-only durable
  checkpoint events, and Arrow-appender batch snapshot storage for
  audit/replay paths.

## Ownership Boundary

Generic DuckDB and DuckLake storage primitives belong here. Wendao may still
own search-specific runtime resolution, Flight-facing behavior, event-lake
schemas, and query routing, but it should consume the generic connection,
catalog attach, and Arrow appender helper surface from this crate.

DuckLake support is intentionally embedded-first. This crate owns local
metadata-file and PostgreSQL-catalog attach configuration, extension bootstrap
SQL, local data-path preparation, typed remote data-path rendering, fully
qualified DuckLake table references, Arrow `RecordBatch` appends into existing
attached tables, reusable Arrow appender handles for high-throughput ingestion,
and DuckDB `httpfs` S3 secret SQL helpers. It does not own Wendao event names,
BPMN payload schemas, SwanLake session orchestration, credential discovery, S3
bucket provisioning, or live PostgreSQL service management.

DuckLake `DATA_PATH` values are typed as local paths or remote URIs. Runtime
attach prepares local directories only for local paths; remote values such as
`s3://bucket/prefix/` are rendered into SQL without filesystem side effects.

### External DuckLake Probe

The external DuckLake probe is ignored by default and must be invoked
explicitly after PostgreSQL and optional S3-compatible storage have already
been provisioned:

```bash
direnv exec . cargo test -p xiuxian-db-store --features duckdb ducklake_external -- --ignored --nocapture
```

Required environment variables:

- `XIUXIAN_DUCKLAKE_EXTERNAL_POSTGRES_DSN`: PostgreSQL connection string for
  DuckLake metadata.
- `XIUXIAN_DUCKLAKE_EXTERNAL_DATA_PATH`: local path or remote URI such as
  `s3://bucket/prefix/`.

Optional environment variables:

- `XIUXIAN_DUCKLAKE_EXTERNAL_ALIAS`: attached catalog alias, default
  `wendao_external_lake`.
- `XIUXIAN_DUCKLAKE_EXTERNAL_S3_SECRET_NAME`: enables DuckDB `httpfs` secret
  creation before attach.
- `XIUXIAN_DUCKLAKE_EXTERNAL_S3_KEY_ID` and
  `XIUXIAN_DUCKLAKE_EXTERNAL_S3_SECRET`: static S3 credentials. When absent,
  the probe uses DuckDB's credential-chain provider.
- `XIUXIAN_DUCKLAKE_EXTERNAL_S3_SESSION_TOKEN`,
  `XIUXIAN_DUCKLAKE_EXTERNAL_S3_CHAIN`,
  `XIUXIAN_DUCKLAKE_EXTERNAL_S3_REGION`,
  `XIUXIAN_DUCKLAKE_EXTERNAL_S3_ENDPOINT`,
  `XIUXIAN_DUCKLAKE_EXTERNAL_S3_URL_STYLE`,
  `XIUXIAN_DUCKLAKE_EXTERNAL_S3_SCOPE`, and
  `XIUXIAN_DUCKLAKE_EXTERNAL_S3_USE_SSL`: optional S3 secret fields.

When the required variables are missing, the ignored probe skips itself
cleanly. It does not provision PostgreSQL, create buckets, discover
credentials, or start SwanLake.

### DuckLake Harness Profile

The db-store Rust harness profile binds `src/duckdb/ducklake/mod.rs` to the
regression verification skill. That profile covers the embedded DuckLake chain:
attach SQL and extension bootstrap, catalog and data-path typing, S3 secret SQL
helpers, Arrow appender behavior, the local live smoke, and the env-gated
external probe.

The primary profile checks are:

```bash
direnv exec . cargo test -p xiuxian-db-store --features duckdb db_store_verification_profile_hints_bind_active_skill_tasks -- --nocapture
direnv exec . cargo test -p xiuxian-db-store --features duckdb -- --nocapture
direnv exec . cargo bench -p xiuxian-db-store --features duckdb --bench db_store_performance db_store_ducklake_arrow_appender
```

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
