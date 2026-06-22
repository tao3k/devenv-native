#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${PRJ_ROOT:-${DEVENV_ROOT:-$(cd "${SCRIPT_DIR}/../.." && pwd)}}"
PROJECT_RUNTIME_ROOT="${PRJ_RUNTIME_DIR:-$PROJECT_ROOT/.run}"
PYTHON_BIN="${QIANJI_SERVER_PYTHON:-${PYO3_PYTHON:-}}"

if [ -z "$PYTHON_BIN" ]; then
  PYTHON_BIN="$(command -v python3 2>/dev/null || true)"
fi
if [ -z "$PYTHON_BIN" ]; then
  echo "Error: python3 not found in PATH." >&2
  exit 1
fi

if [[ "$PROJECT_RUNTIME_ROOT" != /* ]]; then
  PROJECT_RUNTIME_ROOT="$PROJECT_ROOT/$PROJECT_RUNTIME_ROOT"
fi

BIND_ADDR="${QIANJI_SERVER_BIND_ADDR:-127.0.0.1:38130}"
PIDFILE="${QIANJI_SERVER_PIDFILE:-$PROJECT_RUNTIME_ROOT/qianji-server/qianji-server.pid}"
TIMEOUT_SECS="${QIANJI_SERVER_HEALTH_TIMEOUT_SECS:-2}"
HOST="${QIANJI_SERVER_HEALTH_HOST:-${BIND_ADDR%:*}}"
PORT="${BIND_ADDR##*:}"

if [ "$HOST" = "0.0.0.0" ] || [ "$HOST" = "::" ]; then
  HOST="127.0.0.1"
fi
if [[ "$PIDFILE" != /* ]]; then
  PIDFILE="$PROJECT_ROOT/$PIDFILE"
fi

if [ ! -s "$PIDFILE" ]; then
  echo "Error: qianji-server pidfile is missing: $PIDFILE" >&2
  exit 1
fi

PID="$(tr -d '[:space:]' <"$PIDFILE")"
if ! kill -0 "$PID" 2>/dev/null; then
  echo "Error: qianji-server process is not alive: $PID" >&2
  exit 1
fi

"$PYTHON_BIN" - "$HOST" "$PORT" "$TIMEOUT_SECS" <<'PY'
import sys
import urllib.error
import urllib.request

host, port, timeout_secs = sys.argv[1], sys.argv[2], float(sys.argv[3])
origin = f"http://{host}:{port}"
url = f"{origin}/readyz"
try:
    with urllib.request.urlopen(url, timeout=timeout_secs) as response:
        status = response.getcode()
except urllib.error.HTTPError as exc:
    print(f"Error: qianji-server readiness failed with HTTP {exc.code}: {url}", file=sys.stderr)
    raise SystemExit(1)
except Exception as exc:
    print(f"Error: qianji-server readiness check failed for {url}: {exc}", file=sys.stderr)
    raise SystemExit(1)

if status < 200 or status >= 300:
    print(f"Error: qianji-server readiness failed with HTTP {status}: {url}", file=sys.stderr)
    raise SystemExit(1)

worker_route = f"{origin}/control/runs/__health__/workers/openai-compatible-llm/run-and-complete"
request = urllib.request.Request(
    worker_route,
    data=b"{}",
    headers={"content-type": "application/json"},
    method="POST",
)
try:
    with urllib.request.urlopen(request, timeout=timeout_secs) as response:
        route_status = response.getcode()
except urllib.error.HTTPError as exc:
    route_status = exc.code
except Exception as exc:
    print(
        f"Error: qianji-server LLM worker route probe failed for {worker_route}: {exc}",
        file=sys.stderr,
    )
    raise SystemExit(1)

if route_status == 404:
    print(
        "Error: qianji-server LLM worker route is missing: "
        f"{worker_route}",
        file=sys.stderr,
    )
    raise SystemExit(1)
PY
