#!/usr/bin/env python3
"""Guarded runner for cargo nextest with memory/performance telemetry."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import math
import os
import signal
import subprocess
import time
from dataclasses import asdict, dataclass
from pathlib import Path


@dataclass(frozen=True)
class ProcInfo:
    ppid: int
    rss_kb: int
    vsz_kb: int  # Virtual memory size
    cpu_pct: float
    command: str


@dataclass(frozen=True)
class Sample:
    index: int
    ts_iso: str
    elapsed_s: float
    rss_kb: int
    vsz_kb: int  # Virtual memory
    cpu_pct: float
    pid_count: int


@dataclass(frozen=True)
class Offender:
    pid: int
    rss_gb: float
    cpu_pct: float
    command: str


@dataclass(frozen=True)
class SingletonViolation:
    pattern: str
    matched_pids: list[int]


@dataclass(frozen=True)
class ProcessSpikeRule:
    pattern: str
    max_count: int
    max_total_rss_gb: float


@dataclass(frozen=True)
class ProcessSpikeViolation:
    pattern: str
    matched_pids: list[int]
    matched_count: int
    total_rss_gb: float
    max_count: int
    max_total_rss_gb: float


@dataclass(frozen=True)
class AggressiveKillRule:
    """Rule for aggressive process killing based on keyword matching."""

    pattern: str
    max_count: int
    max_rss_gb: float
    kill_immediately: bool  # If True, kill offending process immediately without grace


@dataclass(frozen=True)
class WatchBinaryRule:
    """Rule for watching specific binary for memory violations."""

    binary_name: str
    max_rss_gb: float


def now_iso() -> str:
    return dt.datetime.now().astimezone().isoformat(timespec="seconds")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Run a command under a memory/perf guard and kill process tree on anomalies."
        )
    )
    parser.add_argument(
        "--label",
        default="",
        help="Logical label for this run (for history trend comparison).",
    )
    parser.add_argument(
        "--max-rss-gb",
        type=float,
        default=6.0,
        help="Kill when process-tree RSS exceeds this GB threshold (default: 6.0)",
    )
    parser.add_argument(
        "--max-vsz-gb",
        type=float,
        default=0.0,
        help="Kill when process-tree virtual memory (VSZ) exceeds this GB threshold. 0 disables (default: 0)",
    )
    parser.add_argument(
        "--system-max-rss-gb",
        type=float,
        default=0.0,
        help=(
            "System-wide RSS guard: kill if ANY process (not just children) exceeds this RSS. "
            "Set 0 to disable (default: 0). Useful for catching runaway test processes."
        ),
    )
    parser.add_argument(
        "--max-growth-gb-per-min",
        type=float,
        default=0.0,
        help=(
            "Kill when RSS growth rate exceeds this GB/min over growth-window-sec. "
            "Set 0 to disable (default: 0)."
        ),
    )
    parser.add_argument(
        "--growth-window-sec",
        type=float,
        default=20.0,
        help="Window for growth-rate detection in seconds (default: 20)",
    )
    parser.add_argument(
        "--growth-warmup-sec",
        type=float,
        default=5.0,
        help="Ignore growth guard for first N seconds (default: 5)",
    )
    parser.add_argument(
        "--max-pids",
        type=int,
        default=0,
        help="Kill when process-tree PID count exceeds this value. 0 disables.",
    )
    parser.add_argument(
        "--singleton-substring",
        action="append",
        default=[],
        help=(
            "Enforce only one process globally whose command contains this substring. "
            "Supports `exe=<name>` for executable exact-match (e.g. exe=wendao) "
            "or `exe_prefix=<prefix>` for executable prefix-match (e.g. exe_prefix=llm-). "
            "Repeat flag for multiple patterns."
        ),
    )
    parser.add_argument(
        "--kill-substring",
        action="append",
        default=[],
        help=(
            "On guard trigger, also kill any process whose command contains this substring. "
            "Supports `exe=<name>` for executable exact-match or "
            "`exe_prefix=<prefix>` for executable prefix-match. "
            "Repeat flag for multiple patterns."
        ),
    )
    parser.add_argument(
        "--process-spike-rule",
        action="append",
        default=[],
        help=(
            "Realtime process spike rule in format 'substring:max_count:max_total_rss_gb'. "
            "Example: 'exe=wendao:1:6' or 'exe_prefix=llm-:1:8'. "
            "Set max_total_rss_gb to 0 to disable RSS limit."
        ),
    )
    parser.add_argument(
        "--aggressive-kill",
        action="append",
        default=[],
        help=(
            "Aggressive kill rule in format 'substring:max_count:max_rss_gb'. "
            "Immediately kills any matching process that exceeds count OR RSS limit. "
            "Example: 'exe=ld:2:2' kills excess linker processes. "
            "Example: 'exe=cargo:1:4' limits cargo to 1 instance with max 4GB RSS."
        ),
    )
    parser.add_argument(
        "--watch-binary",
        action="append",
        default=[],
        help=(
            "Watch specific binary name for memory violations. "
            "Format: 'binary_name:max_rss_gb'. "
            "Monitors ALL processes matching the binary name (not just child tree). "
            "Kills immediately when ANY matching process exceeds RSS limit. "
            "Example: 'llm_vision_deepseek_real_metal:4' kills if test binary exceeds 4GB. "
            "Example: 'cargo:3' kills if cargo exceeds 3GB."
        ),
    )
    parser.add_argument(
        "--aggressive-poll-ms",
        type=int,
        default=100,
        help="Polling interval for aggressive kill checks (default: 100ms, faster than normal poll)",
    )
    parser.add_argument(
        "--poll-ms",
        type=int,
        default=500,
        help="Sampling interval in milliseconds (default: 500)",
    )
    parser.add_argument(
        "--grace-ms",
        type=int,
        default=1500,
        help="SIGTERM grace period before SIGKILL in milliseconds (default: 1500)",
    )
    parser.add_argument(
        "--log-every",
        type=int,
        default=4,
        help="Write a human log line every N samples (default: 4)",
    )
    parser.add_argument(
        "--truncate-samples",
        action="store_true",
        help="Clear samples-jsonl before run starts",
    )
    parser.add_argument(
        "--log-file",
        type=Path,
        default=Path(".run/logs/guarded-nextest.log"),
        help="Human-readable log output path",
    )
    parser.add_argument(
        "--report-json",
        type=Path,
        default=Path(".run/reports/guarded-nextest/latest.json"),
        help="Structured report path (default: .run/reports/guarded-nextest/latest.json)",
    )
    parser.add_argument(
        "--samples-jsonl",
        type=Path,
        default=Path(".run/reports/guarded-nextest/samples.jsonl"),
        help="Sample stream path (default: .run/reports/guarded-nextest/samples.jsonl)",
    )
    parser.add_argument(
        "--history-jsonl",
        type=Path,
        default=Path(".run/reports/guarded-nextest/history.jsonl"),
        help="Append run summaries for trend analysis",
    )
    parser.add_argument(
        "command",
        nargs=argparse.REMAINDER,
        help="Command to run (prefix with --, e.g. -- cargo nextest run ...)",
    )
    args = parser.parse_args()
    if args.command and args.command[0] == "--":
        args.command = args.command[1:]
    if not args.command:
        parser.error("Missing command. Use: -- <command> [args...]")
    return args


def log_line(log_file: Path, message: str) -> None:
    line = f"[{now_iso()}] {message}"
    print(line, flush=True)
    with log_file.open("a", encoding="utf-8") as fh:
        fh.write(f"{line}\n")


def parse_cpu(raw: str) -> float:
    normalized = raw.strip().replace(",", ".")
    try:
        return float(normalized)
    except ValueError:
        return 0.0


def snapshot_process_table() -> tuple[dict[int, ProcInfo], str | None]:
    """
    Snapshot process table using ps for fast response.

    Note: On macOS, ps RSS is ~5x lower than Activity Monitor's Memory column.
    The caller should adjust thresholds accordingly (divide by 5).
    This is acceptable because we need fast response to catch memory explosions.
    """
    try:
        result = subprocess.run(
            ["ps", "-Ao", "pid=,ppid=,rss=,vsz=,%cpu=,command="],
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as error:
        return {}, str(error)

    table: dict[int, ProcInfo] = {}
    for raw in result.stdout.splitlines():
        line = raw.strip()
        if not line:
            continue
        parts = line.split(None, 5)
        if len(parts) < 6:
            continue
        try:
            pid = int(parts[0])
            ppid = int(parts[1])
            rss_kb = int(parts[2])
            vsz_kb = int(parts[3])
        except ValueError:
            continue
        table[pid] = ProcInfo(
            ppid=ppid,
            rss_kb=rss_kb,
            vsz_kb=vsz_kb,
            cpu_pct=parse_cpu(parts[4]),
            command=parts[5],
        )
    return table, None


def collect_tree_pids(root_pid: int, table: dict[int, ProcInfo]) -> set[int]:
    children: dict[int, list[int]] = {}
    for pid, info in table.items():
        children.setdefault(info.ppid, []).append(pid)

    stack = [root_pid]
    seen: set[int] = set()
    while stack:
        pid = stack.pop()
        if pid in seen:
            continue
        seen.add(pid)
        stack.extend(children.get(pid, []))
    return seen


def sum_rss_kb(pids: set[int], table: dict[int, ProcInfo]) -> int:
    return sum(table[pid].rss_kb for pid in pids if pid in table)


def sum_cpu_pct(pids: set[int], table: dict[int, ProcInfo]) -> float:
    return float(sum(table[pid].cpu_pct for pid in pids if pid in table))


def sum_vsz_kb(pids: set[int], table: dict[int, ProcInfo]) -> int:
    return sum(table[pid].vsz_kb for pid in pids if pid in table)


def kill_tree(pids: set[int], sig: int) -> None:
    for pid in sorted(pids, reverse=True):
        try:
            os.kill(pid, sig)
        except (ProcessLookupError, PermissionError):
            continue


def truncate_command(command: str, limit: int = 180) -> str:
    if len(command) <= limit:
        return command
    return f"{command[: limit - 3]}..."


def top_offenders(
    pids: set[int], table: dict[int, ProcInfo], limit: int = 8
) -> list[Offender]:
    ranked = [(table[pid], pid) for pid in pids if pid in table]
    ranked.sort(key=lambda item: item[0].rss_kb, reverse=True)
    out: list[Offender] = []
    for info, pid in ranked[:limit]:
        out.append(
            Offender(
                pid=pid,
                rss_gb=kb_to_gb(info.rss_kb),
                cpu_pct=round(info.cpu_pct, 3),
                command=truncate_command(info.command),
            )
        )
    return out


def pids_matching_substring(
    table: dict[int, ProcInfo],
    pattern: str,
    root_pid: int,
) -> list[int]:
    normalized_pattern = pattern.strip()
    lowered = normalized_pattern.lower()
    exe_match = None
    exe_prefix = None
    if lowered.startswith("exe="):
        exe_match = lowered.removeprefix("exe=").strip()
        if not exe_match:
            return []
    elif lowered.startswith("exe_prefix="):
        exe_prefix = lowered.removeprefix("exe_prefix=").strip()
        if not exe_prefix:
            return []
    matches: list[int] = []
    for pid, info in table.items():
        if pid == root_pid:
            continue
        lowered_cmd = info.command.lower()
        if "guarded_nextest.py" in lowered_cmd:
            continue
        exe = lowered_cmd.split(None, 1)[0].rsplit("/", 1)[-1]
        if exe_match is not None:
            if exe == exe_match:
                matches.append(pid)
            continue
        if exe_prefix is not None:
            if exe.startswith(exe_prefix):
                matches.append(pid)
            continue
        if lowered in lowered_cmd:
            matches.append(pid)
    matches.sort()
    return matches


def singleton_violations(
    table: dict[int, ProcInfo],
    patterns: list[str],
    root_pid: int,
) -> list[SingletonViolation]:
    violations: list[SingletonViolation] = []
    for pattern in patterns:
        normalized = pattern.strip()
        if not normalized:
            continue
        matched = pids_matching_substring(table, normalized, root_pid)
        if len(matched) > 1:
            violations.append(
                SingletonViolation(pattern=normalized, matched_pids=matched)
            )
    return violations


def parse_process_spike_rule(raw: str) -> ProcessSpikeRule | None:
    parts = raw.split(":")
    if len(parts) != 3:
        return None
    pattern = parts[0].strip()
    if not pattern:
        return None
    try:
        max_count = int(parts[1].strip())
        max_total_rss_gb = float(parts[2].strip())
    except ValueError:
        return None
    if max_count < 1:
        return None
    if max_total_rss_gb < 0:
        return None
    return ProcessSpikeRule(
        pattern=pattern,
        max_count=max_count,
        max_total_rss_gb=max_total_rss_gb,
    )


def parse_aggressive_kill_rule(raw: str) -> AggressiveKillRule | None:
    """Parse aggressive kill rule: 'pattern:max_count:max_rss_gb'"""
    parts = raw.split(":")
    if len(parts) != 3:
        return None
    pattern = parts[0].strip()
    if not pattern:
        return None
    try:
        max_count = int(parts[1].strip())
        max_rss_gb = float(parts[2].strip())
    except ValueError:
        return None
    if max_count < 1:
        return None
    if max_rss_gb < 0:
        return None
    return AggressiveKillRule(
        pattern=pattern,
        max_count=max_count,
        max_rss_gb=max_rss_gb,
        kill_immediately=True,
    )


def parse_watch_binary_rule(raw: str) -> WatchBinaryRule | None:
    """Parse watch binary rule: 'binary_name:max_rss_gb'"""
    parts = raw.split(":")
    if len(parts) != 2:
        return None
    binary_name = parts[0].strip()
    if not binary_name:
        return None
    try:
        max_rss_gb = float(parts[1].strip())
    except ValueError:
        return None
    if max_rss_gb <= 0:
        return None
    return WatchBinaryRule(binary_name=binary_name, max_rss_gb=max_rss_gb)


def find_processes_by_binary_name(
    table: dict[int, ProcInfo],
    binary_name: str,
    log_file: Path | None = None,
) -> list[tuple[int, float]]:
    """
    Find all processes matching the binary name (prefix match).
    Returns list of (pid, rss_gb) tuples.
    """
    matches: list[tuple[int, float]] = []
    for pid, info in table.items():
        # Extract binary name from command
        cmd_parts = info.command.split(None, 1)
        if not cmd_parts:
            continue
        exe_path = cmd_parts[0]
        exe_name = exe_path.rsplit("/", 1)[-1]
        # Use prefix match to catch binaries like "llm_vision_deepseek_real_metal-abc123"
        if exe_name.startswith(binary_name):
            matches.append((pid, kb_to_gb(info.rss_kb)))
            if log_file:
                log_line(
                    log_file,
                    f"watch_binary_found binary={binary_name!r} exe={exe_name!r} pid={pid} rss_gb={kb_to_gb(info.rss_kb):.2f}",
                )
    return matches


def evaluate_watch_binary_rules(
    table: dict[int, ProcInfo],
    rules: list[WatchBinaryRule],
    log_file: Path | None = None,
) -> list[tuple[WatchBinaryRule, list[tuple[int, float]], str]]:
    """
    Evaluate watch binary rules and return violations.
    Returns list of (rule, [(pid, rss_gb), ...], violation_reason).
    """
    violations: list[tuple[WatchBinaryRule, list[tuple[int, float]], str]] = []

    for rule in rules:
        matches = find_processes_by_binary_name(table, rule.binary_name, log_file)
        if not matches:
            continue

        # Check if any process exceeds RSS limit
        exceeded = [(pid, rss) for pid, rss in matches if rss > rule.max_rss_gb]
        if exceeded:
            violations.append(
                (
                    rule,
                    exceeded,
                    f"rss_exceeded: {exceeded[0][1]:.2f}GB > {rule.max_rss_gb}GB",
                )
            )

    return violations


def find_high_vsz_processes(
    table: dict[int, ProcInfo],
    max_vsz_gb: float,
    exclude_pids: set[int],
    log_file: Path | None = None,
) -> list[tuple[int, float, str]]:
    """
    Find ALL processes (system-wide) with VSZ exceeding threshold.
    Returns list of (pid, vsz_gb, command) tuples.
    """
    matches: list[tuple[int, float, str]] = []
    for pid, info in table.items():
        if pid in exclude_pids:
            continue
        # Skip our own script
        if "guarded_nextest.py" in info.command.lower():
            continue
        vsz_gb = kb_to_gb(info.vsz_kb)
        if vsz_gb > max_vsz_gb:
            matches.append((pid, vsz_gb, truncate_command(info.command)))
            if log_file:
                log_line(
                    log_file,
                    f"high_vsz_process pid={pid} vsz_gb={vsz_gb:.2f} cmd={truncate_command(info.command)}",
                )
    return matches


def find_high_rss_processes(
    table: dict[int, ProcInfo],
    max_rss_gb: float,
    exclude_pids: set[int],
    log_file: Path | None = None,
) -> list[tuple[int, float, str]]:
    """
    Find ALL processes (system-wide) with RSS exceeding threshold.
    Returns list of (pid, rss_gb, command) tuples.
    """
    matches: list[tuple[int, float, str]] = []
    for pid, info in table.items():
        if pid in exclude_pids:
            continue
        # Skip our own script
        if "guarded_nextest.py" in info.command.lower():
            continue
        rss_gb = kb_to_gb(info.rss_kb)
        if rss_gb > max_rss_gb:
            matches.append((pid, rss_gb, truncate_command(info.command)))
            if log_file:
                log_line(
                    log_file,
                    f"high_rss_process pid={pid} rss_gb={rss_gb:.2f} cmd={truncate_command(info.command)}",
                )
    return matches


def evaluate_aggressive_kill_rules(
    table: dict[int, ProcInfo],
    rules: list[AggressiveKillRule],
    root_pid: int,
) -> list[tuple[AggressiveKillRule, list[int], str]]:
    """
    Evaluate aggressive kill rules and return violations.
    Returns list of (rule, matched_pids, violation_reason).
    """
    violations: list[tuple[AggressiveKillRule, list[int], str]] = []

    for rule in rules:
        matched = pids_matching_substring(table, rule.pattern, root_pid)
        if not matched:
            continue

        # Check count violation
        if len(matched) > rule.max_count:
            violations.append(
                (rule, matched, f"count_exceeded: {len(matched)} > {rule.max_count}")
            )
            continue

        # Check RSS violation for any single process
        if rule.max_rss_gb > 0:
            for pid in matched:
                if pid in table:
                    rss_gb = kb_to_gb(table[pid].rss_kb)
                    if rss_gb > rule.max_rss_gb:
                        violations.append(
                            (
                                rule,
                                [pid],
                                f"rss_exceeded: {rss_gb:.2f}GB > {rule.max_rss_gb}GB",
                            )
                        )

    return violations


def evaluate_process_spike_rules(
    table: dict[int, ProcInfo],
    rules: list[ProcessSpikeRule],
    root_pid: int,
) -> list[ProcessSpikeViolation]:
    violations: list[ProcessSpikeViolation] = []
    for rule in rules:
        matched = pids_matching_substring(table, rule.pattern, root_pid)
        if not matched:
            continue
        total_rss_kb = sum(table[pid].rss_kb for pid in matched if pid in table)
        total_rss_gb = kb_to_gb(total_rss_kb)
        exceeds_count = len(matched) > rule.max_count
        exceeds_rss = rule.max_total_rss_gb > 0 and total_rss_gb > rule.max_total_rss_gb
        if exceeds_count or exceeds_rss:
            violations.append(
                ProcessSpikeViolation(
                    pattern=rule.pattern,
                    matched_pids=matched,
                    matched_count=len(matched),
                    total_rss_gb=round(total_rss_gb, 6),
                    max_count=rule.max_count,
                    max_total_rss_gb=rule.max_total_rss_gb,
                )
            )
    return violations


def kb_to_gb(kb: int) -> float:
    return kb / 1024.0 / 1024.0


def percentile(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    if len(values) == 1:
        return float(values[0])
    sorted_values = sorted(values)
    p = max(0.0, min(1.0, p))
    idx = (len(sorted_values) - 1) * p
    lo = math.floor(idx)
    hi = math.ceil(idx)
    if lo == hi:
        return float(sorted_values[lo])
    frac = idx - lo
    return float(sorted_values[lo] * (1.0 - frac) + sorted_values[hi] * frac)


def growth_gb_per_min(samples: list[Sample], window_sec: float) -> float:
    if len(samples) < 2:
        return 0.0
    newest = samples[-1]
    baseline = None
    for sample in reversed(samples):
        if newest.elapsed_s - sample.elapsed_s >= window_sec:
            baseline = sample
            break
    if baseline is None:
        baseline = samples[0]
    delta_s = newest.elapsed_s - baseline.elapsed_s
    if delta_s <= 0.0:
        return 0.0
    delta_gb = kb_to_gb(newest.rss_kb - baseline.rss_kb)
    return (delta_gb / delta_s) * 60.0


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        json.dump(payload, fh, indent=2, ensure_ascii=False)


def append_jsonl(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(payload, ensure_ascii=False))
        fh.write("\n")


def load_last_history_match(
    history_path: Path,
    label: str,
    command_signature: str,
) -> dict | None:
    if not history_path.exists():
        return None

    try:
        lines = history_path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return None

    for raw in reversed(lines):
        raw = raw.strip()
        if not raw:
            continue
        try:
            parsed = json.loads(raw)
        except json.JSONDecodeError:
            continue
        if (
            parsed.get("label", "") == label
            and parsed.get("command_signature", "") == command_signature
        ):
            return parsed
    return None


def run_guarded(args: argparse.Namespace) -> int:
    args.log_file.parent.mkdir(parents=True, exist_ok=True)
    max_rss_kb = int(args.max_rss_gb * 1024 * 1024)
    max_vsz_kb = int(args.max_vsz_gb * 1024 * 1024) if args.max_vsz_gb > 0 else 0
    poll_sec = max(args.poll_ms, 50) / 1000.0
    grace_sec = max(args.grace_ms, 0) / 1000.0
    log_every = max(args.log_every, 1)
    spike_rules = [
        rule
        for raw in args.process_spike_rule
        if raw.strip()
        for rule in [parse_process_spike_rule(raw.strip())]
        if rule is not None
    ]
    invalid_spike_rules = [
        raw.strip()
        for raw in args.process_spike_rule
        if raw.strip() and parse_process_spike_rule(raw.strip()) is None
    ]
    if invalid_spike_rules:
        raise ValueError(
            "Invalid --process-spike-rule values: "
            + ", ".join(repr(value) for value in invalid_spike_rules)
        )

    # Parse aggressive kill rules
    aggressive_rules = [
        rule
        for raw in args.aggressive_kill
        if raw.strip()
        for rule in [parse_aggressive_kill_rule(raw.strip())]
        if rule is not None
    ]
    invalid_aggressive_rules = [
        raw.strip()
        for raw in args.aggressive_kill
        if raw.strip() and parse_aggressive_kill_rule(raw.strip()) is None
    ]
    if invalid_aggressive_rules:
        raise ValueError(
            "Invalid --aggressive-kill values: "
            + ", ".join(repr(value) for value in invalid_aggressive_rules)
        )

    # Parse watch-binary rules
    watch_binary_rules = [
        rule
        for raw in args.watch_binary
        if raw.strip()
        for rule in [parse_watch_binary_rule(raw.strip())]
        if rule is not None
    ]
    invalid_watch_rules = [
        raw.strip()
        for raw in args.watch_binary
        if raw.strip() and parse_watch_binary_rule(raw.strip()) is None
    ]
    if invalid_watch_rules:
        raise ValueError(
            "Invalid --watch-binary values: "
            + ", ".join(repr(value) for value in invalid_watch_rules)
        )

    aggressive_poll_sec = max(args.aggressive_poll_ms, 50) / 1000.0

    # Use faster polling if aggressive rules are present
    effective_poll_sec = (
        min(poll_sec, aggressive_poll_sec) if aggressive_rules else poll_sec
    )

    if args.truncate_samples:
        args.samples_jsonl.parent.mkdir(parents=True, exist_ok=True)
        args.samples_jsonl.write_text("", encoding="utf-8")

    start_ts = time.monotonic()
    start_iso = now_iso()

    log_line(
        args.log_file,
        (
            f"start label={args.label or '-'} max_rss_gb={args.max_rss_gb:.2f} "
            f"max_growth_gb_per_min={args.max_growth_gb_per_min:.2f} "
            f"growth_window_sec={args.growth_window_sec:.2f} "
            f"poll_ms={args.poll_ms} grace_ms={args.grace_ms} "
            f"process_spike_rules={len(spike_rules)} "
            f"command={' '.join(args.command)}"
        ),
    )

    proc = subprocess.Popen(args.command)

    samples: list[Sample] = []
    peak_sample: Sample | None = None
    peak_offenders: list[Offender] = []
    guard_offenders: list[Offender] = []
    guard_triggered = False
    guard_reason = ""
    guard_singleton_violations: list[SingletonViolation] = []
    guard_process_spike_violations: list[ProcessSpikeViolation] = []
    ps_unavailable_reason: str | None = None

    sample_index = 0
    while proc.poll() is None:
        table, ps_error = snapshot_process_table()
        if ps_error is not None:
            if ps_unavailable_reason is None:
                ps_unavailable_reason = ps_error
                log_line(
                    args.log_file,
                    (
                        f"ps_unavailable fallback_enabled=true error={ps_unavailable_reason!r}"
                    ),
                )
            tree = {proc.pid}
        else:
            tree = collect_tree_pids(proc.pid, table)

        sample = Sample(
            index=sample_index,
            ts_iso=now_iso(),
            elapsed_s=round(time.monotonic() - start_ts, 3),
            rss_kb=sum_rss_kb(tree, table) if table else 0,
            vsz_kb=sum_vsz_kb(tree, table) if table else 0,
            cpu_pct=round(sum_cpu_pct(tree, table), 3) if table else 0.0,
            pid_count=len(tree),
        )
        samples.append(sample)
        sample_index += 1

        append_jsonl(
            args.samples_jsonl,
            {
                "index": sample.index,
                "ts_iso": sample.ts_iso,
                "elapsed_s": sample.elapsed_s,
                "rss_kb": sample.rss_kb,
                "rss_gb": round(kb_to_gb(sample.rss_kb), 6),
                "cpu_pct": sample.cpu_pct,
                "pid_count": sample.pid_count,
                "rss_growth_gb_per_min": round(
                    growth_gb_per_min(samples, args.growth_window_sec),
                    6,
                ),
            },
        )

        if peak_sample is None or sample.rss_kb > peak_sample.rss_kb:
            peak_sample = sample
            peak_offenders = top_offenders(tree, table)

        if sample_index % log_every == 0:
            growth = growth_gb_per_min(samples, args.growth_window_sec)
            log_line(
                args.log_file,
                (
                    f"rss_gb={kb_to_gb(sample.rss_kb):.2f} cpu_pct={sample.cpu_pct:.2f} "
                    f"pids={sample.pid_count} elapsed_s={sample.elapsed_s:.1f} "
                    f"growth_gb_per_min={growth:.2f}"
                ),
            )

        if sample.rss_kb > max_rss_kb:
            guard_triggered = True
            guard_reason = "rss_threshold_exceeded"
        elif args.max_vsz_gb > 0 and sample.vsz_kb > max_vsz_kb:
            guard_triggered = True
            guard_reason = "vsz_threshold_exceeded"
        elif args.max_pids > 0 and sample.pid_count > args.max_pids:
            guard_triggered = True
            guard_reason = "pid_threshold_exceeded"
        elif ps_unavailable_reason is not None:
            # Advanced process-level guards require process table snapshots.
            guard_singleton_violations = []
            guard_process_spike_violations = []
        else:
            guard_singleton_violations = singleton_violations(
                table, args.singleton_substring, proc.pid
            )
            if guard_singleton_violations:
                guard_triggered = True
                guard_reason = "singleton_substring_exceeded"
            else:
                guard_process_spike_violations = evaluate_process_spike_rules(
                    table, spike_rules, proc.pid
                )
                if guard_process_spike_violations:
                    guard_triggered = True
                    guard_reason = "process_spike_detected"
                else:
                    growth = growth_gb_per_min(samples, args.growth_window_sec)
                    if (
                        args.max_growth_gb_per_min > 0
                        and sample.elapsed_s >= args.growth_warmup_sec
                        and growth > args.max_growth_gb_per_min
                    ):
                        guard_triggered = True
                        guard_reason = "rss_growth_threshold_exceeded"

        # Aggressive kill check (immediate termination on violation)
        if aggressive_rules and table:
            aggressive_violations = evaluate_aggressive_kill_rules(
                table, aggressive_rules, proc.pid
            )
            if aggressive_violations:
                # Log all violations
                for rule, pids, reason in aggressive_violations:
                    log_line(
                        args.log_file,
                        (
                            f"aggressive_violation pattern={rule.pattern!r} "
                            f"reason={reason} pids={pids}"
                        ),
                    )

                # KILL EVERYTHING IMMEDIATELY - no grace period
                log_line(
                    args.log_file,
                    "aggressive_shutdown initiating full process tree kill",
                )

                # Kill the entire process tree with SIGKILL
                kill_tree(tree, signal.SIGKILL)

                # Also kill any specifically matched processes
                for rule, pids, reason in aggressive_violations:
                    for pid in pids:
                        try:
                            os.kill(pid, signal.SIGKILL)
                        except (ProcessLookupError, PermissionError):
                            pass

                # Wait briefly then exit
                time.sleep(0.1)
                log_line(args.log_file, "aggressive_shutdown_complete")
                return 137  # Exit immediately with killed status

        # Watch binary check - monitor specific binary names for memory violations
        if watch_binary_rules and table:
            watch_violations = evaluate_watch_binary_rules(
                table, watch_binary_rules, args.log_file
            )
            if watch_violations:
                for rule, exceeded, reason in watch_violations:
                    for pid, rss_gb in exceeded:
                        log_line(
                            args.log_file,
                            (
                                f"watch_binary_violation binary={rule.binary_name!r} "
                                f"pid={pid} rss_gb={rss_gb:.2f} max_rss_gb={rule.max_rss_gb} "
                                f"reason={reason}"
                            ),
                        )
                        # Kill the violating process immediately
                        try:
                            os.kill(pid, signal.SIGKILL)
                            log_line(args.log_file, f"watch_binary_killed pid={pid}")
                        except (ProcessLookupError, PermissionError):
                            pass

                # Also kill the process tree to be safe
                log_line(args.log_file, "watch_binary_shutdown killing process tree")
                kill_tree(tree, signal.SIGKILL)
                time.sleep(0.1)
                return 137

        # High VSZ check - system-wide detection of abnormal virtual memory growth
        # On macOS, individual processes may have ~400GB VSZ due to address space layout
        # but if we see VSZ > 10GB on non-system processes, it's suspicious
        if table and args.max_vsz_gb > 0:
            high_vsz_procs = find_high_vsz_processes(
                table, args.max_vsz_gb, tree, args.log_file
            )
            if high_vsz_procs:
                log_line(
                    args.log_file,
                    f"high_vsz_violation threshold={args.max_vsz_gb}GB count={len(high_vsz_procs)}",
                )
                for pid, vsz_gb, cmd in high_vsz_procs:
                    log_line(
                        args.log_file,
                        f"high_vsz_killing pid={pid} vsz_gb={vsz_gb:.2f} cmd={cmd}",
                    )
                    try:
                        os.kill(pid, signal.SIGKILL)
                    except (ProcessLookupError, PermissionError):
                        pass
                # Kill our process tree too
                kill_tree(tree, signal.SIGKILL)
                time.sleep(0.1)
                return 137

        # System-wide RSS check - detect ANY process with RSS exceeding threshold
        if table and args.system_max_rss_gb > 0:
            high_rss_procs = find_high_rss_processes(
                table, args.system_max_rss_gb, tree, args.log_file
            )
            if high_rss_procs:
                log_line(
                    args.log_file,
                    f"system_high_rss_violation threshold={args.system_max_rss_gb}GB count={len(high_rss_procs)}",
                )
                for pid, rss_gb, cmd in high_rss_procs:
                    log_line(
                        args.log_file,
                        f"system_high_rss_killing pid={pid} rss_gb={rss_gb:.2f} cmd={cmd}",
                    )
                    try:
                        os.kill(pid, signal.SIGKILL)
                    except (ProcessLookupError, PermissionError):
                        pass
                # Kill our process tree too
                kill_tree(tree, signal.SIGKILL)
                time.sleep(0.1)
                return 137

        if guard_triggered:
            kill_pids = set(tree)
            for violation in guard_process_spike_violations:
                kill_pids.update(violation.matched_pids)
            for pattern in args.kill_substring:
                kill_pids.update(
                    pids_matching_substring(table, pattern.strip(), proc.pid)
                )
            growth = growth_gb_per_min(samples, args.growth_window_sec)
            guard_offenders = top_offenders(kill_pids, table)

            if guard_reason == "rss_threshold_exceeded":
                log_line(
                    args.log_file,
                    (
                        f"guard_triggered reason={guard_reason} "
                        f"rss_gb={kb_to_gb(sample.rss_kb):.2f} "
                        f"limit_gb={kb_to_gb(max_rss_kb):.2f}"
                    ),
                )
            elif guard_reason == "pid_threshold_exceeded":
                log_line(
                    args.log_file,
                    (
                        f"guard_triggered reason={guard_reason} "
                        f"pid_count={sample.pid_count} "
                        f"limit={args.max_pids}"
                    ),
                )
            elif guard_reason == "singleton_substring_exceeded":
                for violation in guard_singleton_violations:
                    log_line(
                        args.log_file,
                        (
                            f"guard_triggered reason={guard_reason} "
                            f"pattern={violation.pattern!r} "
                            f"matched_pids={violation.matched_pids}"
                        ),
                    )
            elif guard_reason == "process_spike_detected":
                for violation in guard_process_spike_violations:
                    log_line(
                        args.log_file,
                        (
                            f"guard_triggered reason={guard_reason} "
                            f"pattern={violation.pattern!r} "
                            f"matched_count={violation.matched_count} "
                            f"max_count={violation.max_count} "
                            f"total_rss_gb={violation.total_rss_gb:.2f} "
                            f"max_total_rss_gb={violation.max_total_rss_gb:.2f} "
                            f"matched_pids={violation.matched_pids}"
                        ),
                    )
            else:
                growth = growth_gb_per_min(samples, args.growth_window_sec)
                if (
                    args.max_growth_gb_per_min > 0
                    and sample.elapsed_s >= args.growth_warmup_sec
                    and growth > args.max_growth_gb_per_min
                ):
                    guard_triggered = True
                    guard_reason = "rss_growth_threshold_exceeded"

                log_line(
                    args.log_file,
                    (
                        f"guard_triggered reason={guard_reason} "
                        f"growth_gb_per_min={growth:.2f} "
                        f"limit_gb_per_min={args.max_growth_gb_per_min:.2f}"
                    ),
                )

            for offender in guard_offenders:
                log_line(
                    args.log_file,
                    (
                        f"offender pid={offender.pid} rss_gb={offender.rss_gb:.2f} "
                        f"cpu_pct={offender.cpu_pct:.2f} cmd={offender.command}"
                    ),
                )

            kill_tree(kill_pids, signal.SIGTERM)
            if grace_sec > 0:
                deadline = time.monotonic() + grace_sec
                while proc.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.05)

            if proc.poll() is None:
                table, ps_error = snapshot_process_table()
                if ps_error is None and table:
                    kill_pids = set(collect_tree_pids(proc.pid, table))
                    for violation in guard_process_spike_violations:
                        kill_pids.update(violation.matched_pids)
                    for pattern in args.kill_substring:
                        kill_pids.update(
                            pids_matching_substring(table, pattern.strip(), proc.pid)
                        )
                else:
                    kill_pids = {proc.pid}
                kill_tree(kill_pids, signal.SIGKILL)
            break

        time.sleep(effective_poll_sec)

    exit_code = proc.wait()
    end_iso = now_iso()
    duration_s = round(time.monotonic() - start_ts, 3)

    rss_gb_series = [kb_to_gb(s.rss_kb) for s in samples]
    cpu_series = [s.cpu_pct for s in samples]
    pid_series = [float(s.pid_count) for s in samples]
    growth_series = [
        growth_gb_per_min(samples[: idx + 1], args.growth_window_sec)
        for idx in range(len(samples))
    ]

    summary = {
        "start_iso": start_iso,
        "end_iso": end_iso,
        "duration_s": duration_s,
        "command": args.command,
        "exit_code": 137 if guard_triggered else int(exit_code),
        "guard_triggered": guard_triggered,
        "guard_reason": guard_reason,
        "thresholds": {
            "max_rss_gb": args.max_rss_gb,
            "max_growth_gb_per_min": args.max_growth_gb_per_min,
            "growth_window_sec": args.growth_window_sec,
            "growth_warmup_sec": args.growth_warmup_sec,
            "max_pids": args.max_pids,
            "singleton_substring": args.singleton_substring,
            "kill_substring": args.kill_substring,
            "process_spike_rule": args.process_spike_rule,
            "poll_ms": args.poll_ms,
            "grace_ms": args.grace_ms,
        },
        "rss": {
            "peak_gb": round(max(rss_gb_series) if rss_gb_series else 0.0, 6),
            "avg_gb": (
                round(sum(rss_gb_series) / len(rss_gb_series), 6)
                if rss_gb_series
                else 0.0
            ),
            "p95_gb": round(percentile(rss_gb_series, 0.95), 6),
            "p99_gb": round(percentile(rss_gb_series, 0.99), 6),
        },
        "cpu": {
            "peak_pct": round(max(cpu_series) if cpu_series else 0.0, 6),
            "avg_pct": (
                round(sum(cpu_series) / len(cpu_series), 6) if cpu_series else 0.0
            ),
            "p95_pct": round(percentile(cpu_series, 0.95), 6),
            "p99_pct": round(percentile(cpu_series, 0.99), 6),
        },
        "pid_count": {
            "peak": int(max(pid_series) if pid_series else 0),
            "avg": round(sum(pid_series) / len(pid_series), 3) if pid_series else 0.0,
            "p95": round(percentile(pid_series, 0.95), 3),
        },
        "growth": {
            "max_gb_per_min": round(max(growth_series) if growth_series else 0.0, 6),
            "p95_gb_per_min": round(percentile(growth_series, 0.95), 6),
        },
        "samples_count": len(samples),
        "peak_top_offenders": [asdict(item) for item in peak_offenders],
        "guard_top_offenders": [asdict(item) for item in guard_offenders],
        "singleton_violations": [asdict(item) for item in guard_singleton_violations],
        "process_spike_violations": [
            asdict(item) for item in guard_process_spike_violations
        ],
        "report_paths": {
            "log_file": str(args.log_file),
            "samples_jsonl": str(args.samples_jsonl),
            "report_json": str(args.report_json),
            "history_jsonl": str(args.history_jsonl),
        },
        "fallback": {
            "ps_available": ps_unavailable_reason is None,
            "ps_error": ps_unavailable_reason,
        },
    }

    command_signature = " ".join(args.command)
    previous = load_last_history_match(
        args.history_jsonl, args.label, command_signature
    )
    if previous is not None:
        current_peak = float(summary["rss"]["peak_gb"])
        current_duration = float(summary["duration_s"])
        prev_peak = float(previous.get("peak_rss_gb", 0.0))
        prev_duration = float(previous.get("duration_s", 0.0))
        summary["comparison"] = {
            "has_previous": True,
            "previous_start_iso": previous.get("start_iso"),
            "peak_rss_gb_delta": round(current_peak - prev_peak, 6),
            "duration_s_delta": round(current_duration - prev_duration, 6),
        }
    else:
        summary["comparison"] = {"has_previous": False}

    write_json(args.report_json, summary)

    append_jsonl(
        args.history_jsonl,
        {
            "start_iso": start_iso,
            "end_iso": end_iso,
            "label": args.label,
            "command_signature": command_signature,
            "duration_s": duration_s,
            "exit_code": summary["exit_code"],
            "guard_triggered": guard_triggered,
            "guard_reason": guard_reason,
            "peak_rss_gb": summary["rss"]["peak_gb"],
            "avg_rss_gb": summary["rss"]["avg_gb"],
            "p95_rss_gb": summary["rss"]["p95_gb"],
            "peak_cpu_pct": summary["cpu"]["peak_pct"],
            "avg_cpu_pct": summary["cpu"]["avg_pct"],
            "growth_max_gb_per_min": summary["growth"]["max_gb_per_min"],
            "samples_count": summary["samples_count"],
        },
    )

    if guard_triggered:
        log_line(
            args.log_file,
            (
                f"stopped_by_guard reason={guard_reason} "
                f"peak_rss_gb={summary['rss']['peak_gb']:.2f} "
                f"report_json={args.report_json}"
            ),
        )
        return 137

    log_line(
        args.log_file,
        (
            f"command_exit status={exit_code} peak_rss_gb={summary['rss']['peak_gb']:.2f} "
            f"report_json={args.report_json}"
        ),
    )
    return int(exit_code)


def main() -> int:
    args = parse_args()
    return run_guarded(args)


if __name__ == "__main__":
    raise SystemExit(main())
