"""document_service test slice 3."""

from __future__ import annotations

import json

from xiuxian_wendao_analyzer.pdf_ocr import (
    HOSTED_VLM_OCR_BASE_URL_ENV,
    HOSTED_VLM_OCR_SCAFFOLD_MODE_ENV,
    HOSTED_VLM_OCR_TRACE_PATH_ENV,
    PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
)

from .support import (
    DoclingPdfOcrShardWorker,
    Path,
    _sample_pdf_ocr_input_table,
    build_pdf_ocr_shard_result_table,
)
from .support_scaffold import (
    INVALID_REGION_SCAFFOLD_CASES,
    write_ocr2_region_scaffold_sidecar,
)


def test_docling_pdf_ocr_worker_fails_invalid_region_scaffolds(
    tmp_path: Path,
    monkeypatch,
) -> None:
    cases = INVALID_REGION_SCAFFOLD_CASES

    class FakeResponse:
        status = 200

        def __init__(self, content: str) -> None:
            self._content = content

        def __enter__(self) -> FakeResponse:
            return self

        def __exit__(self, *args: object) -> None:
            _ = args

        def read(self) -> bytes:
            return json.dumps(
                {"choices": [{"message": {"content": self._content}}]}
            ).encode("utf-8")

    requests: list[object] = []
    responses: list[str] = []
    trace_path = tmp_path / "invalid-scaffold-trace.jsonl"

    def fake_urlopen(request: object, *, timeout: float) -> FakeResponse:
        _ = timeout
        requests.append(request)
        return FakeResponse(responses.pop(0))

    monkeypatch.setenv(HOSTED_VLM_OCR_BASE_URL_ENV, "http://127.0.0.1:8999/v1")
    monkeypatch.setenv(HOSTED_VLM_OCR_SCAFFOLD_MODE_ENV, "region-table-json")
    monkeypatch.setenv(HOSTED_VLM_OCR_TRACE_PATH_ENV, str(trace_path))
    monkeypatch.setattr(
        "xiuxian_wendao_analyzer.pdf_ocr_ocr2.http.urllib.request.urlopen",
        fake_urlopen,
    )

    for index, (_label, raster_sha256, response) in enumerate(cases):
        case_dir = tmp_path / f"case-{index}"
        case_dir.mkdir()
        image = case_dir / "region-00001.png"
        image.write_bytes(b"region png fixture")
        input_table = _sample_pdf_ocr_input_table(
            image_path=str(image),
            shard_element_id="region-a",
            shard_type="region",
            region_index=1,
            parent_shard_element_id="parent-page",
            ocr_profile=PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
        )
        input_row = input_table.to_pylist()[0]
        if raster_sha256 is not None:
            write_ocr2_region_scaffold_sidecar(
                case_dir,
                [input_row],
                raster_sha256=raster_sha256,
            )
        if response is not None:
            responses.append(response)

        table = build_pdf_ocr_shard_result_table(
            input_table,
            worker=DoclingPdfOcrShardWorker(max_workers=1),
        )

        row = table.to_pylist()[0]
        assert row["status"] == "failed"
        assert "Hosted VLM/OCR failed" in row["errorMessage"]

    trace_records = [
        json.loads(line) for line in trace_path.read_text(encoding="utf-8").splitlines()
    ]
    response_validation_records = [
        record
        for record in trace_records
        if record["status"] == "failed" and record["httpStatus"] == 200
    ]
    assert response_validation_records
    assert all(
        record["scaffoldJsonChars"] > 0 for record in response_validation_records
    )
