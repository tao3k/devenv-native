---
type: knowledge
title: "Spec: Qianji Runtime Config Layering"
category: "architecture"
status: "draft"
authors:
  - codex
created: 2026-04-07
tags:
  - qianji
  - runtime-config
  - valkey
  - checkpoint
  - llm
---

# Spec: Qianji Runtime Config Layering

This note defines the current runtime-config ownership for `xiuxian-qianji`.

## 1. Config Discovery

Qianji resolves runtime configuration in this file order:

1. system defaults:
   `packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml`
2. user overrides:
   `$PRJ_CONFIG_HOME/xiuxian-artisan-workshop/qianji.toml`
3. explicit override:
   `$QIANJI_CONFIG_PATH`

Legacy user `xiuxian.toml` is not part of the Qianji runtime-config overlay
chain.

## 2. Precedence Rule

For runtime-owned settings, `qianji.toml` is authoritative and environment
variables are fallback inputs unless a typed test/tooling override is injected
through `QianjiRuntimeEnv`.

## 3. Checkpoint Persistence

Checkpoint persistence is owned by the `[checkpoint]` section.

```toml
[checkpoint]
valkey_url = "redis://127.0.0.1:6379/0"
```

Effective precedence for `checkpoint.valkey_url` is:

1. explicit `QianjiRuntimeEnv.qianji_checkpoint_valkey_url`
2. `qianji.toml` `[checkpoint].valkey_url`
3. `QIANJI_VALKEY_URL`
4. `VALKEY_URL`
5. `REDIS_URL`
6. built-in localhost fallback

## 4. Local Workflow State

Local no-server BPMN workflow-state persistence is owned by the
`[workflow_state]` section.

```toml
[workflow_state]
local_duckdb_path = "path/to/qianji-workflow-state.duckdb"
```

Effective precedence for `workflow_state.local_duckdb_path` is:

1. explicit `QianjiRuntimeEnv.qianji_workflow_state_duckdb_path`
2. `qianji.toml` `[workflow_state].local_duckdb_path`
3. `QIANJI_WORKFLOW_STATE_DUCKDB_PATH`
4. built-in local runtime-path fallback

This path is used only by local embedded or CLI BPMN control surfaces. HTTP
workflow control through `qianji-server` continues to default to
`checkpoint.valkey_url`.

## 5. LLM Runtime

LLM runtime continues to resolve from `[llm]` in `qianji.toml`, with runtime
environment overrides such as `QIANJI_LLM_MODEL`, `OPENAI_API_BASE`, and
`OPENAI_API_KEY`.

## 6. Validation Expectations

Changes to Qianji runtime-config layering should keep focused coverage on:

1. file discovery order
2. TOML-first precedence
3. invalid TOML error classification
4. legacy config-file ignore behavior

## 7. Shared Path Resolution

Process-scoped `PRJ_ROOT` fallback resolution for Qianji runtime surfaces should
stay on the shared helper lane:

1. `xiuxian-config-core::resolve_project_root_or_cwd_from_value(...)` owns the
   low-level trim and relative-path semantics
2. `xiuxian-qianji::runtime_config::pathing` owns Qianji-side reuse of that
   helper for runtime-config, scheduler preflight, bootcamp, and refresh
   callers
3. Qianji runtime owners should not add new direct `PRJ_ROOT` parsing branches
   when the shared helper path already fits the required fallback policy

Contract-feedback storage path resolution follows the same ownership split:

1. `xiuxian-config-core::resolve_cache_home(...)` and
   `resolve_cache_home_from_value(...)` own the generic `PRJ_CACHE_HOME`
   trimming, relative-path joining, and default `.cache` fallback
2. `xiuxian-qianji/src/bin/qianji.rs` may still enforce the package-specific
   rule that contract-feedback cache storage must stay under the resolved
   workspace root when an absolute override points elsewhere

## 8. Macro Boundary

The earlier config-core rollout intentionally moved ownership and precedence
semantics first, even when that temporarily made some call sites look longer.
That phase reduced drift, but it did not yet change the syntax model for typed
`Option` precedence chains.

The current boundary is:

1. `xiuxian-config-core::first_some!` owns the generic "first present typed
   candidate wins" pattern for runtime-config precedence chains
2. `xiuxian-macros::env_non_empty!` owns raw env-string trimming for direct env
   reads
3. Qianji runtime-config owners still keep package-specific policy local, such
   as typed test overrides, Qianji fallback order, and any package-specific
   post-processing

This keeps macros small and mechanical. `xiuxian-config-core` should not grow a
derive-based projection layer just to reduce a few lines of precedence code.
