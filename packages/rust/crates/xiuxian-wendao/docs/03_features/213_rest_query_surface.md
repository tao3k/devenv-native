# REST Query Surface

:PROPERTIES:
:ID: feat-rest-query-surface
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: feature, rest, search, datafusion
:STATUS: ACTIVE
:VERSION: 1.0
:END:

## Overview

`xiuxian-wendao` now exposes REST-style requests as another thin adapter over
the shared `search/queries/` system. The first REST slice is intentionally
narrow: it proves that a REST-shaped request and response contract can reuse
the same `SearchQueryService` as SQL and GraphQL without introducing a second
planner path. The runtime now also mounts one bounded gateway compatibility
route at `/query` for external clients such as Daochang's native
`wendao.search` tool, but that route is still just a transport shim over the
same shared query service.

The first REST surface stays transport-neutral:

- request decoding lives in `search/queries/rest/`
- execution delegates into the already-landed SQL or GraphQL adapters
- the first proof is the CLI entrypoint `wendao query rest --payload ...`
- the gateway compatibility route `POST /query` reuses the same adapter

## Design Rules

- REST request decoding stays inside `search/queries/rest/`.
- REST execution must delegate into the shared query system through
  `SearchQueryService`.
- REST must not own request-scoped SQL surface assembly.
- The first REST slice must not widen native HTTP routes or OpenAPI.

## First Request Contract

The first request contract is tagged JSON:

```json
{
  "query_language": "sql",
  "query": "SELECT sql_table_name FROM wendao_sql_tables ORDER BY sql_table_name"
}
```

```json
{
  "query_language": "graphql",
  "document": "{ wendao_sql_tables { sql_table_name sql_object_kind } }"
}
```

The first response contract mirrors that boundary and returns tagged payloads:

- `sql`: `SqlQueryPayload`
- `graphql`: `GraphqlQueryPayload`

## Current Entry Points

- shared adapter internals: `search/queries/rest/`
- CLI adapter:
  `wendao query rest --payload '{"query_language":"sql","query":"SELECT ..."}'`
- gateway compatibility route: `POST /query`

## Snapshot Contract

REST now also keeps snapshot-level regression coverage under
`search/queries/rest/tests/snapshots.rs`, with baselines written to
`tests/snapshots/search/queries/rest_query_surface_payload.snap`.

## Boundaries

This first REST slice does not yet include:

- OpenAPI growth for search query execution
- REST-specific planning logic
- REST-specific business semantics outside the shared query system

REST remains a thin adapter over the same shared query core already used by
SQL, GraphQL, FlightSQL, and CLI query entrypoints.
