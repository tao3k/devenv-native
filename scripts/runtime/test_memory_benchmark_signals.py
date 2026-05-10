from __future__ import annotations

import importlib.util
from pathlib import Path


def _load_module():
    module_name = "memory_benchmark_signals_test_module"
    script_path = Path(__file__).resolve().with_name("memory_benchmark_signals.py")
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load module from {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_parse_log_tokens_and_numeric_helpers() -> None:
    module = _load_module()
    line = '\x1b[31mevent="agent.memory.recall.injected"\x1b[0m recalled_selected=3 lambda=0.25'
    tokens = module.parse_log_tokens(line)

    assert tokens["event"] == "agent.memory.recall.injected"
    assert module.token_as_int(tokens, "recalled_selected") == 3
    assert module.token_as_float(tokens, "lambda") == 0.25
    assert module.token_as_int(tokens, "missing") is None
    assert module.token_as_float({"x": "bad"}, "x") is None


def test_parse_turn_signals_extracts_plan_decision_feedback_and_flags() -> None:
    module = _load_module()
    lines = [
        'event="agent.memory.recall.planned" k1=8 k2=4 lambda=0.35',
        'event="agent.memory.recall.injected" recalled_selected=2 query_tokens=110',
        'event="agent.memory.recall.feedback_updated" recall_feedback_bias_before=0.1 recall_feedback_bias_after=0.2',
        'event="agent.memory.embedding.timeout_fallback_hash"',
        'event="agent.memory.embedding.cooldown_fallback_hash"',
        'event="agent.memory.embedding.unavailable_fallback_hash"',
        "tools/call: Tool runtime error",
        "→ Bot: benchmark reply line",
    ]

    signals = module.parse_turn_signals(
        lines,
        forbidden_log_pattern="tools/call: Tool runtime error",
        bot_marker="→ Bot:",
        recall_plan_event="agent.memory.recall.planned",
        recall_injected_event="agent.memory.recall.injected",
        recall_skipped_event="agent.memory.recall.skipped",
        recall_feedback_event="agent.memory.recall.feedback_updated",
        embedding_timeout_fallback_event="agent.memory.embedding.timeout_fallback_hash",
        embedding_cooldown_fallback_event="agent.memory.embedding.cooldown_fallback_hash",
        embedding_unavailable_fallback_event="agent.memory.embedding.unavailable_fallback_hash",
    )

    assert signals["plan"]["k1"] == "8"
    assert signals["decision"]["event"] == "agent.memory.recall.injected"
    assert signals["feedback"]["recall_feedback_bias_before"] == "0.1"
    assert signals["embedding_timeout_fallback"] is True
    assert signals["embedding_cooldown_fallback"] is True
    assert signals["embedding_unavailable_fallback"] is True
    assert signals["tool_error"] is True
    assert signals["bot_line"] == "benchmark reply line"


def test_has_event_and_trim_text() -> None:
    module = _load_module()
    lines = ['event="a.b.c"', 'event="x.y.z"']
    assert module.has_event(lines, "x.y.z")
    assert not module.has_event(lines, "not.found")
    assert module.trim_text("abcdef", max_chars=5) == "ab..."
    assert module.trim_text(None) is None
