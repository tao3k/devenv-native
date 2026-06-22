# Config Import Overlay

## Purpose

Wendao configuration now resolves through one merge engine:
`xiuxian-config-core` owns recursive `imports = [...]`, import-path
environment expansion, and merge order. Wendao and gateway consumers project
typed runtime values from that merged TOML instead of re-implementing line
scanners or local deep-merge helpers.

## Merge Model

The merge order is:

1. embedded defaults from `resources/config/wendao.toml`
2. boot-time base config from `$PRJ_ROOT/wendao.toml` or an explicit override
   path
3. optional legacy Studio overlay wrapper from `wendao.studio.overlay.toml`

`imports` are merged before the importing file body, so the importing file wins
on conflicts. That means any legacy Studio wrapper must import the base file:

```toml
imports = ["wendao.toml"]

[sources.projects.main]
root = "."
dirs = ["docs", "src"]
```

The reverse shape would be wrong because `wendao.toml` would overwrite the
overlay on merge.

## Effective Config Paths

- the checked-in workspace boot config now lives at `$PRJ_ROOT/wendao.toml`
- shared runtime settings in `src/settings/` resolve through
  `xiuxian-config-core`
- gateway CLI config resolution prefers the sibling
  `wendao.studio.overlay.toml` wrapper only when a legacy file still exists
- Studio bootstrap config loading reads the same effective path
- repo-intelligence config loading is import-aware, so base config plus any
  legacy overlay repo registrations use the same merged TOML contract
- `nix/modules/process.nix` now points at the root config and uses a bounded
  TOML-aware helper under `scripts/runtime/` for readiness-port discovery

## Gateway Runtime Knobs

Gateway process knobs now follow the same TOML-first contract instead of
living as env-only parser glue in `command.rs`.

The canonical keyspace is:

```toml
[gateway.runtime]
listen_backlog = 2048
studio_concurrency_limit = 64
studio_request_timeout_secs = 15
```

Resolution order for these fields is:

1. merged `wendao.toml` plus any legacy overlay settings
2. env fallback:
   `XIUXIAN_WENDAO_GATEWAY_LISTEN_BACKLOG`
   `XIUXIAN_WENDAO_GATEWAY_STUDIO_CONCURRENCY_LIMIT`
   `XIUXIAN_WENDAO_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS`
3. built-in defaults plus clamp bounds in the gateway command surface

This keeps operator-facing gateway limits on the same config-core lane as the
other TOML-backed runtime owners while preserving env fallback when the TOML
keys are omitted.

Gateway webhook fallback now follows the same trim-aware precedence hygiene:

1. merged `wendao.toml` plus any legacy overlay `gateway.webhook_*` settings
2. env fallback:
   `WENDAO_WEBHOOK_URL`
   `WENDAO_WEBHOOK_SECRET`
3. built-in webhook defaults

Blank env values are treated as absent instead of becoming authoritative
runtime config.

The gateway notify-status surface reports the effective webhook URL captured at
startup, so `/api/notify/status` now reflects the same resolved TOML-first
runtime config the notification worker is actually using instead of re-reading
`WENDAO_WEBHOOK_*` from the live process environment.

## Environment Variables In Import Paths

`xiuxian-config-core` now expands environment variables inside `imports`
entries. Supported forms are `$VAR` and `${VAR}`.

The intended use is path-like import composition, for example:

```toml
imports = ["${PRJ_ROOT}/packages/rust/crates/xiuxian-wendao/resources/config/wendao.toml"]
```

This support is intentionally limited to import paths. Wendao does not treat
arbitrary string values in TOML as general environment-template fields.

## Studio Bootstrap Ownership

Studio UI state is now backend-loaded from the effective `wendao.toml` import
graph at startup. The runtime still understands legacy overlay composition
while reading config, but it no longer exposes `/api/ui/config` as a live
rewrite surface for mutating `wendao.toml` from the Studio process.

## Ownership Boundary

- `xiuxian-config-core` owns import recursion, path expansion, and merge
  semantics
- `xiuxian-wendao/src/settings/` owns conversion from merged TOML into Wendao
  runtime settings
- `gateway/studio/router/config/` owns legacy overlay compatibility plus
  `UiConfig` projection and base-file persistence
- `src/bin/wendao/execute/gateway/config.rs` consumes merged gateway settings
  instead of scanning TOML text, and now consumes the shared
  `xiuxian-config-core` project-root helper for `$PRJ_ROOT/wendao.toml`
  discovery instead of owning a separate `PRJ_ROOT` lookup path
