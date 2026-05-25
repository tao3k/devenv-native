# Wendao Core Runtime Plugin Surface Inventory

:PROPERTIES:
:ID: wendao-core-runtime-plugin-surface-inventory
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, migration, plugins, core, runtime, julia, inventory
:STATUS: ACTIVE
:END:

## Mission

This note started as the early `P0` ownership map for the Wendao
`core` / `runtime` / plugin-package migration. It is now maintained as the
active late-`M6` outward-surface inventory and compatibility ledger.

Primary references:

- `[[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md]]`
- `[[docs/rfcs/2026-03-27-wendao-arrow-plugin-flight-rfc.md]]`
- `[[06_roadmap/405_large_rust_modularization]]`
- `[[06_roadmap/410_p1_generic_plugin_contract_staging]]`

The active task-scoped blueprint for this migration remains recorded in the
paired ExecPlan rather than this persistent inventory note.

This document now records the live outward surfaces, the remaining
package-owned compatibility seams, and the additive external-plugin proof
coverage that currently governs the migration.

## Gate Intent

The active alignment bundle requires:

1. an accurate inventory of the live outward surfaces
2. one explicit record of which compatibility seams are still intentionally
   package-owned
3. one outward-facing summary of the late-`M6` additive proof
4. active documentation that no longer describes the tree as early-phase
   extraction startup

The historical ownership map remains below, but the document's primary job is
now to keep late-`M6` outward reality synchronized across the active docs.

## Pure Arrow Flight Boundary Classification (2026-04-01)

The active operator boundary is now intentionally simpler than the older
`Flight-first + Arrow IPC fallback` story.

Formal surface classes:

1. `Arrow Flight` business surface
   - high-performance-first query, retrieval, analysis, docs, planner, repo,
     graph, and VFS business contracts
2. `JSON` control surface
   - process liveness/bootstrap, operator config/status/control, and static
     artifact inspection

Arrow IPC is not a third formal surface in this classification. Flight
business calls still serialize record batches as Arrow IPC stream frames, so
IPC encode/decode helpers remain protocol-intrinsic implementation detail.
The remaining retirement debt is only the standalone IPC business transport
vocabulary such as `ArrowIpcHttp` or `LocalProcessArrowIpc`.

The current Stage-A classification snapshot is:

| Formal class            | Current live family                                                                                                              | Current physical transport reality                                                                         | Migration interpretation                                                                                   |
| :---------------------- | :------------------------------------------------------------------------------------------------------------------------------- | :--------------------------------------------------------------------------------------------------------- | :--------------------------------------------------------------------------------------------------------- |
| `Arrow Flight` business | canonical Flight `/search/*` and `/analysis/*`, plus the existing docs/repo/vfs/graph business families                          | semantic search and analysis are already Flight-only; broader docs/repo/vfs/graph migration remains staged | these families are target business surfaces and should be migrated toward Flight rather than grown as JSON |
| `JSON` control          | `/api/notify`, `/api/ui/capabilities`, `GET /api/ui/plugins/{plugin_id}/artifacts/{artifact_id}`, `GET /api/search/index/status` | JSON is already the current live contract                                                                  | these stay on the control plane unless a later RFC explicitly promotes them into Flight business scope     |

`/api/ui/config` is now retired from that control surface. Studio bootstrap is
backend-loaded from the effective config imports instead of round-tripping a
runtime UI-config write through the live gateway process.

Retirement debt ledger:

1. the old `/api/search/*/hits-arrow` browser route family is retired, and the
   unified `/api/search/{intent,attachments,references,symbols}` HTTP business
   routes are now removed from the outward gateway surface as well
2. the old `/api/analysis/*/retrieval-arrow` mirror family is retired, and the
   old `/api/analysis/markdown` plus `/api/analysis/code-ast` HTTP business
   routes are now removed entirely from the outward gateway surface
3. `ArrowFlight` is now the only active transport vocabulary carried by the
   current runtime-owned plugin contract

Current concrete Flight replacement reality:

1. the runtime query-contract tree already owns
   `REPO_SEARCH_ROUTE = /search/repos/main`
2. the runtime query-contract tree now also owns
   `SEARCH_INTENT_ROUTE`, `SEARCH_KNOWLEDGE_ROUTE`,
   `SEARCH_ATTACHMENTS_ROUTE`, `SEARCH_AST_ROUTE`,
   `SEARCH_REFERENCES_ROUTE`, and `SEARCH_SYMBOLS_ROUTE`
3. the runtime query-contract tree now also owns
   `ANALYSIS_MARKDOWN_ROUTE` and `ANALYSIS_CODE_AST_ROUTE`
4. the runtime query-contract tree already owns `RERANK_ROUTE = /rerank`
5. `/search/repos/main` and `/rerank` already have concrete runtime service
   seams
6. the first bounded semantic search/analysis service-materialization slice
   now exists in `xiuxian-wendao-runtime/src/transport/server.rs` via
   `SearchFlightRouteProvider`, shared
   `WENDAO_SEARCH_QUERY_HEADER` / `WENDAO_SEARCH_LIMIT_HEADER` metadata, and
   `WendaoFlightService::new_with_route_providers(...)`
7. the first concrete Studio-backed provider slice now exists in
   `src/gateway/studio/search/handlers/knowledge/intent/flight.rs` via
   `StudioIntentSearchFlightRouteProvider`, which materializes the semantic
   `/search/intent` route into a Studio-owned Arrow batch shaped like the
   current intent `hits-arrow` payload
8. the semantic search contract is now snapshot-locked in
   `tests/snapshots/gateway/studio/search_flight_service_route_contracts.snap`,
   which records `/search/intent`, `/search/knowledge`,
   `/search/attachments`, `/search/references`, `/search/symbols`, and
   `/search/ast` as the current active Flight search family
9. the old `/api/search/ast` HTTP business route has now been removed
   entirely; `src/gateway/studio/search/handlers/ast.rs` remains only as the
   Studio-owned provider-backed batch seam for the canonical Flight route
   `/search/ast`
10. one bounded host-side aggregate dispatch seam now also exists in
    `src/gateway/studio/search/handlers/flight.rs`, where
    `StudioSearchFlightRouteProvider` multiplexes
    `SEARCH_INTENT_ROUTE`, `SEARCH_KNOWLEDGE_ROUTE`,
    `SEARCH_REFERENCES_ROUTE`, `SEARCH_SYMBOLS_ROUTE`, and `SEARCH_AST_ROUTE`
    onto one shared Studio-owned Flight provider contract
11. the general knowledge materialization seam now also exists in
    `src/gateway/studio/search/handlers/knowledge/search.rs`, where
    `load_knowledge_search_response_flight_batch(...)` reuses the shared
    `SearchHit` Arrow batch shape for the canonical `/search/knowledge`
    contract
12. one bounded host-side service-builder seam now also exists in
    `src/gateway/studio/search/handlers/flight.rs` and
    `src/link_graph/plugin_runtime/transport/server.rs`, where
    `build_studio_search_flight_service_with_repo_provider(...)` and
    `build_search_plane_studio_flight_service(...)` materialize one
    `WendaoFlightService` with repo-search, aggregate semantic search,
    attachment, AST, markdown-analysis, and code-AST-analysis providers
13. active Flight service wiring is now landed for the dedicated
    `wendao_search_flight_server` binary, so the remaining retirement debt is
    fallback/runtime compatibility transport seams rather than missing
    binary-side analysis activation
14. the dedicated `src/bin/wendao_search_flight_server.rs` binary now also
    consumes the roots-based Studio builder through
    `build_search_plane_studio_flight_service_for_roots_with_weights(...)`,
    which makes semantic search-family and analysis-family Flight serving
    active in the dedicated binary without depending on outward search or
    analysis HTTP business-route mirrors
15. that dedicated Flight binary is now browser-consumable too:
    `src/bin/wendao_search_flight_server.rs` enables
    `tonic_web::GrpcWebLayer` with `accept_http1(true)`, so browser-side
    gRPC-web clients can consume the semantic Flight contracts directly
16. `.data/wendao-frontend` now owns a real pure-Flight knowledge-search path:
    `wendao.toml` carries `[search_flight]`, the Rspack dev server proxies
    `/arrow.flight.protocol.FlightService/*`, and
    `src/api/flightSearchTransport.ts` plus the generated Arrow Flight client
    now consume the canonical `/search/knowledge` contract
17. live end-to-end proof now exists in
    `.data/wendao-frontend/src/api/liveGateway.test.ts`, which exercises:
    local config push into the gateway, graph resolution from the gateway, and
    pure Flight knowledge search against the dedicated Flight server
18. the next in-crate retirement slice is now also complete: the dead local
    Arrow IPC helper modules
    `src/gateway/studio/search/handlers/arrow_transport.rs` and
    `src/gateway/studio/search/handlers/knowledge/intent/arrow.rs` are gone,
    and the attachments/references/symbols helper roundtrip tests are retired,
    so the semantic search contract is pinned only by the shared Flight
    snapshot instead of duplicated local IPC helper coverage
19. the next test-graph retirement slice is now also complete: the old
    semantic-search HTTP wrapper shells used only by unit tests are gone, and
    the search unit suite now proves intent/attachments/references/symbols
    through the active response seams directly instead of through retired HTTP
    wrapper functions
20. the first bounded `graph/vfs` Flight business cut is now also landed:
    `VFS_RESOLVE_ROUTE = /vfs/resolve` is runtime-owned in the query contract,
    `VfsResolveFlightRouteProvider` is now part of
    `WendaoFlightService::new_with_route_providers(...)`, the Studio-backed
    owner seam lives in `src/gateway/studio/vfs/flight.rs`, the shared
    workspace Flight snapshot now locks `/vfs/resolve`, and the old
    `/api/vfs/resolve` HTTP business route is removed from the outward router
    and bundled OpenAPI surface
21. the next bounded `graph/vfs` Flight business cut is now also landed:
    `GRAPH_NEIGHBORS_ROUTE = /graph/neighbors` is runtime-owned in the query
    contract, `GraphNeighborsFlightRouteProvider` is now part of
    `WendaoFlightService::new_with_route_providers(...)`, the Studio-backed
    owner seam lives in
    `src/gateway/studio/router/handlers/graph/flight.rs`, the shared
    workspace Flight snapshot now locks `/graph/neighbors`, and the old
    `/api/graph/neighbors/{id}` HTTP business route is removed from the
    outward router and bundled OpenAPI surface
22. the first remaining `graph/vfs` utility retirement slice is now also
    landed: the dead legacy `/api/neighbors/{id}` route is removed from the
    outward router and bundled OpenAPI surface, backend `node_neighbors`
    handler/type residue is deleted, frontend `NodeNeighbors` client residue is
    deleted, and `/graph/neighbors` remains the only live graph-neighbor read
    surface while the later workspace utility cuts for scan/topology were
    still queued as separate bounded slices at that point in the migration
23. the next bounded repo-facing Flight cut is now also landed for code
    content search: `REPO_SEARCH_ROUTE = /repo/search` is no longer pinned to a
    fixed gateway repo id, the runtime request contract now carries
    `x-wendao-repo-search-repo`, and `.data/wendao-frontend` consumes that
    same-origin Flight route through `src/api/flightRepoSearchTransport.ts`
    for the repo-aware `all`/`code` lane while the remaining repo-intelligence
    `doc` facet stays on its existing JSON surface as a later bounded
    retirement slice
24. the next frontend integration cut on top of that route is now also landed:
    the default repo-aware `all` / `code` interface no longer fans out to the
    repo-intelligence `module`, `symbol`, or `example` JSON endpoints when no
    explicit repo facet is active, so the main repo-aware search interface now
    stays on repo-content Flight plus Flight-backed references while explicit
    facet requests remain the only repo-facing JSON retirement debt
25. the next explicit-facet retirement slice is now also landed for
    `symbol`: `.data/wendao-frontend` now reuses repo-search Flight plus
    `tag_filters = ["kind:<value>"]` for repo-aware symbol queries, so the
    repo-intelligence symbol endpoint is no longer on the active
    `kind:function` / `kind:method` / `kind:struct` search path
26. the next explicit-facet retirement slice is now also landed for
    `module`: `.data/wendao-frontend` now reuses repo-search Flight plus
    `tag_filters = ["kind:module"]` for repo-aware module queries, and the
    repo-overview fallback now retries that same Flight route instead of
    calling `/api/repo/module-search`
27. the next explicit-facet retirement slice is now also landed for
    `example`: `.data/wendao-frontend` now reuses repo-search Flight plus
    `tag_filters = ["kind:example"]` for repo-aware example queries, and the
    repo-overview fallback now retries that same Flight route instead of
    calling `/api/repo/example-search`
28. `doc` is now Flight-backed too, but it intentionally stays on the
    dedicated `/analysis/repo-doc-coverage` contract instead of repo-search
    hits; that route carries doc rows in Arrow plus coverage summary metadata
    and the optional `module` filter, retiring the last repo-facing JSON seam
29. the next workspace-preview cut is now also landed for raw content:
    `VFS_CONTENT_ROUTE = /vfs/content` is runtime-owned in the query contract,
    `VfsContentFlightRouteProvider` is now part of
    `WendaoFlightService::new_with_route_providers(...)`, the Studio-backed
    owner seam lives in `src/gateway/studio/vfs/flight_content.rs`, the
    shared workspace Flight snapshot now locks canonical `project/path`
    inputs for both `/vfs/resolve` and `/vfs/content`, and
    `.data/wendao-frontend` now routes `api.getVfsContent(...)` through
    same-origin Flight instead of the older `/api/vfs/cat` helper
30. the next document-command cut is now also landed for refine-doc:
    `ANALYSIS_REFINE_DOC_ROUTE = /analysis/refine-doc` is runtime-owned in the
    shared query contract, `RefineDocFlightRouteProvider` is now part of the
    Studio-backed Flight builder, and `.data/wendao-frontend` now routes
    `api.refineEntityDoc(...)` through `src/api/flightRefineEntityDocTransport.ts`
    instead of the older `/api/repo/refine-entity-doc` helper. After this
    slice, the active frontend search/preview/refine lane is Flight-first and
    the remaining `/api/` debt is concentrated in control-plane and
    non-hot-path utility surfaces.
31. the next frontend utility cleanup is now also landed for VFS scans:
    `.data/wendao-frontend/src/components/panels/VfsSidebar/VfsSidebar.tsx`
    no longer bypasses the API layer with a direct `/api/vfs/scan` fetch and
    now consumes `api.scanVfs()` instead. This does not create a new Flight
    contract, but it centralizes the remaining non-Flight control-plane debt
    under `src/api/*` so production component code no longer owns raw `/api/`
    calls on the active studio surface.
32. the next repo-analysis status cut is now also landed for repo overview:
    `ANALYSIS_REPO_OVERVIEW_ROUTE = /analysis/repo-overview` is runtime-owned
    in the shared query contract, `RepoOverviewFlightRouteProvider` is now
    part of the Studio-backed Flight builder, and `.data/wendao-frontend` now
    routes `api.getRepoOverview(...)` through
    `src/api/flightRepoOverviewTransport.ts` instead of `/api/repo/overview`.
33. the next repo-analysis status cut is now also landed for repo index
    status: `ANALYSIS_REPO_INDEX_STATUS_ROUTE = /analysis/repo-index-status`
    is runtime-owned in the shared query contract,
    `RepoIndexStatusFlightRouteProvider` is now part of the Studio-backed
    Flight builder, and `.data/wendao-frontend` now routes
    `api.getRepoIndexStatus(...)` through
    `src/api/flightRepoIndexStatusTransport.ts` instead of
    `/api/repo/index/status`. After this slice, the remaining active repo-aware
    JSON debt is reduced to repo sync rather than search, preview, refine-doc,
    doc coverage, repo overview, or repo index status.
34. the next repo-analysis status cut is now also landed for repo sync:
    `ANALYSIS_REPO_SYNC_ROUTE = /analysis/repo-sync` is runtime-owned in the
    shared query contract, `RepoSyncFlightRouteProvider` is now part of the
    Studio-backed Flight builder, and `.data/wendao-frontend` now routes
    `api.getRepoSync(...)` through `src/api/flightRepoSyncTransport.ts`
    instead of `/api/repo/sync`. After this slice, the active repo-aware
    frontend lane is Flight-first across search, preview, refine-doc, repo
    overview, repo index status, and repo sync; the remaining `/api/` debt is
    now limited to control-plane and utility routes rather than active
    repo-aware business flows.
35. the next workspace utility cut is now also landed for VFS scan:
    `VFS_SCAN_ROUTE = /vfs/scan` is runtime-owned in the shared query
    contract, `VfsScanFlightRouteProvider` is now part of the Studio-backed
    Flight builder, and `.data/wendao-frontend` now routes `api.scanVfs()`
    through `src/api/flightWorkspaceTransport.ts` instead of the older
    `/api/vfs/scan` helper surface.
36. the next graph utility cut is now also landed for topology:
    `TOPOLOGY_3D_ROUTE = /topology/3d` is runtime-owned in the shared query
    contract, `Topology3dFlightRouteProvider` is now part of the
    Studio-backed Flight builder, and `.data/wendao-frontend` now routes
    `api.get3DTopology()` through `src/api/flightGraphTransport.ts` instead of
    the older `/api/topology/3d` helper surface.
37. the next repo service-state cut is now also landed for repo index
    mutation: `ANALYSIS_REPO_INDEX_ROUTE = /analysis/repo-index` is runtime-owned
    in the shared query contract, `RepoIndexFlightRouteProvider` is now part
    of the Studio-backed Flight builder, and `.data/wendao-frontend` now
    routes `api.enqueueRepoIndex(...)` through
    `src/api/flightRepoIndexTransport.ts` instead of `POST /api/repo/index`.
    The command contract also carries an explicit request id so repeated
    enqueue operations do not reuse cached route payloads. After this slice,
    the remaining active frontend non-Flight surface is intentionally limited
    to control-plane routes such as health, UI config/capabilities, and Julia
    deployment artifact inspection. `.data/wendao-frontend` now groups those
    retained JSON routes under `src/api/controlPlane/*`.

Bounded Stage-A retirement mapping:

| IPC debt family                                                                    | Current physical owner                                                                                                                                      | Concrete Flight replacement today                 | Governed next move                                                                                           |
| :--------------------------------------------------------------------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------- | :------------------------------------------------ | :----------------------------------------------------------------------------------------------------------- |
| retired `/api/search/{intent,attachments,references,symbols}` HTTP business routes | `src/gateway/studio/search/handlers/{attachments,references,symbols}.rs` plus `src/gateway/studio/search/handlers/knowledge/intent/{arrow,entry,flight}.rs` | `/search/{intent,attachments,references,symbols}` | keep only as provider/materialization seams and remove downstream HTTP assumptions                           |
| retired non-Flight plugin transport vocabulary                                     | `xiuxian-wendao-core` transport contract plus runtime/studio mirrors                                                                                        | complete                                          | remove retired variants from the active public type system and keep Flight as the only live plugin transport |

## Classification Rules

This inventory uses the following target owners:

1. `core`
   - stable capability, artifact, schema, and transport contracts
   - no process lifecycle, no language-specific runtime settings
2. `runtime`
   - process launch, transport negotiation, routing, health, fallback, and
     UI-facing host assembly
3. `xiuxian-julia-core`
   - Julia-specific capability declarations, launch details, deployment
     artifacts, and plugin-owned transport defaults

This inventory also uses the following structural rule:

1. medium or complex migration slices must end in a feature folder with
   responsibility-oriented leaf files
2. `mod.rs` remains interface-only
3. compatibility shims may preserve public exports, but implementation must
   move behind the new namespace

## Late-M6 Outward Surface Summary

The current live outward surfaces are:

| Outward family                        | Canonical surface                                                                                              | Current late-`M6` status                                                                                                         | Compatibility note                                                   |
| :------------------------------------ | :------------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------- |
| Generic artifact inspection           | `wendao.plugin_artifact`, `GET /api/ui/plugins/{plugin_id}/artifacts/{artifact_id}`                            | canonical and live                                                                                                               | Julia-named outward artifact surfaces are retired from the host      |
| Studio docs gateway family            | `/api/docs/*`                                                                                                  | external Modelica proof now covers docs-facing search, retrieval, navigation, family, gap-report, and planner peers              | no host-owned language-specific route family remains in this surface |
| Studio repo query/projection family   | `/api/repo/overview`, `/api/repo/*search`, `/api/repo/doc-coverage`, `/api/repo/sync`, `/api/repo/projected-*` | external Modelica proof now covers repo-facing query, projection, reopen, and navigation peers                                   | additive proof stays on generic host surfaces                        |
| Studio repo service-state family      | `/analysis/repo-index`, `/analysis/repo-index-status`                                                          | `Stage A` is complete; the active Studio repo service-state bundle now stays on Flight while JSON compatibility remains optional | bounded local helper reuse only; no dead-code suppressions added     |
| Remaining Julia compatibility imports | `xiuxian_julia_runtime::wendao::link_graph::*`                                                                 | runtime-owned only                                                                                                               | host crate-root compatibility shims are retired                      |

The current late-`M6` outward story is therefore:

1. generic outward artifact inspection is canonical
2. repo-facing, docs-facing, and Studio-facing additive proof now all exist on
   one non-Julia external plugin path
3. the `M6` additive-proof exit condition is now satisfied
4. `Phase 7: Flight-First Runtime Negotiation` is now complete
5. `Phase-8 Stage A: Tooling Reality Inventory Bundle` is now complete
6. `Phase-8 Stage B: Contract and Dependency Policy Bundle` is now complete
7. `Phase-8 Stage C: Lane Integration and Gate Bundle` is now complete
8. the explicit `Phase 8` gate decision is `go`
9. the next governed move is no longer more outward-surface expansion inside
   this inventory; it should move to a new macro-phase proposal or a bounded
   remediation plan for the surfaced advisory findings

## Phase-7 Transport Surface Inventory

The `Phase-7 Stage A` inventory resolves the live transport surface as
follows:

| Surface                              | Current owner                 | Stage-A finding                                                                                                                                                    | Phase-7 implication                                                                   |
| :----------------------------------- | :---------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------ |
| Generic transport contract           | `xiuxian-wendao-core`         | `PluginCapabilityBinding` carries `endpoint`, `transport`, and `contract_version`; `PluginTransportKind` is now `ArrowFlight`-only                                 | the public plugin transport contract is now Flight-only                               |
| Runtime transport construction       | `xiuxian-wendao-runtime`      | `src/transport/flight.rs` and `src/transport/negotiation.rs` materialize `ArrowFlight` directly                                                                    | runtime negotiation is Flight-only on the live materialization path                   |
| Host Arrow response encoding         | `xiuxian-wendao` host gateway | `src/gateway/studio/search/handlers/arrow_transport.rs` only serializes local Arrow payload responses                                                              | this seam is not the negotiation owner and should stay out of transport-policy growth |
| Outward transport inspection payload | `xiuxian-wendao` Studio types | `UiPluginArtifact` now exposes `base_url`, `route`, `health_route`, `timeout_secs`, `schema_version`, `selected_transport`, `fallback_from`, and `fallback_reason` | `Stage C` is complete for the outward inspection family                               |

The canonical Phase-7 runtime negotiation order is now fixed as:

1. `ArrowFlight`

Interpretation:

1. `ArrowFlight` is the preferred runtime negotiation result where a provider
   and host path can support it
2. the retired non-Flight transport variants are no longer part of the active
   public plugin transport contract

Current Stage-B implementation status:

1. the runtime-owned negotiation policy now lives in
   `packages/rust/crates/xiuxian-wendao-runtime/src/transport/negotiation.rs`
2. the current rerank transport path now delegates through that negotiation
   seam instead of directly materializing an Arrow IPC client
3. the runtime now also owns a real Flight client seam in
   `packages/rust/crates/xiuxian-wendao-runtime/src/transport/flight.rs`
4. that Flight client intentionally uses `arrow-flight = 57.3.0` so the data
   plane stays aligned with the LanceDB Arrow line already present in
   `lance-arrow`
5. the host-side rerank path stays on the current engine Arrow line by
   reusing `xiuxian-vector`'s existing
   `engine_batches_to_lance_batches(...)` and
   `lance_batches_to_engine_batches(...)` conversion seam
6. when generic bindings advertise retired transport kinds, the runtime now
   rejects them explicitly and keeps the live materialization path Flight-only
7. `Phase-7 Stage B` is complete: runtime policy and Flight materialization
   now live on the LanceDB Arrow line
8. `Phase-7 Stage C` is complete: outward inspection payloads and rerank
   diagnostics now surface negotiated transport and fallback decisions
9. the explicit `Phase 7` gate decision is `go`

## Phase-8 Contract and Dependency Policy Snapshot

`Phase-8 Stage B` fixes the first governance boundaries as:

1. workspace security remains one blocking lane family through the existing
   `cargo-audit` and `cargo-deny` baseline
2. the current `cargo-deny` contract remains pragmatic:
   advisories, bans, and sources are blocking, while duplicate-version
   findings stay warn-only and license gating stays out of the initial
   bounded rollout
3. semver governance is now explicitly owned by `xiuxian-wendao-core`; the
   initial blocking semver lane should target that crate only
4. `xiuxian-wendao`, `xiuxian-wendao-runtime`, `xiuxian-julia-core`, and
   `xiuxian-wendao-modelica` are not part of the initial semver gate because
   their public surfaces are still migration-owned
5. dependency-hygiene rollout stays advisory-first:
   `cargo-machete` should target the Wendao migration cluster, and
   `cargo-udeps` should start with `xiuxian-wendao-core` plus
   `xiuxian-wendao-runtime`

Current Stage-C status:

1. the bounded governance bundle is now physically wired through `justfile`,
   `nix/modules/rust.nix`, `nix/modules/tasks.nix`, and the main Rust CI
   workflows
2. the blocking semver lane for `xiuxian-wendao-core` is live and passed on
   the current tree
3. the advisory `cargo-machete` lane is live and now passes cleanly on the
   current Wendao migration cluster after removing the stale
   `xiuxian-wendao` dependency entries
4. the advisory `cargo-udeps` lane is wired with bounded skip semantics when
   `rustup` nightly is unavailable in the current Nix-managed environment
5. follow-on bounded sibling remediations removed the stale
   `xiuxian-llm` direct dependencies `rmcp`, `fast_image_resize`, and
   `spider_agent`, then retired the dead `xiuxian-zhenfa`
   `tests/support/gateway.rs` helper; `toml` and `xiuxian-config-core`
   stay declared in `xiuxian-llm` as explicit `cargo-machete`-ignored macro
   dependencies, and live `rmcp` ownership is now concentrated in
   `xiuxian-daochang`
6. a further bounded contraction moved `xiuxian-daochang` production callers
   off `rmcp` response/request model types, so direct `rmcp` usage in
   `src/` is now narrowed to `tool_runtime/bridge.rs`; full
   `xiuxian-daochang` crate verification remains blocked by unrelated
   pre-existing module/export failures outside the touched seam
7. the final bounded production-side replacement moved
   `xiuxian-daochang/src/tool_runtime/bridge.rs` off `rmcp` entirely by
   landing a self-owned streamable-HTTP JSON-RPC client for `initialize`,
   `tools/list`, and `tools/call`, and by moving `rmcp` to
   `dev-dependencies` for the test-side server harness only; direct `rmcp`
   usage no longer exists in `xiuxian-daochang/src/`
8. the bounded follow-on advisory cleanup has now removed the stale
   `xiuxian-daochang` direct dependencies `comrak` and `regex`, so the
   package is clean under the current `cargo-machete` lane even though full
   crate verification is still blocked by unrelated pre-existing failures
9. the next bounded crate-health slice has repaired the stale
   `system_prompt_injection_state` owner drift in `xiuxian-daochang`, so the
   compile front now advances past that session prompt-injection seam and
   stops in other pre-existing import/export clusters instead
10. the following bounded crate-health slice has repaired the
    `runtime_agent_factory` test-support visibility seam, so that private
    helper-import cluster no longer appears at the compile front either
11. the next bounded Discord crate-health slice has removed the dead mounted
    `channels/discord/channel/constructors.rs` duplicate, so that stale
    `omni_agent` / `ChannelCommandRequest` constructor drift is no longer
    carried inside the live Discord channel module tree
12. the following bounded channel-runtime slice has restored the missing
    `nodes/channel/common.rs` embedding-memory guard helper, so the live
    `nodes/channel/{discord,telegram}.rs` launch paths no longer depend on a
    nonexistent shared leaf and the compile front now advances into deeper
    Discord runtime dispatch and Telegram channel wiring drift
13. the next bounded Discord runtime slice has now repaired the stale
    `channels/discord/runtime/dispatch/` surface by restoring child-module
    wiring, a local `dispatch/support.rs` logging/preemption leaf, the
    `process_discord_message(...)` compatibility wrapper, crate-internal
    `ForegroundInterruptController` visibility, and the shared
    `compose_turn_content(...)` helper in `channels/managed_runtime/turn.rs`;
    that Discord dispatch cluster no longer appears at the compile front,
    which now starts deeper in Telegram runtime surface drift and other
    pre-existing export failures
14. the following bounded channel path-normalization slice has now repaired
    the remaining live Telegram channel/runtime owner drift by mounting the
    existing `channel/outbound_text.rs` leaf, re-exporting the
    `jobs::{JobRecord, QueuedJob, epoch_millis}` owner seam, switching the
    touched imports from `super::super::...` to `crate::...`, and retiring
    dead duplicate Discord managed-session `admin` / `injection` leaves plus
    the dead Telegram `session_context` duplicate; the
    `xiuxian-daochang` compile front no longer stops in that channel family
    and now exits the crate at a pre-existing `xiuxian-wendao` transport
    import failure
15. the next bounded runtime transport slice has now repaired that
    `xiuxian-wendao` import failure by restoring the missing
    `RepoSearchFlightRouteProvider` re-export in
    `xiuxian-wendao-runtime/src/transport/mod.rs`; the compile front no
    longer exits `xiuxian-daochang` at that transport seam
16. the following bounded helper slice has now repaired the
    `xiuxian-daochang` `jobs/heartbeat` owner import and the initial
    embedding helper drift by replacing the dead `env_first_non_empty`
    macro dependency with a local env scan and by making the currently
    unwired `mistral_sdk` embedding path fail explicitly instead of failing
    the build; the current compile front now starts deeper in `llm/*`,
    `resolve.rs`, and `runtime_agent_factory/*`
17. the following bounded crate-health slice has now repaired the
    `resolve.rs` plus `runtime_agent_factory/*` owner drift by moving the
    channel runtime enums into a crate-owned `channel_runtime.rs` seam,
    rebinding factory imports to crate-owned settings/config owners,
    restoring the missing env-parse helpers, and replacing touched relative
    paths with `crate::...`; the current compile front now starts deeper in
    `session/*`, `test_support/*`, `lib.rs`, and `agent/*` private-module
    exposure consumed by `test_support/*`
18. the following bounded crate-health slice has now repaired the
    `session/*` owner drift by restoring the missing local `SessionStore`,
    switching the live `TurnSlot` owner back to `xiuxian_window`, fixing the
    `RedisSessionBackend` message-content snapshot field, and rebinding the
    touched session tests to `xiuxian_daochang`; the current compile front
    now starts deeper in `test_support/*`, `lib.rs`, and `agent/*`
    private-module exposure consumed by `test_support/*`
19. the following bounded crate-health slice has now repaired the
    `Agent` / session-alignment bundle by restoring the live `Agent`
    owner fields, wiring `agent/bootstrap/memory.rs` back onto the current
    crate-owned tool/admission/embedding seams, restoring the recall-bias and
    session-backup wrappers, and rebinding the touched imports to
    `crate::...`; the current compile front now starts in
    `agent/tool_startup.rs`, `session/redis_backend/executor.rs`, and
    `test_support/memory_metrics.rs` before deeper channel/runtime drift
20. the following bounded crate-health slice has now repaired that next
    compile-front bundle too by cloning the startup connect config at the
    tool-pool call boundary, aligning
    `session/redis_backend/executor.rs` to the current `RwLock` connection
    owner, and moving the memory-recall metric helper methods onto the live
    `agent/memory_recall_metrics.rs` owner seam while retiring the dead
    `test_support/memory_metrics.rs` shim; the current compile front now
    starts deeper in
    `channels/discord/runtime/managed/handlers/command_dispatch/session/budget.rs`,
    `tool_runtime/bridge.rs`, `llm/*`, and the remaining Discord/Telegram
    runtime drift
21. the following bounded crate-health slice has now repaired that next
    compile-front bundle too by rebinding the touched Discord budget handler
    to `crate::...` owner imports and reference-based snapshot formatting,
    aligning `tool_runtime/bridge.rs` to the live `reqwest` response seam
    and touched JSON-RPC/result ownership while retaining connect
    diagnostics, and restoring the touched `llm/*` surface to the live
    `max_tokens` plus `DeepseekRuntime::RemoteHttp` shape; the current
    compile front now starts deeper in Discord runtime gateway/run ownership
    and Telegram send/runtime drift
22. the following bounded crate-health slice has now repaired that
    Discord/Telegram runtime bundle too by restoring the live Discord
    channel constructor owner, borrowing the live gateway join handle,
    wiring the current admission snapshot into gateway/run telemetry,
    restoring Telegram send-rate gate plus chunk-send helpers, moving
    `SessionGate` acquire/drop ownership onto the shared type seam, and
    rebasing the touched runtime/router files from `super::super::...` to
    crate-owned imports; the current compile front no longer stops in
    Discord runtime gateway/run ownership or Telegram send/runtime drift and
    now starts deeper in `gateway/http/*`, `agent/injection/*`,
    `agent/native_tools/zhixing.rs`, `agent/zhenfa/bridge.rs`, Telegram
    ACL/settings, Telegram session-memory reply shaping, and the remaining
    test-support seams

Scope correction:

1. the `xiuxian-daochang` crate-health notes above are retained as workspace
   history only
2. they are not the authoritative next move for this Wendao outward-surface
   inventory unless they directly unblock a Wendao-owned host surface
3. the authoritative next in-program move is now `Phase 9 Stage A`:
   inventory the remaining live consumers that still depend on monolith-era
   `xiuxian-wendao` owner seams instead of `xiuxian-wendao-core` or
   `xiuxian-wendao-runtime`

## Historical Ownership Inventory

The table below remains as the resolved extraction-era ownership map that got
the program to the current late-`M6` shape.

| Current surface                                                                                                                                                                                              | Current path                                                                                                                                                         | Current problem                                                                                                                                                                                                           | Target owner                                                      | Target namespace                                                                                                                                             | Planned phase    |
| :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | :---------------------------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------- | :--------------- |
| Julia-specific runtime config records such as `LinkGraphJuliaRerankRuntimeConfig`, `LinkGraphJuliaAnalyzerServiceDescriptor`, `LinkGraphJuliaAnalyzerLaunchManifest`, and `LinkGraphJuliaDeploymentArtifact` | `src/link_graph/runtime_config/models.rs`                                                                                                                            | Stable runtime config is encoded as Julia-only types inside the host                                                                                                                                                      | `runtime` and `xiuxian-julia-core`                                | `runtime/runtime_config/providers/`, `runtime/artifacts/`, `xiuxian-julia-core/capabilities/`, `xiuxian-julia-core/artifacts/`, `xiuxian-julia-core/launch/` | `P1`, `P3`, `P4` |
| `link_graph.retrieval.julia_rerank` host config path                                                                                                                                                         | `src/link_graph/runtime_config/models.rs`, `src/link_graph/runtime_config/resolve/policy.rs`                                                                         | Provider identity is hardcoded into the config shape, which blocks generic capability routing                                                                                                                             | `runtime`                                                         | `runtime/runtime_config/capabilities/`, `runtime/negotiation/`                                                                                               | `P1`, `P3`       |
| Julia-specific environment variables and launcher defaults such as `XIUXIAN_WENDAO_LINK_GRAPH_JULIA_RERANK_*` and `.data/WendaoAnalyzer/scripts/run_analyzer_service.sh`                                     | `src/link_graph/runtime_config/constants.rs`                                                                                                                         | Host runtime defaults are language-scoped rather than provider-scoped                                                                                                                                                     | `runtime` with Julia-owned defaults in plugin package             | `runtime/runtime_config/providers/`, `runtime/launch/`, `xiuxian-julia-core/manifest/`, `xiuxian-julia-core/launch/`                                         | `P1`, `P3`, `P4` |
| Builtin Julia registration in host bootstrap                                                                                                                                                                 | `src/analyzers/service/bootstrap.rs`                                                                                                                                 | The host owns Julia plugin assembly directly instead of loading a package-defined provider                                                                                                                                | `runtime` and `xiuxian-julia-core`                                | `runtime/registry/`, `runtime/discovery/`, `xiuxian-julia-core/entry/`                                                                                       | `P3`, `P4`       |
| Former sibling-source inclusion hacks for package-local plugin code                                                                                                                                          | `src/analyzers/languages/mod.rs`                                                                                                                                     | Julia and Modelica previously entered the host through `#[path]`; the current tree now uses normal crate dependencies for both, so this row remains only as a resolved retirement checkpoint for future plugin onboarding | resolved for Julia and Modelica                                   | `xiuxian-julia-core/plugin/`, `xiuxian-wendao-modelica/plugin/`, and package dependency registration instead of source inclusion                             | `P4`, `P6`       |
| Julia-specific rerank planning and transport helpers                                                                                                                                                         | `src/link_graph/index/search/plan/payload/quantum.rs`                                                                                                                | Capability execution path is hardcoded to Julia rather than routed through a generic provider binding                                                                                                                     | `runtime` with Julia-specific transport details in plugin package | `runtime/capabilities/rerank/`, `runtime/transport/`, `runtime/negotiation/`, `xiuxian-julia-core/transport/`, `xiuxian-julia-core/capabilities/rerank/`     | `P1`, `P3`, `P4` |
| Julia-specific request-batch builder names in ignition helpers                                                                                                                                               | `src/link_graph/index/search/quantum_fusion/openai_ignition.rs`, `src/link_graph/index/search/quantum_fusion/vector_ignition.rs`                                     | Shared preparation logic is named after one provider even though the long-term host contract is capability-oriented                                                                                                       | `runtime`                                                         | `runtime/capabilities/rerank/request/`                                                                                                                       | `P1`, `P3`       |
| Link-graph public re-exports of Julia-specific types                                                                                                                                                         | `src/link_graph/mod.rs`, `src/link_graph/runtime_config.rs`                                                                                                          | The link-graph domain surface leaks one plugin provider as core vocabulary                                                                                                                                                | `core` compatibility shim plus `runtime` implementation           | `core/capabilities/`, `runtime/capabilities/`, `runtime/artifacts/`                                                                                          | `P1`, `P2`, `P5` |
| Julia-specific test fixtures and planned integration tests                                                                                                                                                   | `tests/integration/planned_search_julia_rerank*.rs`, `packages/rust/crates/xiuxian-julia-core/src/integration_support/`, `src/gateway/studio/router/tests/config.rs` | The WendaoSearch service-fixture launch helpers are now plugin-owned, but the broader integration-test topology still spans host and plugin crates and should keep moving toward plugin-owned or runtime-owned seams      | split across `runtime` and `xiuxian-julia-core`                   | `runtime/tests/capabilities/rerank/`, `runtime/tests/artifacts/`, `xiuxian-julia-core/tests/launch/`, `xiuxian-julia-core/tests/artifacts/`                  | `P3`, `P4`, `P5` |

## Immediate Ownership Decisions

The current inventory resolves the previously ambiguous boundaries as follows:

1. `core` keeps only generic capability, artifact, and schema contracts.
2. `runtime` owns every host behavior that launches, negotiates with, routes
   to, or renders plugin providers.
3. `xiuxian-julia-core` owns Julia-specific launch metadata, deployment
   artifact payload shape, transport defaults, and capability declarations.
4. temporary Julia-named public exports may remain only as compatibility
   shims while the generic runtime surface becomes authoritative.

## Structural Namespace Targets

The first stable namespace targets for migration are:

```text
xiuxian-wendao-core
  capabilities/
  artifacts/
  transport/
  schemas/

xiuxian-wendao-runtime
  capabilities/
    rerank/
  artifacts/
    resolve/
    render/
  runtime_config/
    capabilities/
    providers/
  transport/
  negotiation/
  registry/
  discovery/
  launch/
  health/
  telemetry/
  gateway/
    studio/
      router/
        handlers/
          plugin_artifacts/
      types/
        artifacts/

xiuxian-julia-core
  plugin/
  capabilities/
    rerank/
  artifacts/
  launch/
  manifest/
  transport/
  tests/
```

Every touched medium or complex slice must land in one of these
responsibility-oriented folders rather than in a new flat host file.

## Compatibility Rules

During migration, the following compatibility rules apply:

1. legacy Julia-named public exports may remain temporarily if they delegate to
   the new generic owner
2. new implementation logic must not be added behind the legacy Julia-named
   facade
3. new plugin providers must use the generic capability and artifact surfaces
   rather than copying the Julia naming pattern

Current live status note:

- Julia-owned launch/deployment DTO meaning and selector ownership now live in
  `packages/rust/crates/xiuxian-julia-core/src/compatibility/link_graph/`,
  so the host `launch.rs` and `artifact.rs` files now behave as compatibility
  re-export seams instead of owning those records directly
- the same Julia compatibility folder now also owns
  `LinkGraphJuliaAnalyzerServiceDescriptor` and the Julia analyzer-launch
  CLI-arg mapping, along with the default Julia analyzer launcher path, so
  the remaining host ownership had been narrowed to
  `LinkGraphJuliaRerankRuntimeConfig` plus package-path/default ownership
- the Julia package-path/default seam now lives in
  `packages/rust/crates/xiuxian-julia-core/src/compatibility/link_graph/paths.rs`,
  which owns the default analyzer package dir, launcher path, and example
  config path; the touched host runtime/tests and integration fixtures now
  consume those Julia-owned constants instead of embedding raw
  `.data/WendaoAnalyzer/...` literals
- the Julia rerank runtime-record seam now also lives in
  `packages/rust/crates/xiuxian-julia-core/src/compatibility/link_graph/runtime.rs`,
  which owns `LinkGraphJuliaRerankRuntimeConfig` and its provider-binding /
  launch / artifact normalization helpers; the host `runtime.rs` and
  `conversions.rs` files now serve only as compatibility wrappers
- the `M5` outward artifact cutover is now complete on the Studio/OpenAPI
  side: `UiPluginArtifact` is the primary payload, the canonical schema
  export is generic-only, and
  `/api/ui/plugins/{plugin_id}/artifacts/{artifact_id}` is now the only live
  Studio/OpenAPI UI artifact endpoint
- the old Studio Julia compatibility path is now fully retired from code:
  `UiJuliaDeploymentArtifact`, the dedicated compatibility type leaf, the
  route-local compat adapter, `JuliaDeploymentArtifactQuery`,
  `get_julia_deployment_artifact`, and
  `GET /api/ui/julia-deployment-artifact` are all gone from the live tree
- the OpenAPI Julia route-path aliases are now retired from code too:
  `API_UI_JULIA_DEPLOYMENT_ARTIFACT_*` are gone, and the route inventory now
  validates only the canonical plugin-artifact path
- the `M5` outward artifact cutover is now complete on the Zhenfa side too:
  `wendao.plugin_artifact` is the only live tool/RPC artifact surface, and
  the former Julia outward tool name, compat-specific tool/RPC path, native
  compatibility helper folder, and Julia helper/type aliases are all retired
  from code
- the crate-root and `runtime_config` top-level Julia-named DTO/helper exports
  are now retired too: those names live only under the explicit compatibility
  namespaces instead of leaking through flat public re-export blocks
- the former host crate-root compatibility shim is now retired from code too:
  `src/compatibility/julia.rs`, `src/compatibility/link_graph.rs`, and the
  `pub mod compatibility;` mount in `src/lib.rs` are all gone, so the touched
  internal consumers now read Julia compatibility imports directly from
  `packages/rust/crates/xiuxian-julia-core/src/compatibility/link_graph/`
- the first `M6` additive-plugin proof is now landed too:
  `xiuxian-wendao-modelica` consumes
  `xiuxian-wendao-core::repo_intelligence` for production contracts, the host
  loads Modelica through a normal optional crate dependency, and Modelica
  keeps `xiuxian-wendao` only as a dev-dependency for registry-aware
  integration-query validation
- that same `M6` slice now has a host-side proof too:
  `packages/rust/crates/xiuxian-wendao/tests/integration/repo_overview.rs`
  now exercises the external Modelica plugin through the builtin registry and
  the shared repo-overview/module-search/example-search entry points
- that `M6` host proof is now deeper than the first search slice:
  `repo_symbol_search.rs`, `repo_relations.rs`, and `repo_projected_page.rs`
  now also exercise the external Modelica plugin through builtin-registry
  symbol-search, relation-graph, and projected-page lookup consumers
- that same `M6` host proof now reaches parsed page hierarchy too:
  `repo_projected_page_index_tree.rs` exercises the external Modelica plugin
  through builtin-registry projected page-index tree generation and lookup
- that same `M6` host proof now reaches stable node addressing too:
  `repo_projected_page_index_node.rs` exercises the external Modelica plugin
  through builtin-registry projected page-index node lookup
- that same `M6` host proof now reaches assembled navigation too:
  `repo_projected_page_navigation.rs` exercises the external Modelica plugin
  through builtin-registry projected page navigation bundles
- that same `M6` host proof now reaches grouped family context too:
  `repo_projected_page_family_context.rs` exercises the external Modelica
  plugin through builtin-registry projected page-family context lookup
- that same `M6` host proof now reaches singular family-cluster lookup too:
  `repo_projected_page_family_cluster.rs` exercises the external Modelica
  plugin through builtin-registry projected page-family cluster lookup
- that same `M6` host proof now reaches search-driven family expansion too:
  `repo_projected_page_family_search.rs` exercises the external Modelica
  plugin through builtin-registry projected page-family search
- that same `M6` host proof now reaches search-driven navigation expansion too:
  `repo_projected_page_navigation_search.rs` exercises the external Modelica
  plugin through builtin-registry projected page-navigation search
- that same `M6` additive slice now reaches a docs-facing search consumer too:
  `docs_navigation_search.rs` exercises the external Modelica plugin through
  builtin-registry docs-facing projected page-navigation search
- that same `M6` additive slice now reaches the docs-facing family-search
  peer too: `docs_family_search.rs` exercises the external Modelica plugin
  through builtin-registry docs-facing projected page-family search
- that same `M6` additive slice now reaches the docs-facing family-context
  peer too: `docs_family_context.rs` exercises the external Modelica plugin
  through builtin-registry docs-facing projected page-family context
- that same `M6` additive slice now reaches the docs-facing navigation lookup
  peer too: `docs_navigation.rs` exercises the external Modelica plugin
  through builtin-registry docs-facing projected page navigation lookup
- that same `M6` additive slice now reaches the docs-facing family-cluster
  peer too: `docs_family_cluster.rs` exercises the external Modelica plugin
  through builtin-registry docs-facing projected page-family cluster lookup
- that same `M6` additive slice now reaches the docs-facing page lookup peer
  too: `docs_page.rs` exercises the external Modelica plugin through
  builtin-registry docs-facing projected page lookup
- that same `M6` additive slice now reaches the docs-facing page-index tree
  lookup peer too: `docs_page_index_tree.rs` exercises the external Modelica
  plugin through builtin-registry docs-facing projected page-index tree
  lookup
- that same `M6` additive slice now reaches the docs-facing page-index node
  lookup peer too: `docs_page_index_node.rs` exercises the external Modelica
  plugin through builtin-registry docs-facing projected page-index node
  lookup
- that same `M6` additive slice now reaches the docs-facing page-index tree
  search peer too: `docs_page_index_tree_search.rs` exercises the external
  Modelica plugin through builtin-registry docs-facing projected page-index
  tree search
- that same `M6` additive slice now reaches the docs-facing page-index trees
  peer too: `docs_page_index_trees.rs` exercises the external Modelica plugin
  through builtin-registry docs-facing projected page-index tree listing
- that same `M6` additive slice now reaches the docs-facing page-index
  documents peer too: `docs_page_index_documents.rs` exercises the external
  Modelica plugin through builtin-registry docs-facing projected page-index
  document generation
- that same `M6` additive slice now reaches the docs-facing markdown
  documents peer too: `docs_markdown_documents.rs` exercises the external
  Modelica plugin through builtin-registry docs-facing projected markdown
  document generation
- that same `M6` additive slice now reaches the docs-facing search peer too:
  `docs_search.rs` exercises the external Modelica plugin through
  builtin-registry docs-facing projected page search
- that same `M6` additive slice now reaches the docs-facing retrieval peer
  too: `docs_retrieval.rs` exercises the external Modelica plugin through
  builtin-registry docs-facing mixed projected retrieval
- that same `M6` additive slice now reaches the docs-facing retrieval-
  context peer too: `docs_retrieval_context.rs` exercises the external
  Modelica plugin through builtin-registry docs-facing local projected
  retrieval context
- that same `M6` additive slice now reaches the docs-facing retrieval-hit
  peer too: `docs_retrieval_hit.rs` exercises the external Modelica plugin
  through builtin-registry docs-facing deterministic projected retrieval-hit
  reopening
- that same `M6` additive slice now reaches the docs-facing projected-gap
  report peer too: `docs_projected_gap_report.rs` exercises the external
  Modelica plugin through builtin-registry docs-facing projected gap
  reporting
- that same `M6` additive slice now reaches the docs-facing planner-queue
  peer too: `docs_planner_queue.rs` exercises the external Modelica plugin
  through builtin-registry docs-facing deterministic planner queue shaping
- that same `M6` additive slice now reaches the docs-facing planner-workset
  peer too: `docs_planner_workset.rs` exercises the external Modelica plugin
  through builtin-registry docs-facing deterministic planner workset shaping
- that same `M6` additive slice now reaches the docs-facing planner-rank
  peer too: `docs_planner_rank.rs` exercises the external Modelica plugin
  through builtin-registry docs-facing deterministic planner ranking
- that same `M6` additive slice now reaches the docs-facing planner-item
  peer too: `docs_planner_item.rs` exercises the external Modelica plugin
  through builtin-registry docs-facing deterministic planner item reopening
- that same `M6` additive slice now reaches the docs-facing planner-search
  peer too: `docs_planner_search.rs` exercises the external Modelica plugin
  through builtin-registry docs-facing deterministic planner search
- that same `M6` additive slice now reaches the Studio docs route layer too:
  `tests/unit/studio_repo_sync_api.rs` exercises `/api/docs/planner-search`
  through the builtin-registry external Modelica path, so the additive proof
  now covers one real gateway consumer above the analyzer entrypoint
- that same `M6` additive slice now reaches a second Studio docs route peer
  too: `tests/unit/studio_repo_sync_api.rs` also exercises
  `/api/docs/planner-item` through the builtin-registry external Modelica
  path, so the gateway-layer additive proof now covers deterministic planner
  gap reopening as well as planner search
- that same `M6` additive slice now reaches a third Studio docs route peer
  too: `tests/unit/studio_repo_sync_api.rs` also exercises
  `/api/docs/planner-workset` through the builtin-registry external Modelica
  path, so the gateway-layer additive proof now covers deterministic planner
  workset shaping as well as planner search and planner-item reopening
- that same `M6` additive slice now reaches a fourth Studio docs route peer
  too: `tests/unit/studio_repo_sync_api.rs` also exercises
  `/api/docs/planner-rank` through the builtin-registry external Modelica
  path, so the gateway-layer additive proof now covers deterministic planner
  ranking as well as planner search, planner-item reopening, and
  planner-workset shaping
- that same `M6` additive slice now reaches a fifth Studio docs route peer
  too: `tests/unit/studio_repo_sync_api.rs` also exercises
  `/api/docs/planner-queue` through the builtin-registry external Modelica
  path, so the gateway-layer additive proof now covers deterministic planner
  queue shaping as well as planner search, planner-item reopening,
  planner-workset shaping, and planner ranking
- that same `M6` additive slice now exits the Studio planner subtree too:
  `tests/unit/studio_repo_sync_api.rs` also exercises `/api/docs/search`
  through the builtin-registry external Modelica path, so the gateway-layer
  additive proof now reaches the first non-planner docs-facing route family
  as well
- that same `M6` additive slice now extends the non-planner Studio docs
  route family too: `tests/unit/studio_repo_sync_api.rs` also exercises
  `/api/docs/retrieval` through the builtin-registry external Modelica path,
  so the gateway-layer additive proof now covers mixed docs-facing retrieval
  as well as plain docs search
- that same `M6` additive slice now pushes deeper into the non-planner
  Studio docs route family too: `tests/unit/studio_repo_sync_api.rs` now
  also exercises `/api/docs/retrieval-context` through the builtin-registry
  external Modelica path, so the gateway-layer additive proof now covers
  deterministic node-context reopening as well
- that same `M6` additive slice now closes the sibling deterministic
  reopening peer too: `tests/unit/studio_repo_sync_api.rs` now also
  exercises `/api/docs/retrieval-hit` through the builtin-registry external
  Modelica path, so the gateway-layer additive proof now covers
  deterministic hit reopening as well
- that same `M6` additive slice now closes the deterministic page-lookup
  peer too: `tests/unit/studio_repo_sync_api.rs` now also exercises
  `/api/docs/page` through the builtin-registry external Modelica path, so
  the gateway-layer additive proof now covers deterministic docs page
  lookup as well
- that same `M6` additive slice now closes the deterministic family-context
  peer too: `tests/unit/studio_repo_sync_api.rs` now also exercises
  `/api/docs/family-context` through the builtin-registry external Modelica
  path, so the gateway-layer additive proof now covers grouped family
  context reopening as well
- that same `M6` additive slice now closes the deterministic family-search
  peer too: `tests/unit/studio_repo_sync_api.rs` now also exercises
  `/api/docs/family-search` through the builtin-registry external Modelica
  path, so the gateway-layer additive proof now covers grouped family
  search expansion as well
- that same `M6` additive slice now closes the deterministic family-cluster
  peer too: `tests/unit/studio_repo_sync_api.rs` now also exercises
  `/api/docs/family-cluster` through the builtin-registry external Modelica
  path, so the gateway-layer additive proof now covers single-family
  reopening as well
- that same `M6` additive slice now closes the deterministic navigation peer
  too: `tests/unit/studio_repo_sync_api.rs` now also exercises
  `/api/docs/navigation` through the builtin-registry external Modelica
  path, so the gateway-layer additive proof now covers tree-context plus
  family-cluster reopening as well
- that same `M6` additive slice now closes the deterministic navigation-
  search peer too: `tests/unit/studio_repo_sync_api.rs` now also exercises
  `/api/docs/navigation-search` through the builtin-registry external
  Modelica path, so the gateway-layer additive proof now covers grouped
  navigation-bundle expansion as well
- that same `M6` additive slice now closes the docs projected-gap-report
  peer too: `tests/unit/studio_repo_sync_api.rs` now also exercises
  `/api/docs/projected-gap-report` through the builtin-registry external
  Modelica path, so the gateway-layer additive proof now covers docs-facing
  gap reporting as well
- that same `M6` additive slice now opens the sibling Studio repo route
  family too: `tests/unit/studio_repo_sync_api.rs` now also exercises
  `/api/repo/overview` through the builtin-registry external Modelica path,
  so the gateway-layer additive proof now covers repo-summary reopening
  outside the docs-only surface too
- that same `M6` additive slice now closes the sibling Studio repo module-
  search peer too: `tests/unit/studio_repo_sync_api.rs` now also exercises
  `/api/repo/module-search` through the builtin-registry external Modelica
  path, so the gateway-layer additive proof now covers deterministic module
  search outside the docs-only surface too
- that same `M6` additive slice now closes the sibling Studio repo symbol-
  search peer too: `tests/unit/studio_repo_sync_api.rs` now also exercises
  `/api/repo/symbol-search` through the builtin-registry external Modelica
  path, so the gateway-layer additive proof now covers deterministic symbol
  search outside the docs-only surface too
- that same `M6` additive slice now closes the sibling Studio repo example-
  search peer too: `tests/unit/studio_repo_sync_api.rs` now also exercises
  `/api/repo/example-search` through the builtin-registry external Modelica
  path, so the gateway-layer additive proof now covers deterministic example
  search outside the docs-only surface too
- that same `M6` additive slice now closes the sibling Studio repo doc-
  coverage peer too: `tests/unit/studio_repo_sync_api.rs` now also exercises
  `/api/repo/doc-coverage` through the builtin-registry external Modelica
  path, so the gateway-layer additive proof now covers deterministic
  module-scoped doc coverage outside the docs-only surface too
- that same `M6` additive slice now closes a bundled Studio repo lifecycle-
  and-projection batch too: `tests/unit/studio_repo_sync_api.rs` now also
  exercises `/api/repo/sync`, `/api/repo/projected-pages`, and
  `/api/repo/projected-gap-report` through the builtin-registry external
  Modelica path, so the gateway-layer additive proof now covers repo
  status reopening, projected-page enumeration, and projected-gap reporting
  outside the docs-only surface too
- that same `M6` additive slice now closes a bundled deterministic Studio
  repo projected reopen family too: `tests/unit/studio_repo_sync_api.rs`
  now also exercises `/api/repo/projected-page`,
  `/api/repo/projected-page-index-tree`, `/api/repo/projected-page-index-node`,
  `/api/repo/projected-retrieval-hit`, and
  `/api/repo/projected-retrieval-context` through the builtin-registry
  external Modelica path, so the gateway-layer additive proof now covers
  symbol-page reopening, tree reopening, node reopening, deterministic hit
  reopening, and node-context reopening outside the docs-only surface too
- that same `M6` additive slice now closes the remaining bundled Studio repo
  projected query-and-navigation family too: `tests/unit/studio_repo_sync_api.rs`
  now also exercises `/api/repo/projected-page-index-tree-search`,
  `/api/repo/projected-page-search`, `/api/repo/projected-retrieval`,
  `/api/repo/projected-page-family-context`,
  `/api/repo/projected-page-family-search`,
  `/api/repo/projected-page-family-cluster`,
  `/api/repo/projected-page-navigation`,
  `/api/repo/projected-page-navigation-search`, and
  `/api/repo/projected-page-index-trees` through the builtin-registry
  external Modelica path, so the gateway-layer additive proof now also
  covers deterministic section search, projected page search, mixed
  projected retrieval, grouped family reopening, single-family reopening,
  navigation-bundle reopening, navigation-search expansion, and projected
  tree listing outside the docs-only surface too

## Current Compatibility Ledger

The current live tree now has a narrower set of Julia-named outward surfaces.
These should be treated as an explicit compatibility ledger rather than as
default host vocabulary.

### Legacy compatibility surfaces that still remain intentionally

No host-owned outward compatibility route, tool surface, or crate-root
compatibility namespace remains in the live artifact path. The remaining
Julia-specific compatibility imports are package-owned in
`packages/rust/crates/xiuxian-julia-core/src/compatibility/link_graph/`.

### Julia names that are now compatibility-seam only

These surfaces have already been pushed behind narrower ownership seams and
should not reappear on higher-level host surfaces:

1. `UI_JULIA_DEPLOYMENT_ARTIFACT` route-contract inventory alias
2. `get_julia_deployment_artifact` and `JuliaDeploymentArtifactQuery` as
   higher-level capability-module re-exports
3. Julia deployment-artifact helpers as `link_graph` middle-layer re-exports
4. raw `JULIA_*` ids as high-level host re-exports
5. Julia deployment tool type as the default `zhenfa_router` export
6. `UiJuliaDeploymentArtifact` as a root-level `types::compatibility::*`
   re-export
7. router-level direct deserialization of `UiJuliaDeploymentArtifact`
8. route-layer direct imports of `UiJuliaDeploymentArtifact`
9. Julia-named Zhenfa native deployment Rust tool/helper symbols
10. OpenAPI Julia path alias constants
11. Legacy Julia Zhenfa outward tool name
12. Flat crate-root and `runtime_config` root Julia re-export blocks
13. Julia-named Studio compatibility Rust DTO symbols

## Compatibility Namespace Map

The current compatibility seams are now physically grouped as follows:

```text
packages/rust/crates/xiuxian-julia-core/src/compatibility/
  link_graph/
```

The remaining compatibility import work is now package-owned under
`xiuxian-julia-core`.

The host crate-root compatibility shim is gone. The flat crate-root Julia
re-export block, the `runtime_config` compatibility sub-namespace, the former
`src/compatibility/julia.rs` shim, and the final
`src/compatibility/link_graph.rs` migration module are all retired from the
live tree. The remaining host-side legacy regression no longer imports any
crate-root compatibility helper path.

Downstream migration guidance now becomes:

1. prefer `xiuxian_julia_runtime::wendao::link_graph::*` for
   Julia-specific deployment and launch/runtime DTO imports
2. treat host crate-root compatibility imports as retired from code
3. keep generic plugin-artifact inspection on the canonical `xiuxian-wendao`
   surfaces instead of restoring a host compatibility namespace

### Next removal / generalization candidates

The next outward surfaces most likely to move after `P1` are:

1. post-`M5` downstream import cleanup for any package-owned Julia
   compatibility DTO users
2. post-`M6` additive-proof expansion beyond the landed Modelica slice,
   reusing the canonical generic plugin-artifact and plugin-capability
   surfaces without reintroducing host-owned compatibility seams

## Stage-B Acceptance State

The outward-alignment bundle and exit review are complete when:

1. this inventory describes the live outward surfaces rather than only the
   early extraction map
2. the active docs agree that late-`M6` additive proof now spans repo-facing,
   docs-facing, and Studio-facing consumers plus the repo service-state bundle
3. `Phase 7` transport hardening is closed with a `go` decision
4. `Phase-8 Stage A` has verified the live `deny.toml` / `justfile` /
   Nix security baseline and the missing semver/dependency-hygiene lanes
5. `Phase-8 Stage B` has fixed the policy boundary: blocking workspace
   security, blocking `xiuxian-wendao-core` semver governance, and advisory
   Wendao-scoped dependency hygiene
6. `Phase-8 Stage C` has landed the first bounded lane bundle and the
   explicit `Phase 8` gate decision is now `go`

## Phase-9 Consumer Reality Inventory

The authoritative post-`Phase 8` inventory is no longer adjunct
`xiuxian-daochang` remediation. It is the Wendao-owned `Phase 9` consumer
cutover baseline.

Current dependency reality:

1. live monolith-era `xiuxian-wendao` direct dependencies still exist in
   `xiuxian-qianji`, `xiuxian-zhixing`, and `xiuxian-daochang`
2. `xiuxian-qianhuan` still carries an optional monolith dependency
3. `xiuxian-wendao-modelica` already uses `xiuxian-wendao-core` for
   production code, but still retains a monolith dev-dependency for
   integration tests
4. no surveyed sibling consumer crate currently imports
   `xiuxian_wendao_core::...` or `xiuxian_wendao_runtime::...` directly in
   Rust source

Current source-level concentration:

1. `xiuxian-qianji` still depends on monolith exports for `LinkGraphIndex`,
   graph document types, and later cutover families beyond the now-cleared
   resource/VFS and contract-feedback / knowledge-entry seams
2. `xiuxian-qianhuan` no longer carries root-qualified source imports for the
   cleared resource/VFS family, but it still keeps the optional monolith
   dependency for later contraction work
3. `xiuxian-daochang` still depends on monolith exports for
   `LinkGraphIndex` and related host helpers
4. `xiuxian-zhixing` still depends on monolith exports for repo-intelligence
   and graph-facing types such as `LinkGraphIndex`, `LinkGraphHit`,
   `KnowledgeGraph`, `Entity`, and related repo-intelligence owners

First bounded cutover candidate:

1. the resource/VFS family:
   - `SkillRuntimeResolver`
   - `WendaoResourceUri`
   - `embedded_resource_text_from_wendao_uri`
   - `WendaoResourceRegistry`
2. this family is physically concentrated under `src/skill_runtime/` and
   `src/enhancer/resource_registry/`
3. this family is still re-exported broadly from `src/lib.rs`
4. this family is consumed by multiple sibling crates, which makes it the
   best first Stage-B cutover slice

Current Stage-B progress:

1. the first resource/VFS source-consumer cutover slice is now landed across
   `xiuxian-qianhuan`, `xiuxian-qianji`, and `xiuxian-daochang`
2. those source consumers now import from the owner seams instead of from the
   monolith crate root:
   - `xiuxian_wendao::skill_runtime::*`
   - `xiuxian_wendao::enhancer::WendaoResourceRegistry`
3. the follow-up test-consumer slice for this family is now landed too
4. root-qualified imports for this family are now cleared across the touched
   `src/` and `tests/` scope
5. the same family is now also cleared for Wendao's own internal unit-test
   consumer surface
6. the next bounded ingress/spider consumer slice is now also landed across
   `xiuxian-daochang` source and test consumers
7. those touched consumers now use the owner seam
   `xiuxian_wendao::ingress::{SpiderPagePayload, SpiderWendaoBridge,
canonical_web_uri}`
8. root-qualified imports for that ingress family are now cleared across the
   workspace `packages/**` Rust source and test scope
9. the targeted `xiuxian-daochang --test agent_suite --no-run` probe remains
   blocked only by deeper pre-existing compile failures outside the touched
   ingress files
10. the next bounded incremental-sync policy slice is now also landed across
    `xiuxian-daochang`, `xiuxian-zhixing`, and Wendao's own unit-test
    consumer surface
11. those touched consumers now use the owner seam
    `xiuxian_wendao::sync::IncrementalSyncPolicy`
12. root-qualified imports for `IncrementalSyncPolicy` are now cleared across
    the workspace `packages/**` Rust source and test scope
13. bounded verification is clean on that seam:
    `xiuxian-zhixing --lib` and
    `xiuxian-wendao --test wendao-validation-gate --no-run` pass, while
    `xiuxian-daochang --lib` and
    `xiuxian-zhixing --test test_wendao_indexer --no-run` remain blocked
    only by deeper pre-existing drift outside this family
14. the next bounded Zhixing indexer family slice is now also landed across
    `xiuxian-zhixing` source and test consumers
15. those touched consumers now use the owner seam
    `xiuxian_wendao::skill_runtime::zhixing::{ZhixingIndexSummary,
ZhixingWendaoIndexer}`
16. the owner seam now also carries the embedded skill-reference counters in
    `ZhixingIndexSummary`, which removes the existing downstream summary-field
    drift on the main indexer path
17. root-qualified imports for `ZhixingIndexSummary` and
    `ZhixingWendaoIndexer` are now cleared across the workspace `packages/**`
    Rust source and test scope
18. bounded verification is clean on that seam:
    `xiuxian-zhixing --lib`,
    `xiuxian-zhixing --test test_wendao_indexer --no-run`, and
    `xiuxian-wendao --test wendao-validation-gate --no-run` pass, while
    `xiuxian-zhixing --tests --no-run` and `xiuxian-daochang --lib`
    remain blocked only by deeper pre-existing drift outside this family
19. the next bounded contract-feedback / knowledge-entry slice is now also
    landed across `xiuxian-qianji` source and test consumers
20. those touched consumers now use the owner seams:
    `xiuxian_wendao::contract_feedback::WendaoContractFeedbackAdapter`,
    `xiuxian_wendao::storage::KnowledgeStorage`, and
    `xiuxian_wendao::types::{KnowledgeCategory, KnowledgeEntry}`
21. the slice exposed one bounded visibility seam in the host crate:
    `xiuxian-wendao/src/lib.rs` now exports `pub mod types;` so the owner
    path is physically reachable without relying on the crate-root alias
22. root-qualified imports for `KnowledgeEntry`, `KnowledgeStorage`,
    `WendaoContractFeedbackAdapter`, and `KnowledgeCategory` are now cleared
    across the touched `xiuxian-qianji` `src/` and `tests/` scope
23. bounded verification is clean on that seam:
    `xiuxian-qianji --lib`,
    `xiuxian-qianji --tests --no-run`, and
    `xiuxian-wendao --test wendao-validation-gate --no-run` pass
24. the next bounded graph-primitive slice is now also landed across the
    touched `xiuxian-qianji` and `xiuxian-zhixing` source/test consumers
25. those touched consumers now use the owner seams:
    `xiuxian_wendao::entity::{Entity, EntityType, Relation, RelationType}`
    and `xiuxian_wendao::graph::KnowledgeGraph`
26. the touched `xiuxian-zhixing/tests/test_strict_teacher.rs` seam also now
    matches the current live APIs by using a local `ManifestationInterface`
    stub and the current `ZhixingHeyi::add_task(title, scheduled_at)`
    signature
27. root-qualified imports for `Entity`, `EntityType`, `Relation`,
    `RelationType`, and root-braced `KnowledgeGraph` are now cleared across
    the touched `xiuxian-qianji` / `xiuxian-zhixing` scope, while explicit
    `xiuxian_wendao::graph::KnowledgeGraph` owner imports remain by design
28. bounded verification is clean on that seam:
    `xiuxian-qianji --lib`,
    `xiuxian-zhixing --lib`,
    `xiuxian-zhixing --test test_strict_teacher --no-run`, and
    `xiuxian-zhixing --test test_wendao_indexer --no-run` pass
29. the residual `xiuxian-zhixing/tests/test_heyi.rs` tail is now also
    compile-aligned to the same owner seams and to the current live
    `ZhixingHeyi` API signatures
30. bounded compile verification on that residual tail is clean:
    `xiuxian-zhixing --test test_heyi --no-run` passes
31. an attempted full `xiuxian-zhixing --test test_heyi` run still fails on
    deeper pre-existing reminder/agenda/task behavior drift in that Zhixing
    test surface, not on the owner-path cutover itself
32. the next bounded markdown-config slice is now also landed on the
    `MarkdownConfigBlock / extract_markdown_config_blocks` family
33. the host owner seam now has the minimum required visibility:
    `xiuxian-wendao/src/enhancer/mod.rs` now exports
    `pub mod markdown_config;`
34. the touched consumer in
    `xiuxian-daochang/tests/agent/native_tools_zhixing.rs` now uses the
    owner seam
    `xiuxian_wendao::enhancer::markdown_config::{MarkdownConfigBlock,
extract_markdown_config_blocks}`
35. root-qualified imports for that markdown-config family are now cleared
    across the workspace `packages/**` Rust source and test scope
36. bounded verification is clean on the Wendao-owned seam:
    `xiuxian-wendao --test wendao-validation-gate --no-run` passes, and the
    workspace grep for root-qualified `MarkdownConfigBlock` /
    `extract_markdown_config_blocks` imports is clean
37. the affected-package `xiuxian-daochang --test agent_suite --no-run`
    probe still fails, but the compile front remains in deeper pre-existing
    `gateway/http`, `agent/injection`, Telegram ACL, and session-memory
    drift outside this markdown-config cutover
38. the natural next Stage-B follow-up is still the next bounded consumer
    family; crate-root re-export contraction belongs to `Stage C`
39. the next bounded `Stage B` slice is now also landed on the residual
    graph-primitive tail in `xiuxian-daochang` test consumers
40. the touched `xiuxian-daochang` tests now use owner seams instead of
    crate-root graph primitives:
    - `xiuxian_wendao::entity::{Entity, EntityType}`
    - `xiuxian_wendao::graph::KnowledgeGraph`
41. the touched files are:
    - `xiuxian-daochang/tests/agent/native_tools_zhixing.rs`
    - `xiuxian-daochang/tests/agent/native_tools_zhixing_e2e.rs`
    - `xiuxian-daochang/tests/agent/native_tools_web.rs`
42. bounded verification for this residual tail is clean on the owner-path
    seam:
    - `xiuxian-wendao --test wendao-validation-gate --no-run` passes
    - grep for crate-root `Entity` / `EntityType` / `KnowledgeGraph`
      imports in the touched files is clean
43. the affected-package `xiuxian-daochang --test agent_suite --no-run`
    probe still fails, but the compile front remains in deeper pre-existing
    `gateway/http`, `agent/injection`, `agent/native_tools/zhixing`,
    `agent/zhenfa/bridge`, Telegram ACL, and session-memory drift outside
    this residual graph-primitive cutover
44. the natural next Stage-B follow-up is still another small bounded
    consumer family, not a broad `LinkGraphIndex` cut
45. the next bounded `Stage B` slice is now also landed on the
    `parse_frontmatter / embedded_discover_canonical_uris` family in
    `xiuxian-qianji`
46. the touched consumers now use owner seams instead of crate-root imports:
    - `xiuxian_wendao::enhancer::parse_frontmatter`
    - `xiuxian_wendao::skill_runtime::embedded_discover_canonical_uris`
47. the touched files are:
    - `xiuxian-qianji/src/executors/annotation/persona_markdown.rs`
    - `xiuxian-qianji/src/scheduler/preflight/query.rs`
48. bounded verification for this family is clean:
    - `xiuxian-qianji --lib` passes
    - `xiuxian-qianji --tests --no-run` passes
    - workspace grep for crate-root `parse_frontmatter` /
      `embedded_discover_canonical_uris` imports across sibling consumers is
      clean
49. the natural next Stage-B follow-up is still another small bounded
    consumer family that stays off a broad `LinkGraphIndex` cut
50. the next bounded `Stage B` slice is now also landed on the
    `LinkGraphHit / LinkGraphSearchOptions` family in `xiuxian-zhixing`
51. the touched consumer now uses the owner seam instead of crate-root
    imports:
    - `xiuxian_wendao::link_graph::{LinkGraphHit, LinkGraphSearchOptions}`
52. the touched file is:
    - `xiuxian-zhixing/src/heyi/agenda_render.rs`
53. bounded verification for this family is clean on the library seam:
    - `xiuxian-zhixing --lib` passes
    - sibling-consumer grep for crate-root `LinkGraphHit` /
      `LinkGraphSearchOptions` imports is clean
54. an attempted `xiuxian-zhixing --tests --no-run` still fails, but the
    compile front is in pre-existing `tests/test_storage_markdown.rs`
    crate-path drift outside this owner-path cutover
55. the natural next Stage-B follow-up is still another small bounded
    consumer family that stays off a broad `LinkGraphIndex` cut
56. the next bounded `Stage B` slice is now also landed on the
    `WendaoSearchTool` family across sibling test consumers
57. the touched consumers now use the owner seam instead of crate-root
    imports:
    - `xiuxian_wendao::zhenfa_router::WendaoSearchTool`
58. the touched files are:
    - `xiuxian-qianhuan/tests/test_zhenfa_native_tools.rs`
    - `xiuxian-daochang/tests/scenario_adversarial_evolution.rs`
59. bounded verification for this family is clean on the positive consumer
    path:
    - `xiuxian-qianhuan --test test_zhenfa_native_tools --features zhenfa-router --no-run`
      passes
    - sibling-consumer grep for crate-root `WendaoSearchTool` imports is
      clean
60. the affected-package
    `xiuxian-daochang --test scenario_adversarial_evolution --no-run`
    probe still fails, but the compile front remains in deeper pre-existing
    `gateway/http`, `agent/injection`, `agent/native_tools/zhixing`,
    `agent/zhenfa/bridge`, Telegram ACL, and session-memory drift outside
    this owner-path cutover
61. the natural next Stage-B follow-up is still another small bounded
    consumer family that stays off a broad `LinkGraphIndex` cut
62. the next bounded `Stage B` slice is now also landed on the residual
    resource/VFS test tail in `xiuxian-zhixing`
63. the touched tests now use owner seams instead of crate-root imports:
    - `xiuxian_wendao::enhancer::WendaoResourceRegistry`
    - `xiuxian_wendao::skill_runtime::{...}`
64. the touched files are:
    - `xiuxian-zhixing/tests/test_forge_skill_resources.rs`
    - `xiuxian-zhixing/tests/test_wendao_skill_resources.rs`
65. bounded verification for this residual tail is clean:
    - `xiuxian-zhixing --test test_forge_skill_resources --no-run` passes
    - `xiuxian-zhixing --test test_wendao_skill_resources --no-run` passes
    - sibling-consumer grep for crate-root resource/VFS imports in this
      family is clean
66. the natural next Stage-B follow-up is still another small bounded
    consumer family that stays off a broad `LinkGraphIndex` cut
67. the next bounded `Stage B` slice is now also landed on a test-only
    `LinkGraphIndex` leaf for the native-search scenario consumers
68. the touched tests now use the owner seam instead of the crate-root import:
    - `xiuxian_wendao::link_graph::LinkGraphIndex`
69. the touched files are:
    - `xiuxian-qianhuan/tests/test_zhenfa_native_tools.rs`
    - `xiuxian-daochang/tests/scenario_adversarial_evolution.rs`
70. bounded verification for this test-only leaf is clean on the positive
    consumer path:
    - `xiuxian-qianhuan --test test_zhenfa_native_tools --features zhenfa-router --no-run`
      passes
    - grep for crate-root `LinkGraphIndex` imports in the touched files is
      clean
71. the affected-package
    `xiuxian-daochang --test scenario_adversarial_evolution --no-run`
    probe still fails, but the compile front remains in deeper pre-existing
    `gateway/http`, `agent/injection`, `agent/native_tools/zhixing`,
    `agent/zhenfa/bridge`, Telegram ACL, and session-memory drift outside
    this test-only owner-path cutover
72. the natural next Stage-B follow-up is still another small bounded
    consumer family, and this slice does not authorize a broad
    `LinkGraphIndex` cut
73. the next bounded `Stage B` slice is now also landed on a residual
    `xiuxian-qianji` integration-test `LinkGraphIndex` leaf
74. the touched files now use the owner seam instead of the crate-root import:
    - `xiuxian-qianji/tests/integration/test_qianji_qianhuan_binding.rs`
    - `xiuxian-qianji/tests/integration/test_agenda_validation_pipeline.rs`
    - `xiuxian-qianji/tests/integration/test_qianji_trinity_integration.rs`
    - `xiuxian_wendao::link_graph::LinkGraphIndex`
75. these files are not currently wired as active `[[test]]` targets in
    `xiuxian-qianji/Cargo.toml`, so exact `cargo test --test ... --no-run`
    probes are physically unavailable for this leaf
76. bounded verification is still clean at the touched-file and active-package
    level:
    - grep for crate-root `LinkGraphIndex` imports in the touched files is
      clean
    - `xiuxian-qianji --tests --no-run` passes
77. this is explicitly not a family-complete `LinkGraphIndex` migration:
    residual root-qualified `LinkGraphIndex` imports still exist elsewhere in
    `xiuxian-qianji` source and tests
78. the natural next Stage-B follow-up is still another small bounded
    consumer family rather than a broad `LinkGraphIndex` cut
79. the next bounded `Stage B` slice is now also landed on the
    `xiuxian-qianji` app/runtime boot surfaces
80. the touched files now use the owner seam instead of the crate-root import:
    - `xiuxian-qianji/src/app/qianji_app.rs`
    - `xiuxian-qianji/src/app/build.rs`
    - `xiuxian-qianji/src/bootcamp/model.rs`
    - `xiuxian-qianji/src/bootcamp/runtime.rs`
    - `xiuxian-qianji/src/bin/qianji.rs`
    - `xiuxian-qianji/src/python_module/llm_bridge.rs`
    - `xiuxian_wendao::link_graph::LinkGraphIndex`
81. bounded verification for this bundle is clean:
    - touched-file grep for crate-root `LinkGraphIndex` imports is clean
    - `xiuxian-qianji --lib` passes
    - `xiuxian-qianji --bin qianji --features llm` passes
    - `xiuxian-qianji --lib --features pyo3,llm` passes
82. the natural next Stage-B follow-up is still another small bounded
    consumer family rather than a broad `LinkGraphIndex` cut
83. the current pure Flight same-port checkpoint is now explicit:
    - `/api/search` is removed from the active router and bundled OpenAPI
      surface
    - `/search/knowledge` is the only knowledge-search business contract
    - the search business family is now Flight-only in the active source tree
84. the current search-surface cleanup is warning-clean in the touched scope:
    - `SearchQuery` is now test-only
    - dead references/symbols provider wrappers and batch-helper shims are gone
85. the current runtime transport server is now modularized out of the old
    monolith:
    - retired `xiuxian-wendao-runtime/src/transport/server.rs`
    - added
      `xiuxian-wendao-runtime/src/transport/server/{mod,types,request_metadata,service,tests}.rs`
86. fresh same-port proof on a newly built `127.0.0.1:9519` gateway is clean:
    - `/api/health -> 200`
    - `FlightService/GetFlightInfo -> 400`
    - `/api/search -> 404`
    - frontend same-origin live Flight proof passes end-to-end
87. the same pure-Flight search boundary now also includes definition and
    autocomplete:
    - `/search/definition` and `/search/autocomplete` are part of the active
      Flight snapshot and runtime provider contract
    - `/api/search/definition` and `/api/search/autocomplete` are removed from
      the active router and bundled OpenAPI surface
    - `.data/wendao-frontend` now resolves both flows through same-origin
      Flight instead of through standalone HTTP business routes

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/404_repo_intelligence_for_sciml_and_msl|Repo Intelligence for SciML and MSL]], [[06_roadmap/405_large_rust_modularization|Large Rust File Modularization]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

:FOOTER:
:STANDARDS: v2.0
:LAST_SYNC: 2026-04-02
:END:
