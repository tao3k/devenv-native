---
type: knowledge
title: "Xiuxian OS Testing Guide"
category: "developer"
tags:
  - developer
  - testing
saliency_base: 6.3
decay_rate: 0.04
metadata:
  title: "Xiuxian OS Testing Guide"
---

# Xiuxian OS Testing Guide

## Current Python Scope

Retained Python testing covers only:

1. `packages/python/foundation/tests`
2. `packages/python/core/tests`
3. `packages/python/wendao-core-lib/tests`
4. `packages/python/wendao-arrow-interface/tests`
5. `packages/python/xiuxian-wendao-analyzer/tests`
6. `scripts/tests/test_*.py`

Python agent/skill/runtime test suites are gone with the deleted packages.

Python project-policy checks consume `python-lang-project-harness` from its
standalone repository. Do not add new repo-local Python parser or project
harness forks for package tests.

Rust project-policy gates consume `rust-lang-project-harness` where a crate
only needs layout, modularity, and agent-policy checks. The retired repo-local
testing crate must not be reintroduced; scenario, contract, or performance
helper needs must use explicit successor surfaces owned by the relevant package
or standalone harness.

## Recommended Commands

```bash
# Retained Python package tests
just test-python

# Direct package-level runs
uv run pytest packages/python/foundation/tests
uv run pytest packages/python/core/tests
uv run pytest packages/python/wendao-core-lib/tests
cd packages/python/wendao-arrow-interface && uv run pytest tests
cd packages/python/xiuxian-wendao-analyzer && uv run pytest tests
uv run pytest scripts/tests

# Rust validation
cargo check --workspace --all-targets
cargo nextest run --workspace
```

## Architecture Rule

When a change touches Rust-owned runtime behavior, validate that behavior in
Rust first. Python tests should cover only retained consumer/helper boundaries,
not deleted local runtime systems.
