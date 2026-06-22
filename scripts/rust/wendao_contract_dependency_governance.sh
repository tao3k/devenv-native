#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"

require_command() {
  local tool="$1"
  if ! command -v "${tool}" >/dev/null 2>&1; then
    printf 'error: required tool not found: %s\n' "${tool}" >&2
    return 1
  fi
}

resolve_semver_baseline_rev() {
  if [[ -n ${WENDAO_CORE_SEMVER_BASELINE_REV:-} ]]; then
    printf '%s\n' "${WENDAO_CORE_SEMVER_BASELINE_REV}"
    return 0
  fi

  if git -C "${repo_root}" rev-parse --verify --quiet origin/main >/dev/null 2>&1; then
    git -C "${repo_root}" merge-base HEAD origin/main
    return 0
  fi

  if git -C "${repo_root}" rev-parse --verify --quiet HEAD^ >/dev/null 2>&1; then
    git -C "${repo_root}" rev-parse HEAD^
    return 0
  fi

  printf 'error: unable to resolve a semver baseline revision; set WENDAO_CORE_SEMVER_BASELINE_REV explicitly\n' >&2
  return 1
}

run_semver_core() {
  local baseline_rev
  baseline_rev="$(resolve_semver_baseline_rev)"

  require_command cargo-semver-checks

  printf 'Running cargo-semver-checks for xiuxian-wendao-core against baseline %s\n' "${baseline_rev}"
  cd "${repo_root}"
  direnv exec . cargo semver-checks check-release \
    --manifest-path packages/rust/crates/xiuxian-wendao-core/Cargo.toml \
    --baseline-rev "${baseline_rev}"
}

run_machete_wendao() {
  local crate_path
  local status
  local findings=0
  local crate_paths=(
    "packages/rust/crates/xiuxian-wendao-core"
    "packages/rust/crates/xiuxian-wendao-runtime"
    "packages/rust/crates/xiuxian-wendao"
    "packages/rust/crates/xiuxian-wendao-julia"
  )

  require_command cargo-machete

  cd "${repo_root}"
  for crate_path in "${crate_paths[@]}"; do
    printf 'Running cargo-machete advisory scan for %s\n' "${crate_path}"
    set +e
    direnv exec . cargo machete "${crate_path}"
    status=$?
    set -e

    case "${status}" in
    0) ;;
    1)
      findings=1
      ;;
    *)
      printf 'error: cargo-machete failed for %s with exit code %s\n' "${crate_path}" "${status}" >&2
      return "${status}"
      ;;
    esac
  done

  if [[ ${findings} -eq 1 ]]; then
    printf 'cargo-machete reported advisory findings in the Wendao migration cluster\n'
  fi
}

nightly_rustup_available() {
  if ! command -v rustup >/dev/null 2>&1; then
    return 1
  fi

  rustup toolchain list 2>/dev/null | rg -q '^nightly'
}

run_udeps_wendao() {
  local status
  local output

  require_command cargo-udeps

  if ! nightly_rustup_available; then
    printf 'Skipping cargo-udeps advisory scan: rustup nightly toolchain is not available in the current environment\n'
    return 0
  fi

  printf 'Running cargo-udeps advisory scan for xiuxian-wendao-core and xiuxian-wendao-runtime\n'
  cd "${repo_root}"

  set +e
  output="$(
    rustup run nightly cargo udeps \
      --workspace \
      --package xiuxian-wendao-core \
      --package xiuxian-wendao-runtime \
      --all-targets 2>&1
  )"
  status=$?
  set -e

  printf '%s\n' "${output}"

  if [[ ${status} -eq 0 ]]; then
    return 0
  fi

  if printf '%s\n' "${output}" | rg -qi 'unused crates|unused dependencies'; then
    printf 'cargo-udeps reported advisory findings for the bounded Wendao scope\n'
    return 0
  fi

  printf 'error: cargo-udeps failed with exit code %s\n' "${status}" >&2
  return "${status}"
}

case "${1:-}" in
semver-core)
  run_semver_core
  ;;
machete-wendao)
  run_machete_wendao
  ;;
udeps-wendao)
  run_udeps_wendao
  ;;
*)
  printf 'usage: %s {semver-core|machete-wendao|udeps-wendao}\n' "${0##*/}" >&2
  exit 1
  ;;
esac
