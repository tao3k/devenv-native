# Qianji DuckDB Workflow Data Store

## Boundary

Qianji uses Valkey for `qianji-server` HTTP workflow control, distributed
checkpoint state, and writer ownership. DuckDB is the local no-server
workflow-state store for embedded or CLI BPMN runs when the `duckdb` feature is
enabled. It does not own distributed leases.

The first DuckDB workflow data-store surface is intentionally small:

- `QianjiBpmnDuckDbDataStoreConfig` resolves local DuckDB runtime settings.
- `QianjiBpmnDuckDbDataStore` opens the store through `xiuxian-db-store`.
- `QianjiBpmnDataRecord` stores one JSON-safe payload keyed by workflow
  `instance_id` and caller-owned `record_key`.
- `QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY` reserves the latest local
  workflow-state snapshot record.
- `QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb` lets local control
  surfaces reuse the configured DuckDB path without starting `qianji-server`.
- Append-log checkpoint snapshot helpers use DuckDB's Arrow appender path for
  batch/audit/replay workloads; the hot local checkpoint facade uses a durable
  append-only event log plus a same-process latest checkpoint cache.

## Ownership

Generic DuckDB connection and SQL setup belong to `xiuxian-db-store`. Qianji
owns only the workflow data contract and the BPMN-specific table shape. Wendao
search modules are not part of this dependency path.

The `xiuxian-qianji` crate keeps this boundary explicit with a default
`qianji-full` feature for the historical application, Flowhub, AST, Wendao,
Qianhuan, Zhenfa, and CLI-heavy surfaces. A narrow local BPMN store build can
use `--no-default-features --features duckdb`; that path exposes BPMN workflow
control plus runtime config and telemetry, while the full application modules
stay feature-gated.

## Intended Uses

The data store is suitable for bounded local workflow records such as host-work
outputs, DMN outcomes, dataset references, or adapter scratch payloads that
should stay local to one embedded BPMN run.

The store also persists the latest local workflow-state snapshot for no-server
resume/status/cancel flows. That snapshot uses the same checkpoint envelope
shape as Valkey so local and server paths can share resume semantics, but it is
not a distributed checkpoint truth source and it does not participate in Valkey
lease ownership.

`qianji-server` and HTTP workflow requests continue to default to Valkey. Local
CLI/control requests with no explicit checkpoint backend default to the
configured DuckDB workflow-state path when the `duckdb` feature is enabled.

## Performance Notes

The store adapter exposes three distinct checkpoint write shapes. The hot
runtime facade appends each checkpoint to the durable event log and keeps a
same-process latest checkpoint cache for the active execution loop. The
batch/audit path appends checkpoint snapshots with DuckDB's Arrow appender. The
row upsert path remains as a compatibility and bounded-record path, but is not
the preferred checkpoint hot path. Cold recovery uses a compacted latest table
rebuilt from the append log, then hydrates the same-process latest cache in one
batch.

Focused local probe evidence for the store-owned path:

- Checkpoint codec: 1,000 medium JSON checkpoint payloads encoded in 24.3 ms
  and decoded in 45.3 ms, averaging 715 bytes per payload after empty runtime
  collections are omitted.
- Arrow append-log batch path: 1,000 medium JSON checkpoint payloads appended
  in 97.7 ms and another 1,000 updated snapshots appended in 94.9 ms. Cold
  latest-event reads completed in 5,247 ms.
- Compacted latest path: compacting 2,000 append-log events into 1,000 latest
  checkpoints took 28.9 ms; hydrating all compacted checkpoints took 28.1 ms;
  repeated point reads from the compacted table completed in 1,381 ms.
- Point append event path: 1,000 medium JSON checkpoint payloads appended in
  2,114 ms, another 1,000 updated snapshots appended in 1,714 ms, and cold
  latest-event reads completed in 4,975 ms.
- Reused store with transaction-backed row upserts: 1,000 medium JSON
  checkpoint payloads saved in 6,251 ms, loaded in 1,436 ms, overwritten in
  5,913 ms, and deleted in 1,381 ms.
- Open-per-operation anti-pattern: 128 checkpoint payloads took 8,617 ms to
  save and 6,873 ms to load.
- Qianji cached checkpoint facade hot path: 64 durable checkpoint appends took
  210 ms and same-process latest-cache loads took 0.112 ms.
- Qianji reopened checkpoint facade path: 64 cold loads from a reopened DuckDB
  store, including first-load compaction and cache hydration, took 103 ms.

These probes measure the Qianji/DuckDB integration path, not DuckDB's raw
database benchmark. The open-per-operation probe remains as regression
evidence for why Qianji must reuse local stores instead of reopening DuckDB for
each checkpoint operation. The Arrow append-log probe remains as regression
evidence that batch/audit/replay storage must not be implemented as repeated
row upserts. The cold latest-event read numbers are intentionally tracked as a
resume-path baseline; cold recovery should compact once and hydrate the
same-process latest cache instead of issuing repeated event-log latest scans.
