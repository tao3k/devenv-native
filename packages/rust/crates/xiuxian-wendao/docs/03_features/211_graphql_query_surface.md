# GraphQL Query Surface

:PROPERTIES:
:ID: feat-graphql-query-surface
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: feature, graphql, search, graph
:STATUS: ACTIVE
:VERSION: 1.0
:END:

## Overview

`xiuxian-wendao` now exposes GraphQL as another adapter over the shared
`search/queries/` system. The current GraphQL slice is intentionally narrow:
it proves that GraphQL can stay a table-query frontend without introducing a
GraphQL-only planner or a GraphQL-local dataframe execution path.

The first GraphQL surface follows the ROAPI-style shape:

- the root field name is a SQL-visible table or view name
- query operators are expressed as GraphQL arguments
- the adapter translates the parsed GraphQL table query into SQL text
- execution is delegated into the shared request-scoped SQL surface

## Design Rules

- GraphQL document parsing stays inside `search/queries/graphql/`.
- GraphQL root fields must map to SQL-visible tables or logical views.
- GraphQL operators must compile into SQL text, not adapter-local dataframe
  operators.
- The GraphQL adapter should return GraphQL-style JSON data, but it should not
  own new business semantics that already exist elsewhere in Wendao.

## First Root Field

### `wendao_sql_tables`

The first discovery field exposes request-scoped SQL-visible objects from the
shared SQL discovery surface using the same table name that SQL clients query.

Example:

```graphql
{
  wendao_sql_tables(
    filter: { sql_object_kind: "view" }
    sort: [{ field: "sql_table_name" }]
    limit: 10
  ) {
    sql_table_name
    sql_object_kind
    source_count
    repo_id
  }
}
```

The initial operator set matches the narrow ROAPI-style table-query shape:

- `filter`
- `sort`
- `limit`
- `page`

## Current Entry Points

- shared adapter internals: `search/queries/graphql/`
- CLI adapter: `wendao query graphql --document '{ wendao_sql_tables { sql_table_name } }'`

## Snapshot Contract

GraphQL now also keeps snapshot-level regression coverage under
`search/queries/graphql/tests/snapshots.rs`, with baselines written to
`tests/snapshots/search/queries/graphql_query_surface_payload.snap`.

## Boundaries

This first GraphQL slice does not yet include:

- a full GraphQL HTTP endpoint
- mutations or subscriptions
- GraphQL coverage for every Wendao business route
- custom graph business fields such as `graphNeighbors(...)`
- GraphQL-specific planning rules outside the shared query system

Graph-native Wendao capabilities should stay in native Flight or be surfaced to
GraphQL only after they are materialized as SQL-visible tables or logical
views.
