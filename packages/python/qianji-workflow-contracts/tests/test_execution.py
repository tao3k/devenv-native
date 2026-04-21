from __future__ import annotations

import pytest

from qianji_workflow_contracts import (
    EXECUTION_REF_KIND,
    EXECUTION_STATUS_KIND,
    ContractValidationError,
    WorkflowExecutionRef,
    WorkflowExecutionStatus,
)


def test_execution_ref_roundtrip() -> None:
    execution_ref = WorkflowExecutionRef(
        instance_id="instance-1",
        process_id="credit-risk-review",
        session_id="session-1",
        checkpoint_id="checkpoint-1",
        metadata={"tenant": "alpha"},
    )

    payload = execution_ref.to_payload()

    assert WorkflowExecutionRef.from_payload(payload) == execution_ref
    assert WorkflowExecutionRef.from_json(execution_ref.to_json()) == execution_ref


def test_execution_status_envelope_roundtrip() -> None:
    status = WorkflowExecutionStatus(
        instance_id="instance-1",
        status="waiting",
        updated_at="2026-04-20T10:00:00Z",
        active_node_id="user-approval",
        metadata={"lane": "underwriting"},
    )

    envelope = status.to_envelope()

    assert envelope.kind == EXECUTION_STATUS_KIND
    assert WorkflowExecutionStatus.from_envelope(envelope) == status


def test_execution_ref_envelope_uses_declared_kind() -> None:
    execution_ref = WorkflowExecutionRef(
        instance_id="instance-1",
        process_id="credit-risk-review",
    )

    assert execution_ref.to_envelope().kind == EXECUTION_REF_KIND


def test_execution_status_requires_non_empty_status() -> None:
    with pytest.raises(
        ContractValidationError, match="status must be a non-empty string"
    ):
        WorkflowExecutionStatus(
            instance_id="instance-1",
            status="",
            updated_at="2026-04-20T10:00:00Z",
        )
