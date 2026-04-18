# FlightSQL Query Surface

:PROPERTIES:
:ID: feat-flightsql-query-surface
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: feature, flightsql, flight, datafusion, search
:STATUS: ACTIVE
:VERSION: 1.0
:END:

## Overview

`xiuxian-wendao` now exposes FlightSQL as another adapter over the shared
`search/queries/` system. This adapter is intentionally separate from native
Wendao business Flight routes.

The first FlightSQL slice stays narrow:

- statement-query execution through `CommandStatementQuery`
- minimal server metadata through `CommandGetSqlInfo`
- one dedicated FlightSQL server builder and binary

The next bounded metadata slice is discovery:

- `CommandGetCatalogs`
- `CommandGetDbSchemas`
- `CommandGetTables`
- one stable logical catalog name: `wendao`
- schema names derived from the registered SQL scope

## Design Rules

- FlightSQL request decoding stays inside `search/queries/flightsql/`.
- Statement execution must reuse the shared request-scoped SQL/DataFusion
  surface.
- FlightSQL must not widen the native Wendao business Flight router.
- The first slice must not implement prepared statements, ingest/update, or
  broad JDBC/XDBC metadata coverage.

## First Supported Commands

### `CommandStatementQuery`

The first statement-execution slice accepts SQL text from the FlightSQL
descriptor and executes it against the same request-scoped SQL surface already
used by the SQL and GraphQL adapters.

### `CommandGetSqlInfo`

The first metadata slice exposes a stable minimal identity for the Wendao
FlightSQL server, enough for compatible clients to negotiate the server
surface.

## Current Entry Points

- shared adapter internals: `search/queries/flightsql/`
- gateway export: `build_search_plane_flightsql_service`
- shared service type: `StudioFlightSqlService`
- standalone server binary: `wendao_search_flightsql_server`

## Snapshot Contract

FlightSQL now also keeps snapshot-level regression coverage under
`search/queries/flightsql/tests/snapshots.rs`, with baselines written to
`tests/snapshots/search/queries/flightsql_query_surface_payload.snap`.

## Boundaries

This first FlightSQL slice does not yet include:

- prepared statements
- statement updates or ingest
- catalogs, schemas, tables, or XDBC type metadata
- reuse of the native Wendao business Flight server port

Native Flight remains the business protocol surface. FlightSQL is only the
query-language adapter over the shared SQL/DataFusion execution core.
The next implementation target is to lift catalogs, schemas, and tables into
FlightSQL discovery while still leaving broader XDBC metadata out of scope.
