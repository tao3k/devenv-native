"""Audio diagnostic reference and prompt input helpers."""

from __future__ import annotations

import csv
import json
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

REFERENCE_STATUS_CANDIDATE_DRAFT = "candidate-draft"
REFERENCE_STATUS_CURATED = "curated"


def load_reference_transcripts(path: Path | None) -> dict[tuple[str, int], str]:
    """Load optional JSONL references keyed by source basename and chunk index."""

    if path is None:
        return {}
    references: dict[tuple[str, int], str] = {}
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not raw_line.strip():
            continue
        row = json.loads(raw_line)
        source = row.get("source")
        chunk_index = row.get("chunkIndex", row.get("chunk_index"))
        text = row.get("text")
        if (
            not isinstance(source, str)
            or not isinstance(chunk_index, int)
            or not isinstance(text, str)
        ):
            raise ValueError(f"invalid reference row at line {line_number}")
        references[(Path(source).name, chunk_index)] = text
    return references


def reference_candidate_draft_row_count(path: Path | None) -> int:
    """Count candidate-draft reference rows that are unsafe for promotion."""

    if path is None:
        return 0
    count = 0
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        if not raw_line.strip():
            continue
        row = json.loads(raw_line)
        if not isinstance(row, dict):
            continue
        if is_curated_reference_row(row):
            continue
        if row.get("referenceStatus") == REFERENCE_STATUS_CANDIDATE_DRAFT:
            count += 1
            continue
        if any(field in row for field in ("backend", "model", "reviewStatus")):
            count += 1
    return count


def curated_reference_rows_from_draft(path: Path) -> list[dict[str, object]]:
    """Convert an edited reference draft into promotion-safe curated rows."""

    rows: list[dict[str, object]] = []
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not raw_line.strip():
            continue
        value = json.loads(raw_line)
        if not isinstance(value, dict):
            raise ValueError(f"invalid reference draft row at line {line_number}")
        rows.append(curated_reference_row(value, line_number=line_number))
    return rows


def curated_reference_rows_from_tsv(path: Path) -> list[dict[str, object]]:
    """Convert an edited reference draft TSV into curated JSONL rows."""

    rows: list[dict[str, object]] = []
    with path.open("r", encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, dialect="excel-tab")
        for line_number, row in enumerate(reader, start=2):
            rows.append(curated_reference_row(row, line_number=line_number))
    return rows


def curated_reference_row(
    row: Mapping[str, object],
    *,
    line_number: int,
) -> dict[str, object]:
    """Return one promotion-safe curated reference row."""

    if not _explicitly_curated_reference(row):
        raise ValueError(f"reference row at line {line_number} is not curated")
    source = row.get("source")
    chunk_index = row.get("chunkIndex", row.get("chunk_index"))
    text = row.get("text")
    if isinstance(chunk_index, str) and chunk_index.isdigit():
        chunk_index = int(chunk_index)
    if not isinstance(source, str) or not isinstance(chunk_index, int) or not isinstance(text, str):
        raise ValueError(f"invalid reference draft row at line {line_number}")
    text = text.strip()
    if not text:
        raise ValueError(f"empty reference text at line {line_number}")
    curated: dict[str, object] = {
        "source": Path(source).name,
        "chunkIndex": chunk_index,
        "referenceStatus": REFERENCE_STATUS_CURATED,
        "text": text,
    }
    source_id = row.get("sourceId", row.get("source_id"))
    if isinstance(source_id, str) and source_id:
        curated["sourceId"] = source_id
    for field in ("startSeconds", "durationSeconds"):
        value = row.get(field)
        if isinstance(value, str) and value:
            try:
                value = float(value)
            except ValueError:
                value = None
        if isinstance(value, int | float):
            curated[field] = value
    return curated


def is_curated_reference_row(row: dict[str, object]) -> bool:
    """Return whether a reference row is explicitly curated."""

    return _explicitly_curated_reference(row)


def _explicitly_curated_reference(row: Mapping[str, object]) -> bool:
    """Return whether a row carries an explicit human-curated marker."""

    return row.get("referenceStatus") == REFERENCE_STATUS_CURATED or row.get("curated") is True


def load_term_list(path: Path | None) -> list[str]:
    """Load one precision term per line, ignoring comments and blank lines."""

    if path is None:
        return []
    terms: list[str] = []
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        terms.append(line)
    return terms


def prompt_with_domain_terms(prompt: str, terms: Sequence[str]) -> str:
    """Append domain terms to the transcription prompt when provided."""

    if not terms:
        return prompt
    joined_terms = "、".join(terms)
    return (
        f"{prompt}\n\nDomain vocabulary that may appear in the audio: "
        f"{joined_terms}. Preserve these terms exactly when heard."
    )


def read_transcript(path: str) -> str:
    """Read a transcript path when present."""

    return Path(path).read_text(encoding="utf-8") if path else ""
