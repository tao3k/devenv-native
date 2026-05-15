"""Docling audio diagnostic backend helpers."""

from __future__ import annotations

from collections.abc import Iterable, Mapping
from typing import TYPE_CHECKING

from xiuxian_wendao_analyzer.audio_diagnostic_media_probe import ensure_ffmpeg_on_path

if TYPE_CHECKING:
    from pathlib import Path

    from xiuxian_wendao_analyzer.audio_diagnostic_materialization import AudioChunk


def _row_value(row: object, field: str) -> object:
    if isinstance(row, Mapping):
        return row.get(field)
    return getattr(row, field, None)


def transcript_from_document_rows(rows: Iterable[object]) -> str:
    """Extract transcript-like content from Docling resource rows."""

    preferred: list[str] = []
    fallback: list[str] = []
    for row in rows:
        content = _row_value(row, "content")
        if not isinstance(content, str) or not content.strip():
            continue
        resource_type = _row_value(row, "resourceType")
        if resource_type in {"audio", "document"}:
            preferred.append(content.strip())
        else:
            fallback.append(content.strip())
    return "\n\n".join(preferred or fallback)


def build_docling_audio_converter(asr_model: str, language: str) -> object:
    """Create a Docling audio converter with explicit ASR model and language."""

    from docling.datamodel import asr_model_specs
    from docling.datamodel.base_models import InputFormat
    from docling.datamodel.pipeline_options import AsrPipelineOptions
    from docling.document_converter import AudioFormatOption, DocumentConverter
    from docling.pipeline.asr_pipeline import AsrPipeline

    if not hasattr(asr_model_specs, asr_model):
        raise ValueError(f"unknown Docling ASR model spec: {asr_model}")
    asr_options = getattr(asr_model_specs, asr_model).model_copy(deep=True)
    asr_options.language = language
    if hasattr(asr_options, "verbose"):
        asr_options.verbose = False
    pipeline_options = AsrPipelineOptions()
    pipeline_options.asr_options = asr_options
    return DocumentConverter(
        format_options={
            InputFormat.AUDIO: AudioFormatOption(
                pipeline_cls=AsrPipeline,
                pipeline_options=pipeline_options,
            )
        }
    )


def transcribe_local_docling(
    chunk: AudioChunk, output_dir: Path, *, asr_model: str, language: str
) -> str:
    """Run local Docling ASR for one materialized chunk."""

    from xiuxian_wendao_analyzer.document_extract import extract_document_resources

    ensure_ffmpeg_on_path(output_dir / "_ffmpeg_bin")
    converter = build_docling_audio_converter(asr_model, language)
    rows = extract_document_resources(chunk.path, output_dir, converter=converter)
    return transcript_from_document_rows(rows)
