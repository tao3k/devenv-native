"""Envelope helpers for transport-facing workflow payloads."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping

from .base import (
    JsonObject,
    PayloadConvertible,
    normalize_json_object,
    require_field,
    require_str,
)
from .errors import ContractValidationError
from .version import CONTRACT_VERSION, ensure_supported_contract_version


@dataclass(frozen=True, slots=True)
class WorkflowEnvelope(PayloadConvertible):
    """Shared top-level workflow envelope."""

    kind: str
    payload: JsonObject | Mapping[str, object]
    contract_version: str = CONTRACT_VERSION

    def __post_init__(self) -> None:
        if not isinstance(self.kind, str) or not self.kind:
            raise ContractValidationError("kind must be a non-empty string")
        object.__setattr__(
            self,
            "contract_version",
            ensure_supported_contract_version(self.contract_version),
        )
        object.__setattr__(
            self,
            "payload",
            normalize_json_object(self.payload, field_name="payload"),
        )

    def to_payload(self) -> JsonObject:
        return {
            "kind": self.kind,
            "contract_version": self.contract_version,
            "payload": self.payload,
        }

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> WorkflowEnvelope:
        return cls(
            kind=require_str(payload, "kind"),
            contract_version=require_str(payload, "contract_version"),
            payload=require_field(payload, "payload"),
        )


def make_envelope(
    kind: str,
    payload: Mapping[str, object],
    *,
    contract_version: str = CONTRACT_VERSION,
) -> WorkflowEnvelope:
    """Construct one workflow envelope from a raw payload mapping."""

    return WorkflowEnvelope(
        kind=kind, contract_version=contract_version, payload=payload
    )


def parse_envelope(payload: Mapping[str, object]) -> WorkflowEnvelope:
    """Parse one raw payload mapping into a workflow envelope."""

    return WorkflowEnvelope.from_payload(payload)
