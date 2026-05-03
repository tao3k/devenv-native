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
