"""Audio diagnostic TSV and JSONL report writers."""

from __future__ import annotations

import json
from pathlib import Path
from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_quality import QualityRow, read_transcript

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


def write_text(path: Path, text: str) -> None:
    """Write UTF-8 text, creating parents."""

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def write_jsonl(path: Path, rows: Sequence[Mapping[str, object]]) -> None:
    """Write UTF-8 JSONL rows."""

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        "".join(
            json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n" for row in rows
        ),
        encoding="utf-8",
    )


def write_quality_tsv(path: Path, rows: Sequence[QualityRow]) -> None:
    """Write a compact TSV for human precision review."""

    header = [
        "backend",
        "source",
        "chunkIndex",
        "startSeconds",
        "status",
        "reviewStatus",
        "model",
        "transcriptChars",
        "charsPerMinute",
        "chineseRatio",
        "inaudiblePerMinute",
        "repeatedNgramRatio",
        "referenceCer",
        "requiredTermRecall",
        "missingRequiredTerms",
        "transcriptPath",
        "error",
    ]
    lines = ["\t".join(header)]
    for row in rows:
        lines.append("\t".join(_quality_tsv_values(row)))
    write_text(path, "\n".join(lines) + "\n")


def write_transcript_review_tsv(path: Path, rows: Sequence[QualityRow]) -> None:
    """Write transcript text beside timing metadata for private evidence review."""

    header = [
        "backend",
        "source",
        "chunkIndex",
        "startSeconds",
        "endSeconds",
        "status",
        "reviewStatus",
        "referenceCer",
        "requiredTermRecall",
        "repeatedNgramRatio",
        "text",
    ]
    lines = ["\t".join(header)]
    for row in rows:
        lines.append("\t".join(_transcript_review_values(row)))
    write_text(path, "\n".join(lines) + "\n")


def _quality_tsv_values(row: QualityRow) -> list[str]:
    return [
        row.backend,
        row.source,
        str(row.chunk_index),
        f"{row.start_seconds:.3f}",
        row.status,
        row.review_status,
        row.model,
        str(row.transcript_chars),
        f"{row.chars_per_minute:.3f}",
        "" if row.chinese_ratio is None else f"{row.chinese_ratio:.6f}",
        f"{row.inaudible_per_minute:.3f}",
        f"{row.repeated_ngram_ratio:.6f}",
        "" if row.reference_cer is None else f"{row.reference_cer:.6f}",
        "" if row.required_term_recall is None else f"{row.required_term_recall:.6f}",
        row.missing_required_terms,
        row.transcript_path,
        row.error.replace("\t", " ").replace("\n", " "),
    ]


def _transcript_review_values(row: QualityRow) -> list[str]:
    transcript = read_transcript(row.transcript_path)
    return [
        row.backend,
        Path(row.source).name,
        str(row.chunk_index),
        f"{row.start_seconds:.3f}",
        f"{row.start_seconds + row.duration_seconds:.3f}",
        row.status,
        row.review_status,
        "" if row.reference_cer is None else f"{row.reference_cer:.6f}",
        "" if row.required_term_recall is None else f"{row.required_term_recall:.6f}",
        f"{row.repeated_ngram_ratio:.6f}",
        transcript.replace("\t", " ").replace("\r", " ").replace("\n", "\\n"),
    ]
