"""Run bounded MP3 ASR diagnostics through Docling and hosted audio backends."""

# ruff: noqa: F401

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Sequence

from xiuxian_wendao_analyzer.audio_diagnostic_backends import run_backend
from xiuxian_wendao_analyzer.audio_diagnostic_candidate_compare import (
    compare_audio_candidate_summaries,
)
from xiuxian_wendao_analyzer.audio_diagnostic_docling import transcribe_local_docling
from xiuxian_wendao_analyzer.audio_diagnostic_firered import transcribe_fireredasr2s
from xiuxian_wendao_analyzer.audio_diagnostic_identity import (
    AUDIO_MATERIALIZATION_NATIVE_RATE_WAV,
    AUDIO_MATERIALIZATION_NORMALIZED_16K_WAV,
    AUDIO_MATERIALIZATION_SOURCE_DIRECT,
    AUDIO_SHARD_MANIFEST_SCHEMA,
    DEFAULT_AUDIO_SHARD_PROFILE,
    SUPPORTED_AUDIO_MATERIALIZATION_MODES,
    AudioChunk,
    SpeechSegment,
    audio_shard_manifest,
    safe_file_stem,
    truth_template_rows,
)
from xiuxian_wendao_analyzer.audio_diagnostic_identity import (
    audio_shard_cache_key as audio_shard_cache_key,
)
from xiuxian_wendao_analyzer.audio_diagnostic_identity import (
    build_audio_shard_manifest_item as build_audio_shard_manifest_item,
)
from xiuxian_wendao_analyzer.audio_diagnostic_materialization import (
    materialize_audio_chunks,
)
from xiuxian_wendao_analyzer.audio_diagnostic_media_probe import (
    audio_duration_seconds,
    audio_stream_info,
    ensure_ffmpeg_on_path,
)
from xiuxian_wendao_analyzer.audio_diagnostic_openrouter import (
    build_openrouter_payload,
    extract_openrouter_segments,
    extract_openrouter_transcript,
    transcribe_openrouter,
)
from xiuxian_wendao_analyzer.audio_diagnostic_parser import build_parser
from xiuxian_wendao_analyzer.audio_diagnostic_paths import (
    PRIVATE_INPUT_PRIVACY,
    SHAREABLE_INPUT_PRIVACY,
    default_output_dir,
    discover_audio_sources,
    resolve_openrouter_api_key,
    validate_private_output_dir,
)
from xiuxian_wendao_analyzer.audio_diagnostic_quality import (
    QualityRow,
    build_quality_rows,
)
from xiuxian_wendao_analyzer.audio_diagnostic_quality_inputs import (
    REFERENCE_STATUS_CANDIDATE_DRAFT,
    REFERENCE_STATUS_CURATED,
    curated_reference_rows_from_draft,
    curated_reference_rows_from_tsv,
    load_reference_transcripts,
    load_term_list,
    prompt_with_domain_terms,
    reference_candidate_draft_row_count,
    validate_reference_jsonl,
)
from xiuxian_wendao_analyzer.audio_diagnostic_quality_recheck import (
    recheck_quality_summary,
)
from xiuxian_wendao_analyzer.audio_diagnostic_quality_summary import (
    summarize_precision_gate,
    summarize_quality,
    summarize_reference_subset,
)
from xiuxian_wendao_analyzer.audio_diagnostic_reference_pack import (
    materialize_reference_selection_pack,
    validate_reference_selection_pack,
)
from xiuxian_wendao_analyzer.audio_diagnostic_reference_selection import (
    select_reference_draft_report,
    select_reference_rows,
)
from xiuxian_wendao_analyzer.audio_diagnostic_reporting import (
    reference_draft_rows,
    write_jsonl,
    write_quality_tsv,
    write_reference_draft_jsonl,
    write_reference_draft_tsv,
    write_transcript_review_tsv,
    write_transcript_timeline_jsonl,
    write_transcript_timeline_srt,
    write_transcript_timeline_vtt,
)
from xiuxian_wendao_analyzer.audio_diagnostic_results import (
    OPENAI_COMPATIBLE_AUDIO_BACKENDS,
    AsrResult,
    audio_result_cache_key,
    backend_config_hash,
    summarize_results,
    write_json,
    write_result_cache,
)
from xiuxian_wendao_analyzer.audio_diagnostic_runner import run_diagnostic
from xiuxian_wendao_analyzer.audio_diagnostic_window_plan import (
    build_speech_window_plan_report,
    parse_window_min_candidates,
)
from xiuxian_wendao_analyzer.audio_diagnostic_windows import (
    chunk_start_offsets,
    chunk_windows,
    load_speech_segments,
)

DEFAULT_PROMPT = (
    "Please transcribe this audio verbatim as Simplified Chinese. "
    "Do not summarize, translate, or complete inaudible content. "
    "Preserve English technical terms, model names, code names, and person names. "
    "Mark inaudible spans as [inaudible]. Output only the transcript text."
)
DEFAULT_OPENROUTER_URL = "https://openrouter.ai/api/v1/chat/completions"
DEFAULT_OPENROUTER_MODEL = "xiaomi/mimo-v2.5"
DEFAULT_LOCAL_ASR_MODEL = "WHISPER_TINY"
DEFAULT_LOCAL_LANGUAGE = "zh"
DEFAULT_FIREREDASR2S_COMMAND = "fireredasr2s-cli"
INAUDIBLE_MARKERS = ("[inaudible]", "[听不清]", "听不清")

__all__ = [
    "AUDIO_MATERIALIZATION_NATIVE_RATE_WAV",
    "AUDIO_MATERIALIZATION_NORMALIZED_16K_WAV",
    "AUDIO_MATERIALIZATION_SOURCE_DIRECT",
    "AUDIO_SHARD_MANIFEST_SCHEMA",
    "DEFAULT_AUDIO_SHARD_PROFILE",
    "OPENAI_COMPATIBLE_AUDIO_BACKENDS",
    "PRIVATE_INPUT_PRIVACY",
    "REFERENCE_STATUS_CANDIDATE_DRAFT",
    "REFERENCE_STATUS_CURATED",
    "SHAREABLE_INPUT_PRIVACY",
    "SUPPORTED_AUDIO_MATERIALIZATION_MODES",
    "AsrResult",
    "AudioChunk",
    "QualityRow",
    "SpeechSegment",
    "audio_result_cache_key",
    "audio_shard_cache_key",
    "audio_shard_manifest",
    "backend_config_hash",
    "build_audio_shard_manifest_item",
    "build_openrouter_payload",
    "build_speech_window_plan_report",
    "chunk_start_offsets",
    "chunk_windows",
    "compare_audio_candidate_summaries",
    "curated_reference_rows_from_draft",
    "curated_reference_rows_from_tsv",
    "discover_audio_sources",
    "ensure_ffmpeg_on_path",
    "extract_openrouter_segments",
    "extract_openrouter_transcript",
    "load_reference_transcripts",
    "load_speech_segments",
    "materialize_audio_chunks",
    "materialize_reference_selection_pack",
    "parse_window_min_candidates",
    "recheck_quality_summary",
    "reference_candidate_draft_row_count",
    "reference_draft_rows",
    "resolve_openrouter_api_key",
    "run_diagnostic",
    "select_reference_draft_report",
    "select_reference_rows",
    "summarize_reference_subset",
    "summarize_results",
    "transcribe_fireredasr2s",
    "transcribe_local_docling",
    "transcribe_openrouter",
    "truth_template_rows",
    "validate_private_output_dir",
    "validate_reference_jsonl",
    "validate_reference_selection_pack",
    "write_json",
    "write_reference_draft_jsonl",
    "write_reference_draft_tsv",
    "write_result_cache",
]


def main(argv: Sequence[str] | None = None) -> int:
    """Run the diagnostic command."""

    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        if args.compare_summary_json is not None:
            report = compare_audio_candidate_summaries(args.compare_summary_json)
            if args.comparison_report_json is not None:
                write_json(args.comparison_report_json, report)
            print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
            return 0
        if args.validate_reference_jsonl is not None:
            report = validate_reference_jsonl(
                args.validate_reference_jsonl,
                audio_shards_path=args.reference_audio_shards_json,
            )
            if args.reference_validation_report_json is not None:
                write_json(args.reference_validation_report_json, report)
            print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
            return 0
        if args.recheck_quality_summary_json is not None:
            report = recheck_quality_summary(args)
            if args.recheck_quality_report_json is not None:
                write_json(args.recheck_quality_report_json, report)
            print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
            return 0
        if args.select_reference_draft_jsonl is not None:
            report = select_reference_draft_report(
                draft_jsonl=args.select_reference_draft_jsonl,
                limit=args.reference_selection_limit,
                quality_json=args.reference_selection_quality_json,
                selected_jsonl=args.reference_selection_jsonl,
                selected_tsv=args.reference_selection_tsv,
            )
            if args.reference_selection_report_json is not None:
                write_json(args.reference_selection_report_json, report)
            print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
            return 0
        if args.materialize_reference_selection_jsonl is not None:
            if args.reference_selection_clip_dir is None:
                parser.error(
                    "--reference-selection-clip-dir is required with "
                    "--materialize-reference-selection-jsonl"
                )
            validate_private_output_dir(
                args.reference_selection_clip_dir,
                start=Path.cwd(),
                input_privacy=args.input_privacy,
                allow_private_output_outside_cache=(
                    args.allow_private_output_outside_cache
                ),
            )
            report = materialize_reference_selection_pack(
                selection_jsonl=args.materialize_reference_selection_jsonl,
                clip_dir=args.reference_selection_clip_dir,
                force=args.force,
            )
            if args.reference_selection_pack_report_json is not None:
                write_json(args.reference_selection_pack_report_json, report)
            print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
            return 0
        if args.validate_reference_selection_review_tsv is not None:
            report = validate_reference_selection_pack(
                review_tsv=args.validate_reference_selection_review_tsv,
            )
            if args.reference_selection_validation_report_json is not None:
                write_json(args.reference_selection_validation_report_json, report)
            print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
            return 0
        if args.plan_speech_windows:
            if args.source_root is None:
                parser.error("source_root is required with --plan-speech-windows")
            if args.speech_segments_jsonl is None:
                parser.error(
                    "--speech-segments-jsonl is required with --plan-speech-windows"
                )
            report = build_speech_window_plan_report(
                speech_segments=load_speech_segments(
                    args.speech_segments_jsonl,
                    source=Path(args.source_root),
                ),
                duration_seconds=None,
                chunk_seconds=args.chunk_seconds,
                limit_chunks=args.limit_chunks,
                merge_gap_seconds=args.speech_segment_merge_gap_seconds,
                max_window_seconds=args.speech_segment_max_window_seconds,
                min_window_candidates=parse_window_min_candidates(
                    args.speech_window_plan_min_candidates
                ),
                short_merge_gap_seconds=args.speech_segment_short_merge_gap_seconds,
            )
            if args.speech_window_plan_report_json is not None:
                write_json(args.speech_window_plan_report_json, report)
            print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
            return 0
        curate_inputs = [
            value
            for value in (args.curate_reference_draft, args.curate_reference_tsv)
            if value is not None
        ]
        if curate_inputs:
            if len(curate_inputs) > 1:
                parser.error(
                    "use only one of --curate-reference-draft or --curate-reference-tsv"
                )
            if args.curated_reference_jsonl is None:
                parser.error(
                    "--curated-reference-jsonl is required with reference curation"
                )
            rows = (
                curated_reference_rows_from_draft(args.curate_reference_draft)
                if args.curate_reference_draft is not None
                else curated_reference_rows_from_tsv(args.curate_reference_tsv)
            )
            write_jsonl(args.curated_reference_jsonl, rows)
            print(
                json.dumps(
                    {
                        "curatedReferenceJsonl": str(args.curated_reference_jsonl),
                        "rows": len(rows),
                    },
                    ensure_ascii=False,
                    indent=2,
                    sort_keys=True,
                )
            )
            return 0
        report = run_diagnostic(args)
    except Exception as exc:
        print(f"audio ASR diagnostic failed: {exc}", file=sys.stderr)
        return 1
    print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
