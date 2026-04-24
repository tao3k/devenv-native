# Qianji DuckDB Workflow Data Store

## Boundary

Qianji uses Valkey for BPMN checkpoint state and writer ownership. DuckDB is a
local workflow data-plane store, not a checkpoint backend and not a distributed
lease owner.

The first DuckDB workflow data-store surface is intentionally small:

- `QianjiBpmnDuckDbDataStoreConfig` resolves local DuckDB runtime settings.
- `QianjiBpmnDuckDbDataStore` opens the store through `xiuxian-db-store`.
- `QianjiBpmnDataRecord` stores one JSON-safe payload keyed by workflow
  `instance_id` and caller-owned `record_key`.

## Ownership

Generic DuckDB connection and SQL setup belong to `xiuxian-db-store`. Qianji
owns only the workflow data contract and the BPMN-specific table shape. Wendao
search modules are not part of this dependency path.

## Intended Uses

The data store is suitable for bounded local workflow records such as host-work
outputs, DMN outcomes, dataset references, or adapter scratch payloads that
should not be persisted as distributed checkpoint state.

The store does not replace Valkey checkpoints. A waiting or resumable BPMN
instance still needs Valkey-backed checkpoint state when running through the
service/control-plane path.
