#!/usr/bin/env python3
"""Smoke-test Wendao Gateway chat model-route admission."""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from typing import Any, Callable

Opener = Callable[[urllib.request.Request, float], Any]

CHAT_ROUTE_SCHEMA = "xiuxian_wendao.model_route_chat_admission.v1"
DEFAULT_GATEWAY_ORIGIN = "http://127.0.0.1:9517"
DEFAULT_EXPECTED_PROVIDER = "openrouter"
DEFAULT_EXPECTED_BACKEND_PROFILE = "openai-compatible-chat-v1"
DEFAULT_EXPECTED_ROUTING_MODE = "deterministic"


def default_gateway_origin() -> str:
    return (
        os.environ.get("WENDAO_GATEWAY_ORIGIN")
        or os.environ.get("WENDAO_GATEWAY_HTTP_BASE_URL")
        or DEFAULT_GATEWAY_ORIGIN
    ).rstrip("/")


def build_chat_route_payload(
    *,
    precision_tier: str = "high",
    privacy_tier: str = "private",
    latency_budget_ms: int = 60_000,
    evidence_profile: str = "local-knowledge-chat",
    artifact_refs: list[str] | None = None,
) -> dict[str, object]:
    return {
        "precisionTier": precision_tier,
        "privacyTier": privacy_tier,
        "latencyBudgetMs": latency_budget_ms,
        "evidenceProfile": evidence_profile,
        "artifactRefs": artifact_refs or [],
    }


def admit_chat_model_route(
    gateway_origin: str,
    payload: dict[str, object],
    *,
    timeout_secs: float,
    opener: Opener = urllib.request.urlopen,
) -> dict[str, object]:
    route_url = f"{gateway_origin.rstrip('/')}/api/model-route/chat"
    request = urllib.request.Request(
        route_url,
        data=json.dumps(payload, ensure_ascii=True).encode("utf-8"),
        headers={"content-type": "application/json"},
        method="POST",
    )
    try:
        with opener(request, timeout=timeout_secs) as response:
            status = getattr(response, "status", None)
            body = response.read()
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"Gateway model route returned HTTP {error.code}: {body}") from error
    except urllib.error.URLError as error:
        raise RuntimeError(f"Gateway model route is unreachable: {route_url} ({error.reason})") from error

    if status != 200:
        body_text = body.decode("utf-8", errors="replace")
        raise RuntimeError(f"Gateway model route returned HTTP {status}: {body_text}")
    try:
        decoded = json.loads(body)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"Gateway model route returned invalid JSON: {error}") from error
    if not isinstance(decoded, dict):
        raise RuntimeError("Gateway model route returned a non-object JSON payload")
    return decoded


def validate_chat_route_response(
    payload: dict[str, object],
    *,
    expected_routing_mode: str | None = DEFAULT_EXPECTED_ROUTING_MODE,
    expected_provider: str | None = DEFAULT_EXPECTED_PROVIDER,
    expected_backend_profile: str | None = DEFAULT_EXPECTED_BACKEND_PROFILE,
) -> None:
    if payload.get("schemaVersion") != CHAT_ROUTE_SCHEMA:
        raise RuntimeError("Gateway model route returned an unsupported schemaVersion")
    route_mode = payload.get("modelRoutingMode")
    if route_mode not in {"deterministic", "vllm-sr"}:
        raise RuntimeError("Gateway model route returned an unsupported modelRoutingMode")
    if expected_routing_mode and route_mode != expected_routing_mode:
        raise RuntimeError(
            "Gateway model route modelRoutingMode mismatch: "
            f"expected {expected_routing_mode}, got {route_mode}"
        )

    intent = payload.get("intent")
    if not isinstance(intent, dict):
        raise RuntimeError("Gateway model route response is missing an intent object")
    if intent.get("taskKind") != "chat":
        raise RuntimeError("Gateway model route intent.taskKind must be chat")
    if intent.get("modality") != "text":
        raise RuntimeError("Gateway model route intent.modality must be text")

    decision = payload.get("decision")
    if not isinstance(decision, dict):
        raise RuntimeError("Gateway model route response is missing a model decision object")

    selected_provider = _required_string(decision, "selectedProvider")
    selected_model = _required_string(decision, "selectedModel")
    selected_backend_profile = _required_string(decision, "selectedBackendProfile")
    _required_string(decision, "routeId")

    if expected_provider and selected_provider != expected_provider:
        raise RuntimeError(
            "Gateway model route selectedProvider mismatch: "
            f"expected {expected_provider}, got {selected_provider}"
        )
    if expected_backend_profile and selected_backend_profile != expected_backend_profile:
        raise RuntimeError(
            "Gateway model route selectedBackendProfile mismatch: "
            f"expected {expected_backend_profile}, got {selected_backend_profile}"
        )
    if not selected_model.strip():
        raise RuntimeError("Gateway model route selectedModel must be non-empty")


def _required_string(payload: dict[str, object], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value.strip():
        raise RuntimeError(f"Gateway model route decision.{key} must be a non-empty string")
    return value


def run_smoke(
    *,
    gateway_origin: str,
    timeout_secs: float,
    expected_routing_mode: str | None,
    expected_provider: str | None,
    expected_backend_profile: str | None,
) -> dict[str, object]:
    response = admit_chat_model_route(
        gateway_origin,
        build_chat_route_payload(),
        timeout_secs=timeout_secs,
    )
    validate_chat_route_response(
        response,
        expected_routing_mode=expected_routing_mode,
        expected_provider=expected_provider,
        expected_backend_profile=expected_backend_profile,
    )
    return response


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Smoke-test Wendao Gateway /api/model-route/chat admission"
    )
    parser.add_argument("--gateway-origin", default=default_gateway_origin())
    parser.add_argument("--timeout-secs", type=float, default=10.0)
    parser.add_argument("--expected-routing-mode", default=DEFAULT_EXPECTED_ROUTING_MODE)
    parser.add_argument("--expected-provider", default=DEFAULT_EXPECTED_PROVIDER)
    parser.add_argument(
        "--expected-backend-profile",
        default=DEFAULT_EXPECTED_BACKEND_PROFILE,
    )
    parser.add_argument(
        "--no-provider-assert",
        action="store_true",
        help="Only require a non-empty selectedProvider",
    )
    args = parser.parse_args()

    expected_provider = None if args.no_provider_assert else args.expected_provider
    try:
        response = run_smoke(
            gateway_origin=args.gateway_origin,
            timeout_secs=args.timeout_secs,
            expected_routing_mode=args.expected_routing_mode,
            expected_provider=expected_provider,
            expected_backend_profile=args.expected_backend_profile,
        )
    except RuntimeError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1

    decision = response["decision"]
    assert isinstance(decision, dict)
    print(
        "healthy "
        f"provider={decision['selectedProvider']} "
        f"model={decision['selectedModel']} "
        f"backend={decision['selectedBackendProfile']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
