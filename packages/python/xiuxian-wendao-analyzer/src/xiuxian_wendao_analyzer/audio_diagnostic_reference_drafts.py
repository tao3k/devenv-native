"""Audio diagnostic reference draft writers."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_quality import read_transcript
from xiuxian_wendao_analyzer.audio_diagnostic_reference_inputs import (
    REFERENCE_STATUS_CANDIDATE_DRAFT,
)
from xiuxian_wendao_analyzer.audio_diagnostic_report_writers import (
    write_jsonl,
    write_text,
)

if TYPE_CHECKING:
    from collections.abc import Sequence

    from xiuxian_wendao_analyzer.audio_diagnostic_quality import QualityRow


def reference_draft_rows(rows: Sequence[QualityRow]) -> list[dict[str, object]]:
    """Build editable reference rows prefilled from diagnostic transcripts."""

    draft_rows: list[dict[str, object]] = []
    for row in rows:
        draft_rows.append(
            {
                "source": Path(row.source).name,
                "sourceId": row.source,
                "chunkIndex": row.chunk_index,
                "startSeconds": row.start_seconds,
                "durationSeconds": row.duration_seconds,
                "backend": row.backend,
                "model": row.model,
                "reviewStatus": row.review_status,
                "referenceStatus": REFERENCE_STATUS_CANDIDATE_DRAFT,
                "text": read_transcript(row.transcript_path),
            }
        )
    return draft_rows


def write_reference_draft_jsonl(path: Path, rows: Sequence[QualityRow]) -> None:
    """Write editable reference JSONL rows for manual transcript correction."""

    write_jsonl(path, reference_draft_rows(rows))


def write_reference_draft_tsv(path: Path, rows: Sequence[QualityRow]) -> None:
    """Write editable reference TSV rows for manual transcript correction."""

    header = [
        "source",
        "sourceId",
        "chunkIndex",
        "startSeconds",
        "durationSeconds",
        "referenceStatus",
        "text",
    ]
    lines = ["\t".join(header)]
    for row in reference_draft_rows(rows):
        values = [
            str(row["source"]),
            str(row["sourceId"]),
            str(row["chunkIndex"]),
            str(row["startSeconds"]),
            str(row["durationSeconds"]),
            str(row["referenceStatus"]),
            str(row["text"]).replace("\t", " ").replace("\r", " ").replace("\n", "\\n"),
        ]
        lines.append("\t".join(values))
    write_text(path, "\n".join(lines) + "\n")
