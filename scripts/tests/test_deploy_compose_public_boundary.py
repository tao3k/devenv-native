from __future__ import annotations

import re
from pathlib import Path


def _service_block(compose: str, service_name: str) -> str:
    heading = re.search(rf"^  {re.escape(service_name)}:\n", compose, re.M)
    assert heading is not None, service_name
    block_start = heading.end()
    next_heading = re.search(r"^  [A-Za-z0-9_-]+:\n", compose[block_start:], re.M)
    if next_heading is None:
        return compose[block_start:]
    return compose[block_start : block_start + next_heading.start()]


def test_deploy_compose_keeps_internal_services_off_host_ports() -> None:
    compose = (Path(__file__).resolve().parents[2] / "deploy/docker-compose.yml").read_text(
        encoding="utf-8"
    )

    public_port_services = {
        "wendao-gateway",
        "wendao-frontend",
    }

    for service_name in (
        "valkey",
        "wendaosearch-solver",
        "wendaosearch-parser",
        "xiuxian-daochang",
        "wendao-document-extract",
    ):
        block = _service_block(compose, service_name)
        assert "\n    ports:" not in block, service_name
        assert "\n    expose:" in block, service_name

    for service_name in public_port_services:
        block = _service_block(compose, service_name)
        assert "\n    ports:" in block, service_name
