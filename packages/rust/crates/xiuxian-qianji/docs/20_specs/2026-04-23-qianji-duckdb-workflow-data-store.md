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

## Ownership

Generic DuckDB connection and SQL setup belong to `xiuxian-db-store`. Qianji
owns only the workflow data contract and the BPMN-specific table shape. Wendao
search modules are not part of this dependency path.

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
