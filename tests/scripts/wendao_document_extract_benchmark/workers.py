"""Local Python document worker startup helpers."""

from __future__ import annotations

from .common import (
    Path,
    os,
    subprocess,
)
from .processes import start_logged_process
from .server_code import fixture_server_code, real_docling_server_code


def start_server(
    host: str,
    port: int,
    *,
    real_docling: bool,
    real_fixture_root: Path | None,
    include_audio: bool,
    converter_count_path: Path | None,
    pdf_ocr_worker: str = "skip",
    pdf_ocr_workers: str = "auto",
    python_uv_package: str | None = "xiuxian-wendao-analyzer",
    python_uv_extras: list[str] | None = None,
    log_dir: Path | None = None,
) -> subprocess.Popen[str]:
    if pdf_ocr_worker == "docling" and not real_docling:
        raise SystemExit("--pdf-ocr-worker docling requires --real-docling")
    if real_docling:
        command = python_worker_command(
            real_docling_server_code(
                host,
                port,
                real_fixture_root,
                include_audio,
                converter_count_path,
                pdf_ocr_worker,
                pdf_ocr_workers,
            ),
            uv_package=python_uv_package,
            uv_extras=python_uv_extras,
        )
    else:
        command = python_worker_command(
            fixture_server_code(
                host,
                port,
                converter_count_path,
                pdf_ocr_worker,
                pdf_ocr_workers,
            ),
            uv_package=python_uv_package,
            uv_extras=python_uv_extras,
        )
    effective_log_dir = log_dir or (
        Path(os.environ.get("PRJ_RUNTIME_DIR", ".run"))
        / "document-extract-perf-process-logs"
    )
    return start_logged_process(
        command, log_dir=effective_log_dir, name="python-worker"
    )


def python_worker_command(
    code: str,
    *,
    uv_package: str | None,
    uv_extras: list[str] | None,
) -> list[str]:
    command = ["uv", "run"]
    if uv_package:
        command.extend(["--package", uv_package])
    for extra in uv_extras or []:
        command.extend(["--extra", extra])
    command.extend(["python", "-c", code])
    return command
