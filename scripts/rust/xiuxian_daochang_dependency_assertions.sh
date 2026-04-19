#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo_bin="${CARGO_BIN:-${script_dir}/cargo_exec.sh}"
target_dir="${CARGO_TARGET_DIR:-/tmp/workspace-strict-proof}"

default_tree="$(mktemp)"
no_default_tree="$(mktemp)"
trap 'rm -f "${default_tree}" "${no_default_tree}"' EXIT

CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" tree -p xiuxian-daochang -e all >"${default_tree}"
CARGO_TARGET_DIR="${target_dir}" "${cargo_bin}" tree -p xiuxian-daochang -e all --no-default-features >"${no_default_tree}"

if ! rg -q "litellm-rs v" "${default_tree}"; then
  echo "xiuxian-daochang default profile must include litellm-rs but it was not found."
  exit 1
fi

if rg -q "litellm-rs v" "${no_default_tree}"; then
  echo "xiuxian-daochang --no-default-features unexpectedly includes litellm-rs."
  exit 1
fi

if ! rg -q 'reqwest v0\.13\.' "${default_tree}"; then
  echo "xiuxian-daochang default profile must include reqwest 0.13.x but it was not found."
  exit 1
fi

if ! rg -q 'reqwest v0\.13\.' "${no_default_tree}"; then
  echo "xiuxian-daochang --no-default-features must include reqwest 0.13.x but it was not found."
  exit 1
fi

if rg -q 'reqwest v0\.12\.' "${default_tree}"; then
  echo "xiuxian-daochang default profile unexpectedly includes reqwest 0.12.x."
  exit 1
fi

if rg -q 'reqwest v0\.12\.' "${no_default_tree}"; then
  echo "xiuxian-daochang --no-default-features unexpectedly includes reqwest 0.12.x."
  exit 1
fi

reqwest_default="$( (rg 'reqwest v0\.11' "${default_tree}" || true) | wc -l | tr -d ' ')"
reqwest_no_default="$( (rg 'reqwest v0\.11' "${no_default_tree}" || true) | wc -l | tr -d ' ')"
reqwest13_default="$( (rg 'reqwest v0\.13\.' "${default_tree}" || true) | wc -l | tr -d ' ')"
reqwest13_no_default="$( (rg 'reqwest v0\.13\.' "${no_default_tree}" || true) | wc -l | tr -d ' ')"
rustls_default="$( (rg 'rustls-pemfile v1\.' "${default_tree}" || true) | wc -l | tr -d ' ')"
rustls_no_default="$( (rg 'rustls-pemfile v1\.' "${no_default_tree}" || true) | wc -l | tr -d ' ')"

echo "xiuxian-daochang dependency signal:"
echo "  default: reqwest0.13=${reqwest13_default} reqwest0.11=${reqwest_default} rustls-pemfile1.x=${rustls_default}"
echo "  no-default-features: reqwest0.13=${reqwest13_no_default} reqwest0.11=${reqwest_no_default} rustls-pemfile1.x=${rustls_no_default}"
