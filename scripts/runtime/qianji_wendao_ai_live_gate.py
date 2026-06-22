#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from pathlib import Path
from typing import Any

DEFAULT_WORKFLOWS = ("wf-1", "wf-2", "wf-3", "wf-4", "wf-5", "wf-6")
DEFAULT_REQUIRED_SOURCES = ("bpmn", "tool", "llm")


@dataclass(frozen=True)
class StreamObservation:
    iteration: int
    status: str
    row_count: int
    source_counts: dict[str, int]


@dataclass(frozen=True)
class RunSummary:
    name: str
    workflow_id: str
    instance_id: str
    control_run_id: str
    status: str
    worker_iterations: int
    elapsed_ms: int
    row_count: int
    source_counts: dict[str, int]
    error_count: int
    stream_observations: list[StreamObservation]


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    results: list[RunSummary] = []
    started_at = time.time()

    workflows = DEFAULT_WORKFLOWS if args.workflow == ["all"] else tuple(args.workflow)
    for workflow_id in workflows:
        results.append(run_wendao_ai_workflow(workflow_id, args))

    if args.include_pi_complex:
        results.append(run_pi_complex_workflow(args))

    payload = {
        "schema": "xiuxian.qianji.wendao_ai.live_gate.v1",
        "elapsed_ms": int((time.time() - started_at) * 1000),
        "wendao_ai_origin": args.wendao_ai_origin,
        "qianji_server_origin": args.qianji_server_origin,
        "required_sources": list(args.require_source),
        "runs": [summary_to_json(result) for result in results],
    }
    print(json.dumps(payload, indent=2, sort_keys=True))
    if args.evidence:
        write_evidence(payload, args.evidence)
    return 0


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    project_root = Path(
        os.environ.get("PRJ_ROOT", Path(__file__).resolve().parents[2])
    ).resolve()
    parser = argparse.ArgumentParser(
        description=(
            "Run a live qianji-server and wendao.ai workflow gate through the "
            "same local API routes used by the frontend."
        )
    )
    parser.add_argument(
        "--wendao-ai-origin",
        default=os.environ.get("WENDAO_AI_ORIGIN", "http://127.0.0.1:9518"),
    )
    parser.add_argument(
        "--qianji-server-origin",
        default=os.environ.get("QIANJI_SERVER_ORIGIN", "http://127.0.0.1:38130"),
    )
    parser.add_argument(
        "--workflow",
        action="append",
        default=None,
        help="Built-in wendao.ai workflow id to run. Use all for wf-1..wf-6.",
    )
    parser.add_argument(
        "--include-pi-complex",
        action="store_true",
        help="Also run the pi-wendao complex BPMN fixture through qianji-server start and wendao.ai worker control.",
    )
    parser.add_argument(
        "--pi-complex-bpmn-path",
        default=str(project_root / ".data/pi-wendao/test/fixtures/complex-workflow.bpmn"),
    )
    parser.add_argument("--max-worker-iterations", type=int, default=12)
    parser.add_argument("--poll-delay-seconds", type=float, default=0.2)
    parser.add_argument("--request-timeout-seconds", type=float, default=120.0)
    parser.add_argument("--min-stream-rows", type=int, default=1)
    parser.add_argument(
        "--allow-non-incremental-stream",
        action="store_true",
        help="Do not require multi-step runs to expose growing stream rows before completion.",
    )
    parser.add_argument(
        "--require-source",
        action="append",
        default=None,
        help="Require at least one durable stream row for this source. May be repeated.",
    )
    parser.add_argument(
        "--evidence",
        default=None,
        help="Optional evidence JSON path. Parent directories are created.",
    )
    parsed = parser.parse_args(argv)
    parsed.workflow = parsed.workflow or ["all"]
    parsed.require_source = tuple(parsed.require_source or DEFAULT_REQUIRED_SOURCES)
    return parsed


def run_wendao_ai_workflow(workflow_id: str, args: argparse.Namespace) -> RunSummary:
    start_time = time.time()
    start = post_json(
        f"{args.wendao_ai_origin}/api/qianji/workflows/{quote_path(workflow_id)}/start",
        {},
        args.request_timeout_seconds,
    )
    instance_id = require_str(start, "instanceId")
    control_run_id = require_str(start, "controlRunId")
    process_id = require_str(start, "processId")
    bpmn_path = require_str(start, "bpmnPath")
    return drive_worker_until_complete(
        name=workflow_id,
        workflow_id=workflow_id,
        instance_id=instance_id,
        control_run_id=control_run_id,
        process_id=process_id,
        bpmn_path=bpmn_path,
        args=args,
        started_at=start_time,
    )


def run_pi_complex_workflow(args: argparse.Namespace) -> RunSummary:
    start_time = time.time()
    bpmn_path = Path(args.pi_complex_bpmn_path).resolve()
    if not bpmn_path.is_file():
        raise RuntimeError(f"pi-wendao complex BPMN fixture is missing: {bpmn_path}")
    process_id = bpmn_process_id(bpmn_path)
    instance_id = f"pi-complex-e2e-{int(start_time * 1000)}"
    start = post_json(
        f"{args.qianji_server_origin}/workflows/start",
        {
            "bpmn_path": str(bpmn_path),
            "process_id": process_id,
            "instance_id": instance_id,
            "initial_variables": {
                "source": "qianji-wendao-ai-live-gate",
                "workflowId": "pi-complex",
                "workflowName": "pi-wendao complex BPMN fixture",
            },
        },
        args.request_timeout_seconds,
    )
    if "workflow" not in start:
        raise RuntimeError(f"qianji-server complex BPMN start returned no workflow: {start}")
    return drive_worker_until_complete(
        name="pi-complex",
        workflow_id="pi-complex",
        instance_id=instance_id,
        control_run_id=f"bpmn.workflow.{instance_id}",
        process_id=process_id,
        bpmn_path=str(bpmn_path),
        args=args,
        started_at=start_time,
    )


def drive_worker_until_complete(
    *,
    name: str,
    workflow_id: str,
    instance_id: str,
    control_run_id: str,
    process_id: str,
    bpmn_path: str,
    args: argparse.Namespace,
    started_at: float,
) -> RunSummary:
    console: dict[str, Any] | None = None
    last_worker_result: dict[str, Any] | None = None
    observations: list[StreamObservation] = []
    for iteration in range(1, args.max_worker_iterations + 1):
        last_worker_result = post_json(
            (
                f"{args.wendao_ai_origin}/api/qianji/runs/"
                f"{quote_path(control_run_id)}/workers/openai-compatible-llm/run"
            ),
            {
                "workflowId": workflow_id,
                "instanceId": instance_id,
                "processId": process_id,
                "bpmnPath": bpmn_path,
            },
            args.request_timeout_seconds,
        )
        console = get_console(args, workflow_id, instance_id, control_run_id)
        status = console_status(console)
        observations.append(stream_observation(iteration, status, console))
        if status == "completed":
            return validate_console_summary(
                name=name,
                workflow_id=workflow_id,
                instance_id=instance_id,
                control_run_id=control_run_id,
                status=status,
                worker_iterations=iteration,
                elapsed_ms=int((time.time() - started_at) * 1000),
                console=console,
                observations=observations,
                args=args,
            )
        if terminal_failure_status(status):
            break
        time.sleep(args.poll_delay_seconds)

    if console is None:
        console = get_console(args, workflow_id, instance_id, control_run_id)
    raise RuntimeError(
        f"{name} did not complete after {args.max_worker_iterations} worker iterations: "
        f"status={console_status(console)} diagnostic="
        f"{json.dumps(failure_diagnostic(console, last_worker_result), sort_keys=True)}"
    )


def get_console(
    args: argparse.Namespace,
    workflow_id: str,
    instance_id: str,
    control_run_id: str,
) -> dict[str, Any]:
    query = urllib.parse.urlencode(
        {
            "workflowId": workflow_id,
            "instanceId": instance_id,
            "controlRunId": control_run_id,
        }
    )
    return get_json(
        f"{args.wendao_ai_origin}/api/qianji/runs/{quote_path(control_run_id)}/console?{query}",
        args.request_timeout_seconds,
    )


def validate_console_summary(
    *,
    name: str,
    workflow_id: str,
    instance_id: str,
    control_run_id: str,
    status: str,
    worker_iterations: int,
    elapsed_ms: int,
    console: dict[str, Any],
    observations: list[StreamObservation] | None = None,
    args: argparse.Namespace,
) -> RunSummary:
    rows = durable_stream_rows(console)
    source_counts = count_sources(rows)
    error_count = count_error_rows(rows)
    if len(rows) < args.min_stream_rows:
        raise RuntimeError(f"{name} returned only {len(rows)} stream rows")
    missing_sources = [source for source in args.require_source if source_counts.get(source, 0) == 0]
    if missing_sources:
        raise RuntimeError(f"{name} stream is missing required sources: {missing_sources}")
    if error_count > 0:
        raise RuntimeError(f"{name} stream contains {error_count} error row(s)")
    stream_observations = observations or []
    if not args.allow_non_incremental_stream:
        validate_incremental_stream(name, stream_observations)
    return RunSummary(
        name=name,
        workflow_id=workflow_id,
        instance_id=instance_id,
        control_run_id=control_run_id,
        status=status,
        worker_iterations=worker_iterations,
        elapsed_ms=elapsed_ms,
        row_count=len(rows),
        source_counts=source_counts,
        error_count=error_count,
        stream_observations=stream_observations,
    )


def stream_observation(
    iteration: int,
    status: str,
    console: dict[str, Any],
) -> StreamObservation:
    rows = durable_stream_rows(console)
    return StreamObservation(
        iteration=iteration,
        status=status,
        row_count=len(rows),
        source_counts=count_sources(rows),
    )


def validate_incremental_stream(name: str, observations: list[StreamObservation]) -> None:
    if len(observations) < 2:
        return
    previous_count = -1
    grew = False
    saw_pre_completion_rows = False
    for observation in observations:
        if observation.row_count < previous_count:
            raise RuntimeError(f"{name} stream row count regressed across worker iterations")
        if previous_count >= 0 and observation.row_count > previous_count:
            grew = True
        if observation.status != "completed" and observation.row_count > 0:
            saw_pre_completion_rows = True
        previous_count = observation.row_count
    if not grew:
        raise RuntimeError(f"{name} stream did not grow across worker iterations")
    if not saw_pre_completion_rows:
        raise RuntimeError(f"{name} stream was only visible after completion")


def durable_stream_rows(console: dict[str, Any]) -> list[dict[str, Any]]:
    stream = console.get("stream")
    if not isinstance(stream, dict):
        return []
    rows = stream.get("rows")
    if not isinstance(rows, list):
        return []
    return [row for row in rows if isinstance(row, dict)]


def count_sources(rows: list[dict[str, Any]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for row in rows:
        source = row.get("source")
        if isinstance(source, str):
            counts[source] = counts.get(source, 0) + 1
    return counts


def count_error_rows(rows: list[dict[str, Any]]) -> int:
    count = 0
    for row in rows:
        kind = str(row.get("kind") or "").lower()
        message = str(row.get("message") or "").lower()
        if "failed" in kind or "error" in kind or "failed" in message:
            count += 1
    return count


def failure_diagnostic(
    console: dict[str, Any],
    last_worker_result: dict[str, Any] | None,
) -> dict[str, Any]:
    rows = durable_stream_rows(console)
    worker_response = None
    if isinstance(last_worker_result, dict):
        worker_response = last_worker_result.get("response")
    return {
        "status": console_status(console),
        "stream_row_count": len(rows),
        "source_counts": count_sources(rows),
        "last_worker_response": worker_response,
        "tail": [
            {
                "sequence": row.get("sequence"),
                "source": row.get("source"),
                "kind": row.get("kind"),
                "title": row.get("title"),
                "message": row.get("message"),
            }
            for row in rows[-8:]
        ],
    }


def console_status(console: dict[str, Any]) -> str:
    summary = console.get("controlSummary")
    if isinstance(summary, dict):
        status = summary.get("status")
        if isinstance(status, str) and status:
            return status
    diagnostics = console.get("diagnostics")
    if isinstance(diagnostics, dict):
        nested_summary = diagnostics.get("summary")
        if isinstance(nested_summary, dict):
            status = nested_summary.get("status")
            if isinstance(status, str) and status:
                return status
    state = console.get("state")
    return state if isinstance(state, str) and state else "unknown"


def terminal_failure_status(status: str) -> bool:
    return status in {"failed", "error"}


def bpmn_process_id(path: Path) -> str:
    root = ET.parse(path).getroot()
    for element in root.iter():
        if local_name(element.tag) == "process":
            process_id = element.attrib.get("id")
            if process_id:
                return process_id
    raise RuntimeError(f"BPMN file contains no process id: {path}")


def local_name(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def post_json(url: str, payload: dict[str, Any], timeout: float) -> dict[str, Any]:
    data = json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=data,
        headers={"content-type": "application/json"},
        method="POST",
    )
    return request_json(request, timeout)


def get_json(url: str, timeout: float) -> dict[str, Any]:
    request = urllib.request.Request(url, method="GET")
    return request_json(request, timeout)


def request_json(request: urllib.request.Request, timeout: float) -> dict[str, Any]:
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            body = response.read().decode("utf-8")
            status = response.getcode()
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"{request.full_url} returned HTTP {exc.code}: {body}") from exc
    except Exception as exc:
        raise RuntimeError(f"{request.full_url} request failed: {exc}") from exc
    if status < 200 or status >= 300:
        raise RuntimeError(f"{request.full_url} returned HTTP {status}: {body}")
    value = json.loads(body)
    if not isinstance(value, dict):
        raise RuntimeError(f"{request.full_url} returned non-object JSON")
    return value


def require_str(value: dict[str, Any], key: str) -> str:
    candidate = value.get(key)
    if not isinstance(candidate, str) or not candidate:
        raise RuntimeError(f"response missing required string field: {key}")
    return candidate


def quote_path(value: str) -> str:
    return urllib.parse.quote(value, safe="")


def summary_to_json(summary: RunSummary) -> dict[str, Any]:
    return {
        "name": summary.name,
        "workflow_id": summary.workflow_id,
        "instance_id": summary.instance_id,
        "control_run_id": summary.control_run_id,
        "status": summary.status,
        "worker_iterations": summary.worker_iterations,
        "elapsed_ms": summary.elapsed_ms,
        "row_count": summary.row_count,
        "source_counts": summary.source_counts,
        "error_count": summary.error_count,
        "stream_observations": [
            {
                "iteration": observation.iteration,
                "status": observation.status,
                "row_count": observation.row_count,
                "source_counts": observation.source_counts,
            }
            for observation in summary.stream_observations
        ],
    }


def write_evidence(payload: dict[str, Any], evidence_path: str) -> None:
    path = Path(evidence_path).resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"Error: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
