"""Arrow timing sidecars for Docling document extraction."""

from __future__ import annotations

import time
from contextlib import contextmanager
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import TYPE_CHECKING

import pyarrow as pa

if TYPE_CHECKING:
    from collections.abc import Iterator, Mapping

DOCUMENT_TIMING_ARROW_CACHE_NAME = "_document_metrics.arrow"
DOCUMENT_TIMING_SCHEMA_VERSION = "xiuxian_wendao.document_timing.v1"

DOCUMENT_TIMING_SCHEMA = pa.schema(
    [
        pa.field("contractVersion", pa.utf8()),
        pa.field("sourcePath", pa.utf8()),
        pa.field("sourceSuffix", pa.utf8()),
        pa.field("phase", pa.utf8()),
        pa.field("elapsedMs", pa.float64()),
        pa.field("status", pa.utf8()),
        pa.field("detail", pa.utf8()),
        pa.field("resourceRows", pa.int32()),
        pa.field("structureRows", pa.int32()),
    ]
)


@dataclass(frozen=True)
class DocumentTimingRow:
    """One phase timing row for a full-document extraction cache miss."""

    contractVersion: str
    sourcePath: str
    sourceSuffix: str
    phase: str
    elapsedMs: float
    status: str
    detail: str
    resourceRows: int
    structureRows: int


class DocumentTimingRecorder:
    """Collect phase timings for a single document conversion."""

    def __init__(self, source_path: str | Path) -> None:
        self._source = Path(source_path)
        self._started = time.perf_counter()
        self._rows: list[DocumentTimingRow] = []
        self._finished = False

    @contextmanager
    def phase(self, name: str) -> Iterator[None]:
        """Measure one phase and append a timing row.

        # Errors

        Re-raises any exception raised inside the measured block after recording
        an `error` timing row.
        """

        started = time.perf_counter()
        status = "ok"
        detail = ""
        try:
            yield
        except Exception as exc:
            status = "error"
            detail = str(exc)
            raise
        finally:
            self.record(
                name,
                _elapsed_ms(started),
                status=status,
                detail=detail,
            )

    def record(
        self,
        phase: str,
        elapsed_ms: float,
        *,
        status: str = "ok",
        detail: str = "",
        resource_rows: int = 0,
        structure_rows: int = 0,
    ) -> None:
        """Append one timing row."""

        self._rows.append(
            DocumentTimingRow(
                contractVersion=DOCUMENT_TIMING_SCHEMA_VERSION,
                sourcePath=str(self._source),
                sourceSuffix=self._source.suffix.lower(),
                phase=phase,
                elapsedMs=max(float(elapsed_ms), 0.0),
                status=status,
                detail=_trim_detail(detail),
                resourceRows=_int32_non_negative(resource_rows),
                structureRows=_int32_non_negative(structure_rows),
            )
        )

    def finish(
        self,
        *,
        status: str,
        detail: str = "",
        resource_rows: int = 0,
        structure_rows: int = 0,
    ) -> None:
        """Record the total extraction timing once."""

        if self._finished:
            return
        self._finished = True
        self.record(
            "total",
            _elapsed_ms(self._started),
            status=status,
            detail=detail,
            resource_rows=resource_rows,
            structure_rows=structure_rows,
        )

    @property
    def rows(self) -> list[DocumentTimingRow]:
        """Return collected timing rows."""

        return list(self._rows)


def document_timing_to_table(
    rows: list[DocumentTimingRow | Mapping[str, object]],
) -> pa.Table:
    """Convert timing rows to the stable Arrow schema."""

    return pa.Table.from_pylist(
        [
            asdict(row) if isinstance(row, DocumentTimingRow) else dict(row)
            for row in rows
        ],
        schema=DOCUMENT_TIMING_SCHEMA,
    )


def write_document_timing_cache(
    output_dir: str | Path,
    rows: list[DocumentTimingRow | Mapping[str, object]],
) -> None:
    """Write timing rows into `_document_metrics.arrow`.

    # Errors

    Raises filesystem or Arrow IPC errors when the sidecar cannot be written.
    """

    path = Path(output_dir) / DOCUMENT_TIMING_ARROW_CACHE_NAME
    table = document_timing_to_table(rows)
    with pa.ipc.new_file(path, DOCUMENT_TIMING_SCHEMA) as writer:
        writer.write_table(table)


def _elapsed_ms(started: float) -> float:
    return (time.perf_counter() - started) * 1000.0


def _trim_detail(value: str) -> str:
    if len(value) <= 500:
        return value
    return value[:497] + "..."


def _int32_non_negative(value: int) -> int:
    return max(0, min(int(value), 2_147_483_647))
