# xiuxian-wendao-web

`xiuxian-wendao-web` is the web-facing namespace boundary for Wendao gateway
surfaces.

The crate currently provides compatibility exports backed by
runtime-owned OpenAPI artifacts by default and `xiuxian-wendao::gateway` when
the `studio` feature is enabled. This lets callers adopt the clearer package
boundary before the gateway implementation is physically moved.

## Ownership

This crate owns:

- HTTP and gateway namespace exports.
- OpenAPI document and route contract exports.
- Studio web API namespace exports behind the `studio` feature.
- Web-facing compatibility imports during the migration.

`xiuxian-wendao` continues to own graph, search, repository indexing, parser,
analyzer, and domain-runtime behavior. The dependency direction is one-way:
`xiuxian-wendao-web` may depend on `xiuxian-wendao`, but `xiuxian-wendao` must
not depend on `xiuxian-wendao-web`.

## Migration Rule

New web and gateway callers should prefer `xiuxian_wendao_web`. Existing
`xiuxian_wendao::gateway` callers remain supported while implementation modules
are migrated in later slices.

The default feature set stays light and exposes the OpenAPI document surface
without depending on `xiuxian-wendao`. Enable `studio` when a caller needs the
full Studio router, gateway state, route contracts, and service-boundary
handlers.
