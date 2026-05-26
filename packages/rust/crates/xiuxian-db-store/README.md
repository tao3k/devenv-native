# xiuxian-db-store

`xiuxian-db-store` is the shared storage boundary for database-backed local
helpers that should not be owned by Wendao search internals or Qianji workflow
runtime code.

## Feature Surfaces

- `engine`: exposes Arrow/DataFusion engine record batches, IPC helpers,
  retrieval result schemas generated through the shared Arrow table-contract
  surface, and Parquet write helpers without compiling `xiuxian-vector` or
  LanceDB.
- `arrow-codec`: exposes shared Arrow IPC encode/decode helpers plus reusable
  Arrow table-schema contracts for consumers that need deterministic
  `RecordBatch` and IPC payload validation.
- `artifact-cache`: exposes attachment, document extraction, ontology, and
  agent artifact cache contracts plus the content-addressed filesystem
  baseline. This feature does not enable Foyer, Moka, DuckDB, or Valkey by
  itself.
- `foyer-artifact-cache`: enables the optional Foyer implementation behind
  `ArtifactBlobCache`. This feature admits Foyer as the feature-gated
  mainline artifact byte-cache backend for repeated agent/model artifact reads.
- `vector-store`: re-exports the Lance/vector storage surface from
  `xiuxian-vector` for explicit vector-store consumers.
- `duckdb-types`: exposes generic DuckDB runtime config and SQL helper types
  without compiling the DuckDB runtime.
- `duckdb`: enables real DuckDB connection opening, DuckLake catalog attach
  helpers, and Arrow record-batch append helpers for attached DuckLake tables.
  The Wendao client Org agent read-model command consumes this feature as its
  default local materialization backend; runtime path changes are configured by
  the client through `wendao.toml`, not through a DuckDB selector flag.
- `qianji-bpmn-workflow-state`: enables the Qianji BPMN workflow-state
  DuckDB adapter, checkpoint-envelope storage helpers, append-only durable
  checkpoint events, and Arrow-appender batch snapshot storage for
  audit/replay paths.

## Ownership Boundary

Generic DuckDB and DuckLake storage primitives belong here. Wendao may still
own search-specific runtime resolution, Flight-facing behavior, event-lake
schemas, and query routing, but it should consume the generic connection,
catalog attach, and Arrow appender helper surface from this crate.

Reusable Arrow schema mechanics also belong here. Domain crates still own their
table names, column names, and semantic contracts, but db-store provides the
shared `ArrowSchemaContract` surface for constructing Arrow schemas and
validating `RecordBatch` or Arrow IPC payloads against required columns,
logical data types, exact-order contracts, nullability validation policies, and
`wendao.table` metadata. The logical vocabulary covers the scalar and list
shapes needed by graph-structural request contracts, including `Int32` and
`List(Utf8)`, while route-specific semantics remain in the owning bridge
crate. The lightweight engine retrieval payloads also use this surface
internally so projected result schemas and canonical result schemas share the
same field vocabulary.

Attachment, document extraction, ontology, and agent evidence artifact cache
contracts also belong here behind `artifact-cache`. The contract is
intentionally split from the truth catalog: DuckDB remains responsible for
manifests, indexes, lineage, precision status, ontology truth, and read-model
projection; Arrow IPC and Parquet remain the structured payload and interchange
formats; Valkey remains the cross-process lease and in-flight coordination
surface. The cache stores large derived payload bytes only, such as audio
shards, PDF rasters/crops, OCR/VLM atlases, Arrow IPC batch bytes, ontology
review packets, ontology read-model payloads, parser projections, and
prompt/evidence packs.

`ArtifactBlobCache` is the only consumer interface for those bytes. The
filesystem backend is the contract baseline and stores artifact bytes in a
content-addressed layout keyed by namespace, artifact kind, source digest,
profile digest, and shard digest. The Foyer backend is the mainline
memory-plus-disk backend behind `foyer-artifact-cache`; it stays behind the
same trait so route code never owns backend selection directly. Moka is
intentionally out of scope for this artifact substrate because the active
bottleneck is repeated large artifact reuse, not process-local metadata lookup.

Agent and model loops should use the first-class artifact kinds
`agent-evidence-pack`, `org-projection`, `json-projection`,
`tabular-projection`, and `prompt-context-pack` for generated context that may
be reread across prompts, workflow steps, or process restarts. Use
`agent_artifact_key` to keep the namespace stable and
`read_through_artifact_bytes` to centralize miss/build/write behavior instead
of creating route-local read-through cache logic. Read-through calls return an
artifact receipt with the backend name, artifact key, hit/miss/throttled
status, byte count, optional write outcome, and read/build/write timings so
Gateway reports can distinguish cache reuse, backend pressure, and materializer
cost.
When a builder can be moved into a backend-owned execution path, use
`fetch_through_artifact_bytes` instead. The Foyer backend maps that helper to
Foyer's hybrid `get_or_fetch` path so concurrent same-key misses are coalesced
inside the artifact substrate instead of rebuilding the same agent evidence
pack, prompt context, projection, or shard bytes in every worker.

Attachment and ontology routes should use the dedicated key helpers instead of
custom namespaces. Use `attachment_artifact_key` for source payloads, audio
chunks, PDF rasters, OCR crops, VLM atlases, and Arrow IPC batches. Use
`ontology_artifact_key` for registry snapshots, candidate packets,
candidate read models, RDF drafts, promotion review packets, and reasoning
projections. These helpers only name artifact bytes; they do not move source
truth, ontology approval, or structured manifests out of the owning crates.

Runtime selection is data/config driven:

- `WENDAO_ARTIFACT_CACHE_BACKEND`: `filesystem` or `foyer`; when the
  `foyer-artifact-cache` feature is enabled, the default is `foyer`.
  Contract-only builds without Foyer default to `filesystem`.
- `WENDAO_ARTIFACT_CACHE_ROOT`: artifact root, default
  `$PRJ_CACHE_HOME/wendao/artifacts` when the project cache root is available.
- `WENDAO_ARTIFACT_CACHE_MEMORY_BYTES`: Foyer memory-tier capacity in bytes.
- `WENDAO_ARTIFACT_CACHE_STORAGE_BYTES`: Foyer disk-tier capacity in bytes.
- `WENDAO_ARTIFACT_CACHE_RUNTIME_WORKERS`: Foyer runtime workers, either
  `auto` or a positive integer. The default is `auto`, which uses the system
  parallelism available to the Gateway process.
- `WENDAO_ARTIFACT_CACHE_MEMORY_SHARDS`: Foyer memory shard count, either
  `auto` or a positive integer. The default is system parallelism.
- `WENDAO_ARTIFACT_CACHE_BLOCK_SIZE_BYTES`: Foyer block-engine block size in
  bytes. The default follows the artifact backend block-size contract.
- `WENDAO_ARTIFACT_CACHE_RECOVER_CONCURRENCY`: Foyer disk recovery concurrency,
  either `auto` or a positive integer. The default is system parallelism.
- `WENDAO_ARTIFACT_CACHE_FLUSHERS`: Foyer disk flusher count, either `auto` or
  a positive integer. The default derives from system parallelism for I/O lanes.
- `WENDAO_ARTIFACT_CACHE_RECLAIMERS`: Foyer disk reclaimer count, either `auto`
  or a positive integer. The default derives from system parallelism for I/O
  lanes.

`auto` is machine-adaptive, not a fixed package constant. Runtime workers,
memory shards, and recovery concurrency start from the system parallelism
available to the Gateway process. Disk flusher and reclaimer lanes derive from
that same signal, then the resolver constrains the effective lane count by the
configured block geometry so small test or edge deployments do not ask Foyer
to run more disk lanes than the backing store can make progress on.

The `foyer-artifact-cache` feature closes the synchronous wrapper lifecycle
gate for roundtrip, replace, remove, and close/reopen persistence. Callers still
consume only `ArtifactBlobCache`; Studio, attachments, and future agent parse
artifact loops must not construct route-local cache backends directly.
Gateway and route startup should rely on the shared backend resolver so Foyer
capacity and runtime concurrency are selected once by deployment configuration
instead of being hardcoded in individual routes.
The Foyer memory tier is byte-weighted by artifact key plus payload length, so
`WENDAO_ARTIFACT_CACHE_MEMORY_BYTES` is a byte-capacity control rather than an
entry-count control. The default hybrid policy is `write-on-insertion` to keep
agent artifacts restart-reusable; DuckDB/Arrow remain the truth and query plane.
Foyer disk throttling is surfaced as a distinct read-through pressure status
instead of being collapsed into an ordinary miss. Pressure reads rebuild bytes
for the caller without writing back into the cache, preserving correctness
while avoiding extra disk pressure.

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
replay and audit flows use DuckDB's Arrow appender, and the append-log
`RecordBatch` schema is generated and validated through the shared
`ArrowSchemaContract` surface before ingestion.
