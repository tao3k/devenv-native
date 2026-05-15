"""Materialize private audio clips for reference curation."""

from __future__ import annotations

import csv
import json
import subprocess
from pathlib import Path
from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_media_probe import (
    audio_duration_seconds,
    resolve_ffmpeg_executable,
)
from xiuxian_wendao_analyzer.audio_diagnostic_report_writers import write_text

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence

REFERENCE_SELECTION_PACK_SCHEMA = "xiuxian_wendao.audio_reference_selection_pack.v1"


def materialize_reference_selection_pack(
    *,
    selection_jsonl: Path,
    clip_dir: Path,
    ffmpeg_path: str | None = None,
    force: bool = False,
) -> dict[str, object]:
    """Create private review clips for selected reference rows."""

    rows = _load_selection_rows(selection_jsonl)
    clip_dir.mkdir(parents=True, exist_ok=True)
    ffmpeg = ffmpeg_path or resolve_ffmpeg_executable()
    packed_rows: list[dict[str, object]] = []
    for row in rows:
        source = _resolve_source_path(row)
        chunk_index = _int_field(row, "chunkIndex")
        start_seconds = _float_field(row, "startSeconds")
        duration_seconds = _float_field(row, "durationSeconds")
        clip_path = (
            clip_dir
            / f"{source.stem.replace(' ', '-')[:48]}__chunk_{chunk_index:04d}.wav"
        )
        if force or not clip_path.exists():
            _run_ffmpeg_clip(
                ffmpeg,
                source=source,
                clip_path=clip_path,
                start_seconds=start_seconds,
                duration_seconds=duration_seconds,
            )
        packed_rows.append(
            {
                **dict(row),
                "clipPath": str(clip_path),
                "clipFormat": "wav",
            }
        )
    review_tsv = clip_dir / "reference_selection_review.tsv"
    _write_review_tsv(review_tsv, packed_rows)
    return {
        "schema": REFERENCE_SELECTION_PACK_SCHEMA,
        "selectionJsonl": str(selection_jsonl),
        "clipDir": str(clip_dir),
        "reviewTsv": str(review_tsv),
        "rows": len(packed_rows),
        "clips": [
            {
                "source": row.get("source", ""),
                "chunkIndex": row.get("chunkIndex", 0),
                "startSeconds": row.get("startSeconds", 0.0),
                "durationSeconds": row.get("durationSeconds", 0.0),
                "clipPath": row["clipPath"],
            }
            for row in packed_rows
        ],
    }


def validate_reference_selection_pack(
    *,
    review_tsv: Path,
    duration_tolerance_seconds: float = 0.75,
) -> dict[str, object]:
    """Validate that private review clips are ready for manual curation."""

    rows = _load_review_rows(review_tsv)
    issues: list[dict[str, object]] = []
    duplicate_keys = _duplicate_key_count(rows)
    candidate_draft_rows = 0
    curated_rows = 0
    for index, row in enumerate(rows, start=1):
        row_issues = _review_row_issues(
            row,
            duration_tolerance_seconds=duration_tolerance_seconds,
        )
        if row.get("referenceStatus") == "candidate-draft":
            candidate_draft_rows += 1
        if row.get("referenceStatus") == "curated":
            curated_rows += 1
        if row_issues:
            issues.append(
                {
                    "row": index,
                    "source": row.get("source", ""),
                    "chunkIndex": row.get("chunkIndex", ""),
                    "issues": row_issues,
                }
            )
    pack_ready = bool(rows) and duplicate_keys == 0 and not issues
    curated_ready = (
        pack_ready and candidate_draft_rows == 0 and curated_rows == len(rows)
    )
    return {
        "schema": "xiuxian_wendao.audio_reference_selection_pack_validation.v1",
        "reviewTsv": str(review_tsv),
        "rows": len(rows),
        "packReady": pack_ready,
        "curatedReady": curated_ready,
        "candidateDraftRows": candidate_draft_rows,
        "curatedRows": curated_rows,
        "duplicateKeys": duplicate_keys,
        "issueRows": len(issues),
        "issues": issues,
    }


def _load_selection_rows(path: Path) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for line_number, raw_line in enumerate(
        path.read_text(encoding="utf-8").splitlines(), start=1
    ):
        if not raw_line.strip():
            continue
        row = json.loads(raw_line)
        if not isinstance(row, dict):
            raise ValueError(f"reference selection row {line_number} must be an object")
        rows.append(row)
    return rows


def _load_review_rows(path: Path) -> list[dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        return [dict(row) for row in csv.DictReader(handle, dialect="excel-tab")]


def _resolve_source_path(row: Mapping[str, object]) -> Path:
    source = row.get("sourceId") or row.get("source")
    if not isinstance(source, str) or not source:
        raise ValueError("reference selection row is missing source/sourceId")
    path = Path(source)
    if not path.is_absolute():
        path = Path.cwd() / path
    if not path.exists():
        raise FileNotFoundError(f"reference selection source not found: {path}")
    return path


def _run_ffmpeg_clip(
    ffmpeg: str,
    *,
    source: Path,
    clip_path: Path,
    start_seconds: float,
    duration_seconds: float,
) -> None:
    command = [
        ffmpeg,
        "-hide_banner",
        "-nostdin",
        "-loglevel",
        "error",
        "-y",
        "-ss",
        f"{start_seconds:.3f}",
        "-t",
        f"{duration_seconds:.3f}",
        "-i",
        str(source),
        "-ac",
        "1",
        "-vn",
        "-c:a",
        "pcm_s16le",
        str(clip_path),
    ]
    result = subprocess.run(command, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(
            "ffmpeg reference clip materialization failed for "
            f"{source}: {result.stderr.strip()}"
        )


def _write_review_tsv(path: Path, rows: Sequence[Mapping[str, object]]) -> None:
    header = [
        "clipPath",
        "source",
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
        lines.append("\t".join(_tsv_cell(row.get(field, "")) for field in header))
    write_text(path, "\n".join(lines) + "\n")


def _review_row_issues(
    row: Mapping[str, str],
    *,
    duration_tolerance_seconds: float,
) -> list[str]:
    row_issues: list[str] = []
    clip_path = Path(row.get("clipPath", ""))
    if not clip_path.is_absolute():
        clip_path = Path.cwd() / clip_path
    if not clip_path.exists():
        row_issues.append("missing-clip")
    duration = _parse_float(row.get("durationSeconds", ""))
    if duration is None or duration <= 0:
        row_issues.append("invalid-duration")
    start = _parse_float(row.get("startSeconds", ""))
    if start is None or start < 0:
        row_issues.append("invalid-start")
    chunk_index = row.get("chunkIndex", "")
    if not chunk_index.isdigit():
        row_issues.append("invalid-chunk-index")
    if not row.get("source"):
        row_issues.append("missing-source")
    if not row.get("text", "").strip():
        row_issues.append("empty-text")
    if row.get("referenceStatus") not in {"candidate-draft", "curated"}:
        row_issues.append("invalid-reference-status")
    if clip_path.exists() and duration is not None and duration > 0:
        actual_duration = audio_duration_seconds(clip_path)
        if abs(actual_duration - duration) > duration_tolerance_seconds:
            row_issues.append("clip-duration-mismatch")
    return row_issues


def _duplicate_key_count(rows: Sequence[Mapping[str, str]]) -> int:
    keys = [(row.get("source", ""), row.get("chunkIndex", "")) for row in rows]
    return len(keys) - len(set(keys))


def _int_field(row: Mapping[str, object], field: str) -> int:
    value = row.get(field)
    if isinstance(value, int):
        return value
    raise ValueError(f"reference selection row has invalid {field}")


def _float_field(row: Mapping[str, object], field: str) -> float:
    value = row.get(field)
    if isinstance(value, bool):
        raise ValueError(f"reference selection row has invalid {field}")
    if isinstance(value, int | float):
        return float(value)
    raise ValueError(f"reference selection row has invalid {field}")


def _parse_float(value: str) -> float | None:
    try:
        return float(value)
    except ValueError:
        return None


def _tsv_cell(value: object) -> str:
    return str(value).replace("\t", " ").replace("\r", " ").replace("\n", "\\n")
