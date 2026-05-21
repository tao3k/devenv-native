from __future__ import annotations

import importlib.util
import os
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path


def _load_module():
    module_name = "session_matrix_report_test_module"
    script_path = Path(__file__).resolve().with_name("session_matrix_report.py")
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load module from {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _webhook_url() -> str:
    host = (
        os.environ.get("XIUXIAN_WENDAO_LOCAL_HOST", "localhost").strip() or "localhost"
    )
    return f"http://{host}:18081/telegram/webhook"


@dataclass(frozen=True)
class _StepResult:
    name: str
    kind: str
    session_key: str | None
    prompt: str | None
    event: str | None
    passed: bool
    duration_ms: int
    stderr_tail: str
    stdout_tail: str


@dataclass(frozen=True)
class _Cfg:
    webhook_url: str
    log_file: Path
    chat_id: int
    chat_b: int
    chat_c: int
    user_a: int
    user_b: int
    user_c: int
    thread_a: int | None
    thread_b: int | None
    thread_c: int | None
    mixed_plain_prompt: str
    forbid_log_regexes: tuple[str, ...]


def test_build_report_render_and_write_outputs(tmp_path: Path) -> None:
    module = _load_module()
    cfg = _Cfg(
        webhook_url=_webhook_url(),
        log_file=tmp_path / "runtime.log",
        chat_id=-1001,
        chat_b=-1002,
        chat_c=-1003,
        user_a=1,
        user_b=2,
        user_c=3,
        thread_a=None,
        thread_b=None,
        thread_c=None,
        mixed_plain_prompt="hello",
        forbid_log_regexes=("tools/call: Tool runtime error",),
    )
    results = [
        _StepResult(
            name="baseline",
            kind="concurrent",
            session_key="-1001:1",
            prompt="/session json",
            event="telegram.command.session_status_json.replied",
            passed=True,
            duration_ms=100,
            stderr_tail="",
            stdout_tail="",
        )
    ]
    report = module.build_report(cfg, results, datetime.now(UTC), 0.0)
    assert report["overall_passed"] is True
    assert report["summary"]["passed"] == 1

    markdown = module.render_markdown(report)
    assert "Agent Channel Session Matrix Report" in markdown
    assert "baseline" in markdown

    output_json = tmp_path / "report.json"
    output_markdown = tmp_path / "report.md"
    module.write_outputs(report, output_json, output_markdown)
    assert output_json.exists()
    assert output_markdown.exists()
