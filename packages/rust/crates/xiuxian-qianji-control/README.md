# xiuxian-qianji-control

`xiuxian-qianji-control` is the workflow-neutral control-plane kernel for
Qianji-managed Agent execution.

Qianji workflow crates describe what should happen. This crate records and
manages what is happening:

- run and step lifecycle events
- deterministic replay views
- evidence and artifact references
- budget and cost observations
- gate results
- recovery attempts
- hot scheduling leases and worker heartbeats

The core slice ships Rust contracts plus in-memory stores. The `duckdb`
feature adds the durable append-only ledger adapter. The `valkey` feature adds
the hot-state adapter for queues, leases, and worker heartbeats.

## Boundary

This crate must stay independent from workflow implementations. It does not
depend on `xiuxian-qianji`, `qianji-bpmn-engine`, `xiuxian-wendao`,
`xiuxian-llm`, or `xiuxian-qianhuan`.

The intended split is:

- DuckDB: durable append-only control ledger and replayable run views
- Valkey: hot queues, leases, heartbeats, rate limits, and live progress
- Qianji workflow/BPMN/Flowhub: domain execution semantics
- Agent workers: leased executors that attach evidence and observations

Qianji workflow traces are projected into control events by
[`xiuxian-qianji`](../xiuxian-qianji/README.md). This crate does not depend on
Qianji workflow types.

## Current Surface

- `ControlEvent` and `ControlEventRecord`
- `ControlLedger`
- `HotStateStore`
- `InMemoryControlLedger`
- `InMemoryHotStateStore`
- `RunView` and `StepView`
- `RequiredEvidenceGate`
- `DuckDbControlLedger` behind the `duckdb` feature
- `ValkeyHotStateStore` behind the `valkey` feature
