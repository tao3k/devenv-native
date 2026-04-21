"""DMN-style decision request and outcome contracts."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass, field

from .base import (
    JsonObject,
    PayloadConvertible,
    normalize_json_object,
    normalize_required_json_object,
    optional_str,
    parse_model_tuple,
    parse_string_tuple,
    require_field,
    require_int,
    require_sequence,
    require_str,
    validate_int_value,
    validate_model_tuple,
    validate_optional_str_value,
    validate_str_value,
    validate_string_tuple,
)
from .dataset_ref import WorkflowDatasetRef
from .errors import ContractValidationError

DECISION_BATCH_KIND = "qianji.decision.batch"
DECISION_OUTCOME_KIND = "qianji.decision.outcome"


@dataclass(frozen=True, slots=True)
class DecisionRow(PayloadConvertible):
    """One DMN input row."""

    keys: JsonObject | Mapping[str, object]
    features: JsonObject | Mapping[str, object]

    def __post_init__(self) -> None:
        object.__setattr__(
            self,
            "keys",
            normalize_required_json_object(self.keys, field_name="keys"),
        )
        object.__setattr__(
            self,
            "features",
            normalize_required_json_object(self.features, field_name="features"),
        )

    def to_payload(self) -> JsonObject:
        return {"keys": self.keys, "features": self.features}

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> DecisionRow:
        return cls(
            keys=require_field(payload, "keys"),
            features=require_field(payload, "features"),
        )


@dataclass(frozen=True, slots=True)
class DecisionBatch(PayloadConvertible):
    """Batch request for one decision surface."""

    KIND = DECISION_BATCH_KIND

    decision_key: str
    rule_version: str
    key_columns: tuple[str, ...]
    feature_columns: tuple[str, ...]
    rows: tuple[DecisionRow, ...]
    dataset_ref: WorkflowDatasetRef | None = None
    metadata: JsonObject | Mapping[str, object] = field(default_factory=dict)

    def __post_init__(self) -> None:
        object.__setattr__(
            self,
            "decision_key",
            validate_str_value(self.decision_key, "decision_key"),
        )
        object.__setattr__(
            self,
            "rule_version",
            validate_str_value(self.rule_version, "rule_version"),
        )
        object.__setattr__(
            self,
            "key_columns",
            validate_string_tuple(self.key_columns, "key_columns"),
        )
        object.__setattr__(
            self,
            "feature_columns",
            validate_string_tuple(self.feature_columns, "feature_columns"),
        )
        object.__setattr__(
            self, "rows", validate_model_tuple(self.rows, "rows", DecisionRow)
        )
        if self.dataset_ref is not None and not isinstance(
            self.dataset_ref, WorkflowDatasetRef
        ):
            raise ContractValidationError(
                "dataset_ref must be a WorkflowDatasetRef when provided"
            )
        object.__setattr__(
            self,
            "metadata",
            normalize_json_object(self.metadata, field_name="metadata"),
        )

    def to_payload(self) -> JsonObject:
        payload: JsonObject = {
            "decision_key": self.decision_key,
            "rule_version": self.rule_version,
            "key_columns": list(self.key_columns),
            "feature_columns": list(self.feature_columns),
            "rows": [row.to_payload() for row in self.rows],
            "metadata": self.metadata,
        }
        if self.dataset_ref is not None:
            payload["dataset_ref"] = self.dataset_ref.to_payload()
        return payload

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> DecisionBatch:
        dataset_ref_payload = payload.get("dataset_ref")
        if dataset_ref_payload is not None and not isinstance(
            dataset_ref_payload, Mapping
        ):
            raise ContractValidationError("dataset_ref must be a mapping when provided")
        return cls(
            decision_key=require_str(payload, "decision_key"),
            rule_version=require_str(payload, "rule_version"),
            key_columns=parse_string_tuple(payload, "key_columns"),
            feature_columns=parse_string_tuple(payload, "feature_columns"),
            rows=parse_model_tuple(payload, "rows", DecisionRow),
            dataset_ref=(
                WorkflowDatasetRef.from_payload(dataset_ref_payload)
                if dataset_ref_payload is not None
                else None
            ),
            metadata=payload.get("metadata", {}),
        )


@dataclass(frozen=True, slots=True)
class DecisionReason(PayloadConvertible):
    """Structured explanation attached to one decision outcome row."""

    code: str
    message: str | None = None
    metadata: JsonObject | Mapping[str, object] = field(default_factory=dict)

    def __post_init__(self) -> None:
        object.__setattr__(self, "code", validate_str_value(self.code, "code"))
        object.__setattr__(
            self, "message", validate_optional_str_value(self.message, "message")
        )
        object.__setattr__(
            self,
            "metadata",
            normalize_json_object(self.metadata, field_name="metadata"),
        )

    def to_payload(self) -> JsonObject:
        return {"code": self.code, "message": self.message, "metadata": self.metadata}

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> DecisionReason:
        return cls(
            code=require_str(payload, "code"),
            message=optional_str(payload, "message"),
            metadata=payload.get("metadata", {}),
        )


@dataclass(frozen=True, slots=True)
class DecisionOutcomeRow(PayloadConvertible):
    """One DMN output row."""

    keys: JsonObject | Mapping[str, object]
    outputs: JsonObject | Mapping[str, object]
    reasons: tuple[DecisionReason, ...] = ()

    def __post_init__(self) -> None:
        object.__setattr__(
            self,
            "keys",
            normalize_required_json_object(self.keys, field_name="keys"),
        )
        object.__setattr__(
            self,
            "outputs",
            normalize_required_json_object(self.outputs, field_name="outputs"),
        )
        object.__setattr__(
            self,
            "reasons",
            validate_model_tuple(self.reasons, "reasons", DecisionReason),
        )

    def to_payload(self) -> JsonObject:
        return {
            "keys": self.keys,
            "outputs": self.outputs,
            "reasons": [reason.to_payload() for reason in self.reasons],
        }

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> DecisionOutcomeRow:
        reasons_payload = payload.get("reasons", ())
        if not isinstance(reasons_payload, tuple | list):
            raise ContractValidationError("reasons must be a sequence when provided")
        reasons: list[DecisionReason] = []
        for item in reasons_payload:
            if not isinstance(item, Mapping):
                raise ContractValidationError("reasons items must be mappings")
            reasons.append(DecisionReason.from_payload(item))
        return cls(
            keys=require_field(payload, "keys"),
            outputs=require_field(payload, "outputs"),
            reasons=tuple(reasons),
        )


@dataclass(frozen=True, slots=True)
class DecisionOutcome(PayloadConvertible):
    """Outcome rows emitted from one decision surface."""

    KIND = DECISION_OUTCOME_KIND

    decision_key: str
    rule_version: str
    row_count: int
    rows: tuple[DecisionOutcomeRow, ...]
    metadata: JsonObject | Mapping[str, object] = field(default_factory=dict)

    def __post_init__(self) -> None:
        object.__setattr__(
            self,
            "decision_key",
            validate_str_value(self.decision_key, "decision_key"),
        )
        object.__setattr__(
            self,
            "rule_version",
            validate_str_value(self.rule_version, "rule_version"),
        )
        object.__setattr__(
            self,
            "row_count",
            validate_int_value(self.row_count, "row_count", minimum=0),
        )
        object.__setattr__(
            self,
            "rows",
            validate_model_tuple(self.rows, "rows", DecisionOutcomeRow),
        )
        if self.row_count != len(self.rows):
            raise ContractValidationError("row_count must match the number of rows")
        object.__setattr__(
            self,
            "metadata",
            normalize_json_object(self.metadata, field_name="metadata"),
        )

    def to_payload(self) -> JsonObject:
        return {
            "decision_key": self.decision_key,
            "rule_version": self.rule_version,
            "row_count": self.row_count,
            "rows": [row.to_payload() for row in self.rows],
            "metadata": self.metadata,
        }

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> DecisionOutcome:
        rows_payload = require_sequence(payload, "rows")
        rows: list[DecisionOutcomeRow] = []
        for item in rows_payload:
            if not isinstance(item, Mapping):
                raise ContractValidationError("rows items must be mappings")
            rows.append(DecisionOutcomeRow.from_payload(item))
        return cls(
            decision_key=require_str(payload, "decision_key"),
            rule_version=require_str(payload, "rule_version"),
            row_count=require_int(payload, "row_count"),
            rows=tuple(rows),
            metadata=payload.get("metadata", {}),
        )
