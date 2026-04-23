"""Shared payload-conversion helpers for workflow contracts."""

from __future__ import annotations

import json
import math
from collections.abc import Mapping, Sequence
from datetime import date, datetime, time
from decimal import Decimal
from typing import TYPE_CHECKING, ClassVar, Self

if TYPE_CHECKING:
    from .envelope import WorkflowEnvelope

from .errors import ContractValidationError
from .version import CONTRACT_VERSION

JsonScalar = None | bool | int | float | str
JsonValue = JsonScalar | list["JsonValue"] | dict[str, "JsonValue"]
JsonObject = dict[str, JsonValue]


def normalize_json_value(value: object) -> JsonValue:
    """Normalize one runtime value into a JSON-safe value."""

    if isinstance(value, PayloadConvertible):
        return normalize_json_value(value.to_payload())
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        if not math.isfinite(value):
            raise ContractValidationError("float payload values must be finite")
        return value
    if isinstance(value, str):
        return value
    if isinstance(value, Decimal):
        return str(value)
    if isinstance(value, datetime | date | time):
        return value.isoformat()
    if isinstance(value, bytes | bytearray | memoryview):
        raise ContractValidationError("bytes payload values are not supported")
    if isinstance(value, Mapping):
        normalized: JsonObject = {}
        for key, item in value.items():
            if not isinstance(key, str):
                raise ContractValidationError("payload mapping keys must be strings")
            normalized[key] = normalize_json_value(item)
        return normalized
    if isinstance(value, Sequence) and not isinstance(value, str | bytes | bytearray):
        return [normalize_json_value(item) for item in value]
    raise ContractValidationError(
        f"unsupported payload value type: {type(value).__name__}"
    )


def normalize_json_object(value: object, *, field_name: str) -> JsonObject:
    """Normalize one mapping into a JSON object."""

    if value is None:
        return {}
    if not isinstance(value, Mapping):
        raise ContractValidationError(f"{field_name} must be a mapping")
    normalized = normalize_json_value(value)
    if not isinstance(normalized, dict):
        raise ContractValidationError(f"{field_name} must normalize to a JSON object")
    return normalized


def normalize_required_json_object(value: object, *, field_name: str) -> JsonObject:
    """Normalize one required mapping into a JSON object."""

    if value is None:
        raise ContractValidationError(f"{field_name} is required")
    return normalize_json_object(value, field_name=field_name)


def require_field(payload: Mapping[str, object], field_name: str) -> object:
    """Return one required field or raise a validation error."""

    if field_name not in payload:
        raise ContractValidationError(f"missing required field {field_name!r}")
    return payload[field_name]


def require_str(payload: Mapping[str, object], field_name: str) -> str:
    """Return one required string field."""

    value = require_field(payload, field_name)
    if not isinstance(value, str):
        raise ContractValidationError(f"{field_name} must be a string")
    return value


def optional_str(payload: Mapping[str, object], field_name: str) -> str | None:
    """Return one optional string field."""

    value = payload.get(field_name)
    if value is None:
        return None
    if not isinstance(value, str):
        raise ContractValidationError(f"{field_name} must be a string when provided")
    return value


def require_int(payload: Mapping[str, object], field_name: str) -> int:
    """Return one required integer field."""

    value = require_field(payload, field_name)
    if isinstance(value, bool) or not isinstance(value, int):
        raise ContractValidationError(f"{field_name} must be an integer")
    return value


def require_sequence(
    payload: Mapping[str, object], field_name: str
) -> Sequence[object]:
    """Return one required list-like field."""

    value = require_field(payload, field_name)
    if not isinstance(value, Sequence) or isinstance(value, str | bytes | bytearray):
        raise ContractValidationError(f"{field_name} must be a sequence")
    return value


def validate_str_value(
    value: object,
    field_name: str,
    *,
    allow_empty: bool = False,
) -> str:
    """Validate one runtime string value."""

    if not isinstance(value, str):
        raise ContractValidationError(f"{field_name} must be a string")
    if not allow_empty and not value:
        raise ContractValidationError(f"{field_name} must be a non-empty string")
    return value


def validate_optional_str_value(value: object, field_name: str) -> str | None:
    """Validate one optional runtime string value."""

    if value is None:
        return None
    return validate_str_value(value, field_name)


def validate_int_value(
    value: object,
    field_name: str,
    *,
    minimum: int | None = None,
) -> int:
    """Validate one runtime integer value."""

    if isinstance(value, bool) or not isinstance(value, int):
        raise ContractValidationError(f"{field_name} must be an integer")
    if minimum is not None and value < minimum:
        raise ContractValidationError(f"{field_name} must be >= {minimum}")
    return value


def validate_string_tuple(values: object, field_name: str) -> tuple[str, ...]:
    """Validate one sequence of strings."""

    if not isinstance(values, Sequence) or isinstance(values, str | bytes | bytearray):
        raise ContractValidationError(f"{field_name} must be a sequence")
    items: list[str] = []
    for item in values:
        items.append(validate_str_value(item, f"{field_name} item"))
    return tuple(items)


def validate_model_tuple[T](
    values: object,
    field_name: str,
    model_type: type[T],
) -> tuple[T, ...]:
    """Validate one sequence of model instances."""

    if not isinstance(values, Sequence) or isinstance(values, str | bytes | bytearray):
        raise ContractValidationError(f"{field_name} must be a sequence")
    items: list[T] = []
    for item in values:
        if not isinstance(item, model_type):
            raise ContractValidationError(
                f"{field_name} items must be {model_type.__name__} instances"
            )
        items.append(item)
    return tuple(items)


def parse_string_tuple(
    payload: Mapping[str, object], field_name: str
) -> tuple[str, ...]:
    """Parse one sequence of strings into a tuple."""

    sequence = require_sequence(payload, field_name)
    items: list[str] = []
    for item in sequence:
        if not isinstance(item, str):
            raise ContractValidationError(f"{field_name} items must be strings")
        items.append(item)
    return tuple(items)


def parse_model_tuple[T](
    payload: Mapping[str, object],
    field_name: str,
    model_type: type[T],
) -> tuple[T, ...]:
    """Parse one sequence of nested payload models."""

    sequence = require_sequence(payload, field_name)
    items: list[T] = []
    for item in sequence:
        if not isinstance(item, Mapping):
            raise ContractValidationError(f"{field_name} items must be mappings")
        items.append(model_type.from_payload(item))
    return tuple(items)


class PayloadConvertible:
    """Mixin for payload and JSON conversion helpers."""

    KIND: ClassVar[str | None] = None

    def to_payload(self) -> JsonObject:
        """Convert the object into a stable payload mapping."""

        raise NotImplementedError

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> Self:
        """Build the object from a payload mapping."""

        raise NotImplementedError

    def to_json(self) -> str:
        """Serialize the payload as stable JSON."""

        return json.dumps(
            self.to_payload(),
            ensure_ascii=False,
            separators=(",", ":"),
            sort_keys=True,
        )

    @classmethod
    def from_json(cls, raw: str) -> Self:
        """Build the object from one JSON payload string."""

        value = json.loads(raw)
        if not isinstance(value, Mapping):
            raise ContractValidationError("JSON payload must decode to an object")
        return cls.from_payload(value)

    def to_envelope(
        self,
        *,
        kind: str | None = None,
        contract_version: str = CONTRACT_VERSION,
    ) -> WorkflowEnvelope:
        """Wrap the model in a workflow envelope."""

        from .envelope import WorkflowEnvelope

        resolved_kind = kind or self.KIND
        if not resolved_kind:
            raise ContractValidationError("envelope kind is required for this model")
        return WorkflowEnvelope(
            kind=resolved_kind,
            contract_version=contract_version,
            payload=self.to_payload(),
        )

    @classmethod
    def from_envelope(
        cls,
        envelope: WorkflowEnvelope | Mapping[str, object],
        *,
        kind: str | None = None,
    ) -> Self:
        """Build the model from an envelope object or payload mapping."""

        from .envelope import WorkflowEnvelope

        parsed = (
            envelope
            if isinstance(envelope, WorkflowEnvelope)
            else WorkflowEnvelope.from_payload(envelope)
        )
        expected_kind = kind or cls.KIND
        if expected_kind is not None and parsed.kind != expected_kind:
            raise ContractValidationError(
                f"expected envelope kind {expected_kind!r}, got {parsed.kind!r}"
            )
        return cls.from_payload(parsed.payload)
