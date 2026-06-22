"""Rust document extraction provider lifecycle helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

from .http_status import normalize_rest_endpoint, pick_free_port, wait_for_http_endpoint
from .processes import terminate_server
from .providers import (
    start_gateway_server,
    start_rust_provider_server,
    start_valkey_server,
)
from .runtime import wait_for_port, wait_for_process_stdout_contains

if TYPE_CHECKING:
    from .common import Any, Path, argparse


class RustProviderRuntime:
    """Manage a benchmark-local Rust document extraction provider."""

    def __init__(
        self,
        args: argparse.Namespace,
        *,
        temp_root: Path,
        process_log_dir: Path,
        python_host: str,
        python_port: int,
    ) -> None:
        self.args = args
        self.temp_root = temp_root
        self.process_log_dir = process_log_dir
        self.python_host = python_host
        self.python_port = python_port
        self.server: Any | None = None
        self.valkey_server: Any | None = None
        self.valkey_port: int | None = None
        self._rust_rest_endpoint_was_explicit = (
            normalize_rest_endpoint(args.rust_rest_endpoint) is not None
        )

    def start_if_needed(self) -> None:
        if self.args.external_endpoint or not should_start_local_rust_provider(self.args):
            return
        self._start()

    def restart(self, reason: str) -> None:
        if self.args.external_endpoint or not should_start_local_rust_provider(self.args):
            return
        _ = reason
        terminate_server(self.server)
        self.server = None
        self._start()

    def terminate(self) -> None:
        terminate_server(self.server)
        terminate_server(self.valkey_server)
        self.server = None
        self.valkey_server = None

    def _start(self) -> None:
        if self.args.rust_provider_mode == "gateway":
            self._start_gateway()
            return
        self._start_flight_provider()

    def _start_gateway(self) -> None:
        gateway_host = self.args.rust_provider_host or self.args.host
        gateway_port = resolve_local_rust_provider_port(self.args)
        self._ensure_valkey()
        assert self.valkey_port is not None
        self.args.benchmark_host = gateway_host
        self.args.benchmark_port = gateway_port
        if not self._rust_rest_endpoint_was_explicit:
            self.args.rust_rest_endpoint = f"http://{gateway_host}:{gateway_port}"
        self.server = start_gateway_server(
            self.args,
            gateway_port=gateway_port,
            python_host=self.python_host,
            python_port=self.python_port,
            valkey_url=f"redis://{self.args.host}:{self.valkey_port}/0",
            temp_root=self.temp_root,
            log_dir=self.process_log_dir,
        )
        wait_for_http_endpoint(
            f"http://{gateway_host}:{gateway_port}/api/health",
            self.server,
            timeout_seconds=self.args.server_start_timeout,
        )

    def _ensure_valkey(self) -> None:
        if self.valkey_server is not None and self.valkey_server.poll() is None:
            return
        self.valkey_port = self.args.gateway_valkey_port or pick_free_port(self.args.host)
        self.valkey_server = start_valkey_server(
            host=self.args.host,
            port=self.valkey_port,
            temp_root=self.temp_root,
            log_dir=self.process_log_dir,
        )
        wait_for_port(
            self.args.host,
            self.valkey_port,
            self.valkey_server,
            timeout_seconds=self.args.server_start_timeout,
        )

    def _start_flight_provider(self) -> None:
        rust_host = self.args.rust_provider_host or self.args.host
        rust_port = resolve_local_rust_provider_port(self.args)
        self.args.benchmark_host = rust_host
        self.args.benchmark_port = rust_port
        self.server = start_rust_provider_server(
            self.args,
            rust_host=rust_host,
            rust_port=rust_port,
            python_host=self.python_host,
            python_port=self.python_port,
            temp_root=self.temp_root,
            log_dir=self.process_log_dir,
        )
        wait_for_rust_provider_ready(
            rust_host,
            rust_port,
            self.server,
            timeout_seconds=self.args.server_start_timeout,
        )


def should_start_local_rust_provider(args: argparse.Namespace) -> bool:
    return args.flight_mode in {"async", "hybrid-page-ocr", "audio-shards"} or bool(
        args.artifact_registry_reuse_probe
    )


def resolve_local_rust_provider_port(args: object) -> int:
    explicit_port = getattr(args, "rust_provider_port", None)
    if explicit_port is not None:
        return explicit_port
    return pick_free_port(getattr(args, "host", "127.0.0.1"))


def wait_for_rust_provider_ready(
    host: str,
    port: int,
    server: Any,
    *,
    timeout_seconds: float,
) -> None:
    """Wait until the Rust Flight provider has bound and emitted its ready marker."""
    wait_for_port(
        host,
        port,
        server,
        timeout_seconds=timeout_seconds,
    )
    wait_for_process_stdout_contains(
        server,
        f"READY http://{host}:{port}",
        timeout_seconds=timeout_seconds,
    )
