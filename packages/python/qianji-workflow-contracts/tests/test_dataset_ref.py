from __future__ import annotations

import pytest

from qianji_workflow_contracts import ContractValidationError, WorkflowDatasetRef


def test_dataset_ref_payload_roundtrip() -> None:
    dataset_ref = WorkflowDatasetRef(
        dataset_name="risk_features_daily",
        source="wendao.arrow",
        route="/datasets/risk_features_daily",
        schema_digest="sha256:features-v1",
        row_count=128,
        column_count=12,
        flight_endpoint="grpc://wendao.internal:50051",
        ticket="ticket-1",
        partition={"date": "2026-04-20"},
        window_start="2026-04-20T00:00:00",
        window_end="2026-04-20T23:59:59",
        metadata={"owner": "risk-team"},
    )

    payload = dataset_ref.to_payload()

    assert payload["dataset_name"] == "risk_features_daily"
    assert payload["partition"] == {"date": "2026-04-20"}
    assert WorkflowDatasetRef.from_payload(payload) == dataset_ref


def test_dataset_ref_optional_fields_roundtrip() -> None:
    dataset_ref = WorkflowDatasetRef(
        dataset_name="daily_summary",
        source="materialized.report",
        route=None,
        schema_digest="sha256:summary-v1",
        row_count=0,
        column_count=0,
    )

    roundtrip = WorkflowDatasetRef.from_json(dataset_ref.to_json())

    assert roundtrip.route is None
    assert roundtrip.ticket is None
    assert roundtrip.window_start is None
    assert roundtrip.window_end is None
    assert roundtrip.metadata == {}


def test_dataset_ref_ignores_extra_fields_during_parse() -> None:
    payload = {
        "dataset_name": "daily_summary",
        "source": "materialized.report",
        "route": None,
        "schema_digest": "sha256:summary-v1",
        "row_count": 10,
        "column_count": 4,
        "extra_field": "ignored",
    }

    dataset_ref = WorkflowDatasetRef.from_payload(payload)

    assert dataset_ref.dataset_name == "daily_summary"


def test_dataset_ref_constructor_rejects_bool_row_count() -> None:
    with pytest.raises(ContractValidationError):
        WorkflowDatasetRef(
            dataset_name="daily_summary",
            source="materialized.report",
            route=None,
            schema_digest="sha256:summary-v1",
            row_count=True,
            column_count=0,
        )
