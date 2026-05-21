from __future__ import annotations

import importlib.util
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path


def _load_module():
    module_name = "command_events_report_test_module"
    script_path = Path(__file__).resolve().with_name("command_events_report.py")
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load module from {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


@dataclass(frozen=True)
class _Attempt:
    mode: str
    case_id: str
    prompt: str
    event_name: str
    suites: tuple[str, ...]
    chat_id: int | None
    user_id: int | None
    thread_id: int | None
    attempt: int
    max_attempts: int
    returncode: int
    passed: bool
    duration_ms: int
    retry_scheduled: bool


def test_build_report_and_markdown_rendering(tmp_path: Path) -> None:
    module = _load_module()
    started_dt = datetime.now(UTC)
    attempts = [
        _Attempt(
            mode="default",
            case_id="session_status_json",
            prompt="/session json",
            event_name="telegram.command.session_status_json.replied",
            suites=("core",),
            chat_id=-1001,
            user_id=42,
            thread_id=None,
            attempt=1,
            max_attempts=1,
            returncode=0,
            passed=True,
            duration_ms=120,
            retry_scheduled=False,
        )
    ]
    report = module.build_report(
        suites=("all",),
        case_ids=("session_status_json",),
        allow_chat_ids=("-1001",),
        matrix_chat_ids=tuple(),
        attempts=attempts,
        started_dt=started_dt,
        started_mono=0.0,
        exit_code=0,
        runtime_partition_mode="chat_user",
        admin_matrix=False,
        assert_admin_isolation=False,
        assert_admin_topic_isolation=False,
        group_thread_id=None,
        group_thread_id_b=None,
        max_wait=25,
        max_idle_secs=25,
        matrix_retries=2,
        matrix_backoff_secs=2.0,
    )

    assert report["summary"]["passed"] == 1
    assert report["overall_passed"] is True
    markdown = module.render_markdown(report)
    assert "Agent Channel Command Events Report" in markdown
    assert "`session_status_json`" in markdown

    output_json = tmp_path / "report.json"
    output_markdown = tmp_path / "report.md"
    module.write_outputs(report, output_json, output_markdown)
    assert output_json.exists()
    assert output_markdown.exists()
