#!/usr/bin/env bash
set -euo pipefail

_pick_python() {
  local candidate=""
  for candidate in "${PYO3_PYTHON:-}" "${PYTHON:-}"; do
    if [[ -n ${candidate} && -x ${candidate} ]]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done

  candidate="$(command -v python 2>/dev/null || true)"
  if [[ -n ${candidate} && -x ${candidate} ]]; then
    printf '%s\n' "${candidate}"
    return 0
  fi

  candidate="$(command -v python3 2>/dev/null || true)"
  if [[ -n ${candidate} && -x ${candidate} ]]; then
    printf '%s\n' "${candidate}"
    return 0
  fi

  return 1
}

if [[ -n ${PYO3_PYTHON:-} && ! -x ${PYO3_PYTHON} ]]; then
  unset PYO3_PYTHON
fi
unset PYO3_NO_PYTHON

if python_bin="$(_pick_python)"; then
  export PYO3_PYTHON="${python_bin}"
fi

# Ensure macOS SDK path is available for Rust/C toolchain probes.
if [[ "$(uname -s)" == "Darwin" && -z ${SDKROOT:-} ]]; then
  if sdkroot_path="$(xcrun --sdk macosx --show-sdk-path 2>/dev/null)"; then
    if [[ -n ${sdkroot_path} ]]; then
      export SDKROOT="${sdkroot_path}"
    fi
  fi
fi

# Ensure system libraries such as libiconv are discoverable for build scripts
# that invoke the platform linker directly during macOS cargo builds.
if [[ "$(uname -s)" == "Darwin" && -n ${SDKROOT:-} ]]; then
  sdk_lib_path="${SDKROOT}/usr/lib"
  if [[ -d ${sdk_lib_path} ]]; then
    case ":${LIBRARY_PATH:-}:" in
      *":${sdk_lib_path}:"*) ;;
      *)
        if [[ -n ${LIBRARY_PATH:-} ]]; then
          export LIBRARY_PATH="${sdk_lib_path}:${LIBRARY_PATH}"
        else
          export LIBRARY_PATH="${sdk_lib_path}"
        fi
        ;;
    esac
  fi
fi

# Prefer the system Clang toolchain on macOS for crates that compile C/C++ code.
if [[ "$(uname -s)" == "Darwin" ]]; then
  export CC="${CC:-/usr/bin/clang}"
  export CXX="${CXX:-/usr/bin/clang++}"
fi

# Prefer precompiled Metal kernels on local macOS builds.
# Auto-fallback for isolated environments where `metal` tool is unavailable.
if [[ "$(uname -s)" == "Darwin" && -z ${MISTRALRS_METAL_PRECOMPILE:-} ]]; then
  if ! xcrun -sdk macosx metal -v >/dev/null 2>&1; then
    export MISTRALRS_METAL_PRECOMPILE=0
  fi
fi

exec cargo "$@"
