from __future__ import annotations

from pathlib import Path


def test_valkey_process_nix_bootstraps_dirs_and_waits_for_readiness() -> None:
    process_nix = Path(__file__).resolve().parents[1] / "nix/modules/process.nix"
    content = process_nix.read_text(encoding="utf-8")

    assert "ROOT_DIR=\"''${PRJ_ROOT:-$PWD}\"" in content
    assert 'VALKEY_RUNTIME_DIR="$ROOT_DIR/${valkeyRuntimeDir}"' in content
    assert 'VALKEY_DATA_DIR="$ROOT_DIR/${valkeyDataDir}"' in content
    assert 'VALKEY_PIDFILE="$ROOT_DIR/${valkeyPidFile}"' in content
    assert 'mkdir -p "$VALKEY_RUNTIME_DIR" "$VALKEY_DATA_DIR"' in content
    assert 'rm -f "$VALKEY_PIDFILE"' in content
    assert 'export VALKEY_PIDFILE="$VALKEY_PIDFILE"' in content
    assert "export VALKEY_DAEMONIZE=no" in content
    assert "bash scripts/channel/valkey-launch.sh" in content
    assert "bash scripts/channel/valkey-healthcheck.sh >/dev/null" in content
    assert "initial_delay_seconds = 5;" in content
    assert "timeout_seconds = 2;" in content
    assert "failure_threshold = 30;" in content
