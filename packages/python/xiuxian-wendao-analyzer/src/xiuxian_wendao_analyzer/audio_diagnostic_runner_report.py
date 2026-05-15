"""Audio diagnostic report assembly and persistence."""

from __future__ import annotations

from datetime import UTC, datetime
from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_identity import (
    AUDIO_SHARD_MANIFEST_SCHEMA,
    DEFAULT_AUDIO_SHARD_PROFILE,
    AudioChunk,
    audio_shard_manifest,
    truth_template_rows,
)
from xiuxian_wendao_analyzer.audio_diagnostic_reporting import (
    write_jsonl,
    write_quality_tsv,
    write_reference_draft_jsonl,
    write_reference_draft_tsv,
    write_transcript_review_tsv,
    write_transcript_timeline_jsonl,
    write_transcript_timeline_srt,
    write_transcript_timeline_vtt,
)
from xiuxian_wendao_analyzer.audio_diagnostic_results import write_json

if TYPE_CHECKING:
    import argparse
    from pathlib import Path

    from xiuxian_wendao_analyzer.audio_diagnostic_quality import QualityRow


def build_diagnostic_report(
    args: argparse.Namespace,
    *,
    source_root: Path,
    output_dir: Path,
    sources_count: int,
    backends: list[str],
    hosted_audio_enabled: bool,
    openai_compatible_audio_enabled: bool,
    api_key: str | None,
    result_cache_dir: Path | None,
    speech_segment_row_count: int,
    truth_template_path: Path,
    references_configured: bool,
    domain_terms_count: int,
    required_terms_count: int,
    summary: dict[str, object],
    quality_summary: dict[str, object],
    timeline_summary: dict[str, object],
    precision_summary: dict[str, object],
    diagnostic_wall_seconds: float,
) -> dict[str, object]:
    """Build the stable diagnostic summary payload."""

    return {
        "createdAt": datetime.now(tz=UTC).isoformat(),
        "sourceRoot": str(source_root),
        "outputDir": str(output_dir),
        "diagnosticWallSeconds": diagnostic_wall_seconds,
        "sourceCount": sources_count,
        "chunkSeconds": args.chunk_seconds,
        "limitFiles": args.limit_files,
        "limitChunks": args.limit_chunks,
        "sampleStrategy": args.sample_strategy,
        "audioMaterializationMode": args.audio_materialization_mode,
        "startOffsetSeconds": args.start_offset_seconds,
        "chunkContextSeconds": args.chunk_context_seconds,
        "speechSegmentsConfigured": getattr(args, "speech_segments_jsonl", None)
        is not None,
        "speechSegmentRows": speech_segment_row_count,
        "speechSegmentsPath": (
            ""
            if getattr(args, "speech_segments_jsonl", None) is None
            else str(args.speech_segments_jsonl)
        ),
        "speechSegmentMergeGapSeconds": getattr(
            args, "speech_segment_merge_gap_seconds", 0.0
        ),
        "speechSegmentMinWindowSeconds": getattr(
            args, "speech_segment_min_window_seconds", 0.0
        ),
        "speechSegmentShortMergeGapSeconds": getattr(
            args, "speech_segment_short_merge_gap_seconds", None
        ),
        "speechSegmentMaxWindowSeconds": getattr(
            args, "speech_segment_max_window_seconds", None
        ),
        "audioShardManifestSchema": AUDIO_SHARD_MANIFEST_SCHEMA,
        "audioShardProfile": DEFAULT_AUDIO_SHARD_PROFILE,
        "resultCacheEnabled": result_cache_dir is not None,
        "resultCacheDir": "" if result_cache_dir is None else str(result_cache_dir),
        "hostedRequestConcurrency": getattr(args, "hosted_request_concurrency", 1),
        "inputPrivacy": args.input_privacy,
        "privateOutputOutsideCacheAllowed": args.allow_private_output_outside_cache,
        "privateInputPolicy": (
            "private local recordings may be used for diagnostics only; do not "
            "commit source audio, derived chunks, transcripts, raw model "
            "responses, or direct private clips"
        ),
        "requestedBackends": backends,
        "openAiCompatibleAudioEnabled": openai_compatible_audio_enabled,
        "hostedAudioEnabled": hosted_audio_enabled,
        "hostedAudioModel": args.openrouter_model,
        "hostedAudioApiKeyConfigured": bool(api_key) if hosted_audio_enabled else False,
        "truthTemplatePath": str(truth_template_path),
        "referenceDraftPath": str(output_dir / "reference_draft.jsonl"),
        "audioShardManifestPath": str(output_dir / "audio_shards.json"),
        "qualityReviewPath": str(output_dir / "review.tsv"),
        "transcriptReviewPath": str(output_dir / "transcript_review.tsv"),
        "transcriptTimelineJsonlPath": str(output_dir / "transcript_timeline.jsonl"),
        "transcriptTimelineVttPath": str(output_dir / "transcript_timeline.vtt"),
        "transcriptTimelineSrtPath": str(output_dir / "transcript_timeline.srt"),
        "externalTruthTemplatePath": (
            "" if args.truth_template_jsonl is None else str(args.truth_template_jsonl)
        ),
        "openRouterModel": args.openrouter_model,
        "openRouterApiKeyConfigured": bool(api_key) if hosted_audio_enabled else False,
        "localAsrModel": args.local_asr_model,
        "localLanguage": args.local_language,
        "fireRedAsr2sCommand": args.fireredasr2s_command,
        "referenceConfigured": references_configured,
        "domainTermsConfigured": domain_terms_count > 0,
        "domainTermCount": domain_terms_count,
        "requiredTermsConfigured": required_terms_count > 0,
        "requiredTermCount": required_terms_count,
        "minRequiredTermRecall": args.min_required_term_recall,
        "maxReferenceCer": args.max_reference_cer,
        "maxRepeatedNgramRatio": args.max_repeated_ngram_ratio,
        **summary,
        **quality_summary,
        **timeline_summary,
        **precision_summary,
    }


def write_diagnostic_outputs(
    args: argparse.Namespace,
    *,
    output_dir: Path,
    manifest_chunks: list[AudioChunk],
    result_rows: list[dict[str, object]],
    quality_rows: list[QualityRow],
    truth_template_path: Path,
    report: dict[str, object],
) -> None:
    """Write all diagnostic evidence files."""

    write_json(output_dir / "results.json", result_rows)
    write_json(
        output_dir / "audio_shards.json",
        audio_shard_manifest(
            profile=DEFAULT_AUDIO_SHARD_PROFILE,
            sample_strategy=args.sample_strategy,
            audio_materialization_mode=args.audio_materialization_mode,
            chunks=manifest_chunks,
        ),
    )
    write_json(output_dir / "quality.json", [row.__dict__ for row in quality_rows])
    write_quality_tsv(output_dir / "review.tsv", quality_rows)
    write_transcript_review_tsv(output_dir / "transcript_review.tsv", quality_rows)
    write_transcript_timeline_jsonl(
        output_dir / "transcript_timeline.jsonl", quality_rows
    )
    write_transcript_timeline_vtt(output_dir / "transcript_timeline.vtt", quality_rows)
    write_transcript_timeline_srt(output_dir / "transcript_timeline.srt", quality_rows)
    write_reference_draft_jsonl(output_dir / "reference_draft.jsonl", quality_rows)
    write_reference_draft_tsv(output_dir / "reference_draft.tsv", quality_rows)
    template_rows = truth_template_rows(manifest_chunks)
    write_jsonl(truth_template_path, template_rows)
    if args.truth_template_jsonl is not None:
        write_jsonl(args.truth_template_jsonl, template_rows)
    write_json(output_dir / "summary.json", report)
