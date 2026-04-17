# Wendao Package Boundary Matrix

:PROPERTIES:
:ID: wendao-package-boundary-matrix
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, migration, packages, core, runtime, ownership
:STATUS: ACTIVE
:END:

## Purpose

This note is the contributor-facing answer to one recurring question:

> new Wendao code should land in `xiuxian-wendao`, `xiuxian-wendao-core`, or
> `xiuxian-wendao-runtime`?

The roadmap already defines the migration intent. This note turns that intent
into one package-selection rule that can be applied during everyday work.

Primary references:

- `[[06_roadmap/409_core_runtime_plugin_surface_inventory]]`
- `[[06_roadmap/412_core_runtime_plugin_program]]`
- `[[06_roadmap/413_m2_core_extraction_package_list]]`
- `[[06_roadmap/414_m3_runtime_extraction_package_list]]`

## One-Sentence Rule

- `xiuxian-wendao-core`: stable shared contracts
- `xiuxian-wendao-runtime`: generic host behavior
- `xiuxian-wendao`: Wendao business domain and the remaining migration shell

## Package Matrix

| Package                  | Owns                                                                                                                                                                  | Must not own                                                                                                                            | Typical examples                                                                                                |
| :----------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------- |
| `xiuxian-wendao-core`    | stable identifiers, descriptors, traits, payload records, transport descriptors, repo-intelligence contract shapes                                                    | filesystem discovery, env resolution, network clients, host bootstrap, gateway assembly, plugin-specific runtime defaults               | `ids`, `capabilities`, `artifacts`, `transport`, stable repo-intelligence records                               |
| `xiuxian-wendao-runtime` | runtime config resolution, settings merge, transport negotiation, Flight client/server helpers, artifact resolution/rendering from live state                         | business graph/search/storage logic, product semantics, long-lived stable DTO ownership, language/plugin algorithms                     | `runtime_config`, `settings`, `transport`, runtime artifact helpers                                             |
| `xiuxian-wendao`         | knowledge graph, search/retrieval, storage, enhancers, analyzers, search plane, Wendao-specific business services and temporary compatibility seams not yet extracted | stable plugin contracts that plugins consume directly, generic runtime infrastructure that does not depend on Wendao business semantics | `graph`, `link_graph`, `search`, `storage`, `gateway` business handlers, `zhenfa_router` integration, analyzers |

## Decision Procedure

When adding a new boundary, apply this order:

1. If the type must be consumed by plugins or external packages without
   dragging in runtime lifecycle behavior, it belongs in `xiuxian-wendao-core`.
2. If the code resolves config, talks to the network, negotiates transport,
   reads runtime state, or materializes host-side clients/servers, it belongs
   in `xiuxian-wendao-runtime`.
3. If the code expresses Wendao domain behavior, retrieval semantics, graph
   logic, storage behavior, or business-route materialization, it belongs in
   `xiuxian-wendao`.

## Boundary Rules

### `xiuxian-wendao-core`

Use `core` for records and traits that should remain valid even if the host
process, transport implementation, or plugin package changes.

Allowed:

1. ids and selectors
2. artifact payload and launch-spec records
3. capability and transport descriptors
4. repo-intelligence traits and record shapes

Reject from `core`:

1. reading config files
2. applying env-var overrides
3. opening network connections
4. spawning processes
5. embedding Julia or Modelica runtime defaults as primary ownership

### `xiuxian-wendao-runtime`

Use `runtime` for generic host behavior that can change with deployment,
environment, transport policy, or runtime state, but should not be coupled to
the full Wendao domain crate.

Allowed:

1. config parsing and override resolution
2. transport negotiation and client/server construction
3. runtime artifact resolution and rendering
4. host-side request metadata and query-contract helpers

Reject from `runtime`:

1. stable contract ownership that plugins import directly
2. knowledge-graph or retrieval-scoring logic
3. plugin-owned language intelligence implementation

### `xiuxian-wendao`

Use the main crate for Wendao product semantics and business behavior.
This includes the actual graph, search, storage, analyzer, and business-route
materialization logic.

Allowed:

1. knowledge graph and link-graph logic
2. search-plane and retrieval orchestration
3. storage and Valkey/Lance domain behavior
4. business handlers that shape Wendao-specific responses
5. temporary compatibility seams that have not yet been extracted

Reject from the main crate for new code:

1. new stable plugin contract types that could live in `core`
2. new generic runtime negotiation helpers that could live in `runtime`

## Current Reality Versus Target Ownership

The current tree is still mid-migration. Some seams remain physically in
`xiuxian-wendao` even though the target owner is `runtime`.

Treat this document as the rule for **new ownership**, not as a claim that the
tree is already fully migrated.

The main mismatches today are:

1. some gateway assembly and host integration seams still live in
   `xiuxian-wendao`
2. some compatibility wrappers remain in the main crate to preserve existing
   import paths
3. most downstream consumers still depend on `xiuxian-wendao` directly, even
   when they only need a narrower `core` or `runtime` surface

One bounded follow-up is now closed: `WendaoResourceUri` moved into
`xiuxian-wendao-core`, and URI-only consumers in `xiuxian-qianji` and
`xiuxian-qianhuan` now import that shared contract from `core` directly.
Another bounded follow-up is now closed: `KnowledgeEntry` moved into
`xiuxian-wendao-core`, `xiuxian-qianji` record-only contract-feedback
consumers now import that shared payload from `core` directly, and
`KnowledgeStorage` remains in `xiuxian-wendao`.
Another bounded follow-up is now closed:
`WendaoContractFeedbackAdapter` moved into `xiuxian-wendao-core`,
`xiuxian-qianji` now imports that pure contract-feedback projection helper
from `core` directly, and `xiuxian-wendao` keeps only a thin compatibility
re-export for the adapter path.
Another bounded follow-up is now closed: `CognitiveTraceRecord`,
`LinkGraphSemanticDocument`, and `LinkGraphSemanticDocumentKind` moved into
`xiuxian-wendao-core`, `xiuxian-qianji` sovereign-memory consumers now import
those shared semantic-document payloads from `core` directly, and
`xiuxian-wendao` keeps only a thin compatibility re-export while retaining
link-graph indexing and ingestion behavior.
Another bounded follow-up is now closed: the pure link-graph query-contract
family moved into `xiuxian-wendao-core`, `xiuxian-qianji` now imports
`LinkGraphSearchOptions` from `core` directly, and `xiuxian-wendao` keeps
query execution and retrieval semantics while only re-exporting the stable
query DTO family.
Another bounded follow-up is now closed: the stable entity-record family moved
into `xiuxian-wendao-core`, `xiuxian-qianji` now imports `Entity`,
`Relation`, `EntityType`, and `RelationType` from `core` directly, and
`xiuxian-wendao` keeps graph execution and persistence semantics while only
re-exporting the stable entity payload family.
Another bounded follow-up is now closed: `LinkGraphRefreshMode` moved into
`xiuxian-wendao-core`, `xiuxian-qianji` now imports that stable refresh-mode
enum from `core` directly, and `xiuxian-wendao` keeps `LinkGraphIndex`
refresh execution while only re-exporting the shared status enum.
Another bounded follow-up is now closed: the stable SQL result DTO family
(`SqlQueryPayload`, `SqlQueryMetadata`, `SqlBatchPayload`, and
`SqlColumnPayload`) moved into `xiuxian-wendao-core`, `xiuxian-qianji` now
imports `SqlQueryPayload` from `core` directly, and `xiuxian-wendao` keeps
SQL execution and bounded-work query wiring while only re-exporting the
shared result payload family.
Another bounded follow-up is now closed: the bounded-work markdown SQL helper
family moved into `xiuxian-wendao-sql`, `xiuxian-qianji` now imports the
direct bounded-work payload helper from that crate, and `xiuxian-wendao`
keeps only a thin compatibility re-export under
`search::queries::sql::bounded_work_markdown`.
Another bounded follow-up is now closed: the embedded zhixing text/path/mount
helper family moved into `xiuxian-wendao-runtime`, `xiuxian-qianji` now
imports the direct text helper from `runtime`, and `xiuxian-wendao` keeps
discovery, registry, and resolver semantics while only re-exporting the thin
runtime compatibility seam for embedded artifact access.
Another bounded follow-up is now closed: the bundled Wendao gateway `OpenAPI`
artifact helper family moved into `xiuxian-wendao-runtime`, `xiuxian-qianji`
now imports the direct bundled artifact helpers from `runtime`, and
`xiuxian-wendao` keeps gateway route inventory and `OpenAPI` semantics while
only re-exporting the thin runtime compatibility seam for bundled artifact
access.

## Practical Heuristics

If unsure, ask these questions in order:

1. Could a plugin crate depend on this without pulling host lifecycle code?
   If yes, move toward `core`.
2. Does this require filesystem, env, network, process, or transport
   negotiation?
   If yes, move toward `runtime`.
3. Does this encode Wendao graph/search/storage/business semantics?
   If yes, keep it in `xiuxian-wendao`.

## Migration Bias

For new work, prefer shrinking `xiuxian-wendao` by moving only the following
classes outward:

1. stable shared contracts into `xiuxian-wendao-core`
2. generic host behavior into `xiuxian-wendao-runtime`

Do not move domain logic out of `xiuxian-wendao` merely to make package sizes
look balanced. The split is by ownership semantics, not by file count.
