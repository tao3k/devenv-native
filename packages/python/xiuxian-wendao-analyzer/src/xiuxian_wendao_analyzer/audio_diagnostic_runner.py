"""Audio diagnostic orchestration runner."""

from __future__ import annotations

import os
import time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse

from xiuxian_wendao_analyzer.audio_diagnostic_paths import (
    default_output_dir,
    discover_audio_sources,
    resolve_openrouter_api_key,
    validate_private_output_dir,
)
from xiuxian_wendao_analyzer.audio_diagnostic_quality import build_quality_rows
from xiuxian_wendao_analyzer.audio_diagnostic_quality_inputs import (
    load_reference_transcripts,
    load_term_list,
    prompt_with_domain_terms,
    prompt_with_primary_language,
    reference_candidate_draft_row_count,
)
from xiuxian_wendao_analyzer.audio_diagnostic_quality_summary import (
    summarize_precision_gate,
    summarize_quality,
    summarize_reference_subset,
    summarize_timeline_structure,
)
from xiuxian_wendao_analyzer.audio_diagnostic_results import summarize_results
from xiuxian_wendao_analyzer.audio_diagnostic_runner_pipeline import (
    backend_flags,
    materialize_diagnostic_sources,
    run_diagnostic_backends,
    selected_audio_backends,
)
from xiuxian_wendao_analyzer.audio_diagnostic_runner_report import (
    build_diagnostic_report,
    write_diagnostic_outputs,
)

DEFAULT_PROMPT = (
    "Please transcribe this audio verbatim. "
    "Do not summarize, translate, or complete inaudible content. "
    "Preserve English technical terms, model names, code names, and person names. "
    "Mark inaudible spans as [inaudible]. Output only the transcript text."
)
DEFAULT_PRIMARY_LANGUAGE = "zh"
DEFAULT_OPENROUTER_URL = "https://openrouter.ai/api/v1/audio/transcriptions"
DEFAULT_OPENROUTER_MODEL = "qwen/qwen3-asr-flash-2026-02-10"
DEFAULT_LOCAL_ASR_MODEL = "WHISPER_TINY"
DEFAULT_LOCAL_LANGUAGE = "zh"
DEFAULT_FIREREDASR2S_COMMAND = "fireredasr2s-cli"


def run_diagnostic(args: argparse.Namespace) -> dict[str, object]:
    """Run the bounded ASR diagnostic and write evidence files."""

    diagnostic_started = time.perf_counter()
    output_dir = args.output_dir or default_output_dir(Path.cwd())
    validate_private_output_dir(
        output_dir,
        start=Path.cwd(),
        input_privacy=args.input_privacy,
        allow_private_output_outside_cache=args.allow_private_output_outside_cache,
    )
    output_dir.mkdir(parents=True, exist_ok=True)
    if args.source_root is None:
        raise RuntimeError("source_root is required unless curating a reference draft")
    source_root = Path(args.source_root)
    sources = discover_audio_sources(source_root, limit_files=args.limit_files)
    if not sources:
        raise RuntimeError(f"no supported audio files found under {source_root}")

    api_key = resolve_openrouter_api_key(os.environ, env_file=args.env_file)
    domain_terms = load_term_list(args.domain_terms_file)
    required_terms = load_term_list(args.required_terms_file)
    primary_language = getattr(args, "primary_language", DEFAULT_PRIMARY_LANGUAGE)
    prompt = prompt_with_domain_terms(
        prompt_with_primary_language(args.prompt, primary_language),
        domain_terms,
    )
    admission_cache_dir = (
        None if args.no_admission_cache else (args.admission_cache_dir or output_dir / "admissions")
    )
    backends = selected_audio_backends(args.backend)
    hosted_audio_enabled, openai_compatible_audio_enabled = backend_flags(backends)
    (
        manifest_chunks,
        speech_segment_row_count,
        explicit_window_row_count,
    ) = materialize_diagnostic_sources(args, sources=sources, output_dir=output_dir)
    results = run_diagnostic_backends(
        args,
        chunks=manifest_chunks,
        backends=backends,
        output_dir=output_dir,
        api_key=api_key,
        prompt=prompt,
        admission_cache_dir=admission_cache_dir,
    )

    references = load_reference_transcripts(args.reference_jsonl)
    reference_candidate_draft_rows = reference_candidate_draft_row_count(args.reference_jsonl)
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
    report = build_diagnostic_report(
        args,
        source_root=source_root,
        output_dir=output_dir,
        sources_count=len(sources),
        backends=backends,
        hosted_audio_enabled=hosted_audio_enabled,
        openai_compatible_audio_enabled=openai_compatible_audio_enabled,
        api_key=api_key,
        admission_cache_dir=admission_cache_dir,
        speech_segment_row_count=speech_segment_row_count,
        explicit_window_row_count=explicit_window_row_count,
        truth_template_path=output_dir / "truth_template.jsonl",
        references_configured=bool(references),
        domain_terms_count=len(domain_terms),
        required_terms_count=len(required_terms),
        summary=summarize_results(results),
        quality_summary={
            **summarize_quality(quality_rows),
            **summarize_reference_subset(quality_rows),
        },
        timeline_summary=summarize_timeline_structure(
            quality_rows,
            allow_planned_gaps=args.sample_strategy in {"speech-segments", "explicit-windows"},
        ),
        precision_summary=summarize_precision_gate(
            quality_rows,
            reference_configured=bool(references),
            reference_candidate_draft_rows=reference_candidate_draft_rows,
            max_reference_cer=args.max_reference_cer,
            required_terms_configured=bool(required_terms),
        ),
        diagnostic_wall_seconds=time.perf_counter() - diagnostic_started,
    )
    write_diagnostic_outputs(
        args,
        output_dir=output_dir,
        manifest_chunks=manifest_chunks,
        result_rows=[result.__dict__ for result in results],
        quality_rows=quality_rows,
        truth_template_path=output_dir / "truth_template.jsonl",
        report=report,
    )
    return report
