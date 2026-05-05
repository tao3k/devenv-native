from __future__ import annotations

from pathlib import Path


def test_wendao_gateway_process_nix_delegates_to_health_script_and_uses_stable_cleanup_patterns() -> (
    None
):
    process_nix = Path(__file__).resolve().parents[1] / "nix/modules/process.nix"
    content = process_nix.read_text(encoding="utf-8")

    assert 'gatewayRuntimeDir = ".run/wendao-gateway";' in content
    assert 'gatewayPidFile = "${gatewayRuntimeDir}/wendao.pid";' in content
    assert 'gatewayLogDir = ".run/logs";' in content
    assert 'gatewayStdoutLog = "${gatewayLogDir}/wendao-gateway.stdout.log";' in content
    assert 'gatewayStderrLog = "${gatewayLogDir}/wendao-gateway.stderr.log";' in content
    assert "doclingWorkerHost" not in content
    assert "doclingWorkerPort" not in content
    assert "documentExtractJobDb" not in content
    assert "documentExtractArtifactRoot" not in content
    assert (
        'mkdir -p "$ROOT_DIR/${gatewayRuntimeDir}" "$ROOT_DIR/${gatewayLogDir}"'
        in content
    )
    assert (
        'managed_cleanup_pidfile_process "$ROOT_DIR/${gatewayPidFile}" wendao-gateway "$ROOT_DIR/target/debug/wendao" " gateway start"'
        in content
    )
    assert (
        'managed_cleanup_listener "$PORT" wendao-gateway "$ROOT_DIR/target/debug/wendao" " gateway start"'
        in content
    )
    assert '> >(tee -a "$ROOT_DIR/${gatewayStdoutLog}")' in content
    assert '2> >(tee -a "$ROOT_DIR/${gatewayStderrLog}" >&2)' in content
    assert (
        'printf \'%s\\n\' "$GATEWAY_CHILD_PID" > "$ROOT_DIR/${gatewayPidFile}"'
        in content
    )
    assert (
        'bash "$ROOT_DIR/scripts/channel/wendao-gateway-healthcheck.sh" >/dev/null'
        in content
    )
    assert "export WENDAO_DOCUMENT_EXTRACT_ENDPOINT=" not in content
    assert "wendao-document-extract = {" in content
    assert (
        'bash "$ROOT_DIR/scripts/channel/wendao-document-extract-launch.sh"' in content
    )
    assert (
        'bash "$ROOT_DIR/scripts/channel/wendao-document-extract-healthcheck.sh" >/dev/null'
        in content
    )
    assert 'wendao-gateway.condition = "process_healthy";' in content
