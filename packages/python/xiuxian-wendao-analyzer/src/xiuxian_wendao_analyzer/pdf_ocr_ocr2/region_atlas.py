"""OCR2 same-page region atlas recognition path."""

from __future__ import annotations

import io
import time
import urllib.error
from pathlib import Path
from typing import TYPE_CHECKING, Any, Protocol

from .http import extract_openai_message_content
from .payloads import region_atlas_request_payload
from .results import succeeded_markdown_results
from .scaffold import extract_ocr2_scaffold_markdown

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


class RegionAtlasClient(Protocol):
    _model: str
    _prompt: str

    def _max_tokens_for_region_composite(
        self,
        input_rows: Sequence[Mapping[str, Any]],
    ) -> int: ...

    def _send_completion_request(
        self, payload: Mapping[str, Any]
    ) -> tuple[int | None, Any]: ...

    def _write_trace(
        self,
        input_row: Mapping[str, Any],
        *,
        status: str,
        started: float,
        http_status: int | None,
        image_bytes: int,
        markdown_chars: int,
        error: BaseException | None,
        page_count: int = 1,
        input_rows: Sequence[Mapping[str, Any]] | None = None,
        max_tokens: int | None = None,
        request_kind: str | None = None,
        scaffold_applied_count: int = 0,
        scaffold_validation_failure_count: int = 0,
        scaffold_json_chars: int = 0,
        canonical_markdown_chars: int = 0,
    ) -> None: ...


class RegionAtlasResponseValidationError(ValueError):
    def __init__(self, message: str, response_text: str) -> None:
        super().__init__(message)
        self.response_text = response_text


def try_recognize_region_atlas(
    client: RegionAtlasClient,
    input_rows: Sequence[Mapping[str, Any]],
) -> list[Mapping[str, Any]] | None:
    image_paths = [Path(str(input_row["imagePath"])) for input_row in input_rows]
    missing_path = next(
        (image_path for image_path in image_paths if not image_path.is_file()), None
    )
    if missing_path is not None:
        return None
    max_tokens = client._max_tokens_for_region_composite(input_rows)
    started = time.perf_counter()
    http_status = None
    try:
        atlas_image_bytes = build_region_atlas_png(image_paths, input_rows)
        payload = region_atlas_request_payload(
            model=client._model,
            prompt=client._prompt,
            input_rows=input_rows,
            atlas_image_bytes=atlas_image_bytes,
            max_tokens=max_tokens,
        )
        http_status, response_payload = client._send_completion_request(payload)
        response_text = extract_openai_message_content(response_payload)
        try:
            region_texts = extract_ocr2_scaffold_markdown(response_text, input_rows)
        except ValueError as exc:
            raise RegionAtlasResponseValidationError(str(exc), response_text) from exc
    except urllib.error.HTTPError as exc:
        _write_failure_trace(
            client,
            input_rows,
            exc,
            exc.code,
            0,
            max_tokens,
            started,
        )
        return None
    except (OSError, ValueError, urllib.error.URLError) as exc:
        _write_failure_trace(
            client,
            input_rows,
            exc,
            http_status,
            _atlas_image_bytes_len(locals()),
            max_tokens,
            started,
        )
        return None
    markdown_chars = sum(len(text) for text in region_texts)
    client._write_trace(
        input_rows[0],
        status="succeeded",
        started=started,
        http_status=http_status,
        image_bytes=len(atlas_image_bytes),
        markdown_chars=markdown_chars,
        error=None,
        input_rows=input_rows,
        max_tokens=max_tokens,
        request_kind="region-atlas",
        scaffold_json_chars=len(response_text),
        canonical_markdown_chars=markdown_chars,
    )
    return succeeded_markdown_results(region_texts)


def build_region_atlas_png(
    image_paths: Sequence[Path],
    input_rows: Sequence[Mapping[str, Any]],
) -> bytes:
    from PIL import Image, ImageDraw, ImageFont

    if len(image_paths) != len(input_rows):
        raise ValueError("OCR2 region atlas image and row count mismatch")
    panels = [Image.open(image_path).convert("RGB") for image_path in image_paths]
    try:
        label_height = 96
        gap = 12
        margin = 16
        atlas_width = max(panel.width for panel in panels) + margin * 2
        atlas_height = (
            sum(panel.height + label_height + gap for panel in panels) + margin
        )
        atlas = Image.new("RGB", (atlas_width, atlas_height), "white")
        draw = ImageDraw.Draw(atlas)
        font = ImageFont.load_default()
        y = margin
        for index, (panel, row) in enumerate(
            zip(panels, input_rows, strict=True),
            start=1,
        ):
            label = (
                f"REGION {index}  PAGE {row.get('pageIndex')}  "
                f"SHARD {row.get('shardElementId')}"
            )
            draw.rectangle(
                (margin, y, atlas_width - margin, y + label_height - 1),
                fill="white",
                outline="black",
                width=3,
            )
            _paste_scaled_label(atlas, label, margin + 12, y + 16, font)
            y += label_height
            atlas.paste(panel, (margin, y))
            draw.rectangle(
                (margin, y, margin + panel.width - 1, y + panel.height - 1),
                outline="black",
                width=2,
            )
            y += panel.height + gap
        output = io.BytesIO()
        atlas.save(output, format="PNG", optimize=True)
        return output.getvalue()
    finally:
        for panel in panels:
            panel.close()


def _paste_scaled_label(
    atlas: Any,
    label: str,
    x: int,
    y: int,
    font: Any,
) -> None:
    from PIL import Image, ImageDraw

    width = max(len(label) * 8, 1)
    label_image = Image.new("RGB", (width + 12, 20), "white")
    ImageDraw.Draw(label_image).text((6, 4), label, fill="black", font=font)
    label_image = label_image.resize(
        (label_image.width * 4, label_image.height * 4),
        Image.Resampling.NEAREST,
    )
    atlas.paste(label_image, (x, y))


def _write_failure_trace(
    client: RegionAtlasClient,
    input_rows: Sequence[Mapping[str, Any]],
    error: BaseException,
    http_status: int | None,
    image_bytes: int,
    max_tokens: int,
    started: float,
) -> None:
    client._write_trace(
        input_rows[0],
        status="failed",
        started=started,
        http_status=http_status,
        image_bytes=image_bytes,
        markdown_chars=0,
        error=error,
        input_rows=input_rows,
        max_tokens=max_tokens,
        request_kind="region-atlas",
        scaffold_validation_failure_count=len(input_rows),
        scaffold_json_chars=_response_text_len(error),
    )


def _atlas_image_bytes_len(local_values: Mapping[str, Any]) -> int:
    atlas_image_bytes = local_values.get("atlas_image_bytes")
    if isinstance(atlas_image_bytes, bytes):
        return len(atlas_image_bytes)
    return 0


def _response_text_len(error: BaseException) -> int:
    response_text = getattr(error, "response_text", "")
    if isinstance(response_text, str):
        return len(response_text)
    return 0
