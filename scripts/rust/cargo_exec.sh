#!/usr/bin/env bash
set -euo pipefail

_repo_root() {
  local script_dir=""
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  cd "${script_dir}/../.." && pwd
}

_rust_toolchain_from_file() {
  local file=""
  local channel=""
  for file in "$@"; do
    if [[ -f ${file} ]]; then
      channel="$(awk -F'"' '/^[[:space:]]*channel[[:space:]]*=/ { print $2; exit }' "${file}")"
      if [[ -n ${channel} ]]; then
        printf '%s\n' "${channel}"
        return 0
      fi
    fi
  done
  return 1
}

_pick_rustup_toolchain() {
  local root=""
  if [[ -n ${RUSTUP_TOOLCHAIN:-} ]]; then
    printf '%s\n' "${RUSTUP_TOOLCHAIN}"
    return 0
  fi
  root="$(_repo_root)"
  _rust_toolchain_from_file \
    "${root}/packages/rust/rust-toolchain.toml" \
    "${root}/rust-toolchain.toml" \
    || printf '%s\n' "stable"
}

if [[ -z ${RUSTUP_TOOLCHAIN:-} ]]; then
  export RUSTUP_TOOLCHAIN="$(_pick_rustup_toolchain)"
fi

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

_is_real_cargo() {
  local candidate="${1:-}"
  local resolved=""
  local version=""
  if [[ -z ${candidate} || ! -x ${candidate} ]]; then
    return 1
  fi
  if [[ $(basename "${candidate}") == "rustup-init" || $(basename "${candidate}") == "rustup-init.exe" ]]; then
    return 1
  fi
  resolved="$(command -v greadlink >/dev/null 2>&1 && greadlink -f "${candidate}" 2>/dev/null || true)"
  if [[ -z ${resolved} ]]; then
    resolved="$(readlink -f "${candidate}" 2>/dev/null || true)"
  fi
  if [[ -n ${resolved} && ( $(basename "${resolved}") == "rustup-init" || $(basename "${resolved}") == "rustup-init.exe" ) ]]; then
    return 1
  fi
  version="$("${candidate}" --version 2>/dev/null || true)"
  [[ ${version} == cargo\ * ]] && _has_real_rustc "${candidate}"
}

_is_real_rustc() {
  local candidate="${1:-}"
  local resolved=""
  local version=""
  if [[ -z ${candidate} || ! -x ${candidate} ]]; then
    return 1
  fi
  if [[ $(basename "${candidate}") == "rustup-init" || $(basename "${candidate}") == "rustup-init.exe" ]]; then
    return 1
  fi
  resolved="$(command -v greadlink >/dev/null 2>&1 && greadlink -f "${candidate}" 2>/dev/null || true)"
  if [[ -z ${resolved} ]]; then
    resolved="$(readlink -f "${candidate}" 2>/dev/null || true)"
  fi
  if [[ -n ${resolved} && ( $(basename "${resolved}") == "rustup-init" || $(basename "${resolved}") == "rustup-init.exe" ) ]]; then
    return 1
  fi
  version="$("${candidate}" --version 2>/dev/null || true)"
  [[ ${version} == rustc\ * ]]
}

_has_real_rustc() {
  local cargo_candidate="${1:-}"
  local rustc_candidate=""
  for rustc_candidate in \
    "${RUSTC:-}" \
    "$(dirname "${cargo_candidate}")/rustc" \
    "$(command -v rustc 2>/dev/null || true)"
  do
    if _is_real_rustc "${rustc_candidate}"; then
      return 0
    fi
  done
  return 1
}

_is_real_rustup() {
  local candidate="${1:-}"
  local resolved=""
  local version=""
  if [[ -z ${candidate} || ! -x ${candidate} ]]; then
    return 1
  fi
  if [[ $(basename "${candidate}") == "rustup-init" || $(basename "${candidate}") == "rustup-init.exe" ]]; then
    return 1
  fi
  resolved="$(command -v greadlink >/dev/null 2>&1 && greadlink -f "${candidate}" 2>/dev/null || true)"
  if [[ -z ${resolved} ]]; then
    resolved="$(readlink -f "${candidate}" 2>/dev/null || true)"
  fi
  if [[ -n ${resolved} && ( $(basename "${resolved}") == "rustup-init" || $(basename "${resolved}") == "rustup-init.exe" ) ]]; then
    return 1
  fi
  version="$("${candidate}" --version 2>/dev/null || true)"
  [[ ${version} == rustup\ * ]]
}

_pick_cargo() {
  local candidate=""
  local rustup_bin=""
  local rustup_cargo=""
  local rustup_toolchain=""
  for candidate in \
    "${CARGO:-}" \
    "${DEVENV_PROFILE:-}/bin/cargo" \
    "${HOME:-}/.cargo/bin/cargo"
  do
    if _is_real_cargo "${candidate}"; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done

  candidate="$(command -v cargo 2>/dev/null || true)"
  if _is_real_cargo "${candidate}"; then
    printf '%s\n' "${candidate}"
    return 0
  fi

  rustup_bin="$(command -v rustup 2>/dev/null || true)"
  if _is_real_rustup "${rustup_bin}"; then
    rustup_toolchain="$(_pick_rustup_toolchain)"
    rustup_cargo="$("${rustup_bin}" which --toolchain "${rustup_toolchain}" cargo 2>/dev/null || true)"
    if _is_real_cargo "${rustup_cargo}"; then
      printf '%s\n' "${rustup_cargo}"
      return 0
    fi
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
  if [[ -z ${CC:-} || ${CC} == "clang" ]]; then
    export CC="/usr/bin/clang"
  fi
  if [[ -z ${CXX:-} || ${CXX} == "clang++" ]]; then
    export CXX="/usr/bin/clang++"
  fi
  export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="${CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER:-/usr/bin/clang}"
  export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER="${CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER:-/usr/bin/clang}"
fi

# Prefer precompiled Metal kernels on local macOS builds.
# Auto-fallback for isolated environments where `metal` tool is unavailable.
if [[ "$(uname -s)" == "Darwin" && -z ${MISTRALRS_METAL_PRECOMPILE:-} ]]; then
  if ! xcrun -sdk macosx metal -v >/dev/null 2>&1; then
    export MISTRALRS_METAL_PRECOMPILE=0
  fi
fi

if cargo_bin="$(_pick_cargo)"; then
  cargo_bin_dir="$(dirname "${cargo_bin}")"
  case ":${PATH:-}:" in
    *":${cargo_bin_dir}:"*) ;;
    *) export PATH="${cargo_bin_dir}:${PATH:-}" ;;
  esac
  exec "${cargo_bin}" "$@"
fi

rustup_bin="$(command -v rustup 2>/dev/null || true)"
if _is_real_rustup "${rustup_bin}"; then
  rustup_toolchain="$(_pick_rustup_toolchain)"
  exec "${rustup_bin}" run "${rustup_toolchain}" cargo "$@"
fi

printf 'error: usable cargo executable was not found; PATH cargo may be rustup-init without an installed Rust toolchain\n' >&2
exit 127
