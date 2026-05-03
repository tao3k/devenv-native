# xiuxian-wendao-web

`xiuxian-wendao-web` is the web-facing namespace boundary for Wendao gateway
surfaces.

The crate currently provides compatibility exports backed by
`xiuxian-wendao::gateway`. This lets callers adopt the clearer package boundary
before the gateway implementation is physically moved.

## Ownership

This crate owns:

- HTTP and gateway namespace exports.
- Studio web API namespace exports.
- OpenAPI document and route contract exports.
- Web-facing compatibility imports during the migration.

`xiuxian-wendao` continues to own graph, search, repository indexing, parser,
analyzer, and domain-runtime behavior. The dependency direction is one-way:
`xiuxian-wendao-web` may depend on `xiuxian-wendao`, but `xiuxian-wendao` must
not depend on `xiuxian-wendao-web`.

## Migration Rule

New web and gateway callers should prefer `xiuxian_wendao_web`. Existing
`xiuxian_wendao::gateway` callers remain supported while implementation modules
are migrated in later slices.
