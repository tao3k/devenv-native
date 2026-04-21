"""Dataset reference contracts for workflow payloads."""

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
    require_int,
    require_str,
    validate_int_value,
    validate_optional_str_value,
    validate_str_value,
)

DATASET_REF_KIND = "qianji.dataset_ref"


@dataclass(frozen=True, slots=True)
class WorkflowDatasetRef(PayloadConvertible):
    """Stable reference to one workflow dataset."""

    KIND = DATASET_REF_KIND

    dataset_name: str
    source: str
    route: str | None
    schema_digest: str
    row_count: int
    column_count: int
    flight_endpoint: str | None = None
    ticket: str | None = None
    partition: JsonObject | Mapping[str, object] = field(default_factory=dict)
    window_start: str | None = None
    window_end: str | None = None
    metadata: JsonObject | Mapping[str, object] = field(default_factory=dict)

    def __post_init__(self) -> None:
        object.__setattr__(
            self,
            "dataset_name",
            validate_str_value(self.dataset_name, "dataset_name"),
        )
        object.__setattr__(self, "source", validate_str_value(self.source, "source"))
        object.__setattr__(
            self, "route", validate_optional_str_value(self.route, "route")
        )
        object.__setattr__(
            self,
            "schema_digest",
            validate_str_value(self.schema_digest, "schema_digest"),
        )
        object.__setattr__(
            self,
            "row_count",
            validate_int_value(self.row_count, "row_count", minimum=0),
        )
        object.__setattr__(
            self,
            "column_count",
            validate_int_value(self.column_count, "column_count", minimum=0),
        )
        object.__setattr__(
            self,
            "flight_endpoint",
            validate_optional_str_value(self.flight_endpoint, "flight_endpoint"),
        )
        object.__setattr__(
            self, "ticket", validate_optional_str_value(self.ticket, "ticket")
        )
        object.__setattr__(
            self,
            "window_start",
            validate_optional_str_value(self.window_start, "window_start"),
        )
        object.__setattr__(
            self,
            "window_end",
            validate_optional_str_value(self.window_end, "window_end"),
        )
        object.__setattr__(
            self,
            "partition",
            normalize_json_object(self.partition, field_name="partition"),
        )
        object.__setattr__(
            self,
            "metadata",
            normalize_json_object(self.metadata, field_name="metadata"),
        )

    def to_payload(self) -> JsonObject:
        return {
            "dataset_name": self.dataset_name,
            "source": self.source,
            "route": self.route,
            "schema_digest": self.schema_digest,
            "row_count": self.row_count,
            "column_count": self.column_count,
            "flight_endpoint": self.flight_endpoint,
            "ticket": self.ticket,
            "partition": self.partition,
            "window_start": self.window_start,
            "window_end": self.window_end,
            "metadata": self.metadata,
        }

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> WorkflowDatasetRef:
        return cls(
            dataset_name=require_str(payload, "dataset_name"),
            source=require_str(payload, "source"),
            route=optional_str(payload, "route"),
            schema_digest=require_str(payload, "schema_digest"),
            row_count=require_int(payload, "row_count"),
            column_count=require_int(payload, "column_count"),
            flight_endpoint=optional_str(payload, "flight_endpoint"),
            ticket=optional_str(payload, "ticket"),
            partition=payload.get("partition", {}),
            window_start=optional_str(payload, "window_start"),
            window_end=optional_str(payload, "window_end"),
            metadata=payload.get("metadata", {}),
        )
