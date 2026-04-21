from __future__ import annotations

import pytest

from qianji_workflow_contracts import (
    ContractVersionError,
    DecisionBatch,
    WorkflowDatasetRef,
    WorkflowExecutionStatus,
)


def test_model_from_envelope_accepts_future_minor_version() -> None:
    dataset_ref = WorkflowDatasetRef.from_envelope(
        {
            "kind": "qianji.dataset_ref",
            "contract_version": "0.2",
            "payload": {
                "dataset_name": "daily_summary",
                "source": "wendao.arrow",
                "route": "/datasets/daily_summary",
                "schema_digest": "sha256:daily-summary-v1",
                "row_count": 32,
                "column_count": 6,
            },
        }
    )

    assert dataset_ref.dataset_name == "daily_summary"


def test_model_from_envelope_rejects_major_version_mismatch() -> None:
    with pytest.raises(ContractVersionError):
        WorkflowExecutionStatus.from_envelope(
            {
                "kind": "qianji.execution.status",
                "contract_version": "1.0",
                "payload": {
                    "instance_id": "instance-1",
                    "status": "waiting",
                    "updated_at": "2026-04-21T08:00:00Z",
                },
            }
        )


def test_dataset_ref_from_payload_ignores_extra_fields() -> None:
    dataset_ref = WorkflowDatasetRef.from_payload(
        {
            "dataset_name": "daily_summary",
            "source": "wendao.arrow",
            "route": "/datasets/daily_summary",
            "schema_digest": "sha256:daily-summary-v1",
            "row_count": 32,
            "column_count": 6,
            "future_extension": {"owner": "risk"},
        }
    )

    assert dataset_ref == WorkflowDatasetRef(
        dataset_name="daily_summary",
        source="wendao.arrow",
        route="/datasets/daily_summary",
        schema_digest="sha256:daily-summary-v1",
        row_count=32,
        column_count=6,
    )


def test_decision_batch_from_payload_ignores_nested_extra_fields() -> None:
    decision_batch = DecisionBatch.from_payload(
        {
            "decision_key": "risk_decision",
            "rule_version": "2026-04-21",
            "key_columns": ["account_id"],
            "feature_columns": ["risk_score"],
            "rows": [
                {
                    "keys": {"account_id": "acct-1"},
                    "features": {"risk_score": 0.91},
                    "future_extension": {"reason": "reserved"},
                }
            ],
            "metadata": {"lane": "underwriting"},
            "future_extension": {"producer": "dmn-host"},
        }
    )

    assert decision_batch.rows[0].keys == {"account_id": "acct-1"}
    assert decision_batch.rows[0].features == {"risk_score": 0.91}
    assert decision_batch.metadata == {"lane": "underwriting"}
