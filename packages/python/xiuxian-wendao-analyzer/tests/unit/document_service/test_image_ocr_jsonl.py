from __future__ import annotations

import csv
import hashlib
import json
from pathlib import Path
from typing import Any

from xiuxian_wendao_analyzer.image_ocr_jsonl import run_image_ocr_jsonl_tasks
from xiuxian_wendao_analyzer.pdf_ocr_ocr2.config import Ocr2ClientConfig


def _config() -> Ocr2ClientConfig:
    return Ocr2ClientConfig(
        base_url="https://example.test/v1",
        model="test/ocr-model",
        api_key="test-token",
        prompt="Convert the image to markdown.",
        max_tokens=512,
        region_max_tokens=256,
        region_composite_size=1,
        region_atlas_mode="disabled",
        timeout_seconds=30.0,
        request_concurrency=1,
        speculative_retry_delay_seconds=0.0,
        page_window_size=1,
        scaffold_mode="disabled",
        image_optimization_mode="disabled",
        extra_headers={"HTTP-Referer": "https://wendao.local"},
    )


def _write_tasks(
    path: Path, *, source_sha256: str, relative_path: str = "images/a.jpg"
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(
            handle,
            delimiter="\t",
            fieldnames=[
                "queue_id",
                "file_id",
                "relative_path",
                "category",
                "language",
                "extraction_route",
                "priority",
                "source_sha256",
                "planned_output_path",
                "output_contract",
                "status",
            ],
        )
        writer.writeheader()
        writer.writerow(
            {
                "queue_id": "ltc.extract.image.001",
                "file_id": "ltc.file.image.001",
                "relative_path": relative_path,
                "category": "synthetic",
                "language": "zh-CN",
                "extraction_route": "image_ocr_evidence",
                "priority": "normal",
                "source_sha256": source_sha256,
                "planned_output_path": "outputs/ltc.extract.image.001.json",
                "output_contract": "cache_only_no_rdf_promotion",
                "status": "planned",
            }
        )


def test_image_ocr_jsonl_adapter_writes_queue_keyed_sidecar(tmp_path: Path) -> None:
    corpus_root = tmp_path / "corpus"
    image_path = corpus_root / "images" / "a.jpg"
    image_path.parent.mkdir(parents=True)
    image_bytes = b"\xff\xd8synthetic-jpeg\xff\xd9"
    image_path.write_bytes(image_bytes)
    tasks_path = tmp_path / "run" / "tasks.tsv"
    _write_tasks(tasks_path, source_sha256=hashlib.sha256(image_bytes).hexdigest())
    output_jsonl = tmp_path / "run" / "ocr_results.jsonl"
    requests: list[dict[str, Any]] = []

    def fake_sender(
        completion_url: str,
        headers: dict[str, str],
        timeout_seconds: float,
        payload: dict[str, Any],
    ) -> tuple[int, dict[str, Any]]:
        requests.append(
            {
                "completion_url": completion_url,
                "headers": headers,
                "timeout_seconds": timeout_seconds,
                "payload": payload,
            }
        )
        return 200, {"choices": [{"message": {"content": "\n# OCR text\n"}}]}

    report = run_image_ocr_jsonl_tasks(
        tasks_path=tasks_path,
        corpus_root=corpus_root,
        output_jsonl_path=output_jsonl,
        config=_config(),
        send_request=fake_sender,
    )

    assert report["passed"] is True
    assert report["succeeded_count"] == 1
    row = json.loads(output_jsonl.read_text(encoding="utf-8"))
    assert row["queue_id"] == "ltc.extract.image.001"
    assert row["text"] == "# OCR text"
    assert row["ocr_profile"] == "hosted-vlm-direct-ocr-v1"
    assert row["text_mime_type"] == "text/markdown"
    assert row["model"] == "test/ocr-model"
    request = requests[0]
    assert request["completion_url"] == "https://example.test/v1/chat/completions"
    assert request["headers"]["Authorization"] == "Bearer test-token"
    assert request["headers"]["HTTP-Referer"] == "https://wendao.local"
    payload = request["payload"]
    assert payload["model"] == "test/ocr-model"
    image_url = payload["messages"][0]["content"][1]["image_url"]["url"]
    assert image_url.startswith("data:image/jpeg;base64,")


def test_image_ocr_jsonl_adapter_blocks_source_hash_drift(tmp_path: Path) -> None:
    corpus_root = tmp_path / "corpus"
    image_path = corpus_root / "images" / "a.jpg"
    image_path.parent.mkdir(parents=True)
    image_path.write_bytes(b"\xff\xd8changed\xff\xd9")
    tasks_path = tmp_path / "run" / "tasks.tsv"
    _write_tasks(tasks_path, source_sha256="0" * 64)
    output_jsonl = tmp_path / "run" / "ocr_results.jsonl"

    def unexpected_sender(
        completion_url: str,
        headers: dict[str, str],
        timeout_seconds: float,
        payload: dict[str, Any],
    ) -> tuple[int, dict[str, Any]]:
        raise AssertionError("OCR request should not be sent on hash drift")

    report = run_image_ocr_jsonl_tasks(
        tasks_path=tasks_path,
        corpus_root=corpus_root,
        output_jsonl_path=output_jsonl,
        config=_config(),
        send_request=unexpected_sender,
    )

    assert report["passed"] is False
    assert report["succeeded_count"] == 0
    assert report["failed_count"] == 1
    assert "sha256 drift" in report["errors"][0]
    assert output_jsonl.read_text(encoding="utf-8") == ""


def test_image_ocr_jsonl_adapter_blocks_corpus_path_escape(tmp_path: Path) -> None:
    corpus_root = tmp_path / "corpus"
    outside_path = tmp_path / "outside" / "a.jpg"
    outside_path.parent.mkdir(parents=True)
    image_bytes = b"\xff\xd8outside-jpeg\xff\xd9"
    outside_path.write_bytes(image_bytes)
    tasks_path = tmp_path / "run" / "tasks.tsv"
    _write_tasks(
        tasks_path,
        source_sha256=hashlib.sha256(image_bytes).hexdigest(),
        relative_path="../outside/a.jpg",
    )
    output_jsonl = tmp_path / "run" / "ocr_results.jsonl"

    def unexpected_sender(
        completion_url: str,
        headers: dict[str, str],
        timeout_seconds: float,
        payload: dict[str, Any],
    ) -> tuple[int, dict[str, Any]]:
        raise AssertionError("OCR request should not be sent on path escape")

    report = run_image_ocr_jsonl_tasks(
        tasks_path=tasks_path,
        corpus_root=corpus_root,
        output_jsonl_path=output_jsonl,
        config=_config(),
        send_request=unexpected_sender,
    )

    assert report["passed"] is False
    assert report["succeeded_count"] == 0
    assert "escapes corpus root" in report["errors"][0]
    assert output_jsonl.read_text(encoding="utf-8") == ""


def test_image_ocr_jsonl_adapter_blocks_corpus_path_escape(tmp_path: Path) -> None:
    corpus_root = tmp_path / "corpus"
    outside_path = tmp_path / "outside" / "a.jpg"
    outside_path.parent.mkdir(parents=True)
    image_bytes = b"\xff\xd8outside-jpeg\xff\xd9"
    outside_path.write_bytes(image_bytes)
    tasks_path = tmp_path / "run" / "tasks.tsv"
    _write_tasks(
        tasks_path,
        source_sha256=hashlib.sha256(image_bytes).hexdigest(),
        relative_path="../outside/a.jpg",
    )
    output_jsonl = tmp_path / "run" / "ocr_results.jsonl"

    def unexpected_sender(
        completion_url: str,
        headers: dict[str, str],
        timeout_seconds: float,
        payload: dict[str, Any],
    ) -> tuple[int, dict[str, Any]]:
        raise AssertionError("OCR request should not be sent on path escape")

    report = run_image_ocr_jsonl_tasks(
        tasks_path=tasks_path,
        corpus_root=corpus_root,
        output_jsonl_path=output_jsonl,
        config=_config(),
        send_request=unexpected_sender,
    )

    assert report["passed"] is False
    assert report["succeeded_count"] == 0
    assert report["failed_count"] == 1
    assert "escapes corpus root" in report["errors"][0]
    assert output_jsonl.read_text(encoding="utf-8") == ""
