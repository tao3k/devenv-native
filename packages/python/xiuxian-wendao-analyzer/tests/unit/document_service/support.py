"""Shared helpers for document_service tests."""

from __future__ import annotations

import threading
import time
from pathlib import Path

import pyarrow as pa
import pyarrow.flight as flight
import pytest

from xiuxian_wendao_analyzer import (
    ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    ANALYSIS_PDF_OCR_SHARDS_ROUTE,
    DOCUMENT_RESOURCE_SCHEMA,
    EXPECTED_SCHEMA_VERSION,
    PDF_OCR_SHARD_INPUT_SCHEMA,
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
    PDF_OCR_SHARD_RESULT_SCHEMA,
    PDF_OCR_SHARD_RESULT_SCHEMA_VERSION,
    SUPPORTED_DOCUMENT_ROUTES,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER,
    WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_PDF_OCR_WORKERS_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER,
    DoclingPdfOcrShardWorker,
    DocumentExtractFlightServer,
    build_document_extract_table,
    build_pdf_ocr_shard_result_table,
    succeeded_pdf_ocr_shard_result,
)
from xiuxian_wendao_analyzer.document_service import _build_pdf_ocr_worker
from xiuxian_wendao_analyzer.pdf_ocr import (
    SkippingPdfOcrShardWorker,
    resolve_pdf_ocr_worker_count,
)


class FakeDoclingDocument:
    def __init__(
        self,
        markdown: str,
        *,
        markdown_by_page: dict[int, str] | None = None,
    ) -> None:
        self.markdown = markdown
        self.markdown_by_page = markdown_by_page or {}

    def export_to_markdown(self, **kwargs: object) -> str:
        page_no = kwargs.get("page_no")
        if isinstance(page_no, int) and page_no in self.markdown_by_page:
            return self.markdown_by_page[page_no]
        return self.markdown


class FakeDoclingResult:
    def __init__(
        self,
        markdown: str,
        *,
        markdown_by_page: dict[int, str] | None = None,
    ) -> None:
        self.document = FakeDoclingDocument(
            markdown,
            markdown_by_page=markdown_by_page,
        )


class FakeDoclingConverter:
    def __init__(self, markdown: str = "# Service\n") -> None:
        self.markdown = markdown
        self.calls: list[Path] = []
        self.kwargs_calls: list[dict[str, object]] = []

    def convert(self, source: str | Path, **kwargs: object) -> FakeDoclingResult:
        self.calls.append(Path(source))
        self.kwargs_calls.append(dict(kwargs))
        return FakeDoclingResult(self.markdown)


class FailingDoclingConverter:
    def convert(self, source: str | Path, **kwargs: object) -> FakeDoclingResult:
        _ = kwargs
        raise RuntimeError(f"cannot OCR {source}")


class FakePdfOcrShardWorker:
    def __init__(self) -> None:
        self.inputs: list[dict[str, object]] = []
        self.max_workers: int | str | None = None

    def recognize(
        self,
        inputs: list[dict[str, object]],
        *,
        max_workers: int | str | None = None,
    ) -> list[dict[str, object]]:
        self.inputs = list(inputs)
        self.max_workers = max_workers
        return [succeeded_pdf_ocr_shard_result(inputs[0], "page text", 0.91)]


def _sample_pdf_ocr_input_table(
    image_path: str = "/tmp/page-00000.png",
    *,
    source_path: str = "/tmp/source.pdf",
    page_index: int = 0,
    shard_element_id: str = "shard-id",
    shard_type: str = "page",
    region_index: int = 0,
    parent_shard_element_id: str = "",
    reading_order_key: str = "000000.000000",
    ocr_profile: str = "docling-compatible-page-ocr-v1",
):
    return pa.Table.from_pylist(
        [
            {
                "contractVersion": PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
                "sourcePath": source_path,
                "sourceContentHash": "sourcehash",
                "pageIndex": page_index,
                "imagePath": image_path,
                "imageMimeType": "image/png",
                "rasterSha256": "rasterhash",
                "renderProfile": "pdfium-render-page-shards-v1",
                "ocrProfile": ocr_profile,
                "ocrEngine": "docling-compatible-ocr",
                "preferredLanguages": "auto",
                "minConfidence": 0.0,
                "preserveLayout": True,
                "rasterWidthPx": 2400,
                "rasterHeightPx": 3100,
                "renderDpi": 300,
                "rotationDegrees": 0,
                "cropLeft": 0.0,
                "cropBottom": 0.0,
                "cropRight": 612.0,
                "cropTop": 792.0,
                "pointToPixelScaleX": 3.921568627,
                "pointToPixelScaleY": 3.914141414,
                "shardElementId": shard_element_id,
                "shardType": shard_type,
                "regionIndex": region_index,
                "parentShardElementId": parent_shard_element_id,
                "readingOrderKey": reading_order_key,
                "sourcePagePixelLeft": 0,
                "sourcePagePixelTop": 0,
                "sourcePagePixelRight": 2400,
                "sourcePagePixelBottom": 3100,
            }
        ],
        schema=PDF_OCR_SHARD_INPUT_SCHEMA,
    )


__all__ = [
    "ANALYSIS_DOCUMENT_EXTRACT_ROUTE",
    "ANALYSIS_PDF_OCR_SHARDS_ROUTE",
    "DOCUMENT_RESOURCE_SCHEMA",
    "EXPECTED_SCHEMA_VERSION",
    "PDF_OCR_SHARD_INPUT_SCHEMA",
    "PDF_OCR_SHARD_INPUT_SCHEMA_VERSION",
    "PDF_OCR_SHARD_RESULT_SCHEMA",
    "PDF_OCR_SHARD_RESULT_SCHEMA_VERSION",
    "SUPPORTED_DOCUMENT_ROUTES",
    "WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_PAGE_RANGE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER",
    "WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER",
    "WENDAO_PDF_OCR_WORKERS_HEADER",
    "WENDAO_SCHEMA_VERSION_HEADER",
    "DoclingPdfOcrShardWorker",
    "DocumentExtractFlightServer",
    "FailingDoclingConverter",
    "FakeDoclingConverter",
    "FakeDoclingDocument",
    "FakeDoclingResult",
    "FakePdfOcrShardWorker",
    "Path",
    "SkippingPdfOcrShardWorker",
    "_build_pdf_ocr_worker",
    "_sample_pdf_ocr_input_table",
    "build_document_extract_table",
    "build_pdf_ocr_shard_result_table",
    "flight",
    "pa",
    "pytest",
    "resolve_pdf_ocr_worker_count",
    "succeeded_pdf_ocr_shard_result",
    "threading",
    "time",
]
