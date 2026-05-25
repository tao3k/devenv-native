"""Audio diagnostic reference and prompt input helpers."""

from __future__ import annotations

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


def curated_reference_rows_from_org(path: Path) -> list[dict[str, object]]:
    """Convert an edited Org review checklist into curated JSONL rows."""

    entries = _load_reference_org_entries(path)
    rows: list[dict[str, object]] = []
    for index, entry in enumerate(entries, start=1):
        properties = entry["properties"]
        text = entry["referenceText"].strip()
        row: dict[str, object] = {
            "source": properties.get("SOURCE", ""),
            "sourceId": properties.get("SOURCE_ID", ""),
            "chunkIndex": properties.get("CHUNK_INDEX", ""),
            "startSeconds": properties.get("START_SECONDS", ""),
            "durationSeconds": properties.get("DURATION_SECONDS", ""),
            "referenceStatus": properties.get("REFERENCE_STATUS", ""),
            "text": text,
        }
        if entry["state"] != "DONE":
            raise ValueError(f"reference Org row {index} is not marked DONE")
        rows.append(curated_reference_row(row, line_number=index))
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


def _load_reference_org_entries(path: Path) -> list[dict[str, object]]:
    entries: list[dict[str, object]] = []
    current: dict[str, object] | None = None
    in_properties = False
    in_reference_text = False
    text_lines: list[str] = []
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.rstrip("\n")
        if line.startswith("** "):
            if current is not None:
                current["referenceText"] = "\n".join(text_lines).strip()
                entries.append(current)
            heading_parts = line.split(maxsplit=2)
            state = heading_parts[1] if len(heading_parts) > 1 else ""
            current = {"state": state, "properties": {}, "referenceText": ""}
            in_properties = False
            in_reference_text = False
            text_lines = []
            continue
        if current is None:
            continue
        stripped = line.strip()
        if stripped == ":PROPERTIES:":
            in_properties = True
            continue
        if stripped == ":END:" and in_properties:
            in_properties = False
            continue
        if in_properties:
            key, _, value = stripped[1:].partition(":")
            if stripped.startswith(":") and key:
                properties = current["properties"]
                assert isinstance(properties, dict)
                properties[key] = value.strip()
            continue
        if stripped.lower().startswith("#+begin_src") and "reference_text" in stripped:
            in_reference_text = True
            text_lines = []
            continue
        if stripped.lower().startswith("#+end_src") and in_reference_text:
            in_reference_text = False
            continue
        if in_reference_text:
            text_lines.append(line)
    if current is not None:
        current["referenceText"] = "\n".join(text_lines).strip()
        entries.append(current)
    if not entries:
        raise ValueError(f"reference Org has no review rows: {path}")
    return entries


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
