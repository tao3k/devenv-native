from __future__ import annotations

import argparse
import tempfile
from pathlib import Path

import qianji_wendao_ai_live_gate as live_gate


def test_console_status_prefers_control_summary() -> None:
    assert (
        live_gate.console_status(
            {
                "state": "connected",
                "controlSummary": {"status": "completed"},
                "diagnostics": {"summary": {"status": "running"}},
            }
        )
        == "completed"
    )


def test_blocked_status_is_intermediate_not_terminal_failure() -> None:
    assert live_gate.terminal_failure_status("blocked") is False
    assert live_gate.terminal_failure_status("running") is False
    assert live_gate.terminal_failure_status("completed") is False
    assert live_gate.terminal_failure_status("failed") is True
    assert live_gate.terminal_failure_status("error") is True


def test_validate_console_summary_counts_sources_and_errors() -> None:
    args = argparse.Namespace(min_stream_rows=1, require_source=("bpmn", "tool", "llm"))
    summary = live_gate.validate_console_summary(
        name="wf-test",
        workflow_id="wf-test",
        instance_id="instance-test",
        control_run_id="bpmn.workflow.instance-test",
        status="completed",
        worker_iterations=3,
        elapsed_ms=42,
        args=args,
        console={
            "stream": {
                "rows": [
                    {"source": "bpmn", "kind": "step_started", "message": "started"},
                    {"source": "tool", "kind": "activity_scheduled", "message": "scheduled"},
                    {"source": "llm", "kind": "activity_completed", "message": "completed"},
                ]
            }
        },
    )

    assert summary.source_counts == {"bpmn": 1, "tool": 1, "llm": 1}
    assert summary.error_count == 0
    assert summary.row_count == 3


def test_validate_console_summary_rejects_missing_required_source() -> None:
    args = argparse.Namespace(min_stream_rows=1, require_source=("bpmn", "tool", "llm"))

    try:
        live_gate.validate_console_summary(
            name="wf-test",
            workflow_id="wf-test",
            instance_id="instance-test",
            control_run_id="bpmn.workflow.instance-test",
            status="completed",
            worker_iterations=1,
            elapsed_ms=1,
            args=args,
            console={"stream": {"rows": [{"source": "bpmn", "kind": "step_started"}]}},
        )
    except RuntimeError as exc:
        assert "missing required sources" in str(exc)
    else:
        raise AssertionError("expected missing-source rejection")


def test_bpmn_process_id_reads_namespaced_process() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        bpmn_path = Path(tmpdir) / "process.bpmn"
        bpmn_path.write_text(
            """<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <bpmn:process id="Process_test" isExecutable="true" />
</bpmn:definitions>
""",
            encoding="utf-8",
        )

        assert live_gate.bpmn_process_id(bpmn_path) == "Process_test"


def test_quote_path_encodes_control_run_id() -> None:
    assert live_gate.quote_path("bpmn.workflow.wf-1/run") == "bpmn.workflow.wf-1%2Frun"
