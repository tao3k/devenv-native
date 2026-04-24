# Wendao Vector Boundary Split

## Purpose

`xiuxian-wendao` currently mixes two concerns too closely:

1. lightweight Arrow/DataFusion substrate helpers used by bounded local
   analytics, parser-adjacent helper surfaces, and request-scoped SQL payloads
2. Lance-backed vector-store access through `xiuxian-db-store`

That boundary matters for downstream consumers such as `xiuxian-qianji`. Qianji
uses non-vector Wendao surfaces such as link-graph core build/search, skill
VFS, contract-feedback, gateway OpenAPI helpers, and bounded-work markdown
query payloads. Those consumers should not inherit `lance` unless they actually
request vector-store behavior.

## Current Slice

The active bounded slice for this feature does three things:

1. move generic Arrow/DataFusion substrate ownership out of
   `xiuxian-vector`
2. add an explicit `vector-store` feature boundary in `xiuxian-wendao`
3. retarget `xiuxian-qianji` to the non-vector Wendao surface and prove the
   compile-graph change with `cargo tree`

This is intentionally narrower than a full search-plane rearchitecture. The
goal is compile-time ownership clarity first.

## Current Status

As of 2026-04-10, the bounded non-vector consumer slice is implemented and
validated:

1. `xiuxian-wendao` exposes `vector-store` as an optional feature and the full
   package `cargo check -p xiuxian-wendao --no-default-features` build is now
   warning-clean.
2. `xiuxian-qianji` consumes `xiuxian-wendao` with `default-features = false`.
3. Qianji's normal dependency tree no longer includes the heavy
   `xiuxian-db-store` `vector-store` feature path
   or `lance`.
4. Focused compile proof also passes for the heavier Wendao matrix:
   `--no-default-features --features studio,zhenfa-router,julia,builtin-plugins`.
5. An extra `search-runtime`-only test pass exposed a separate feature-coherence
   cleanup between standalone `search-runtime` and studio-owned search DTOs.
   That follow-up is explicitly outside this bounded slice.
6. `xiuxian-vector` is being reduced to a Lance storage-format shell, while
   active Wendao callers use `xiuxian-db-store` as the explicit storage facade.

## Target Boundary

The target package boundary is:

1. a lightweight substrate crate owns generic Arrow `RecordBatch` aliases,
   request-scoped DataFusion helpers, and other non-Lance compute primitives
2. `xiuxian-db-store` owns the compatibility facade for Lance-backed
   vector-store persistence while `xiuxian-vector` is retired behind it
3. `xiuxian-wendao` exposes non-vector product surfaces without a mandatory
   vector-store dependency and gates vector-only behavior explicitly

## Acceptance Signals

The feature is aligned only when all of the following are true:

1. Qianji's normal dependency tree no longer includes the heavy
   `xiuxian-db-store` `vector-store` feature path
2. Qianji's normal dependency tree no longer includes `lance`
3. touched Wendao and Qianji slices still compile with focused cargo checks
4. bounded-work markdown payload contracts remain stable for Qianji callers

## Stable References

- [DuckDB bounded analytics RFC](../../../../../docs/rfcs/2026-04-08-wendao-qianji-duckdb-bounded-analytics-rfc.md)
