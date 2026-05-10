"""
xiuxian_foundation.embedding - Unified Embedding Service

Python layer is Rust-first: embeddings are fetched from an external embedding HTTP
service (typically backed by Rust runtime). Python does not manage local Ollama or
other local model runtimes for embeddings.

Configuration (packages/conf/settings.yaml, user: $PRJ_CONFIG_HOME/xiuxian-artisan-workshop/settings.yaml):
- embedding.provider: "client" | "" (treated as client)
- embedding.client_url: embedding service base URL (default http://127.0.0.1:<embedding.http_port>)
- embedding.dimension: output vector dimension (default 1024)
"""

from __future__ import annotations

import os
import socket
from contextvars import ContextVar
from typing import TYPE_CHECKING, Any, Protocol
from urllib.parse import urlparse

import structlog

from xiuxian_foundation.config.dirs import PRJ_CONFIG
from xiuxian_foundation.config.settings import get_setting
from xiuxian_foundation.config.prj import get_project_root

if TYPE_CHECKING:
    from pathlib import Path

logger = structlog.get_logger(__name__)

_DEFAULT_EMBED_HTTP_PORT = 3002
_DEFAULT_EMBED_HOST = "127.0.0.1"

_KNOWN_TOOL_RUNTIME_PATH_SUFFIXES = ("/sse", "/messages")

try:
    import tomllib
except ModuleNotFoundError:
    try:
        import tomli as tomllib  # type: ignore[no-redef]
    except ModuleNotFoundError:
        tomllib = None  # type: ignore[assignment]


def _int_setting(path: str, default: int) -> int:
    """Read int setting with defensive fallback."""
    try:
        value = get_setting(path)
        return int(value)
    except (TypeError, ValueError):
        return default


def _float_setting(path: str, default: float) -> float:
    """Read float setting with defensive fallback."""
    try:
        value = get_setting(path)
        parsed = float(value)
        return parsed if parsed > 0 else default
    except (TypeError, ValueError):
        return default


def _read_toml_document(path: Path) -> dict[str, Any] | None:
    if tomllib is None or not path.exists():
        return None
    try:
        with path.open("rb") as handle:
            data = tomllib.load(handle)
            return data if isinstance(data, dict) else None
    except Exception:
        return None


def _xiuxian_toml_candidates() -> list[Path]:
    user_path = PRJ_CONFIG("xiuxian-artisan-workshop", "xiuxian.toml")
    try:
        project_root = get_project_root()
    except Exception:
        project_root = user_path.parent.parent
    system_path = (
        project_root
        / "packages"
        / "rust"
        / "crates"
        / "xiuxian-wendao"
        / "resources"
        / "config"
        / "xiuxian.toml"
    )
    return [user_path, system_path]


def _dig(mapping: object, *keys: str) -> object | None:
    cursor = mapping
    for key in keys:
        if not isinstance(cursor, dict) or key not in cursor:
            return None
        cursor = cursor[key]
    return cursor


def _normalize_http_base_url(raw: object) -> str | None:
    if not isinstance(raw, str):
        return None
    text = raw.strip()
    if not text:
        return None
    if "://" not in text:
        text = f"http://{text}"

    parsed = urlparse(text)
    if not parsed.netloc:
        return None
    scheme = parsed.scheme or "http"
    path = parsed.path.rstrip("/")
    lowered = path.lower()
    if lowered in _KNOWN_TOOL_RUNTIME_PATH_SUFFIXES:
        path = ""
    return f"{scheme}://{parsed.netloc}{path}".rstrip("/")


def _resolve_embedding_candidates_from_xiuxian() -> list[str]:
    host = (
        os.environ.get("XIUXIAN_WENDAO_LOCAL_HOST", "localhost").strip() or "localhost"
    )
    resolved: list[str] = []

    for path in _xiuxian_toml_candidates():
        doc = _read_toml_document(path)
        if not isinstance(doc, dict):
            continue

        gateway_bind = _dig(doc, "gateway", "bind")
        gateway_url = _normalize_http_base_url(gateway_bind)
        if gateway_url:
            resolved.append(gateway_url)

        memory_embed_url = _normalize_http_base_url(
            _dig(doc, "memory", "embedding_base_url")
        )
        if memory_embed_url:
            resolved.append(memory_embed_url)

        tool_runtime_base_url = _normalize_http_base_url(
            _dig(doc, "tool_runtime", "base_url")
        )
        if tool_runtime_base_url:
            resolved.append(tool_runtime_base_url)

        tool_runtime_port = _dig(doc, "tool_runtime", "port")
        try:
            tool_runtime_port_int = (
                int(tool_runtime_port) if tool_runtime_port is not None else None
            )
        except (TypeError, ValueError):
            tool_runtime_port_int = None
        if (
            isinstance(tool_runtime_port_int, int)
            and 1 <= tool_runtime_port_int <= 65535
        ):
            resolved.append(f"http://{host}:{tool_runtime_port_int}")

    return resolved


def _dedupe_preserve_order(urls: list[str]) -> list[str]:
    seen: set[str] = set()
    ordered: list[str] = []
    for raw in urls:
        normalized = _normalize_http_base_url(raw)
        if not normalized or normalized in seen:
            continue
        seen.add(normalized)
        ordered.append(normalized)
    return ordered


# Context override so skill execution can use runtime-first embedding.
_embedding_override: ContextVar[Any | None] = ContextVar(
    "embedding_override", default=None
)


class EmbeddingOverrideProtocol(Protocol):
    """Protocol for embedding override used during skill execution."""

    def embed(self, text: str) -> list[list[float]]: ...
    def embed_batch(self, texts: list[str]) -> list[list[float]]: ...


def get_embedding_override() -> EmbeddingOverrideProtocol | None:
    """Return the current embedding override, if any (used by skill execution path)."""
    return _embedding_override.get()


def set_embedding_override(provider: EmbeddingOverrideProtocol | None) -> None:
    """Set the embedding override for the current context."""
    _embedding_override.set(provider)


class EmbeddingUnavailableError(Exception):
    """Raised when embedding HTTP service is unavailable."""


class EmbeddingService:
    """Singleton embedding service. Python is client-only; Rust provides embeddings."""

    _instance: EmbeddingService | None = None
    _dimension: int = 1024
    _backend: str = "unavailable"
    _initialized: bool = False
    _client_mode: bool = False
    _client_url: str | None = None
    _embed_cache_key: str | None = None
    _embed_cache_value: list[list[float]] | None = None
    _client_retried: bool = False
    _candidate_client_urls: list[str] | None = None

    def _reset_runtime_state(self) -> None:
        """Reset mutable runtime state."""
        self._dimension = 1024
        self._backend = "unavailable"
        self._initialized = False
        self._client_mode = False
        self._client_url = None
        self._embed_cache_key = None
        self._embed_cache_value = None
        self._client_retried = False
        self._candidate_client_urls = None

    def _resolve_client_url_candidates(self) -> list[str]:
        http_port = _int_setting("embedding.http_port", _DEFAULT_EMBED_HTTP_PORT)
        configured_url = str(get_setting("embedding.client_url") or "").strip()
        default_url = f"http://{_DEFAULT_EMBED_HOST}:{http_port}"
        normalized_configured = _normalize_http_base_url(configured_url)
        normalized_default = _normalize_http_base_url(default_url)

        candidates: list[str] = []
        # Prefer explicit non-default overrides first. Keep default fallback at the end
        # so xiuxian.toml gateway/memory candidates can win when available.
        if (
            configured_url
            and normalized_configured
            and normalized_configured != normalized_default
        ):
            candidates.append(configured_url)

        candidates.extend(_resolve_embedding_candidates_from_xiuxian())
        if configured_url:
            candidates.append(configured_url)
        candidates.append(default_url)
        return _dedupe_preserve_order(candidates)

    def _preferred_client_url(self) -> str:
        if self._client_url:
            return self._client_url
        if self._candidate_client_urls:
            return self._candidate_client_urls[0]
        candidates = self._resolve_client_url_candidates()
        self._candidate_client_urls = candidates
        if candidates:
            return candidates[0]
        return f"http://{_DEFAULT_EMBED_HOST}:{_DEFAULT_EMBED_HTTP_PORT}"

    def __new__(cls) -> EmbeddingService:
        if cls._instance is None:
            instance = super().__new__(cls)
            instance._reset_runtime_state()
            cls._instance = instance
        return cls._instance

    def _check_http_server_healthy(self, url: str, timeout: float = 1.0) -> bool:
        """Synchronously check if HTTP server is healthy (single request, short timeout)."""
        import json
        import urllib.error
        import urllib.request

        try:
            with urllib.request.urlopen(f"{url}/health", timeout=timeout) as response:
                if response.status != 200:
                    return False
                payload = response.read().decode("utf-8")
                data = json.loads(payload) if payload else {}
                status = str(data.get("status", "")).lower()
                return status in {"healthy", "ok"}
        except (urllib.error.URLError, TimeoutError, ValueError):
            return False

    def _activate_http_backend(self, client_url: str) -> None:
        self._client_mode = True
        self._client_url = client_url
        self._backend = "http"
        self._dimension = _int_setting("embedding.dimension", 1024)
        self._initialized = True

    def _verify_embedding_service_works(
        self, url: str, timeout: float = 5.0
    ) -> tuple[bool, bool]:
        """Verify embedding service via /embed/single.

        Returns:
            (verified, timed_out):
                verified=True when vector payload is valid.
                timed_out=True when upstream likely cold-started and request timed out.
        """
        import json
        import urllib.error
        import urllib.request

        try:
            body = json.dumps({"text": "_probe"}).encode("utf-8")
            req = urllib.request.Request(
                f"{url}/embed/single",
                data=body,
                headers={"Content-Type": "application/json"},
                method="POST",
            )
            with urllib.request.urlopen(req, timeout=timeout) as response:
                if response.status != 200:
                    return False, False
                payload = response.read().decode("utf-8")
                data = json.loads(payload) if payload else {}
                vector = data.get("vector", [])
                if not isinstance(vector, list) or not vector:
                    return False, False
                return all(isinstance(x, (int, float)) for x in vector[:10]), False
        except TimeoutError:
            return False, True
        except urllib.error.URLError as error:
            if isinstance(error.reason, (TimeoutError, socket.timeout)):
                return False, True
            return False, False
        except (ValueError, TypeError):
            return False, False

    def _configure_http_backend(self, client_url: str) -> bool:
        if not self._check_http_server_healthy(client_url, timeout=2.0):
            return False

        probe_timeout = _float_setting("embedding.client_probe_timeout_seconds", 20.0)
        verification = self._verify_embedding_service_works(
            client_url, timeout=probe_timeout
        )
        if isinstance(verification, tuple):
            verified, timed_out = verification
        else:
            verified, timed_out = bool(verification), False
        if not verified and not timed_out:
            return False
        if not verified and timed_out:
            logger.warning(
                "Embedding probe did not return vector in time; accepting healthy endpoint.",
                client_url=client_url,
                probe_timeout_seconds=probe_timeout,
            )

        self._activate_http_backend(client_url)
        logger.info("Embedding: client mode", client_url=self._client_url)
        return True

    @staticmethod
    def _normalize_provider(raw_provider: str) -> str:
        return raw_provider.strip().lower()

    def initialize(self) -> None:
        """Initialize embedding service in client-only mode."""
        if self._initialized:
            return

        provider = self._normalize_provider(
            str(get_setting("embedding.provider") or "")
        )
        self._dimension = _int_setting("embedding.dimension", 1024)

        if provider not in {
            "",
            "client",
            "ollama",
            "litellm_rs",
            "mistral_sdk",
            "http",
        }:
            logger.warning(
                "Unknown embedding.provider '%s'; forcing client mode.",
                provider,
            )

        candidates = self._resolve_client_url_candidates()
        self._candidate_client_urls = candidates
        for client_url in candidates:
            if self._configure_http_backend(client_url):
                return

        self._backend = "unavailable"
        self._initialized = True
        logger.warning(
            "Embedding: client_url unreachable; embedding unavailable.",
            client_url=self._preferred_client_url(),
        )

    def _retry_client_once(self) -> None:
        """Retry once for unavailable client backend."""
        if self._backend != "unavailable" or self._client_retried:
            return
        self._client_retried = True
        candidates = self._resolve_client_url_candidates()
        self._candidate_client_urls = candidates
        for client_url in candidates:
            if self._configure_http_backend(client_url):
                break

    def _auto_detect_and_init(self) -> None:
        """Lazy initialization path for embed/embed_batch."""
        if self._initialized:
            return
        self.initialize()

    def embed(self, text: str) -> list[list[float]]:
        """Generate embedding for a single text input."""
        override = get_embedding_override()
        if override is not None:
            return override.embed(text)

        if not self._initialized:
            self._auto_detect_and_init()

        if (
            self._embed_cache_key is not None
            and self._embed_cache_key == text
            and self._embed_cache_value is not None
        ):
            return self._embed_cache_value

        if self._backend == "unavailable":
            self._retry_client_once()
            if self._backend == "unavailable":
                _url = self._preferred_client_url()
                raise EmbeddingUnavailableError(
                    "Embedding unavailable (client mode). "
                    f"Start the Rust embedding service at {_url} "
                    "(GET /health, POST /embed/single)."
                )

        out = self._embed_http([text])

        self._embed_cache_key = text
        self._embed_cache_value = out
        return out

    def _embed_http(self, texts: list[str]) -> list[list[float]]:
        """Generate embeddings via HTTP client. Raises EmbeddingUnavailableError on failure."""
        from xiuxian_foundation.embedding_client import get_embedding_client

        try:
            client = get_embedding_client(self._client_url)
            return client.sync_embed_batch(texts)
        except Exception as exc:
            raise EmbeddingUnavailableError(
                f"Embedding HTTP service unavailable at {self._client_url}: {exc}"
            ) from exc

    def embed_batch(self, texts: list[str]) -> list[list[float]]:
        """Generate embeddings for multiple texts."""
        if not texts:
            return []

        override = get_embedding_override()
        if override is not None:
            return override.embed_batch(texts)

        if not self._initialized:
            self._auto_detect_and_init()

        if self._backend == "unavailable":
            self._retry_client_once()
            if self._backend == "unavailable":
                _url = self._preferred_client_url()
                raise EmbeddingUnavailableError(
                    "Embedding unavailable (client mode). "
                    f"Start the Rust embedding service at {_url} "
                    "(GET /health, POST /embed/single)."
                )

        return self._embed_http(texts)

    def embed_force_local(self, texts: list[str]) -> list[list[float]]:
        """Force non-override path using HTTP embedding backend."""
        if not texts:
            return []
        if not self._initialized:
            self._auto_detect_and_init()
        return self._embed_http(texts)

    @property
    def backend(self) -> str:
        """Return the embedding backend."""
        return self._backend

    @property
    def dimension(self) -> int:
        """Return the embedding dimension."""
        return self._dimension

    @property
    def is_loaded(self) -> bool:
        """True when initialized."""
        return self._initialized

    @property
    def is_loading(self) -> bool:
        """Always False (no in-process model load)."""
        return False


# Singleton accessor
_service: EmbeddingService | None = None


def get_embedding_service() -> EmbeddingService:
    """Get the singleton EmbeddingService instance."""
    global _service
    if _service is None or EmbeddingService._instance is None:
        _service = EmbeddingService()
    return _service


# Convenience functions
def embed_text(text: str) -> list[float]:
    """Generate embedding for a single text."""
    return get_embedding_service().embed(text)[0]


def embed_batch(texts: list[str]) -> list[list[float]]:
    """Generate embeddings for multiple texts."""
    return get_embedding_service().embed_batch(texts)


def get_dimension() -> int:
    """Get the current embedding dimension."""
    return get_embedding_service().dimension


__all__ = [
    "EmbeddingOverrideProtocol",
    "EmbeddingService",
    "EmbeddingUnavailableError",
    "embed_batch",
    "embed_text",
    "get_dimension",
    "get_embedding_override",
    "get_embedding_service",
    "set_embedding_override",
]
