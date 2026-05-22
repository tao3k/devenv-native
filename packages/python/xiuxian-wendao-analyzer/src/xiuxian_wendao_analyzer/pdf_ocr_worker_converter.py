"""Docling converter construction for PDF OCR workers."""

from __future__ import annotations

import os
from inspect import signature
from typing import TYPE_CHECKING, Any

from .pdf_ocr_contracts import (
    PDF_OCR_BACKEND_TEXT_PROFILE,
    PDF_OCR_DEFAULT_PROFILE,
    PDF_OCR_DOCLING_VLM_DEEPSEEK_OCR_PROFILE,
    PDF_OCR_FAST_TEXT_PROFILE,
)
from .pdf_ocr_worker_options import (
    PDF_OCR_FAST_TEXT_DEFAULT_THREADS,
    PDF_OCR_FAST_TEXT_THREADS_ENV,
)

if TYPE_CHECKING:
    from collections.abc import Callable

    from .documents import DocumentConverterProtocol

PDF_OCR_FAST_TEXT_SOURCE_BACKEND_TABLE_PROFILE = (
    "docling-fast-text-source-backend-table"
)


def _factory_accepts_ocr_profile(factory: Callable[..., Any] | None) -> bool:
    if factory is None:
        return False
    try:
        signature(factory).bind(PDF_OCR_DEFAULT_PROFILE)
    except (TypeError, ValueError):
        return False
    return True


def _new_docling_converter(
    ocr_profile: str = PDF_OCR_DEFAULT_PROFILE,
) -> DocumentConverterProtocol:
    try:
        from docling.datamodel.base_models import InputFormat
        from docling.datamodel.pipeline_options import (
            AcceleratorOptions,
            PdfPipelineOptions,
            TableFormerMode,
        )
        from docling.document_converter import DocumentConverter, PdfFormatOption
    except ModuleNotFoundError as exc:
        raise RuntimeError(
            "docling is not installed; install xiuxian-wendao-analyzer[documents] "
            "to enable Docling-backed PDF OCR shards"
        ) from exc
    if ocr_profile == PDF_OCR_FAST_TEXT_PROFILE:
        options = PdfPipelineOptions()
        options.accelerator_options = AcceleratorOptions(
            num_threads=_fast_text_accelerator_threads()
        )
        options.table_structure_options.mode = TableFormerMode.FAST
        return DocumentConverter(
            format_options={
                InputFormat.PDF: PdfFormatOption(pipeline_options=options),
            }
        )
    if ocr_profile == PDF_OCR_FAST_TEXT_SOURCE_BACKEND_TABLE_PROFILE:
        options = PdfPipelineOptions()
        options.accelerator_options = AcceleratorOptions(
            num_threads=_fast_text_accelerator_threads()
        )
        options.table_structure_options.mode = TableFormerMode.FAST
        options.do_ocr = False
        options.force_backend_text = True
        options.ocr_batch_size = 1
        options.layout_batch_size = 1
        options.table_batch_size = 1
        return DocumentConverter(
            format_options={
                InputFormat.PDF: PdfFormatOption(pipeline_options=options),
            }
        )
    if ocr_profile == PDF_OCR_BACKEND_TEXT_PROFILE:
        options = PdfPipelineOptions()
        options.accelerator_options = AcceleratorOptions(
            num_threads=_fast_text_accelerator_threads()
        )
        options.do_ocr = False
        options.do_table_structure = False
        options.force_backend_text = True
        options.ocr_batch_size = 1
        options.layout_batch_size = 1
        options.table_batch_size = 1
        return DocumentConverter(
            format_options={
                InputFormat.PDF: PdfFormatOption(pipeline_options=options),
            }
        )
    if ocr_profile == PDF_OCR_DOCLING_VLM_DEEPSEEK_OCR_PROFILE:
        try:
            from docling.datamodel.base_models import InputFormat
            from docling.datamodel.pipeline_options import (
                VlmConvertOptions,
                VlmPipelineOptions,
            )
            from docling.document_converter import DocumentConverter, PdfFormatOption
            from docling.pipeline.vlm_pipeline import VlmPipeline
        except ModuleNotFoundError as exc:
            raise RuntimeError(
                "Docling VLM dependencies are not installed; install "
                "xiuxian-wendao-analyzer[documents] to enable Docling VLM OCR"
            ) from exc
        vlm_options = VlmConvertOptions.from_preset("deepseek_ocr")
        pipeline_options = VlmPipelineOptions(
            enable_remote_services=True,
            vlm_options=vlm_options,
        )
        return DocumentConverter(
            format_options={
                InputFormat.PDF: PdfFormatOption(
                    pipeline_cls=VlmPipeline,
                    pipeline_options=pipeline_options,
                ),
            }
        )
    return DocumentConverter()


def _fast_text_accelerator_threads() -> int:
    return _fast_text_accelerator_threads_with_lookup(os.environ.get)


def _fast_text_accelerator_threads_with_lookup(
    lookup: Callable[[str], str | None],
) -> int:
    try:
        value = int(lookup(PDF_OCR_FAST_TEXT_THREADS_ENV) or "")
    except (TypeError, ValueError):
        value = PDF_OCR_FAST_TEXT_DEFAULT_THREADS
    return value if value > 0 else PDF_OCR_FAST_TEXT_DEFAULT_THREADS
