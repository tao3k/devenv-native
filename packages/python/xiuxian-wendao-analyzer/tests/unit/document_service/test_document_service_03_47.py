"""document_service test slice 47."""

from __future__ import annotations

import json

from xiuxian_wendao_analyzer.pdf_ocr import (
    HOSTED_VLM_OCR_BASE_URL_ENV,
    HOSTED_VLM_OCR_REGION_COMPOSITE_MAX_SOURCE_PIXELS_ENV,
    HOSTED_VLM_OCR_REGION_COMPOSITE_MODE_ENV,
    HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV,
    PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
)

from .support import (
    DoclingPdfOcrShardWorker,
    Path,
    _sample_pdf_ocr_input_table,
    build_pdf_ocr_shard_result_table,
    pa,
)


def _region_input_table(
    source: Path,
    image_paths: list[Path],
    *,
    source_pixel_right: int,
    source_pixel_bottom: int,
) -> pa.Table:
    rows = []
    schema = None
    for index, image_path in enumerate(image_paths):
        table = _sample_pdf_ocr_input_table(
            source_path=str(source),
            image_path=str(image_path),
            page_index=0,
            shard_element_id=f"region-{index}",
            shard_type="region",
            region_index=index + 1,
            parent_shard_element_id="parent-page",
            reading_order_key=f"000000.0{index + 1}0000",
            ocr_profile=PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
        )
        schema = table.schema
        row = table.to_pylist()[0]
        row["sourcePagePixelRight"] = source_pixel_right
        row["sourcePagePixelBottom"] = source_pixel_bottom
        rows.append(row)
    assert schema is not None
    return pa.Table.from_pylist(rows, schema=schema)


def test_docling_pdf_ocr_worker_adaptive_composites_small_regions(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    region_images = [tmp_path / f"region-{index:05}.png" for index in range(2)]
    for image in region_images:
        image.write_bytes(b"small region png")
    request_image_counts: list[int] = []

    class FakeResponse:
        status = 200

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {
                    "choices": [
                        {
                            "message": {
                                "content": (
                                    "<!-- xiuxian-wendao-hosted-vlm-region:0:1:region-0 -->\n"
                                    "small A\n"
                                    "<!-- xiuxian-wendao-hosted-vlm-region:0:2:region-1 -->\n"
                                    "small B\n"
                                )
                            }
                        }
                    ]
                }
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        payload = json.loads(request.data.decode("utf-8"))
        request_image_counts.append(
            sum(
                1
                for part in payload["messages"][0]["content"]
                if part["type"] == "image_url"
            )
        )
        return FakeResponse()

    monkeypatch.setenv(HOSTED_VLM_OCR_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setenv(
        HOSTED_VLM_OCR_REGION_COMPOSITE_MODE_ENV,
        "adaptive-small-region",
    )
    monkeypatch.setenv(HOSTED_VLM_OCR_REGION_COMPOSITE_MAX_SOURCE_PIXELS_ENV, "30000")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )

    table = build_pdf_ocr_shard_result_table(
        _region_input_table(
            source,
            region_images,
            source_pixel_right=100,
            source_pixel_bottom=100,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["text"] for row in table.to_pylist()] == ["small A", "small B"]
    assert request_image_counts == [2]


def test_docling_pdf_ocr_worker_adaptive_keeps_dense_regions_singleton(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    region_images = [tmp_path / f"region-{index:05}.png" for index in range(2)]
    for image in region_images:
        image.write_bytes(b"dense region png")
    request_image_counts: list[int] = []

    class FakeResponse:
        status = 200

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {
                    "choices": [
                        {"message": {"content": f"single {len(request_image_counts)}"}}
                    ]
                }
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        payload = json.loads(request.data.decode("utf-8"))
        request_image_counts.append(
            sum(
                1
                for part in payload["messages"][0]["content"]
                if part["type"] == "image_url"
            )
        )
        return FakeResponse()

    monkeypatch.setenv(HOSTED_VLM_OCR_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setenv(
        HOSTED_VLM_OCR_REGION_COMPOSITE_MODE_ENV,
        "adaptive-small-region",
    )
    monkeypatch.setenv(HOSTED_VLM_OCR_REGION_COMPOSITE_MAX_SOURCE_PIXELS_ENV, "30000")
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )

    table = build_pdf_ocr_shard_result_table(
        _region_input_table(
            source,
            region_images,
            source_pixel_right=2400,
            source_pixel_bottom=3100,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["text"] for row in table.to_pylist()] == ["single 1", "single 2"]
    assert request_image_counts == [1, 1]


def test_docling_pdf_ocr_worker_adaptive_default_budget_rejects_moderate_pair(
    tmp_path: Path,
    monkeypatch,
) -> None:
    source = tmp_path / "source.pdf"
    source.write_bytes(b"%PDF fixture")
    region_images = [tmp_path / f"region-{index:05}.png" for index in range(2)]
    for image in region_images:
        image.write_bytes(b"moderate region png")
    request_image_counts: list[int] = []

    class FakeResponse:
        status = 200

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {
                    "choices": [
                        {"message": {"content": f"single {len(request_image_counts)}"}}
                    ]
                }
            ).encode("utf-8")

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        payload = json.loads(request.data.decode("utf-8"))
        request_image_counts.append(
            sum(
                1
                for part in payload["messages"][0]["content"]
                if part["type"] == "image_url"
            )
        )
        return FakeResponse()

    monkeypatch.setenv(HOSTED_VLM_OCR_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(HOSTED_VLM_OCR_REGION_COMPOSITE_SIZE_ENV, "2")
    monkeypatch.setenv(
        HOSTED_VLM_OCR_REGION_COMPOSITE_MODE_ENV,
        "adaptive-small-region",
    )
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )

    table = build_pdf_ocr_shard_result_table(
        _region_input_table(
            source,
            region_images,
            source_pixel_right=1800,
            source_pixel_bottom=1000,
        ),
        worker=DoclingPdfOcrShardWorker(max_workers=1),
    )

    assert [row["text"] for row in table.to_pylist()] == ["single 1", "single 2"]
    assert request_image_counts == [1, 1]
