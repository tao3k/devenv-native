from __future__ import annotations

import pytest

from qianji_workflow_contracts import (
    ContractValidationError,
    DecisionBatch,
    DecisionOutcome,
    DecisionOutcomeRow,
    DecisionReason,
    DecisionRow,
    WorkflowDatasetRef,
)


def build_dataset_ref() -> WorkflowDatasetRef:
    return WorkflowDatasetRef(
        dataset_name="risk_features_daily",
        source="wendao.arrow",
        route="/datasets/risk_features_daily",
        schema_digest="sha256:features-v1",
        row_count=128,
        column_count=12,
    )


def test_decision_batch_roundtrip() -> None:
    batch = DecisionBatch(
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
        dataset_ref=build_dataset_ref(),
        metadata={"lane": "batch"},
    )

    payload = batch.to_payload()
    roundtrip = DecisionBatch.from_payload(payload)

    assert payload["rows"][0]["keys"]["account_id"] == "acct-1"
    assert roundtrip == batch


def test_decision_outcome_roundtrip() -> None:
    outcome = DecisionOutcome(
        decision_key="risk_decision",
        rule_version="2026-04-20",
        row_count=1,
        rows=(
            DecisionOutcomeRow(
                keys={"account_id": "acct-1"},
                outputs={"risk_band": "high"},
                reasons=(
                    DecisionReason(
                        code="threshold-crossed",
                        message="risk_score exceeded the high threshold",
                        metadata={"threshold": 0.9},
                    ),
                ),
            ),
        ),
        metadata={"source": "dmn"},
    )

    roundtrip = DecisionOutcome.from_json(outcome.to_json())

    assert roundtrip == outcome
    assert roundtrip.rows[0].reasons[0].code == "threshold-crossed"


def test_decision_outcome_requires_matching_row_count() -> None:
    with pytest.raises(ContractValidationError, match="row_count"):
        DecisionOutcome(
            decision_key="risk_decision",
            rule_version="2026-04-20",
            row_count=2,
            rows=(
                DecisionOutcomeRow(
                    keys={"account_id": "acct-1"}, outputs={"risk_band": "high"}
                ),
            ),
        )


def test_decision_row_requires_keys_and_features() -> None:
    with pytest.raises(ContractValidationError):
        DecisionRow.from_payload({"keys": {"account_id": "acct-1"}})


def test_decision_batch_constructor_rejects_non_model_rows() -> None:
    with pytest.raises(ContractValidationError):
        DecisionBatch(
            decision_key="risk_decision",
            rule_version="2026-04-20",
            key_columns=("account_id",),
            feature_columns=("risk_score",),
            rows=(
                {"keys": {"account_id": "acct-1"}, "features": {"risk_score": 0.91}},
            ),
        )
