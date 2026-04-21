# qianji-workflow-contracts

`qianji-workflow-contracts` is the pure contract package for Qianji workflow
integrations.

It defines stable JSON-safe payload contracts for:

1. workflow dataset references
2. BPMN host-work requests and results
3. DMN decision batches and outcomes
4. shared workflow envelopes
5. minimal workflow execution references and status snapshots

It does not own:

1. Arrow or DataFrame objects
2. Flight transport
3. runtime orchestration
4. BPMN or DMN execution logic

## Position

This package defines what workflow systems exchange, not how they transport,
compute, or execute those exchanges.

Boundary mapping:

1. `wendao-arrow-interface`: Flight consumer facade
2. `wendao-datascience`: future builder layer that can produce these contracts
3. `xiuxian-qianji`: runtime consumer of these contracts
4. `qianji-workflow-contracts`: stable payload contract owner

## Quick Start

```python
from qianji_workflow_contracts import (
    DecisionBatch,
    DecisionRow,
    HostWorkRequest,
    WorkflowDatasetRef,
)

dataset_ref = WorkflowDatasetRef(
    dataset_name="risk_features_daily",
    source="wendao.arrow",
    route="/datasets/risk_features_daily",
    schema_digest="sha256:features-v1",
    row_count=128,
    column_count=12,
)

decision_batch = DecisionBatch(
    decision_key="risk_decision",
    rule_version="2026-04-20",
    key_columns=("account_id",),
    feature_columns=("risk_score", "segment"),
    rows=(
        DecisionRow(
            keys={"account_id": "acct-1"},
            features={"risk_score": 0.91, "segment": "enterprise"},
        ),
    ),
    dataset_ref=dataset_ref,
)

host_work = HostWorkRequest(
    work_id="work-1",
    node_id="business-rule-task",
    token_id="token-1",
    work_kind="business_rule",
    input_payload={"decision_batch": decision_batch.to_payload()},
    dataset_refs=(dataset_ref,),
)

print(host_work.to_envelope().to_json())
```

## Design Rules

1. Runtime dependencies remain zero.
2. All payload values are JSON-safe.
3. Envelope-first interop uses `kind`, `contract_version`, and `payload`.
4. Field names stay stable and cross-language friendly.

## Compatibility Rules

1. `contract_version` is a protocol version, not the package version.
2. Exact supported versions are accepted.
3. Future minor versions with the same major component are accepted.
4. Different major versions are rejected.
5. Extra fields are ignored during parsing for forward compatibility.
6. Missing required fields still raise `ContractValidationError`.

## Public Surface

The v1 public surface is:

1. `WorkflowEnvelope`
2. `WorkflowDatasetRef`
3. `DecisionRow`
4. `DecisionBatch`
5. `DecisionReason`
6. `DecisionOutcomeRow`
7. `DecisionOutcome`
8. `HostWorkRequest`
9. `HostWorkResult`
10. `WorkflowExecutionRef`
11. `WorkflowExecutionStatus`
