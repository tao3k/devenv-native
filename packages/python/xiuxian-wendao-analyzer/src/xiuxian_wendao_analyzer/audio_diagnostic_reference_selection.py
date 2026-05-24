"""Select high-value rows for audio reference curation."""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path
from typing import TYPE_CHECKING, Any

from xiuxian_wendao_analyzer.audio_diagnostic_report_writers import (
    write_jsonl,
    write_text,
)

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

REFERENCE_SELECTION_SCHEMA = "xiuxian_wendao.audio_reference_selection.v1"


def select_reference_draft_report(
    *,
    draft_jsonl: Path,
    limit: int,
    quality_json: Path | None = None,
    selected_jsonl: Path | None = None,
    selected_tsv: Path | None = None,
) -> dict[str, object]:
    """Select the most useful draft rows for manual CER curation."""

    rows = _load_draft_rows(draft_jsonl)
    if quality_json is not None:
        rows = _apply_quality_overrides(rows, quality_json)
    selected = select_reference_rows(rows, limit=limit)
    if selected_jsonl is not None:
        write_jsonl(selected_jsonl, selected)
    if selected_tsv is not None:
        _write_selection_tsv(selected_tsv, selected)
    return {
        "schema": REFERENCE_SELECTION_SCHEMA,
        "draftJsonl": str(draft_jsonl),
        "qualityJson": "" if quality_json is None else str(quality_json),
        "selectedJsonl": "" if selected_jsonl is None else str(selected_jsonl),
        "selectedTsv": "" if selected_tsv is None else str(selected_tsv),
        "totalRows": len(rows),
        "selectedRows": len(selected),
        "limit": limit,
        "reviewStatusCounts": dict(
            sorted(Counter(str(row.get("reviewStatus", "")) for row in rows).items())
        ),
        "selectionReasonCounts": dict(
            sorted(
                Counter(
                    reason
                    for row in selected
                    for reason in str(row.get("selectionReason", "")).split("|")
                    if reason
                ).items()
            )
        ),
        "selected": selected,
    }


def select_reference_rows(
    rows: Sequence[Mapping[str, object]],
    *,
    limit: int,
) -> list[dict[str, object]]:
    """Return priority rows plus an even timeline spread for reference curation."""

    if limit <= 0:
        raise ValueError("reference selection limit must be positive")
    sorted_rows = sorted(rows, key=lambda row: (_float(row, "startSeconds"), _index(row)))
    selected: dict[int, dict[str, object]] = {}
    for row in sorted_rows:
        reason = _priority_reason(row)
        if reason and len(selected) < limit:
            _add_selection(selected, row, reason)
    remaining_slots = max(0, limit - len(selected))
    spread_candidates = [row for row in sorted_rows if _index(row) not in selected]
    for row in _evenly_spaced_rows(spread_candidates, remaining_slots):
        _add_selection(selected, row, "timeline-spread")
    remaining_slots = max(0, limit - len(selected))
    for row in sorted_rows:
        if remaining_slots <= 0:
            break
        if _index(row) in selected:
            continue
        _add_selection(selected, row, "timeline-fill")
        remaining_slots -= 1
    return sorted(selected.values(), key=lambda row: (_float(row, "startSeconds"), _index(row)))


def _load_draft_rows(path: Path) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not raw_line.strip():
            continue
        row = json.loads(raw_line)
        if not isinstance(row, dict):
            raise ValueError(f"reference draft row {line_number} must be an object")
        rows.append(row)
    return rows


def _apply_quality_overrides(
    rows: Sequence[Mapping[str, object]], quality_json: Path
) -> list[dict[str, object]]:
    payload = json.loads(quality_json.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"quality report must be a JSON object: {quality_json}")
    quality_rows = payload.get("qualityRows")
    if not isinstance(quality_rows, list):
        raise ValueError(f"quality report is missing qualityRows: {quality_json}")
    status_by_key: dict[tuple[str, int], str] = {}
    for row in quality_rows:
        if not isinstance(row, dict):
            continue
        source = str(row.get("source", ""))
        chunk_index = row.get("chunk_index")
        status = row.get("review_status")
        if isinstance(chunk_index, int) and isinstance(status, str):
            status_by_key[(Path(source).name, chunk_index)] = status
    updated: list[dict[str, object]] = []
    for row in rows:
        item = dict(row)
        key = (str(item.get("source", "")), _index(item))
        status = status_by_key.get(key)
        if status is not None:
            item["reviewStatus"] = status
        updated.append(item)
    return updated


def _priority_reason(row: Mapping[str, object]) -> str:
    status = str(row.get("reviewStatus", ""))
    if status == "short-utterance-review":
        return "short-utterance"
    if status.startswith("weak-"):
        return "weak-quality"
    if status in {"failed", "reference-fail", "required-term-miss"}:
        return status
    return ""


def _evenly_spaced_rows(
    rows: Sequence[Mapping[str, object]],
    limit: int,
) -> list[Mapping[str, object]]:
    if limit <= 0 or not rows:
        return []
    if limit >= len(rows):
        return list(rows)
    if limit == 1:
        return [rows[len(rows) // 2]]
    indexes = {round(index * (len(rows) - 1) / (limit - 1)) for index in range(limit)}
    return [rows[index] for index in sorted(indexes)]


def _add_selection(
    selected: dict[int, dict[str, object]],
    row: Mapping[str, object],
    reason: str,
) -> None:
    key = _index(row)
    item = selected.setdefault(key, dict(row))
    reasons = {part for part in str(item.get("selectionReason", "")).split("|") if part}
    reasons.add(reason)
    item["selectionReason"] = "|".join(sorted(reasons))


def _write_selection_tsv(path: Path, rows: Sequence[Mapping[str, object]]) -> None:
    header = [
        "source",
        "sourceId",
        "chunkIndex",
        "startSeconds",
        "durationSeconds",
        "reviewStatus",
        "selectionReason",
        "referenceStatus",
        "text",
    ]
    lines = ["\t".join(header)]
    for row in rows:
        lines.append("\t".join(_tsv_cell(row.get(name, "")) for name in header))
    write_text(path, "\n".join(lines) + "\n")


def _tsv_cell(value: object) -> str:
    return str(value).replace("\t", " ").replace("\r", " ").replace("\n", "\\n")


def _index(row: Mapping[str, object]) -> int:
    value = row.get("chunkIndex")
    return value if isinstance(value, int) else 0


def _float(row: Mapping[str, object], key: str) -> float:
    value: Any = row.get(key)
    if isinstance(value, bool):
        return 0.0
    if isinstance(value, int | float):
        return float(value)
    return 0.0
