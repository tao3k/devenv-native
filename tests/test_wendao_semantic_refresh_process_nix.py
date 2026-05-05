from __future__ import annotations

from pathlib import Path


def test_wendao_semantic_refresh_process_nix_delegates_to_managed_scripts() -> None:
    root = Path(__file__).resolve().parents[1]
    process_nix = root / "nix/modules/process.nix"
    entrypoint = (
        root / "scripts/channel/processes/wendao-semantic-refresh/entrypoint.sh"
    )
    healthcheck = (
        root / "scripts/channel/processes/wendao-semantic-refresh/healthcheck.sh"
    )

    content = process_nix.read_text(encoding="utf-8")
    entrypoint_content = entrypoint.read_text(encoding="utf-8")
    healthcheck_content = healthcheck.read_text(encoding="utf-8")

    assert "wendao-semantic-refresh = {" in content
    assert 'exec = processEntrypoint "wendao-semantic-refresh";' in content
    assert 'exec.command = processHealthcheck "wendao-semantic-refresh";' in content
    assert "period_seconds = 10;" in content
    assert "failure_threshold = 6;" in content

    assert entrypoint.exists()
    assert healthcheck.exists()
    assert (
        "cargo build -p xiuxian-wendao-client --bin wendao-client --locked"
        in entrypoint_content
    )
    assert "WENDAO_SEMANTIC_REFRESH_INTERVAL_SECS" in entrypoint_content
    assert "WENDAO_SEMANTIC_REFRESH_MAX_RUNS" in entrypoint_content
    assert "--require-clean-worktree" in entrypoint_content
    assert "--interval-secs" in entrypoint_content
    assert "managed_cleanup_pidfile_process" in entrypoint_content
    assert '" semantic refresh-projections"' in entrypoint_content

    assert "managed_pid_matches_patterns" in healthcheck_content
    assert '" semantic refresh-projections"' in healthcheck_content
    assert "target/debug/wendao-client" in healthcheck_content
