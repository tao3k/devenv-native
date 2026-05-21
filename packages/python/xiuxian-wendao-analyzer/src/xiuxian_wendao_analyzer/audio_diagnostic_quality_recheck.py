"""Recompute audio diagnostic quality gates from persisted ASR results."""

from __future__ import annotations

import json
from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_quality import build_quality_rows
from xiuxian_wendao_analyzer.audio_diagnostic_quality_inputs import (
    load_reference_transcripts,
    load_term_list,
    reference_candidate_draft_row_count,
)
from xiuxian_wendao_analyzer.audio_diagnostic_quality_summary import (
    summarize_precision_gate,
    summarize_quality,
    summarize_reference_subset,
    summarize_timeline_structure,
)
from xiuxian_wendao_analyzer.audio_diagnostic_results import (
    AsrResult,
    summarize_results,
)

if TYPE_CHECKING:
    import argparse


def recheck_quality_summary(args: argparse.Namespace) -> dict[str, object]:
    """Recompute quality and precision summaries without running ASR."""

    summary_path = Path(args.recheck_quality_summary_json)
    summary = _read_json_object(summary_path)
    results_path = (
        Path(args.recheck_quality_results_json)
        if args.recheck_quality_results_json is not None
        else summary_path.parent / "results.json"
    )
    results = _read_results(results_path)
    references = load_reference_transcripts(args.reference_jsonl)
    required_terms = load_term_list(args.required_terms_file)
    reference_candidate_draft_rows = reference_candidate_draft_row_count(
        args.reference_jsonl
    )
    quality_rows = build_quality_rows(
        results,
        references=references,
        max_reference_cer=args.max_reference_cer,
        required_terms=required_terms,
        min_required_term_recall=args.min_required_term_recall,
        min_chars_per_minute=args.min_chars_per_minute,
        min_chinese_ratio=args.min_chinese_ratio,
        max_inaudible_per_minute=args.max_inaudible_per_minute,
        max_repeated_ngram_ratio=args.max_repeated_ngram_ratio,
    )
    return {
        **summary,
        **summarize_results(results),
        **summarize_quality(quality_rows),
        **summarize_reference_subset(quality_rows),
        **summarize_timeline_structure(
            quality_rows,
            allow_planned_gaps=summary.get("sampleStrategy") == "speech-segments",
        ),
        **summarize_precision_gate(
            quality_rows,
            reference_configured=bool(references),
            reference_candidate_draft_rows=reference_candidate_draft_rows,
            max_reference_cer=args.max_reference_cer,
            required_terms_configured=bool(required_terms),
        ),
        "qualityRecheckedAt": datetime.now(tz=UTC).isoformat(),
        "qualityRecheckSourceSummaryPath": str(summary_path),
        "qualityRecheckResultsPath": str(results_path),
        "qualityRows": [row.__dict__ for row in quality_rows],
    }


def _read_json_object(path: Path) -> dict[str, object]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"expected JSON object: {path}")
    return payload


def _read_results(path: Path) -> list[AsrResult]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, list):
        raise ValueError(f"expected JSON array: {path}")
    rows = [row for row in payload if isinstance(row, dict)]
    if len(rows) != len(payload):
        raise ValueError(f"result rows must be JSON objects: {path}")
    return [AsrResult(**row) for row in rows]
