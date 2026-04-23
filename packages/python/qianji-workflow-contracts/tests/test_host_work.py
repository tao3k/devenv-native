from __future__ import annotations

import pytest

from qianji_workflow_contracts import (
    ContractValidationError,
    DecisionOutcome,
    DecisionOutcomeRow,
    HostWorkRequest,
    HostWorkResult,
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


def test_host_work_request_roundtrip() -> None:
    request = HostWorkRequest(
        work_id="work-1",
        node_id="business-rule-task",
        token_id="token-1",
        work_kind="business_rule",
        input_payload={"decision_key": "risk_decision"},
        dataset_refs=(build_dataset_ref(),),
        metadata={"lane": "host-work"},
    )

    roundtrip = HostWorkRequest.from_json(request.to_json())

    assert roundtrip == request
    assert roundtrip.dataset_refs[0].dataset_name == "risk_features_daily"


def test_host_work_result_completed_roundtrip() -> None:
    result = HostWorkResult(
        work_id="work-1",
        node_id="business-rule-task",
        token_id="token-1",
        status="completed",
        output_payload={"approved": True},
        decision_outcome=DecisionOutcome(
            decision_key="risk_decision",
            rule_version="2026-04-20",
            row_count=1,
            rows=(
                DecisionOutcomeRow(
                    keys={"account_id": "acct-1"}, outputs={"risk_band": "high"}
                ),
            ),
        ),
    )

    roundtrip = HostWorkResult.from_payload(result.to_payload())

    assert roundtrip == result


def test_host_work_result_failed_and_waiting_statuses() -> None:
    failed = HostWorkResult(
        work_id="work-2",
        node_id="service-task",
        token_id=None,
        status="failed",
        error_code="timeout",
        error_message="worker timeout",
    )
    waiting = HostWorkResult(
        work_id="work-3",
        node_id="user-task",
        token_id="token-2",
        status="waiting",
        metadata={"reason": "human_approval"},
    )

    assert HostWorkResult.from_json(failed.to_json()) == failed
    assert HostWorkResult.from_json(waiting.to_json()) == waiting


def test_host_work_request_requires_input_payload() -> None:
    with pytest.raises(ContractValidationError):
        HostWorkRequest.from_payload(
            {
                "work_id": "work-1",
                "node_id": "business-rule-task",
                "token_id": "token-1",
                "work_kind": "business_rule",
            }
        )


def test_host_work_result_constructor_rejects_non_decision_outcome() -> None:
    with pytest.raises(ContractValidationError):
        HostWorkResult(
            work_id="work-1",
            node_id="business-rule-task",
            token_id="token-1",
            status="completed",
            decision_outcome={"decision_key": "risk_decision"},
        )
