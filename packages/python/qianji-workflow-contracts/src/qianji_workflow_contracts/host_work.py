"""BPMN host-work request and result contracts."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass, field

from .base import (
    JsonObject,
    PayloadConvertible,
    normalize_json_object,
    normalize_required_json_object,
    optional_str,
    require_field,
    require_str,
    validate_model_tuple,
    validate_optional_str_value,
    validate_str_value,
)
from .dataset_ref import WorkflowDatasetRef
from .decision import DecisionOutcome
from .errors import ContractValidationError

HOST_WORK_REQUEST_KIND = "qianji.host_work.request"
HOST_WORK_RESULT_KIND = "qianji.host_work.result"
ALLOWED_HOST_WORK_STATUSES = ("completed", "failed", "waiting")


@dataclass(frozen=True, slots=True)
class HostWorkRequest(PayloadConvertible):
    """Host bridge request emitted from a workflow node."""

    KIND = HOST_WORK_REQUEST_KIND

    work_id: str
    node_id: str
    token_id: str | None
    work_kind: str
    input_payload: JsonObject | Mapping[str, object]
    dataset_refs: tuple[WorkflowDatasetRef, ...] = ()
    metadata: JsonObject | Mapping[str, object] = field(default_factory=dict)

    def __post_init__(self) -> None:
        object.__setattr__(self, "work_id", validate_str_value(self.work_id, "work_id"))
        object.__setattr__(self, "node_id", validate_str_value(self.node_id, "node_id"))
        object.__setattr__(
            self, "token_id", validate_optional_str_value(self.token_id, "token_id")
        )
        object.__setattr__(
            self,
            "work_kind",
            validate_str_value(self.work_kind, "work_kind"),
        )
        object.__setattr__(
            self,
            "input_payload",
            normalize_required_json_object(
                self.input_payload, field_name="input_payload"
            ),
        )
        object.__setattr__(
            self,
            "dataset_refs",
            validate_model_tuple(self.dataset_refs, "dataset_refs", WorkflowDatasetRef),
        )
        object.__setattr__(
            self,
            "metadata",
            normalize_json_object(self.metadata, field_name="metadata"),
        )

    def to_payload(self) -> JsonObject:
        return {
            "work_id": self.work_id,
            "node_id": self.node_id,
            "token_id": self.token_id,
            "work_kind": self.work_kind,
            "input_payload": self.input_payload,
            "dataset_refs": [
                dataset_ref.to_payload() for dataset_ref in self.dataset_refs
            ],
            "metadata": self.metadata,
        }

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> HostWorkRequest:
        refs_payload = payload.get("dataset_refs", ())
        if not isinstance(refs_payload, tuple | list):
            raise ContractValidationError(
                "dataset_refs must be a sequence when provided"
            )
        dataset_refs: list[WorkflowDatasetRef] = []
        for item in refs_payload:
            if not isinstance(item, Mapping):
                raise ContractValidationError("dataset_refs items must be mappings")
            dataset_refs.append(WorkflowDatasetRef.from_payload(item))
        return cls(
            work_id=require_str(payload, "work_id"),
            node_id=require_str(payload, "node_id"),
            token_id=optional_str(payload, "token_id"),
            work_kind=require_str(payload, "work_kind"),
            input_payload=require_field(payload, "input_payload"),
            dataset_refs=tuple(dataset_refs),
            metadata=payload.get("metadata", {}),
        )


@dataclass(frozen=True, slots=True)
class HostWorkResult(PayloadConvertible):
    """Host bridge result returned to a workflow node."""

    KIND = HOST_WORK_RESULT_KIND

    work_id: str
    node_id: str
    token_id: str | None
    status: str
    output_payload: JsonObject | Mapping[str, object] = field(default_factory=dict)
    decision_outcome: DecisionOutcome | None = None
    error_code: str | None = None
    error_message: str | None = None
    metadata: JsonObject | Mapping[str, object] = field(default_factory=dict)

    def __post_init__(self) -> None:
        object.__setattr__(self, "work_id", validate_str_value(self.work_id, "work_id"))
        object.__setattr__(self, "node_id", validate_str_value(self.node_id, "node_id"))
        object.__setattr__(
            self, "token_id", validate_optional_str_value(self.token_id, "token_id")
        )
        object.__setattr__(self, "status", validate_str_value(self.status, "status"))
        if self.status not in ALLOWED_HOST_WORK_STATUSES:
            raise ContractValidationError(
                f"status must be one of {ALLOWED_HOST_WORK_STATUSES!r}"
            )
        object.__setattr__(
            self,
            "output_payload",
            normalize_json_object(self.output_payload, field_name="output_payload"),
        )
        if self.decision_outcome is not None and not isinstance(
            self.decision_outcome, DecisionOutcome
        ):
            raise ContractValidationError(
                "decision_outcome must be a DecisionOutcome when provided"
            )
        object.__setattr__(
            self,
            "error_code",
            validate_optional_str_value(self.error_code, "error_code"),
        )
        object.__setattr__(
            self,
            "error_message",
            validate_optional_str_value(self.error_message, "error_message"),
        )
        object.__setattr__(
            self,
            "metadata",
            normalize_json_object(self.metadata, field_name="metadata"),
        )

    def to_payload(self) -> JsonObject:
        payload: JsonObject = {
            "work_id": self.work_id,
            "node_id": self.node_id,
            "token_id": self.token_id,
            "status": self.status,
            "output_payload": self.output_payload,
            "error_code": self.error_code,
            "error_message": self.error_message,
            "metadata": self.metadata,
        }
        if self.decision_outcome is not None:
            payload["decision_outcome"] = self.decision_outcome.to_payload()
        return payload

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> HostWorkResult:
        decision_outcome_payload = payload.get("decision_outcome")
        if decision_outcome_payload is not None and not isinstance(
            decision_outcome_payload, Mapping
        ):
            raise ContractValidationError(
                "decision_outcome must be a mapping when provided"
            )
        return cls(
            work_id=require_str(payload, "work_id"),
            node_id=require_str(payload, "node_id"),
            token_id=optional_str(payload, "token_id"),
            status=require_str(payload, "status"),
            output_payload=payload.get("output_payload", {}),
            decision_outcome=(
                DecisionOutcome.from_payload(decision_outcome_payload)
                if decision_outcome_payload is not None
                else None
            ),
            error_code=optional_str(payload, "error_code"),
            error_message=optional_str(payload, "error_message"),
            metadata=payload.get("metadata", {}),
        )
