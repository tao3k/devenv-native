"""Audio diagnostic CLI parser."""

from __future__ import annotations

import argparse
import os
from pathlib import Path

from xiuxian_wendao_analyzer.audio_diagnostic_identity import (
    AUDIO_MATERIALIZATION_NORMALIZED_16K_WAV,
    SUPPORTED_AUDIO_MATERIALIZATION_MODES,
)
from xiuxian_wendao_analyzer.audio_diagnostic_paths import (
    PRIVATE_INPUT_PRIVACY,
    SHAREABLE_INPUT_PRIVACY,
)
from xiuxian_wendao_analyzer.audio_diagnostic_runner import (
    DEFAULT_FIREREDASR2S_COMMAND,
    DEFAULT_LOCAL_ASR_MODEL,
    DEFAULT_LOCAL_LANGUAGE,
    DEFAULT_OPENROUTER_MODEL,
    DEFAULT_OPENROUTER_URL,
    DEFAULT_PRIMARY_LANGUAGE,
    DEFAULT_PROMPT,
)


def build_parser() -> argparse.ArgumentParser:
    """Build the diagnostic CLI parser."""

    parser = argparse.ArgumentParser(
        description="Run bounded MP3 ASR diagnostics for Docling and hosted audio."
    )
    parser.add_argument(
        "source_root",
        nargs="?",
        help="Directory or audio file to diagnose.",
    )
    parser.add_argument(
        "--backend",
        choices=[
            "local-docling",
            "local-fireredasr2s",
            "local-openai-audio",
            "openrouter-audio",
            "both",
            "firered-openrouter",
            "all",
        ],
        default="both",
    )
    parser.add_argument("--output-dir", type=Path, default=None)
    parser.add_argument(
        "--input-privacy",
        choices=[PRIVATE_INPUT_PRIVACY, SHAREABLE_INPUT_PRIVACY],
        default=PRIVATE_INPUT_PRIVACY,
        help=(
            "Classify diagnostic inputs. The default keeps outputs under "
            ".cache/agent/evidence because source audio may be private."
        ),
    )
    parser.add_argument(
        "--allow-private-output-outside-cache",
        action="store_true",
        help=(
            "Allow private diagnostic transcripts outside .cache/agent/evidence "
            "for local scratch runs. Do not commit those outputs."
        ),
    )
    parser.add_argument("--env-file", type=Path, default=Path(".env"))
    parser.add_argument("--chunk-seconds", type=int, default=60)
    parser.add_argument("--limit-files", type=int, default=2)
    parser.add_argument("--limit-chunks", type=int, default=1)
    parser.add_argument(
        "--sample-strategy",
        choices=[
            "head",
            "uniform",
            "full-coverage",
            "speech-segments",
            "explicit-windows",
        ],
        default="head",
        help=(
            "Audio shard selection strategy. full-coverage emits contiguous "
            "windows and clamps only the final chunk to the media tail; "
            "explicit-windows runs a caller-provided risk/review plan."
        ),
    )
    parser.add_argument("--start-offset-seconds", type=float, default=0.0)
    parser.add_argument("--chunk-context-seconds", type=float, default=0.0)
    parser.add_argument(
        "--audio-materialization-mode",
        choices=sorted(SUPPORTED_AUDIO_MATERIALIZATION_MODES),
        default=AUDIO_MATERIALIZATION_NORMALIZED_16K_WAV,
        help=(
            "Audio materialization profile. normalized-16k-wav decodes shards "
            "to 16 kHz mono WAV, native-rate-wav decodes shards to mono WAV at "
            "the source sample rate, and source-direct sends the full source "
            "file without ffmpeg chunking for backend compatibility diagnostics."
        ),
    )
    parser.add_argument(
        "--speech-segments-jsonl",
        type=Path,
        default=None,
        help=(
            "Optional VAD/planner JSONL sidecar used with "
            "--sample-strategy speech-segments. Rows accept source/sourceId, "
            "startSeconds or startMs, and durationSeconds/durationMs or endSeconds/endMs."
        ),
    )
    parser.add_argument(
        "--explicit-windows-json",
        type=Path,
        default=None,
        help=(
            "Optional explicit risk/review window JSON used with "
            "--sample-strategy explicit-windows. Accepts a JSON array or an "
            "object with rows containing startSeconds/startMs and "
            "durationSeconds/durationMs or endSeconds/endMs."
        ),
    )
    parser.add_argument(
        "--speech-segment-merge-gap-seconds",
        type=float,
        default=1.0,
        help=(
            "Maximum silence gap used to pack adjacent speech segment rows into "
            "one ASR shard when --sample-strategy speech-segments is active."
        ),
    )
    parser.add_argument(
        "--speech-segment-max-window-seconds",
        type=float,
        default=30.0,
        help=(
            "Maximum packed speech-window duration for --sample-strategy "
            "speech-segments. This bounds per-request ASR context."
        ),
    )
    parser.add_argument(
        "--speech-segment-min-window-seconds",
        type=float,
        default=0.0,
        help=(
            "Optional minimum effective speech-window duration for "
            "--sample-strategy speech-segments. Short VAD rows may be packed "
            "with nearby speech when the merged shard stays within the max window."
        ),
    )
    parser.add_argument(
        "--speech-segment-short-merge-gap-seconds",
        type=float,
        default=None,
        help=(
            "Optional silence-gap cap for minimum-window short-utterance packing. "
            "When omitted, the legacy behavior uses the minimum window seconds "
            "as the short merge gap."
        ),
    )
    parser.add_argument(
        "--plan-speech-windows",
        action="store_true",
        help=(
            "Report candidate speech-window plans from --speech-segments-jsonl "
            "without materializing audio or calling a model."
        ),
    )
    parser.add_argument(
        "--speech-window-plan-min-candidates",
        default="0,3,4,5,6,8,10",
        help=(
            "Comma-separated minimum speech-window seconds to compare when "
            "--plan-speech-windows is enabled."
        ),
    )
    parser.add_argument(
        "--speech-window-plan-report-json",
        type=Path,
        default=None,
        help="Optional output path for --plan-speech-windows report JSON.",
    )
    parser.add_argument("--sample-rate", type=int, default=16000)
    parser.add_argument("--audio-format", choices=["wav", "flac"], default="wav")
    parser.add_argument("--openrouter-model", default=DEFAULT_OPENROUTER_MODEL)
    parser.add_argument("--openrouter-base-url", default=DEFAULT_OPENROUTER_URL)
    parser.add_argument("--local-asr-model", default=DEFAULT_LOCAL_ASR_MODEL)
    parser.add_argument("--local-language", default=DEFAULT_LOCAL_LANGUAGE)
    parser.add_argument(
        "--primary-language",
        default=os.environ.get(
            "WENDAO_AUDIO_PRIMARY_LANGUAGE",
            DEFAULT_PRIMARY_LANGUAGE,
        ),
        help=(
            "Model-neutral primary spoken language hint used to shape prompts. "
            "Use unknown to omit language guidance."
        ),
    )
    parser.add_argument("--fireredasr2s-command", default=DEFAULT_FIREREDASR2S_COMMAND)
    parser.add_argument("--prompt", default=DEFAULT_PROMPT)
    parser.add_argument(
        "--domain-terms-file",
        type=Path,
        default=None,
        help="Optional UTF-8 text file with one domain term per line for prompts.",
    )
    parser.add_argument(
        "--required-terms-file",
        type=Path,
        default=None,
        help="Optional UTF-8 text file with one required term per line for scoring.",
    )
    parser.add_argument("--max-tokens", type=int, default=4096)
    parser.add_argument("--temperature", type=float, default=0.0)
    parser.add_argument("--timeout-seconds", type=int, default=300)
    parser.add_argument(
        "--hosted-request-concurrency",
        default=os.environ.get("WENDAO_AUDIO_HOSTED_REQUEST_CONCURRENCY"),
        help=(
            "Diagnostic-only maximum concurrent OpenAI-compatible hosted/local "
            "audio requests. Production Rust-to-Python calls use the worker "
            "budget supplied over Flight metadata. Use a positive integer to "
            "override the diagnostic runner; omit or use auto for ordered serial "
            "diagnostics."
        ),
    )
    parser.add_argument("--admission-cache-dir", type=Path, default=None)
    parser.add_argument("--no-admission-cache", action="store_true")
    parser.add_argument("--reference-jsonl", type=Path, default=None)
    parser.add_argument("--truth-template-jsonl", type=Path, default=None)
    parser.add_argument(
        "--validate-reference-jsonl",
        type=Path,
        default=None,
        help="Validate curated reference JSONL readiness without running ASR.",
    )
    parser.add_argument(
        "--reference-audio-shards-json",
        type=Path,
        default=None,
        help="Optional audio_shards.json used by --validate-reference-jsonl.",
    )
    parser.add_argument(
        "--reference-validation-report-json",
        type=Path,
        default=None,
        help="Optional output path for --validate-reference-jsonl report JSON.",
    )
    parser.add_argument(
        "--compare-summary-json",
        type=Path,
        nargs="+",
        default=None,
        help=(
            "Compare one or more audio diagnostic summary.json files without "
            "running ASR. Precision gates decide eligibility before speed."
        ),
    )
    parser.add_argument(
        "--comparison-report-json",
        type=Path,
        default=None,
        help="Optional output path for --compare-summary-json report JSON.",
    )
    parser.add_argument(
        "--recheck-quality-summary-json",
        type=Path,
        default=None,
        help=(
            "Recompute quality, timeline, and precision summaries from a saved "
            "summary.json and results.json without running ASR."
        ),
    )
    parser.add_argument(
        "--recheck-quality-results-json",
        type=Path,
        default=None,
        help=(
            "Optional results.json for --recheck-quality-summary-json. Defaults "
            "to results.json beside the summary."
        ),
    )
    parser.add_argument(
        "--recheck-quality-report-json",
        type=Path,
        default=None,
        help="Optional output path for --recheck-quality-summary-json report JSON.",
    )
    parser.add_argument(
        "--select-reference-draft-jsonl",
        type=Path,
        default=None,
        help=(
            "Select high-value rows from a reference_draft.jsonl for manual CER "
            "curation without running ASR."
        ),
    )
    parser.add_argument(
        "--build-risk-recovery-plan-quality-json",
        type=Path,
        default=None,
        help=(
            "Build a timestamp-based short-window recovery plan from saved "
            "quality.json without running ASR."
        ),
    )
    parser.add_argument(
        "--risk-recovery-results-json",
        type=Path,
        default=None,
        help=(
            "Optional results.json used by --build-risk-recovery-plan-quality-json "
            "for latency-based risk signals."
        ),
    )
    parser.add_argument(
        "--risk-recovery-output-json",
        type=Path,
        default=None,
        help="Optional output path for the generated risk recovery plan JSON.",
    )
    parser.add_argument(
        "--risk-recovery-split-seconds",
        type=float,
        default=30.0,
        help="Short-window duration for risk recovery plan rows.",
    )
    parser.add_argument(
        "--risk-recovery-limit-parents",
        type=int,
        default=20,
        help="Maximum parent windows selected for short-window recovery.",
    )
    parser.add_argument(
        "--build-risk-recovery-patch-gate-base-quality-json",
        type=Path,
        default=None,
        help=(
            "Build parent-level accept/reject decisions for short-window "
            "recovery patches from saved diagnostic evidence without running ASR."
        ),
    )
    parser.add_argument(
        "--risk-recovery-patch-base-results-json",
        type=Path,
        default=None,
        help="Base full-window results.json used by recovery patch gating.",
    )
    parser.add_argument(
        "--risk-recovery-patch-recovery-quality-json",
        type=Path,
        default=None,
        help="Short-window recovery quality.json used by recovery patch gating.",
    )
    parser.add_argument(
        "--risk-recovery-patch-recovery-results-json",
        type=Path,
        default=None,
        help="Short-window recovery results.json used by recovery patch gating.",
    )
    parser.add_argument(
        "--risk-recovery-patch-plan-json",
        type=Path,
        default=None,
        help="Recovery plan JSON that maps short-window rows to parent rows.",
    )
    parser.add_argument(
        "--risk-recovery-patch-output-json",
        type=Path,
        default=None,
        help="Optional output path for the recovery patch gate report JSON.",
    )
    parser.add_argument(
        "--risk-recovery-patch-max-chinese-ratio-drop",
        type=float,
        default=0.03,
        help="Maximum accepted Chinese-ratio drop for a recovery patch.",
    )
    parser.add_argument(
        "--risk-recovery-patch-min-char-ratio",
        type=float,
        default=0.65,
        help="Minimum recovery/base transcript character ratio for patch acceptance.",
    )
    parser.add_argument(
        "--risk-recovery-patch-max-char-ratio",
        type=float,
        default=1.40,
        help="Maximum recovery/base transcript character ratio for patch acceptance.",
    )
    parser.add_argument(
        "--risk-recovery-patch-max-part-repeat",
        type=float,
        default=0.35,
        help="Maximum repeated n-gram ratio allowed on any short-window part.",
    )
    parser.add_argument(
        "--reference-selection-limit",
        type=int,
        default=12,
        help="Maximum rows selected by --select-reference-draft-jsonl.",
    )
    parser.add_argument(
        "--reference-selection-quality-json",
        type=Path,
        default=None,
        help=(
            "Optional rechecked quality report used to refresh reviewStatus "
            "before selecting reference rows."
        ),
    )
    parser.add_argument(
        "--reference-selection-report-json",
        type=Path,
        default=None,
        help="Optional report path for --select-reference-draft-jsonl.",
    )
    parser.add_argument(
        "--reference-selection-jsonl",
        type=Path,
        default=None,
        help="Optional selected reference draft JSONL output path.",
    )
    parser.add_argument(
        "--materialize-reference-selection-jsonl",
        type=Path,
        default=None,
        help=(
            "Create private review audio clips from selected reference rows without running ASR."
        ),
    )
    parser.add_argument(
        "--reference-selection-clip-dir",
        type=Path,
        default=None,
        help="Private output directory for --materialize-reference-selection-jsonl.",
    )
    parser.add_argument(
        "--reference-selection-pack-report-json",
        type=Path,
        default=None,
        help="Optional report path for materialized reference selection clips.",
    )
    parser.add_argument(
        "--validate-reference-selection-review-table",
        type=Path,
        default=None,
        help="Validate private reference review clips from a Parquet review table.",
    )
    parser.add_argument(
        "--reference-selection-validation-report-json",
        type=Path,
        default=None,
        help="Optional report path for reference review clip validation.",
    )
    parser.add_argument(
        "--reference-selection-review-org",
        type=Path,
        default=None,
        help=(
            "Optional private Org checklist path for "
            "--validate-reference-selection-review-table. The checklist omits "
            "transcript text and points reviewers back to the Parquet review table."
        ),
    )
    parser.add_argument(
        "--model-review-reference-selection-review-table",
        type=Path,
        default=None,
        help=(
            "Run the configured OpenRouter audio model over private review clips "
            "and write a redacted consistency report. This does not create "
            "curated reference truth."
        ),
    )
    parser.add_argument(
        "--reference-selection-model-review-report-json",
        type=Path,
        default=None,
        help="Optional report path for --model-review-reference-selection-review-table.",
    )
    parser.add_argument(
        "--reference-selection-model-review-max-cer",
        type=float,
        default=0.15,
        help=(
            "Maximum candidate-to-model CER for a model-consistent review row. "
            "This is a stability signal, not a curated-reference promotion gate."
        ),
    )
    parser.add_argument(
        "--curate-reference-org",
        type=Path,
        default=None,
        help=(
            "Convert an edited Org review checklist into promotion-safe "
            "curated reference JSONL without running ASR."
        ),
    )
    parser.add_argument(
        "--curated-reference-jsonl",
        type=Path,
        default=None,
        help="Output path for reference curation commands.",
    )
    parser.add_argument("--max-reference-cer", type=float, default=0.15)
    parser.add_argument("--min-required-term-recall", type=float, default=1.0)
    parser.add_argument("--min-chars-per-minute", type=float, default=40.0)
    parser.add_argument("--min-chinese-ratio", type=float, default=0.35)
    parser.add_argument("--max-inaudible-per-minute", type=float, default=30.0)
    parser.add_argument("--max-repeated-ngram-ratio", type=float, default=0.35)
    parser.add_argument("--force", action="store_true")
    return parser
