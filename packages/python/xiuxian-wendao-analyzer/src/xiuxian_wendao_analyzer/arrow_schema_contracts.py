"""Lightweight Arrow schema contract helpers for analyzer-owned adapters."""

from __future__ import annotations

from dataclasses import dataclass

import pyarrow as pa

WENDAO_TABLE_METADATA_KEY = "wendao.table"


@dataclass(frozen=True, slots=True)
class ArrowSchemaColumn:
    """One field in an analyzer Arrow table contract."""

    name: str
    data_type: pa.DataType
    nullable: bool = True

    def to_field(self) -> pa.Field:
        """Return the PyArrow field represented by this contract column."""

        return pa.field(self.name, self.data_type, nullable=self.nullable)


def build_arrow_schema(table_name: str, columns: tuple[ArrowSchemaColumn, ...]) -> pa.Schema:
    """Build a PyArrow schema with Wendao table metadata."""

    return pa.schema(
        [column.to_field() for column in columns],
        metadata={WENDAO_TABLE_METADATA_KEY: table_name},
    )


def schema_table_name(schema: pa.Schema) -> str | None:
    """Return the Wendao table name recorded in schema metadata, if present."""

    metadata = schema.metadata or {}
    value = metadata.get(WENDAO_TABLE_METADATA_KEY.encode("utf-8"))
    if value is None:
        return None
    return value.decode("utf-8")
