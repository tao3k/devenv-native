"""Audio diagnostic report writer facade."""

from __future__ import annotations

from xiuxian_wendao_analyzer.audio_diagnostic_reference_drafts import (
    reference_draft_rows,
    write_reference_draft_jsonl,
)
from xiuxian_wendao_analyzer.audio_diagnostic_report_writers import (
    write_jsonl,
    write_text,
)
from xiuxian_wendao_analyzer.audio_diagnostic_timeline_reporting import (
    format_srt_timestamp,
    format_vtt_timestamp,
    timeline_review_rows,
    write_transcript_timeline_jsonl,
    write_transcript_timeline_org,
    write_transcript_timeline_srt,
    write_transcript_timeline_vtt,
)

__all__ = [
    "format_srt_timestamp",
    "format_vtt_timestamp",
    "reference_draft_rows",
    "timeline_review_rows",
    "write_jsonl",
    "write_reference_draft_jsonl",
    "write_text",
    "write_transcript_timeline_jsonl",
    "write_transcript_timeline_org",
    "write_transcript_timeline_srt",
    "write_transcript_timeline_vtt",
]
