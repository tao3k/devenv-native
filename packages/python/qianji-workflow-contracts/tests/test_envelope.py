from __future__ import annotations

import pytest

from qianji_workflow_contracts import (
    ContractValidationError,
    ContractVersionError,
    WorkflowEnvelope,
    make_envelope,
    parse_envelope,
)


def test_make_and_parse_envelope() -> None:
    envelope = make_envelope("qianji.dataset_ref", {"dataset_name": "daily_summary"})
    parsed = parse_envelope(envelope.to_payload())

    assert parsed == envelope
    assert parsed.kind == "qianji.dataset_ref"


def test_envelope_allows_future_minor_version_with_same_major() -> None:
    envelope = WorkflowEnvelope(
        kind="qianji.dataset_ref",
        contract_version="0.2",
        payload={"dataset_name": "daily_summary"},
    )

    assert envelope.contract_version == "0.2"


def test_envelope_rejects_major_version_mismatch() -> None:
    with pytest.raises(ContractVersionError):
        WorkflowEnvelope(
            kind="qianji.dataset_ref",
            contract_version="1.0",
            payload={"dataset_name": "daily_summary"},
        )


def test_parse_envelope_requires_payload_field() -> None:
    with pytest.raises(ContractValidationError):
        parse_envelope({"kind": "qianji.dataset_ref", "contract_version": "0.1"})
