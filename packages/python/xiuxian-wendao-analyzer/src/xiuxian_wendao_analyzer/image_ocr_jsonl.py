"""Queue-keyed image OCR JSONL adapter for cache-only evidence bridges."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import sys
from collections.abc import Callable, Mapping
from pathlib import Path
from typing import Any

from .pdf_ocr_contracts import HOSTED_VLM_OCR_DEFAULT_API_KEY
from .pdf_ocr_ocr2.config import Ocr2ClientConfig, ocr2_client_config_from_env
from .pdf_ocr_ocr2.http import (
    chat_completion_url,
    extract_openai_message_content,
    send_completion_request,
)
from .pdf_ocr_ocr2.payloads import request_payload

IMAGE_OCR_JSONL_RUN_SCHEMA_VERSION = "xiuxian_wendao.image_ocr_jsonl_run.v1"
IMAGE_OCR_JSONL_TASK_ROUTE = "image_ocr_evidence"
IMAGE_OCR_JSONL_DEFAULT_PROFILE = "hosted-vlm-direct-ocr-v1"
IMAGE_OCR_JSONL_TEXT_MIME_TYPE = "text/markdown"

CompletionSender = Callable[
    [str, Mapping[str, str], float, Mapping[str, Any]],
    tuple[int | None, Any],
]


def run_image_ocr_jsonl_tasks(
    *,
    tasks_path: Path,
    corpus_root: Path,
    output_jsonl_path: Path,
    config: Ocr2ClientConfig | None = None,
    send_request: CompletionSender | None = None,
) -> dict[str, Any]:
    """Run hosted VLM OCR for image tasks and write queue-keyed JSONL.

    Raises:
        ValueError: If task rows are malformed or source hashes drift.
        OSError: If input or output paths cannot be read or written.
    """
    resolved_config = config or ocr2_client_config_from_env()
    sender = send_request or _send_completion
    tasks = read_image_ocr_tasks(tasks_path)
    output_rows: list[dict[str, Any]] = []
    errors: list[str] = []
    completion_url = chat_completion_url(resolved_config.base_url)
    headers = hosted_vlm_headers(resolved_config)
    for row in tasks:
        queue_id = row["queue_id"]
        source_path = corpus_root / row["relative_path"]
        try:
            if not source_path.is_file():
                raise ValueError(f"source image does not exist: {source_path}")
            source_sha256 = sha256_file(source_path)
            expected_sha256 = row.get("source_sha256") or row.get("sha256") or ""
            if source_sha256 != expected_sha256:
                raise ValueError(f"source sha256 drift for {queue_id}")
            image_mime_type = image_mime_type_for_path(source_path)
            payload = request_payload(
                model=resolved_config.model,
                prompt=resolved_config.prompt,
                input_row={
                    "imageMimeType": image_mime_type,
                    "shardType": "image",
                    "shardElementId": queue_id,
                },
                image_path=source_path,
                max_tokens=resolved_config.max_tokens,
                image_optimization_mode=resolved_config.image_optimization_mode,
            )
            http_status, response_payload = sender(
                completion_url,
                headers,
                resolved_config.timeout_seconds,
                payload,
            )
            text = normalize_text(extract_openai_message_content(response_payload))
            if not text:
                raise ValueError(f"OCR returned empty text for {queue_id}")
            output_rows.append(
                {
                    "queue_id": queue_id,
                    "text": text,
                    "ocr_engine": "hosted-vlm-openai-compatible",
                    "ocr_profile": IMAGE_OCR_JSONL_DEFAULT_PROFILE,
                    "text_mime_type": IMAGE_OCR_JSONL_TEXT_MIME_TYPE,
                    "model": resolved_config.model,
                    "http_status": http_status,
                    "source_sha256": source_sha256,
                }
            )
        except Exception as exc:  # noqa: BLE001 - report all row-level failures.
            errors.append(f"{queue_id}: {exc}")
    write_jsonl(output_jsonl_path, output_rows)
    return {
        "schema_version": IMAGE_OCR_JSONL_RUN_SCHEMA_VERSION,
        "passed": not errors,
        "tasks_path": str(tasks_path),
        "output_jsonl_path": str(output_jsonl_path),
        "attempted_count": len(tasks),
        "succeeded_count": len(output_rows),
        "failed_count": len(errors),
        "errors": errors,
        "model": resolved_config.model,
        "base_url": resolved_config.base_url,
        "output_contract": "queue_keyed_ocr_jsonl",
    }


def read_image_ocr_tasks(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fieldnames = set(reader.fieldnames or [])
        required = {
            "queue_id",
            "relative_path",
            "extraction_route",
            "source_sha256",
        }
        missing = sorted(required - fieldnames)
        if missing:
            raise ValueError(f"image OCR task TSV missing fields: {', '.join(missing)}")
        rows = [
            dict(row)
            for row in reader
            if row.get("extraction_route") == IMAGE_OCR_JSONL_TASK_ROUTE
        ]
    seen: set[str] = set()
    for row in rows:
        queue_id = row.get("queue_id", "").strip()
        if not queue_id:
            raise ValueError("image OCR task row missing queue_id")
        if queue_id in seen:
            raise ValueError(f"duplicate image OCR queue_id: {queue_id}")
        seen.add(queue_id)
        if not row.get("relative_path", "").strip():
            raise ValueError(f"image OCR task {queue_id} missing relative_path")
        if not row.get("source_sha256", "").strip():
            raise ValueError(f"image OCR task {queue_id} missing source_sha256")
    return rows


def hosted_vlm_headers(config: Ocr2ClientConfig) -> dict[str, str]:
    headers = {"Content-Type": "application/json", **dict(config.extra_headers or {})}
    if config.api_key and config.api_key != HOSTED_VLM_OCR_DEFAULT_API_KEY:
        headers["Authorization"] = f"Bearer {config.api_key}"
    return headers


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


def image_mime_type_for_path(path: Path) -> str:
    extension = path.suffix.lower()
    if extension in {".jpg", ".jpeg"}:
        return "image/jpeg"
    if extension == ".png":
        return "image/png"
    raise ValueError(f"unsupported image OCR extension: {extension}")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def normalize_text(value: str) -> str:
    return "\n".join(
        line.rstrip() for line in value.replace("\r", "\n").split("\n")
    ).strip()


def write_jsonl(path: Path, rows: list[Mapping[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(dict(row), ensure_ascii=False, sort_keys=True))
            handle.write("\n")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Run hosted VLM OCR for image task TSV rows."
    )
    parser.add_argument(
        "--tasks", required=True, help="Path to a source-contract tasks.tsv file."
    )
    parser.add_argument(
        "--corpus-root", required=True, help="Root directory for task relative paths."
    )
    parser.add_argument(
        "--output-jsonl", required=True, help="Queue-keyed OCR JSONL output path."
    )
    args = parser.parse_args(argv)
    try:
        report = run_image_ocr_jsonl_tasks(
            tasks_path=Path(args.tasks).expanduser().resolve(),
            corpus_root=Path(args.corpus_root).expanduser().resolve(),
            output_jsonl_path=Path(args.output_jsonl).expanduser().resolve(),
        )
    except Exception as exc:  # noqa: BLE001 - CLI prints deterministic JSON failures.
        report = {
            "schema_version": IMAGE_OCR_JSONL_RUN_SCHEMA_VERSION,
            "passed": False,
            "errors": [str(exc)],
        }
    print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
