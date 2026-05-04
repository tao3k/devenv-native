# xiuxian-wendao-web

`xiuxian-wendao-web` is the narrow Flight and gRPC transport boundary for
Wendao.

This crate does not own Studio HTTP routes, OpenAPI contracts, repository
analysis, parsers, config loading, DuckDB access, git repository management, or
frontend-facing state. Those heavier gateway concerns live in
`xiuxian-wendao-studio`.

## Features

- `transport`: exposes the runtime-owned Flight/gRPC route contracts, route
  provider traits, request validators, and `WendaoFlightService` facade.

The default feature set is empty so downstream crates do not accidentally pull
transport or domain dependencies.

## Dependency Boundary

The crate's direct dependency set is intentionally limited to Arrow Flight,
Arrow schema/array types, Tokio streams, Futures, Base64, Async Trait, and
Tonic. A full dependency tree can still show `axum` because `arrow-flight`
depends on Tonic's generated Flight service support, and Tonic currently routes
that support through its internal router feature. That transitive implementation
detail must not become a direct `xiuxian-wendao-web` dependency or a reason to
move Studio HTTP ownership back into this crate.
