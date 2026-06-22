#!/usr/bin/env bash
set -euo pipefail

run_cargo() {
  if [[ -n ${CARGO_BIN:-} ]]; then
    "${CARGO_BIN}" "$@"
  else
    direnv exec . cargo "$@"
  fi
}

# Temporary transitive exceptions for unresolved upstream advisories.
# Remove entries as dependency chains are upgraded.
ignore_args=(
  --ignore RUSTSEC-2023-0071
  --ignore RUSTSEC-2025-0141
  --ignore RUSTSEC-2024-0436
  --ignore RUSTSEC-2025-0134
  --ignore RUSTSEC-2026-0002
)
run_cargo audit --deny warnings "${ignore_args[@]}"
