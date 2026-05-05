from __future__ import annotations

from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[2]
COMMON_SETUP = PROJECT_ROOT / ".github/actions/common-setup/action.yml"
VALKEY_LIVE = PROJECT_ROOT / ".github/workflows/xiuxian-daochang-valkey-live.yaml"
EMBEDDING_GATES = (
    PROJECT_ROOT / ".github/workflows/xiuxian-daochang-embedding-gates.yaml"
)
WENDAO_PERF = PROJECT_ROOT / ".github/workflows/xiuxian-wendao-performance-gates.yaml"


def _workflow_steps(workflow: str) -> list[str]:
    return ["      - " + block for block in workflow.split("\n      - ")[1:]]


def _step_containing(workflow: str, needle: str) -> str:
    for step in _workflow_steps(workflow):
        if needle in step:
            return step
    raise AssertionError(f"missing workflow step containing {needle!r}")


def _assert_secret_guarded(workflow: str, needle: str) -> None:
    step = _step_containing(workflow, needle)
    assert "MIMO_API_KEY_AVAILABLE == 'true'" in step


def test_common_setup_exports_non_secret_mimo_availability_flag() -> None:
    action = COMMON_SETUP.read_text(encoding="utf-8")

    assert "MIMO_API_KEY_AVAILABLE=true" in action
    assert "MIMO_API_KEY_AVAILABLE=false" in action


def test_valkey_live_skips_when_mimo_secret_is_unavailable() -> None:
    workflow = VALKEY_LIVE.read_text(encoding="utf-8")

    assert "MIMO_API_KEY_AVAILABLE != 'true'" in workflow
    assert "skipped: MIMO_API_KEY is not configured for this repository." in workflow
    _assert_secret_guarded(workflow, "devenv tasks run ci:valkey-live")


def test_embedding_gate_skips_when_mimo_secret_is_unavailable() -> None:
    workflow = EMBEDDING_GATES.read_text(encoding="utf-8")

    assert "MIMO_API_KEY_AVAILABLE != 'true'" in workflow
    assert "skipped: MIMO_API_KEY is not configured for this repository." in workflow
    _assert_secret_guarded(
        workflow,
        "devenv tasks run ci:rust-xiuxian-daochang-embedding-role-perf-medium-gate",
    )
    _assert_secret_guarded(
        workflow,
        "devenv tasks run ci:rust-xiuxian-daochang-embedding-role-perf-heavy-gate",
    )


def test_wendao_performance_gates_skip_when_mimo_secret_is_unavailable() -> None:
    workflow = WENDAO_PERF.read_text(encoding="utf-8")

    assert workflow.count("MIMO_API_KEY_AVAILABLE != 'true'") == 2
    assert (
        workflow.count("skipped: MIMO_API_KEY is not configured for this repository.")
        >= 4
    )
    for task in [
        "devenv tasks run ci:rust-wendao-performance-quick",
        "devenv tasks run ci:rust-wendao-performance-gateway-formal",
        "devenv tasks run ci:wendao-gateway-perf-summary",
        "devenv tasks run ci:rust-wendao-performance-stress",
        "devenv tasks run ci:rust-wendao-performance-bench-fast",
    ]:
        _assert_secret_guarded(workflow, task)
