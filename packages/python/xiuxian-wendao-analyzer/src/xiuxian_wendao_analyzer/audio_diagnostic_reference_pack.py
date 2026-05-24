"""Materialize private audio clips for reference curation."""

from __future__ import annotations

import csv
import hashlib
import json
import subprocess
import time
import urllib.error
import urllib.request
from collections.abc import Mapping
from pathlib import Path
from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_media_probe import (
    audio_duration_seconds,
    resolve_ffmpeg_executable,
)
from xiuxian_wendao_analyzer.audio_diagnostic_metrics import character_error_rate
from xiuxian_wendao_analyzer.audio_diagnostic_openrouter import (
    build_openrouter_payload,
    build_openrouter_transcription_payload,
    extract_openrouter_transcript,
    is_openrouter_transcription_url,
)
from xiuxian_wendao_analyzer.audio_diagnostic_report_writers import write_text

if TYPE_CHECKING:
    from collections.abc import Callable, Sequence

REFERENCE_SELECTION_PACK_SCHEMA = "xiuxian_wendao.audio_reference_selection_pack.v1"
REFERENCE_SELECTION_MODEL_REVIEW_SCHEMA = "xiuxian_wendao.audio_reference_selection_model_review.v1"


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
        clip_path = clip_dir / f"{source.stem.replace(' ', '-')[:48]}__chunk_{chunk_index:04d}.wav"
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
    pending_row_summaries: list[dict[str, object]] = []
    curated_row_summaries: list[dict[str, object]] = []
    for index, row in enumerate(rows, start=1):
        row_issues = _review_row_issues(
            row,
            duration_tolerance_seconds=duration_tolerance_seconds,
        )
        if row.get("referenceStatus") == "candidate-draft":
            candidate_draft_rows += 1
            pending_row_summaries.append(_redacted_review_row_summary(index, row))
        if row.get("referenceStatus") == "curated":
            curated_rows += 1
            curated_row_summaries.append(_redacted_review_row_summary(index, row))
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
    curated_ready = pack_ready and candidate_draft_rows == 0 and curated_rows == len(rows)
    return {
        "schema": "xiuxian_wendao.audio_reference_selection_pack_validation.v1",
        "reviewTsv": str(review_tsv),
        "rows": len(rows),
        "packReady": pack_ready,
        "curatedReady": curated_ready,
        "candidateDraftRows": candidate_draft_rows,
        "curatedRows": curated_rows,
        "pendingRows": pending_row_summaries,
        "curatedRowSummaries": curated_row_summaries,
        "duplicateKeys": duplicate_keys,
        "issueRows": len(issues),
        "issues": issues,
    }


def model_review_reference_selection_pack(
    *,
    review_tsv: Path,
    api_key: str,
    model: str,
    base_url: str,
    prompt: str,
    max_tokens: int,
    temperature: float,
    timeout_seconds: int,
    max_candidate_to_model_cer: float = 0.15,
    request_sender: (
        Callable[[str, str, Mapping[str, object], int], Mapping[str, object]] | None
    ) = None,
) -> dict[str, object]:
    """Run a hosted model review over private reference clips without curating them."""

    rows = _load_review_rows(review_tsv)
    sender = request_sender or _send_openrouter_review_request
    reviewed_rows: list[dict[str, object]] = []
    succeeded_rows = 0
    failed_rows = 0
    model_consistent_rows = 0
    model_divergent_rows = 0
    latencies_ms: list[float] = []
    for index, row in enumerate(rows, start=1):
        row_issues = _review_row_issues(row, duration_tolerance_seconds=0.75)
        if row_issues:
            failed_rows += 1
            reviewed_rows.append(
                {
                    **_redacted_review_row_summary(index, row),
                    "modelReviewStatus": "failed",
                    "modelReviewError": ",".join(row_issues),
                }
            )
            continue
        started = time.perf_counter()
        try:
            response = sender(
                base_url,
                api_key,
                _build_model_review_payload(
                    row,
                    model=model,
                    base_url=base_url,
                    prompt=prompt,
                    max_tokens=max_tokens,
                    temperature=temperature,
                ),
                timeout_seconds,
            )
            model_text = extract_openrouter_transcript(response).strip()
            if not model_text:
                raise ValueError("empty model review transcript")
        except Exception as exc:
            failed_rows += 1
            reviewed_rows.append(
                {
                    **_redacted_review_row_summary(index, row),
                    "modelReviewStatus": "failed",
                    "modelReviewError": _short_error(exc),
                }
            )
            continue
        latency_ms = (time.perf_counter() - started) * 1000
        latencies_ms.append(latency_ms)
        candidate_text = row.get("text", "").strip()
        candidate_to_model_cer = character_error_rate(candidate_text, model_text)
        model_consistent = (
            candidate_to_model_cer is not None
            and candidate_to_model_cer <= max_candidate_to_model_cer
        )
        succeeded_rows += 1
        if model_consistent:
            model_consistent_rows += 1
        else:
            model_divergent_rows += 1
        reviewed_rows.append(
            {
                **_redacted_review_row_summary(index, row),
                "modelReviewStatus": (
                    "model-consistent" if model_consistent else "needs-human-review"
                ),
                "model": model,
                "candidateToModelCer": candidate_to_model_cer,
                "modelTextCharCount": len(model_text),
                "modelTextSha256": hashlib.sha256(model_text.encode("utf-8")).hexdigest(),
                "latencyMs": round(latency_ms, 3),
            }
        )
    return {
        "schema": REFERENCE_SELECTION_MODEL_REVIEW_SCHEMA,
        "reviewTsv": str(review_tsv),
        "provider": "openrouter",
        "model": model,
        "baseUrl": base_url,
        "rows": len(rows),
        "succeededRows": succeeded_rows,
        "failedRows": failed_rows,
        "modelConsistentRows": model_consistent_rows,
        "modelDivergentRows": model_divergent_rows,
        "maxCandidateToModelCer": max_candidate_to_model_cer,
        "latencyMsP50": _percentile(latencies_ms, 0.50),
        "latencyMsP95": _percentile(latencies_ms, 0.95),
        "rowsReviewed": reviewed_rows,
        "promotionSafety": {
            "createsCuratedReferences": False,
            "requiresHumanCuratedReferenceText": True,
        },
    }


def _load_selection_rows(path: Path) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
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
            f"ffmpeg reference clip materialization failed for {source}: {result.stderr.strip()}"
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


def _build_model_review_payload(
    row: Mapping[str, str],
    *,
    model: str,
    base_url: str,
    prompt: str,
    max_tokens: int,
    temperature: float,
) -> dict[str, object]:
    clip_path = Path(row.get("clipPath", ""))
    audio_format = clip_path.suffix.lower().lstrip(".") or "wav"
    audio_bytes = clip_path.read_bytes()
    if is_openrouter_transcription_url(base_url):
        return build_openrouter_transcription_payload(
            model=model,
            audio_bytes=audio_bytes,
            audio_format=audio_format,
        )
    return build_openrouter_payload(
        model=model,
        prompt=prompt,
        audio_bytes=audio_bytes,
        audio_format=audio_format,
        max_tokens=max_tokens,
        temperature=temperature,
    )


def _send_openrouter_review_request(
    base_url: str,
    api_key: str,
    payload: Mapping[str, object],
    timeout_seconds: int,
) -> Mapping[str, object]:
    request = urllib.request.Request(
        base_url,
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
            "HTTP-Referer": "https://github.com/tao3k/xiuxian-artisan-workshop",
            "X-Title": "Wendao audio reference model review",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
            parsed = json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        error_body = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"OpenRouter HTTP {exc.code}: {error_body}") from exc
    if not isinstance(parsed, Mapping):
        raise ValueError("OpenRouter model review response is not an object")
    return parsed


def _percentile(values: Sequence[float], percentile: float) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    index = min(len(ordered) - 1, max(0, round((len(ordered) - 1) * percentile)))
    return round(ordered[index], 3)


def _short_error(exc: BaseException) -> str:
    text = str(exc).strip()
    return text[:240] if text else exc.__class__.__name__


def _redacted_review_row_summary(
    row_number: int,
    row: Mapping[str, str],
) -> dict[str, object]:
    text = row.get("text", "")
    return {
        "row": row_number,
        "clipPath": row.get("clipPath", ""),
        "source": row.get("source", ""),
        "chunkIndex": _int_or_string(row.get("chunkIndex", "")),
        "startSeconds": _float_or_string(row.get("startSeconds", "")),
        "durationSeconds": _float_or_string(row.get("durationSeconds", "")),
        "reviewStatus": row.get("reviewStatus", ""),
        "selectionReason": row.get("selectionReason", ""),
        "referenceStatus": row.get("referenceStatus", ""),
        "textCharCount": len(text),
        "textSha256": hashlib.sha256(text.encode("utf-8")).hexdigest() if text else "",
    }


def _int_or_string(value: str) -> int | str:
    return int(value) if value.isdigit() else value


def _float_or_string(value: str) -> float | str:
    parsed = _parse_float(value)
    return parsed if parsed is not None else value


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
