"""Hosted VLM extraction for standalone image documents."""

from __future__ import annotations

import urllib.error
from collections.abc import Callable, Mapping
from dataclasses import dataclass
from io import BytesIO
from typing import TYPE_CHECKING, Any

from .document_cache import (
    _file_sha256,
    _write_cached_resources,
    _write_cached_structure,
    _write_document_timing_sidecar,
)
from .document_metrics import DocumentTimingRecorder
from .document_profiles import (
    DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE,
    normalize_document_extract_profile,
)
from .document_types import (
    DOCUMENT_STRUCTURE_SCHEMA_VERSION,
    DocumentResourceRow,
    DocumentStructureBlock,
)
from .pdf_ocr_contracts import HOSTED_VLM_OCR_DEFAULT_API_KEY
from .pdf_ocr_ocr2.config import Ocr2ClientConfig, ocr2_client_config_from_env
from .pdf_ocr_ocr2.http import (
    chat_completion_url,
    extract_openai_message_content,
    send_completion_request,
)
from .pdf_ocr_ocr2.payload_helpers import image_bytes_data_url
from .pdf_ocr_ocr2.payloads import request_payload

if TYPE_CHECKING:
    from pathlib import Path

CompletionSender = Callable[
    [str, Mapping[str, str], float, Mapping[str, Any]],
    tuple[int | None, Any],
]

DIRECT_IMAGE_MIME_TYPES = {
    ".gif": "image/gif",
    ".jpeg": "image/jpeg",
    ".jpg": "image/jpeg",
    ".png": "image/png",
    ".webp": "image/webp",
}
PNG_CONVERTED_IMAGE_SUFFIXES = {".bmp", ".tif", ".tiff"}
SUPPORTED_IMAGE_SUFFIXES = frozenset(
    set(DIRECT_IMAGE_MIME_TYPES) | PNG_CONVERTED_IMAGE_SUFFIXES
)
IMAGE_DOCUMENT_TEXT_MIME_TYPE = "text/markdown"


@dataclass(frozen=True)
class ImageDocumentPayload:
    data_url: str
    mime_type: str


def is_hosted_vlm_image_source(source: Path, profile: str | None) -> bool:
    """Return true when the source should use hosted VLM image extraction."""

    return (
        normalize_document_extract_profile(profile)
        == DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE
        and source.suffix.lower() in SUPPORTED_IMAGE_SUFFIXES
    )


def extract_image_document_resources(
    source: Path,
    output_dir: Path,
    *,
    config: Ocr2ClientConfig | None = None,
    send_request: CompletionSender | None = None,
    error_row: bool = False,
) -> list[DocumentResourceRow]:
    """Extract one standalone image through a hosted VLM backend.

    # Errors

    Raises when the source suffix is unsupported, the hosted endpoint fails,
    the response shape is malformed, or the response text is empty. When
    `error_row` is true, failures are returned as table-shaped error rows.
    """

    output_dir.mkdir(parents=True, exist_ok=True)
    timing = DocumentTimingRecorder(source)
    try:
        with timing.phase("sourceHash"):
            source_content_hash = _file_sha256(source)
        with timing.phase("imagePayloadBuild"):
            image_payload = image_document_payload_for_source(source)
        resolved_config = config or ocr2_client_config_from_env()
        payload = request_payload(
            model=resolved_config.model,
            prompt=resolved_config.prompt,
            input_row={
                "imageMimeType": image_payload.mime_type,
                "shardElementId": "_main",
                "shardType": "image",
            },
            image_path=source,
            image_data_url_value=image_payload.data_url,
            max_tokens=resolved_config.max_tokens,
            image_optimization_mode=resolved_config.image_optimization_mode,
        )
        sender = send_request or _send_completion
        with timing.phase("hostedVlmRequest"):
            _, response_payload = sender(
                chat_completion_url(resolved_config.base_url),
                hosted_vlm_headers(resolved_config),
                resolved_config.timeout_seconds,
                payload,
            )
        with timing.phase("hostedVlmNormalize"):
            markdown_text = normalize_image_text(
                extract_openai_message_content(response_payload)
            )
            if not markdown_text:
                raise ValueError("hosted VLM image extraction returned empty text")
        markdown_path = output_dir / f"{source.stem}.md"
        with timing.phase("writeMarkdown"):
            markdown_path.write_text(markdown_text, encoding="utf-8")
        resources = [
            DocumentResourceRow(
                sourcePath=str(source),
                resourceType="document",
                resourcePath=str(markdown_path),
                pageIndex=0,
                caption="",
                content=markdown_text,
                mimeType=IMAGE_DOCUMENT_TEXT_MIME_TYPE,
                status="ok",
                elementId="_main",
            )
        ]
        structure = [
            DocumentStructureBlock(
                contractVersion=DOCUMENT_STRUCTURE_SCHEMA_VERSION,
                sourcePath=str(source),
                sourceContentHash=source_content_hash,
                blockId="hosted-vlm-image-main",
                parentBlockId="",
                pageIndex=0,
                blockIndex=0,
                readingOrderKey="000000.000000",
                blockType="image_text",
                resourceElementId="_main",
                content=markdown_text,
                mimeType=IMAGE_DOCUMENT_TEXT_MIME_TYPE,
                status="ok",
                engine=DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE,
                confidence=None,
                bboxLeft=None,
                bboxTop=None,
                bboxRight=None,
                bboxBottom=None,
                provenance='{"route":"document-image-vlm"}',
            )
        ]
        with timing.phase("writeStructureArrow"):
            _write_cached_structure(output_dir, structure)
        with timing.phase("writeResourcesArrow"):
            _write_cached_resources(output_dir, resources)
        timing.finish(
            status="ok",
            resource_rows=len(resources),
            structure_rows=len(structure),
        )
        _write_document_timing_sidecar(output_dir, timing)
        return resources
    except Exception as exc:
        timing.finish(status="error", detail=row_error_message(exc))
        _write_document_timing_sidecar(output_dir, timing)
        if not error_row:
            raise
        return [_image_extract_error_row(source, row_error_message(exc))]


def image_document_payload_for_source(source: Path) -> ImageDocumentPayload:
    """Return an OpenAI-compatible image data URL for one local image."""

    suffix = source.suffix.lower()
    if suffix in DIRECT_IMAGE_MIME_TYPES:
        mime_type = DIRECT_IMAGE_MIME_TYPES[suffix]
        return ImageDocumentPayload(
            data_url=image_bytes_data_url(source.read_bytes(), mime_type),
            mime_type=mime_type,
        )
    if suffix in PNG_CONVERTED_IMAGE_SUFFIXES:
        return ImageDocumentPayload(
            data_url=image_bytes_data_url(_convert_image_to_png_bytes(source), "image/png"),
            mime_type="image/png",
        )
    raise ValueError(f"unsupported image document extension: {suffix}")


def hosted_vlm_headers(config: Ocr2ClientConfig) -> dict[str, str]:
    """Build OpenAI-compatible request headers."""

    headers = {"Content-Type": "application/json", **dict(config.extra_headers or {})}
    if config.api_key and config.api_key != HOSTED_VLM_OCR_DEFAULT_API_KEY:
        headers["Authorization"] = f"Bearer {config.api_key}"
    return headers


def normalize_image_text(value: str) -> str:
    """Normalize hosted VLM image text without changing recognized content."""

    return "\n".join(
        line.rstrip() for line in value.replace("\r", "\n").split("\n")
    ).strip()


def row_error_message(error: BaseException) -> str:
    """Return a bounded error message for resource error rows."""

    if isinstance(error, urllib.error.HTTPError):
        body = http_error_body(error)
        if body:
            return f"{error}; response body: {body}"
    message = str(error)
    if len(message) <= 480:
        return message
    return f"{message[:477]}..."


def http_error_body(error: urllib.error.HTTPError) -> str:
    """Read a bounded HTTP response body from an error."""

    try:
        body = error.read().decode("utf-8", errors="replace").strip()
    except Exception:
        return ""
    if len(body) <= 480:
        return body
    return f"{body[:477]}..."


def _send_completion(
    completion_url: str,
    headers: Mapping[str, str],
    timeout_seconds: float,
    payload: Mapping[str, Any],
) -> tuple[int | None, Any]:
    return send_completion_request(
        completion_url=completion_url,
        headers=headers,
        timeout_seconds=timeout_seconds,
        payload=payload,
    )


def _convert_image_to_png_bytes(source: Path) -> bytes:
    try:
        from PIL import Image
    except ImportError as exc:
        raise RuntimeError("Pillow is required for TIFF/BMP image extraction") from exc
    with Image.open(source) as image:
        converted = image.convert("RGB")
        output = BytesIO()
        converted.save(output, format="PNG", optimize=True)
        return output.getvalue()


def _image_extract_error_row(source: Path, content: str) -> DocumentResourceRow:
    return DocumentResourceRow(
        sourcePath=str(source),
        resourceType="error",
        resourcePath="",
        pageIndex=0,
        caption="",
        content=content,
        mimeType="text/plain",
        status="error",
        elementId="",
    )
