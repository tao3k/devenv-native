#!/usr/bin/env python3
"""Compatibility entrypoint for the Wendao frontend healthcheck."""

from __future__ import annotations

from check_runtime_web_health import (
    is_expected_web_app_command,
    is_runtime_web_healthy,
    main,
    read_expected_pid,
)

is_frontend_healthy = is_runtime_web_healthy


if __name__ == "__main__":
    raise SystemExit(main(default_service_name="wendao-frontend"))
