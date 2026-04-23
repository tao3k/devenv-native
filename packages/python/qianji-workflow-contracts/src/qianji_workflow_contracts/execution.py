"""Execution reference contracts for workflow checkpoints and status exchange."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping

from .base import (
    JsonObject,
    PayloadConvertible,
    normalize_json_object,
    optional_str,
    require_str,
    validate_optional_str_value,
    validate_str_value,
)

EXECUTION_REF_KIND = "qianji.execution.ref"
EXECUTION_STATUS_KIND = "qianji.execution.status"


@dataclass(frozen=True, slots=True)
class WorkflowExecutionRef(PayloadConvertible):
    """Stable reference to one workflow execution instance."""

    KIND = EXECUTION_REF_KIND

    instance_id: str
    process_id: str
    session_id: str | None = None
    checkpoint_id: str | None = None
    metadata: JsonObject | Mapping[str, object] = field(default_factory=dict)

    def __post_init__(self) -> None:
        object.__setattr__(
            self,
            "instance_id",
            validate_str_value(self.instance_id, "instance_id"),
        )
        object.__setattr__(
            self,
            "process_id",
            validate_str_value(self.process_id, "process_id"),
        )
        object.__setattr__(
            self,
            "session_id",
            validate_optional_str_value(self.session_id, "session_id"),
        )
        object.__setattr__(
            self,
            "checkpoint_id",
            validate_optional_str_value(self.checkpoint_id, "checkpoint_id"),
        )
        object.__setattr__(
            self,
            "metadata",
            normalize_json_object(self.metadata, field_name="metadata"),
        )

    def to_payload(self) -> JsonObject:
        return {
            "instance_id": self.instance_id,
            "process_id": self.process_id,
            "session_id": self.session_id,
            "checkpoint_id": self.checkpoint_id,
            "metadata": self.metadata,
        }

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> WorkflowExecutionRef:
        return cls(
            instance_id=require_str(payload, "instance_id"),
            process_id=require_str(payload, "process_id"),
            session_id=optional_str(payload, "session_id"),
            checkpoint_id=optional_str(payload, "checkpoint_id"),
            metadata=payload.get("metadata", {}),
        )


@dataclass(frozen=True, slots=True)
class WorkflowExecutionStatus(PayloadConvertible):
    """Stable status snapshot for one workflow execution instance."""

    KIND = EXECUTION_STATUS_KIND

    instance_id: str
    status: str
    updated_at: str
    active_node_id: str | None = None
    metadata: JsonObject | Mapping[str, object] = field(default_factory=dict)

    def __post_init__(self) -> None:
        object.__setattr__(
            self,
            "instance_id",
            validate_str_value(self.instance_id, "instance_id"),
        )
        object.__setattr__(self, "status", validate_str_value(self.status, "status"))
        object.__setattr__(
            self,
            "updated_at",
            validate_str_value(self.updated_at, "updated_at"),
        )
        object.__setattr__(
            self,
            "active_node_id",
            validate_optional_str_value(self.active_node_id, "active_node_id"),
        )
        object.__setattr__(
            self,
            "metadata",
            normalize_json_object(self.metadata, field_name="metadata"),
        )

    def to_payload(self) -> JsonObject:
        return {
            "instance_id": self.instance_id,
            "status": self.status,
            "updated_at": self.updated_at,
            "active_node_id": self.active_node_id,
            "metadata": self.metadata,
        }

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> WorkflowExecutionStatus:
        return cls(
            instance_id=require_str(payload, "instance_id"),
            status=require_str(payload, "status"),
            updated_at=require_str(payload, "updated_at"),
            active_node_id=optional_str(payload, "active_node_id"),
            metadata=payload.get("metadata", {}),
        )


__all__ = [
    "EXECUTION_REF_KIND",
    "EXECUTION_STATUS_KIND",
    "WorkflowExecutionRef",
    "WorkflowExecutionStatus",
]
