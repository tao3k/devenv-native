#!/usr/bin/env bash
set -euo pipefail

run_cargo() {
  if [[ -n ${CARGO_BIN:-} ]]; then
    "${CARGO_BIN}" "$@"
  else
    direnv exec . cargo "$@"
  fi
}
ROOT_DIR="$(git rev-parse --show-toplevel)"
cd "${ROOT_DIR}"

_pick_python() {
  local candidate=""
  for candidate in "${PYO3_PYTHON:-}" "${PYTHON:-}"; do
    if [[ -n ${candidate} && -x ${candidate} ]]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done

  for candidate in python python3; do
    if command -v "${candidate}" >/dev/null 2>&1; then
      command -v "${candidate}"
      return 0
    fi
  done

  return 1
}

if ! PYTHON_BIN="$(_pick_python)"; then
  echo "Python is required to resolve libpython for xiuxian-core-rs tests." >&2
  exit 1
fi

if ! PYLIB_PATH="$("${PYTHON_BIN}" scripts/rust/resolve_libpython_path.py)"; then
  PYLIB_PATH=""
fi

if [[ -z ${PYLIB_PATH} || ! -f ${PYLIB_PATH} ]]; then
  echo "failed to resolve libpython path from active Python: ${PYTHON_BIN}" >&2
  exit 1
fi

TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}"

if [[ $# -eq 0 ]]; then
  set -- --no-fail-fast
fi

echo "Running xiuxian-core-rs tests with CARGO_TARGET_DIR=${TARGET_DIR}"
echo "Resolved Python: ${PYTHON_BIN}"
echo "Resolved libpython: ${PYLIB_PATH}"

case "$(uname -s)" in
Darwin)
  DYLD_INSERT_LIBRARIES="${PYLIB_PATH}" \
    CARGO_TARGET_DIR="${TARGET_DIR}" \
    run_cargo test -p xiuxian-core-rs "$@"
  ;;
*)
  CARGO_TARGET_DIR="${TARGET_DIR}" run_cargo test -p xiuxian-core-rs "$@"
  ;;
esac
