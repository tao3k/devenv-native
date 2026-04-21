from __future__ import annotations

from datetime import date, datetime, time
from decimal import Decimal

import pytest

from qianji_workflow_contracts import (
    ContractValidationError,
    DecisionBatch,
    DecisionRow,
    HostWorkRequest,
    WorkflowDatasetRef,
)


def test_object_payload_json_object_roundtrip() -> None:
    dataset_ref = WorkflowDatasetRef(
        dataset_name="risk_features_daily",
        source="wendao.arrow",
        route="/datasets/risk_features_daily",
        schema_digest="sha256:features-v1",
        row_count=128,
        column_count=12,
        metadata={"precision": Decimal("0.95")},
    )
    decision_batch = DecisionBatch(
        decision_key="risk_decision",
        rule_version="2026-04-20",
        key_columns=("account_id",),
        feature_columns=("risk_score",),
        rows=(
            DecisionRow(keys={"account_id": "acct-1"}, features={"risk_score": 0.91}),
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

    payload = host_work.to_payload()
    encoded = host_work.to_json()
    decoded = HostWorkRequest.from_json(encoded)

    assert payload["dataset_refs"][0]["metadata"]["precision"] == "0.95"
    assert decoded == host_work
    assert HostWorkRequest.from_payload(payload) == host_work


def test_json_normalization_serializes_temporal_scalars() -> None:
    dataset_ref = WorkflowDatasetRef(
        dataset_name="risk_features_daily",
        source="wendao.arrow",
        route="/datasets/risk_features_daily",
        schema_digest="sha256:features-v1",
        row_count=128,
        column_count=12,
        metadata={
            "batch_date": date(2026, 4, 20),
            "batch_time": time(9, 30, 0),
            "batch_started_at": datetime(2026, 4, 20, 9, 30, 15),
        },
    )

    assert dataset_ref.to_payload()["metadata"] == {
        "batch_date": "2026-04-20",
        "batch_time": "09:30:00",
        "batch_started_at": "2026-04-20T09:30:15",
    }


def test_json_normalization_rejects_bytes_payload_values() -> None:
    with pytest.raises(
        ContractValidationError, match="bytes payload values are not supported"
    ):
        HostWorkRequest(
            work_id="work-1",
            node_id="service-task",
            token_id=None,
            work_kind="service",
            input_payload={"blob": b"binary"},
        )


def test_model_envelope_roundtrip_uses_declared_kind() -> None:
    dataset_ref = WorkflowDatasetRef(
        dataset_name="risk_features_daily",
        source="wendao.arrow",
        route="/datasets/risk_features_daily",
        schema_digest="sha256:features-v1",
        row_count=128,
        column_count=12,
    )
    batch = DecisionBatch(
        decision_key="risk_decision",
        rule_version="2026-04-20",
        key_columns=("account_id",),
        feature_columns=("risk_score",),
        rows=(
            DecisionRow(keys={"account_id": "acct-1"}, features={"risk_score": 0.91}),
        ),
        dataset_ref=dataset_ref,
    )
    request = HostWorkRequest(
        work_id="work-1",
        node_id="business-rule-task",
        token_id="token-1",
        work_kind="business_rule",
        input_payload={"decision_batch": batch.to_payload()},
        dataset_refs=(dataset_ref,),
    )

    envelope = request.to_envelope()
    roundtrip = HostWorkRequest.from_envelope(envelope)

    assert envelope.kind == "qianji.host_work.request"
    assert roundtrip == request


def test_model_from_envelope_rejects_wrong_kind() -> None:
    dataset_ref = WorkflowDatasetRef(
        dataset_name="risk_features_daily",
        source="wendao.arrow",
        route="/datasets/risk_features_daily",
        schema_digest="sha256:features-v1",
        row_count=128,
        column_count=12,
    )

    with_type_mismatch = dataset_ref.to_envelope(kind="qianji.host_work.request")

    with pytest.raises(ContractValidationError, match="expected envelope kind"):
        WorkflowDatasetRef.from_envelope(with_type_mismatch)
