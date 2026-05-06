from __future__ import annotations

import os
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[2]
PROCESS_NIX = PROJECT_ROOT / "nix/modules/process.nix"
PROCESS_ROOT = PROJECT_ROOT / "scripts/channel/processes"

ENTRYPOINT_PROCESSES = (
    "agent",
    "carfox",
    "valkey",
    "wendao-document-extract",
    "wendao-frontend",
    "wendao-gateway",
    "wendao-sentinel",
    "wendaosearch-parser-summary",
    "wendaosearch-solver-demo",
)

HEALTHCHECK_PROCESSES = (
    "valkey",
    "wendao-document-extract",
    "wendao-frontend",
    "wendao-gateway",
    "wendao-sentinel",
    "wendaosearch-parser-summary",
    "wendaosearch-solver-demo",
)


def test_process_nix_delegates_startup_to_process_entrypoints() -> None:
    process_nix = PROCESS_NIX.read_text(encoding="utf-8")

    assert (
        'exec bash "$ROOT_DIR/scripts/channel/processes/${processName}/${scriptName}.sh"'
        in process_nix
    )
    for forbidden in (
        ".run",
        ".data",
        "127.0.0.1",
        "6379",
        "9518",
        "WENDAO_",
        "WENDAOSEARCH_",
        "VALKEY_",
        "wendao.toml",
        "WendaoSearch.jl",
    ):
        assert forbidden not in process_nix


def test_process_entrypoints_are_owned_by_process_directories() -> None:
    for process_name in ENTRYPOINT_PROCESSES:
        entrypoint = PROCESS_ROOT / process_name / "entrypoint.sh"

        assert entrypoint.is_file()
        assert os.access(entrypoint, os.X_OK)


def test_process_healthchecks_are_owned_by_process_directories() -> None:
    for process_name in HEALTHCHECK_PROCESSES:
        healthcheck = PROCESS_ROOT / process_name / "healthcheck.sh"

        assert healthcheck.is_file()
        assert os.access(healthcheck, os.X_OK)
