from __future__ import annotations

import importlib.util
import json
import sys
import urllib.error
from pathlib import Path


class _FakeResponse:
    def __init__(self, status: int, payload: dict[str, object]) -> None:
        self.status = status
        self._payload_bytes = json.dumps(payload, ensure_ascii=True).encode("utf-8")

    def read(self) -> bytes:
        return self._payload_bytes

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb) -> bool:
        return False


def _load_module():
    script_path = Path(__file__).resolve().with_name("smoke_wendao_chat_model_route.py")
    module_name = "test_smoke_wendao_chat_model_route_module"
    spec = importlib.util.spec_from_file_location(module_name, script_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


def _route_response() -> dict[str, object]:
    return {
        "schemaVersion": "xiuxian_wendao.model_route_chat_admission.v1",
        "modelRoutingMode": "deterministic",
        "intent": {
            "taskKind": "chat",
            "modality": "text",
            "sourceKind": "conversation",
            "precisionTier": "high",
            "privacyTier": "private",
            "latencyBudgetMs": 60000,
            "evidenceProfile": "local-knowledge-chat",
            "artifactRefs": [],
        },
        "decision": {
            "routeId": "route-001",
            "selectedProvider": "openrouter",
            "selectedModel": "deepseek/deepseek-v4-pro",
            "selectedBackendProfile": "openai-compatible-chat-v1",
        },
    }


def test_smoke_accepts_gateway_chat_decision() -> None:
    module = _load_module()
    captured: list[object] = []

    def _fake_open(request, timeout: float):
        assert timeout == 3.0
        assert request.full_url == "http://127.0.0.1:9517/api/model-route/chat"
        captured.append(json.loads(request.data.decode("utf-8")))
        return _FakeResponse(200, _route_response())

    response = module.admit_chat_model_route(
        "http://127.0.0.1:9517",
        module.build_chat_route_payload(),
        timeout_secs=3.0,
        opener=_fake_open,
    )
    module.validate_chat_route_response(response)

    assert captured[0]["precisionTier"] == "high"


def test_smoke_rejects_missing_model_decision() -> None:
    module = _load_module()
    response = _route_response()
    response.pop("decision")

    try:
        module.validate_chat_route_response(response)
    except RuntimeError as error:
        assert "missing a model decision" in str(error)
    else:
        raise AssertionError("missing decision should fail the smoke")


def test_smoke_reports_gateway_http_error() -> None:
    module = _load_module()

    def _fake_open(_request, timeout: float):
        assert timeout == 3.0
        raise urllib.error.HTTPError(
            "http://127.0.0.1:9517/api/model-route/chat",
            503,
            "unavailable",
            {},
            fp=None,
        )

    try:
        module.admit_chat_model_route(
            "http://127.0.0.1:9517",
            module.build_chat_route_payload(),
            timeout_secs=3.0,
            opener=_fake_open,
        )
    except RuntimeError as error:
        assert "HTTP 503" in str(error)
    else:
        raise AssertionError("HTTP error should fail the smoke")
