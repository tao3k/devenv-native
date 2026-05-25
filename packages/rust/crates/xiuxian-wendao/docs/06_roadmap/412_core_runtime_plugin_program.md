# Wendao Core Runtime Plugin Program

:PROPERTIES:
:ID: wendao-core-runtime-plugin-program
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, migration, plugins, core, runtime, program
:STATUS: ACTIVE
:END:

## Purpose

This note is the program-level execution entrypoint for the Wendao
core/runtime/plugin migration.

It exists to stop fragmented implementation drift and to turn the active RFC,
blueprint, inventory, and `P1` notes into one coordinated rollout plan.

Primary references:

- `[[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md]]`
- `[[docs/rfcs/2026-03-27-wendao-arrow-plugin-flight-rfc.md]]`
- `[[06_roadmap/409_core_runtime_plugin_surface_inventory]]`
- `[[06_roadmap/410_p1_generic_plugin_contract_staging]]`
- `[[06_roadmap/411_p1_first_code_slice_plan]]`

The active task-scoped blueprint for this migration remains recorded in the
paired ExecPlan rather than this persistent program note.

## Program Position

The active tree has now cleared `M6` and is at the handoff to the next
macro-phase target defined in the RFC: `Phase 7: Flight-First Runtime
Negotiation`.

Completed baseline:

1. generic plugin-runtime vocabulary exists in the current tree
2. compatibility seams are explicit feature folders
3. host crate-root compatibility export maps have now been retired after
   serving the migration cutover
4. the remaining Julia-specific compatibility ownership is package-owned in
   `xiuxian-julia-core`
5. the first physical `xiuxian-wendao-core` crate cut now exists in the
   workspace
6. the first `M6` additive plugin proof now exists too: Modelica uses normal
   package dependencies instead of host-side source inclusion

Incomplete baseline:

1. `xiuxian-wendao-core` is not yet wired into every main-crate consumer
2. `xiuxian-wendao-runtime` is not yet wired into every live host assembly and
   resolver seam
3. package-owned Julia compatibility names may still need downstream cleanup
   after the host-side retirement cut

## Macro Phases

### M1: Contract and Compatibility Stabilization

Scope:

1. complete contract normalization
2. finish narrowing host-side compatibility seams
3. freeze the `core` extraction candidate surface

Exit criteria:

1. no new language-specific host types land outside explicit compatibility
   namespaces
2. `core` extraction package list is complete
3. `runtime` extraction package list is complete

### M2: Core Extraction

Scope:

1. create `xiuxian-wendao-core`
2. move stable contracts there
3. preserve compatibility re-exports temporarily

Exit criteria:

1. `core` builds independently
2. semver-governed contract surface is physically separated

### M3: Runtime Extraction

Scope:

1. create `xiuxian-wendao-runtime`
2. move launch, negotiation, routing, health, and fallback ownership
3. connect binaries and host assembly through runtime

Exit criteria:

1. runtime behavior no longer depends on the monolithic crate boundary
2. `core` remains free of lifecycle ownership

### M4: Julia Ownership Externalization

Scope:

1. create or finalize the independently publishable Julia package path
2. move Julia-owned launch and artifact responsibilities there
3. remove in-tree source inclusion hacks

Exit criteria:

1. Julia package owns Julia-specific runtime details
2. host integration uses plugin contracts, not host-owned Julia DTOs

### M5: Generic Artifact Cutover

Scope:

1. make generic plugin-artifact endpoints canonical
2. demote Julia outward surfaces to compatibility-only

Exit criteria:

1. Studio, OpenAPI, and Zhenfa prefer generic artifact contracts
2. Julia-named outward endpoints are compatibility shims only

### M6: Additional Plugin Proof

Scope:

1. onboard one more language/plugin path
2. verify that the new architecture is additive

Exit criteria:

1. one additional plugin path lands without core expansion

## Program Deliverables

The following deliverables must now be kept current:

1. `core` extraction package list
2. `runtime` extraction package list
3. Julia externalization package list
4. compatibility retirement ledger

## Immediate Next Program Move

Authoritative current position:

1. the additive-proof track has now cleared `M6`
2. the current risk is no longer proof coverage; it is transport/runtime
   hardening ambiguity for the next macro phase
3. the next push should therefore target `Phase 7: Flight-First Runtime
Negotiation` rather than another additive-proof slice

Phase-7 staged push plan:

1. `Stage A: Transport Surface Inventory Bundle`
   identify the live negotiation seams, fallback callers, and outward
   diagnostics surfaces, then document one canonical transport preference
   order
2. `Stage B: Negotiation Policy Bundle`
   harden runtime selection so Flight is preferred where supported while Arrow
   IPC remains the bounded fallback path
3. `Stage C: Observability and Gate Bundle`
   expose negotiated transport plus fallback reason, then run one explicit
   `Phase 7` go/no-go review

Current staged position:

1. `Stage A` is complete
2. `Stage B` is complete
3. `Stage C` is complete
4. the explicit `Phase 7` gate decision is `go`

Stage-A inventory summary:

1. the generic contract surface is already stable in `xiuxian-wendao-core`
   via `PluginCapabilityBinding`, `PluginTransportEndpoint`, and
   `PluginTransportKind`
2. the only live runtime-owned transport-construction seam today is
   `xiuxian-wendao-runtime/src/transport/flight.rs`, and the runtime now
   materializes only `ArrowFlight` on the live negotiation path
3. the current outward inspection seam is `UiPluginArtifact`, which now
   carries endpoint metadata plus `selected_transport`, `fallback_from`, and
   `fallback_reason`
4. the canonical Phase-7 runtime negotiation order is now fixed as
   `ArrowFlight`
5. the first Stage-B runtime cut had landed in
   `xiuxian-wendao-runtime/src/transport/negotiation.rs`, and the rerank path
   now delegates through that runtime-owned negotiation seam
6. the second Stage-B cut has now landed in
   `xiuxian-wendao-runtime/src/transport/flight.rs`, where the runtime owns a
   real Flight client materialization seam aligned to the LanceDB Arrow
   `57.3` line
7. the runtime now bridges host-side Arrow-58 rerank batches onto that
   LanceDB Arrow-57 Flight line through the existing
   `xiuxian-vector::{engine_batches_to_lance_batches,
lance_batches_to_engine_batches}` compat seam
8. the rerank path now negotiates and materializes only the `ArrowFlight`
   path, and the public plugin transport contract is now Flight-only as well
9. `Stage C` has now landed the outward and telemetry diagnostics bundle, so
   `Phase 7` closes with a `go` decision
10. a new operator-directed boundary override now supersedes that older target
    for future external surfaces: high-performance-first `Arrow Flight` is the
    desired business surface, `JSON` remains the control surface for
    process/bootstrap, operator config/status/control, and static artifact
    inspection, with `ArrowFlight` as the only remaining plugin data-exchange
    transport surface
11. the first retirement-ledger slice is now explicit: the old
    `/api/search/*/hits-arrow` family is retired, and the old
    `/api/analysis/*/retrieval-arrow` mirror family is retired too; the
    unified `/api/search/{intent,attachments,references,symbols}` HTTP
    business routes are retired as well, and semantic search plus analysis now
    keep only the canonical Flight business contracts
12. the next bounded contract slice has now declared canonical Flight route
    constants for the current semantic search and analysis business families
13. the first bounded service-materialization slice has now landed in
    `xiuxian-wendao-runtime/src/transport/server.rs`, where
    `SearchFlightRouteProvider`,
    `MarkdownAnalysisFlightRouteProvider`,
    `CodeAstAnalysisFlightRouteProvider`, and
14. the same-process same-port gateway slice is now warning-clean in the
    touched scope: retired code-search compatibility seams no longer remain in
    the production compile graph, and
    `cargo check -p xiuxian-wendao --bin wendao --features julia` now closes
    without the old `code_search/query/intent` warning front
    `WendaoFlightService::new_with_route_providers(...)` accept the semantic
    `/search/*` route family plus the bounded analysis routes using stable
    `x-wendao-search-query` / `x-wendao-search-limit` metadata for search and
    `x-wendao-analysis-path` / `x-wendao-analysis-repo` /
    `x-wendao-analysis-line` for analysis
15. the first concrete Studio-backed provider slice has now landed in
    `src/gateway/studio/search/handlers/knowledge/intent/flight.rs`, where
    `StudioIntentSearchFlightRouteProvider` materializes the semantic
    `/search/intent` route into a Studio-owned Arrow batch shaped like the
    current intent `hits-arrow` payload
16. the former `/api/search/intent` HTTP business route is now retired, so
    that same provider-backed batch seam serves only the canonical Flight
    contract `/search/intent`
17. active Flight service wiring and wider Studio handler cutover are now
    landed, so semantic-search IPC retirement is no longer blocked on
    consumer materialization
18. the next bounded Studio-backed provider-backed bridge has now landed in
    `src/gateway/studio/search/handlers/symbols.rs`, where
    `StudioSymbolSearchFlightRouteProvider` materializes the semantic
    `/search/symbols` route into a Studio-owned Arrow batch shaped like the
    current symbol `hits-arrow` payload
19. the former `/api/search/symbols` HTTP business route is now retired, so
    that same provider-backed batch seam serves only the canonical Flight
    contract `/search/symbols`
20. the former `/api/search/attachments` HTTP business route is now retired
    too, and the dedicated attachment provider seam now serves only the
    canonical Flight contract `/search/attachments`
21. the next bounded Studio-backed provider-backed bridge has now landed in
    `src/gateway/studio/search/handlers/ast.rs`, where
    `StudioAstSearchFlightRouteProvider` materializes the semantic
    `/search/ast` route into a Studio-owned Arrow batch shaped like the
    current AST `hits-arrow` payload
22. the old `/api/search/ast` HTTP business route is now removed entirely, so
    that same provider-backed batch seam in `ast.rs` only serves the
    canonical `/search/ast` Flight contract
23. one bounded host-side aggregate dispatch seam now also exists in
    `src/gateway/studio/search/handlers/flight.rs`, where
    `StudioSearchFlightRouteProvider` multiplexes
    `SEARCH_INTENT_ROUTE`, `SEARCH_REFERENCES_ROUTE`,
    `SEARCH_SYMBOLS_ROUTE`, and `SEARCH_AST_ROUTE` onto one shared
    Studio-owned Flight provider contract; this is not IPC retirement yet
24. the next bounded host-side service-materialization seam now also exists in
    `src/gateway/studio/search/handlers/flight.rs` and
    `src/link_graph/plugin_runtime/transport/server.rs`, where
    `build_studio_search_flight_service_with_repo_provider(...)` and
    `build_search_plane_studio_flight_service(...)` materialize one
    `WendaoFlightService` with repo-search, aggregate semantic search,
    attachment, AST, markdown-analysis, and code-AST-analysis providers; this
    is still not IPC retirement
25. the old `/api/analysis/markdown` and `/api/analysis/code-ast` HTTP
    business routes are now removed entirely from the outward gateway
    contract; the canonical business contracts are the Flight routes
    `/analysis/markdown` and `/analysis/code-ast` plus the stable
    `x-wendao-analysis-*` metadata contract
26. the dedicated `src/bin/wendao_search_flight_server.rs` binary now also
    consumes the roots-based Studio builder through
    `build_search_plane_studio_flight_service_for_roots_with_weights(...)`,
    which makes semantic search-family and analysis-family Flight serving
    active in the dedicated binary while the remaining transport debt sits in
    fallback/runtime compatibility seams rather than outward route mirrors
27. the direct-cut policy is now active for semantic search too: the outward
    HTTP business routes `/api/search/intent`, `/api/search/attachments`,
    `/api/search/references`, `/api/search/symbols`, and `/api/search/ast`
    are retired from the gateway surface
28. the canonical semantic search surface is now only the Flight contract
    family `/search/intent`, `/search/knowledge`, `/search/attachments`,
    `/search/references`, `/search/symbols`, and `/search/ast`
29. the general knowledge search seam is now aligned too:
    `SEARCH_KNOWLEDGE_ROUTE = /search/knowledge` is runtime-owned, the Studio
    aggregate provider now serves it, and
    `load_knowledge_search_response_flight_batch(...)` materializes it through
    the shared `SearchHit` Arrow batch shape
30. that live semantic Flight contract is now pinned by
    `tests/snapshots/gateway/studio/search_flight_service_route_contracts.snap`
    instead of by HTTP Arrow IPC route coverage
31. the next cleanup slice has now removed the dead in-crate search Arrow IPC
    helper modules and their roundtrip helper tests, so semantic search
    verification is anchored on the shared Flight snapshot plus active provider
    tests instead of duplicate local IPC helper coverage
32. the next cleanup slice has now also removed the retired semantic-search
    HTTP wrapper functions from the unit-test graph; the remaining unit proofs
    call the live response seams directly, so test coverage no longer keeps
    dead HTTP wrapper shells alive after the pure-Flight cut
33. the dedicated `src/bin/wendao_search_flight_server.rs` binary is now
    browser-consumable too: it enables `tonic_web::GrpcWebLayer` with
    `accept_http1(true)`, so browser-side gRPC-web clients can consume the
    semantic Flight service directly
34. `.data/wendao-frontend` now has an explicit pure-Flight search lane:
    `wendao.toml` carries `[search_flight]`, the Rspack dev server proxies
    `/arrow.flight.protocol.FlightService/*`, and
    `src/api/flightSearchTransport.ts` plus generated Arrow Flight client code
    now consume the canonical `/search/knowledge` contract
35. `src/api/clientRuntime.ts` now routes knowledge search through that pure
    Flight client instead of through retired HTTP search business routes
36. live end-to-end proof is now green in
    `.data/wendao-frontend/src/api/liveGateway.test.ts`, which validates the
    full frontend -> Flight server -> Arrow IPC decode -> gateway graph
    resolution path against a running gateway and a running dedicated Flight
    server

## M6 Exit Review

Decision: `go`

The `M6` completion conditions are now satisfied:

1. one non-Julia plugin path has landed without new language-specific host
   structs
2. repo-facing, docs-facing, and Studio-facing consumers all now have bounded
   additive proof coverage
3. the RFC, active ExecPlan, program note, outward inventory, and package note
   now agree on the same position and governed next move

Next macro-phase target:

1. `Phase 8: Contract and Dependency Governance`
2. tooling-backed contract stability and dependency hygiene should now replace
   transport hardening as the active program concern
3. the workspace already has a `cargo-audit` / `cargo-deny` baseline, so the
   next phase should extend and unify that reality rather than restart it

Phase-8 staged push plan:

1. `Stage A: Tooling Reality Inventory Bundle`
   verify the live `deny.toml`, `justfile`, Nix inputs, and missing
   semver/dependency-hygiene seams
2. `Stage B: Contract and Dependency Policy Bundle`
   define owner crates and pass/fail scope for `cargo-audit`,
   `cargo-deny`, `cargo-semver-checks`, `cargo-machete`, and `cargo-udeps`
3. `Stage C: Lane Integration and Gate Bundle`
   land the first bounded lane set and record an explicit `Phase 8`
   go/no-go review

Current staged position:

1. `Phase-8 Stage A` is complete
2. `Phase-8 Stage B` is complete
3. `Phase-8 Stage C` is complete
4. the explicit `Phase 8` gate decision is `go`

Stage-A inventory summary:

1. a live repo-root `deny.toml` baseline already exists
2. the live `justfile` already exposes `rust-security-audit`,
   `rust-security-deny`, and `rust-security-gate`
3. Nix already provisions `cargo-audit` and `cargo-deny`
4. no live `justfile`, Nix, or CI lane currently references
   `cargo-semver-checks`, `cargo-machete`, or `cargo-udeps`

Stage-B policy summary:

1. `cargo-audit` stays workspace-wide and blocking through the existing
   `rust-security-gate`
2. `cargo-deny` stays workspace-wide and blocking for advisories, bans, and
   sources, while duplicate-version findings remain warn-only under the
   current `deny.toml`
3. the initial `Phase 8` lane set does not expand `cargo-deny` into a license
   gate yet; license policy remains out of scope for the first bounded
   rollout
4. `cargo-semver-checks` is now the blocking contract-governance lane for
   `xiuxian-wendao-core` only
5. `xiuxian-wendao`, `xiuxian-wendao-runtime`, `xiuxian-julia-core`, and
   `xiuxian-wendao-modelica` stay out of the initial semver gate because
   those public surfaces are still migration-owned rather than frozen
   contract packages
6. `cargo-machete` is the initial advisory dependency-hygiene lane for the
   Wendao migration cluster:
   `xiuxian-wendao-core`, `xiuxian-wendao-runtime`, `xiuxian-wendao`,
   `xiuxian-julia-core`, and `xiuxian-wendao-modelica`
7. `cargo-udeps` is the initial advisory unused-dependency lane, with the
   first bounded scope starting at `xiuxian-wendao-core` and
   `xiuxian-wendao-runtime`
8. the governed `Stage C` target bundle is now fixed as:
   existing blocking workspace security, blocking semver checks on
   `xiuxian-wendao-core`, and advisory Wendao-scoped dependency-hygiene lanes

Stage-C landing summary:

1. the local governance bundle now lives in `justfile` through
   `rust-contract-semver-core`,
   `rust-dependency-hygiene-machete-wendao`,
   `rust-dependency-hygiene-udeps-wendao`, and
   `rust-contract-dependency-governance`
2. Nix provisioning now includes `cargo-semver-checks`,
   `cargo-machete`, and `cargo-udeps`
3. `devenv` now exposes `ci:rust-contract-dependency-governance`, and both
   main Rust workflows call that task
4. the blocking semver lane for `xiuxian-wendao-core` is now live and passed
   on the current tree
5. the advisory `cargo-machete` lane is now live and passes cleanly on the
   current Wendao migration cluster after removing the stale
   `xiuxian-wendao` dependency entries
6. the advisory `cargo-udeps` lane is now wired with bounded skip semantics
   when `rustup` nightly is unavailable in the current environment
7. follow-on bounded sibling remediations removed the stale
   `xiuxian-llm` direct dependencies `rmcp`, `fast_image_resize`, and
   `spider_agent`, then retired the dead `xiuxian-zhenfa`
   `tests/support/gateway.rs` helper; `toml` and `xiuxian-config-core`
   stay present in `xiuxian-llm` as explicit `cargo-machete`-ignored macro
   dependencies, and live `rmcp` ownership is now concentrated in
   `xiuxian-daochang`
8. the final bounded production-side replacement moved
   `xiuxian-daochang/src/tool_runtime/bridge.rs` off `rmcp` entirely by
   landing a self-owned streamable-HTTP JSON-RPC client for `initialize`,
   `tools/list`, and `tools/call`, and by moving `rmcp` to
   `dev-dependencies` for the test-side server harness only; direct `rmcp`
   usage no longer exists in `xiuxian-daochang/src/`
9. full `xiuxian-daochang` crate verification still remains blocked by
   unrelated pre-existing module/export failures outside the touched seam
10. the bounded follow-on remediation has now removed the stale
    `xiuxian-daochang` direct dependencies `comrak` and `regex`, so
    `cargo-machete` is now clean on that package
11. the next bounded crate-health slice has repaired
    `xiuxian-daochang/src/agent/system_prompt_injection_state.rs` by
    restoring a local session prompt-injection snapshot/cache/storage seam;
    the compile front no longer stops there and has moved on to larger
    pre-existing import/export failures elsewhere in `xiuxian-daochang`
12. the following bounded slice has repaired the
    `runtime_agent_factory` test-support visibility seam by promoting the
    reused helper functions to `pub(crate)`; that private-import cluster no
    longer appears at the compile front
13. the next bounded Discord crate-health slice has removed the dead mounted
    `channels/discord/channel/constructors.rs` duplicate from the live
    module tree, so the stale `omni_agent` / `ChannelCommandRequest` import
    cluster is retired instead of being kept alive through new visibility
    widening
14. the following bounded channel-runtime slice has restored the missing
    `nodes/channel/common.rs` embedding-memory guard helper and its test
    shim, so the live `nodes/channel/{discord,telegram}.rs` launch paths no
    longer depend on a nonexistent shared leaf; the next compile-front risk
    is now deeper Discord runtime dispatch and Telegram channel wiring drift
15. the next bounded Discord runtime slice has now repaired the stale
    `channels/discord/runtime/dispatch/` surface by restoring child-module
    wiring, a local `dispatch/support.rs` leaf, the
    `process_discord_message(...)` compatibility wrapper, crate-internal
    `ForegroundInterruptController` visibility, and the shared
    `compose_turn_content(...)` helper in `channels/managed_runtime/turn.rs`;
    that Discord dispatch cluster no longer appears at the compile front
16. the following bounded channel path-normalization slice has now repaired
    the remaining live Telegram channel/runtime owner drift by mounting the
    existing `channel/outbound_text.rs` leaf, re-exporting the
    `jobs::{JobRecord, QueuedJob, epoch_millis}` owner seam, switching the
    touched imports from `super::super::...` to `crate::...`, and retiring
    dead duplicate Discord managed-session `admin` / `injection` leaves plus
    the dead Telegram `session_context` duplicate; the
    `xiuxian-daochang` compile front no longer stops in that channel family
    and now exits the crate at a pre-existing `xiuxian-wendao` transport
    import failure
17. the next bounded runtime transport slice has now repaired that
    `xiuxian-wendao` import failure by restoring the missing
    `RepoSearchFlightRouteProvider` re-export in
    `xiuxian-wendao-runtime/src/transport/mod.rs`; both
    `xiuxian-wendao-runtime --features julia` and
    `xiuxian-wendao --features julia` now clear that transport seam again
18. the following bounded helper slice has now repaired the
    `xiuxian-daochang` `jobs/heartbeat` owner import and the initial
    embedding helper drift by replacing the dead `env_first_non_empty`
    macro dependency with a local env scan and by making the currently
    unwired `mistral_sdk` embedding path fail explicitly instead of failing
    the build; the compile front now starts deeper in `llm/*`,
    `resolve.rs`, and `runtime_agent_factory/*`
19. the following bounded crate-health slice has now repaired the
    `Agent` / session-alignment bundle by restoring the live `Agent` owner
    fields, initializing them in `agent/bootstrap/memory.rs`, restoring the
    recall-bias and session-backup wrappers in `agent/feedback.rs` and
    `agent/session_context/backup.rs`, and rebinding the touched imports to
    `crate::...`; the compile front no longer stops in that bundle and now
    starts in `agent/tool_startup.rs`,
    `session/redis_backend/executor.rs`, and
    `test_support/memory_metrics.rs` before deeper channel/runtime drift
20. the following bounded crate-health slice has now repaired that next
    compile-front bundle too by retaining the startup connect config for
    diagnostics in `agent/tool_startup.rs`, aligning
    `session/redis_backend/executor.rs` to the current `RwLock` connection
    owner, and moving the memory-recall metric helper methods onto the live
    `agent/memory_recall_metrics.rs` owner seam while retiring the dead
    `test_support/memory_metrics.rs` shim; the compile front now starts
    deeper in
    `channels/discord/runtime/managed/handlers/command_dispatch/session/budget.rs`,
    `tool_runtime/bridge.rs`, `llm/*`, and remaining Discord/Telegram
    runtime drift

Phase-8 exit review:

1. decision: `go`
2. the required governance baseline now exists physically in local lanes,
   `devenv` tasks, and CI
3. the next macro-phase move should be either a new RFC-governed phase
   proposal or a bounded remediation bundle for the remaining live package
   health issues; the current smallest bounded risk is no longer stale
   dependencies, but the pre-existing `xiuxian-daochang` crate-health
   breakage outside the already-cleaned dependency surface; the current next
   bounded cluster has now moved past the dead Discord constructor duplicate,
   the missing channel memory-guard leaf, the repaired channel/runtime owner
   drift, and the runtime transport export seam; the compile front now starts
   deeper in `xiuxian-daochang` `llm/*`, `resolve.rs`, and
   `runtime_agent_factory/*`
4. the following bounded crate-health slice has now repaired the
   `resolve.rs` plus `runtime_agent_factory/*` owner drift by introducing a
   crate-owned `channel_runtime.rs` enum seam, rebinding the factory modules
   to crate-owned settings/config owners, restoring the missing
   environment-parsing helpers, and replacing touched relative paths with
   `crate::...`; the compile front now starts deeper in `session/*`,
   `test_support/*`, `lib.rs`, and `agent/*` private-module exposure
   consumed by `test_support/*`
5. the following bounded crate-health slice has now repaired the
   `session/*` owner drift by restoring the missing local `SessionStore`,
   switching the live `TurnSlot` owner back to `xiuxian_window`, fixing the
   `RedisSessionBackend` message-content snapshot field, and rebinding the
   touched session tests to `xiuxian_daochang`; the compile front now starts
   deeper in `test_support/*`, `lib.rs`, and `agent/*` private-module
   exposure consumed by `test_support/*`
6. the following bounded crate-health slice has now repaired the
   `Agent` / session-alignment bundle by restoring the live `Agent`
   owner fields, wiring `agent/bootstrap/memory.rs` back onto the current
   crate-owned tool/admission/embedding seams, restoring the recall-bias and
   session-backup wrappers, and replacing the touched deep relative imports
   with `crate::...`; the compile front now starts in
   `agent/tool_startup.rs`, `session/redis_backend/executor.rs`, and
   `test_support/memory_metrics.rs`
7. the following bounded crate-health slice has now repaired that next
   compile-front bundle too by cloning the startup connect config at the
   call boundary, aligning the Redis executor to the current `RwLock`
   connection owner, and moving the memory-recall metric helper methods onto
   the live `agent/memory_recall_metrics.rs` owner seam while retiring the
   dead `test_support/memory_metrics.rs` shim; the compile front now starts
   deeper in
   `channels/discord/runtime/managed/handlers/command_dispatch/session/budget.rs`,
   `tool_runtime/bridge.rs`, `llm/*`, and the remaining Discord/Telegram
   runtime drift
8. the following bounded crate-health slice has now repaired that next
   compile-front bundle too by rebinding the touched Discord budget handler
   to crate-owned imports and reference-based snapshot formatting, aligning
   `tool_runtime/bridge.rs` to the current `reqwest` and JSON-RPC owner
   seams while retaining connect diagnostics, and restoring the touched
   `llm/*` surface to the live `max_tokens` plus
   `DeepseekRuntime::RemoteHttp` shape; the compile front now starts deeper
   in Discord runtime gateway/run ownership and Telegram send/runtime drift

Current status:

1. the `M2` core extraction package list now exists in
   `[[06_roadmap/413_m2_core_extraction_package_list]]`
2. the first physical `xiuxian-wendao-core` crate cut now exists in the
   workspace and the first plugin-runtime contract slice in
   `xiuxian-wendao` now re-exports from it
3. the `M3` runtime extraction package list now exists in
   `[[06_roadmap/414_m3_runtime_extraction_package_list]]`
4. the Julia externalization package list now exists in
   `[[06_roadmap/415_m4_julia_externalization_package_list]]`
5. the compatibility retirement ledger now exists in
   `[[06_roadmap/416_compatibility_retirement_ledger]]`
6. the program artifact set is now complete for `M2` through `M5` planning
7. the next overall implementation move is to expand consumer cutover beyond
   the `plugin_runtime` barrel and into the remaining `M2` contract surface
8. that expansion has now started in `runtime_config`, where the pure
   contract-side Julia rerank selector, binding, launch, artifact, and
   transport imports are being sourced from `xiuxian-wendao-core`
9. the same `M2` slice has now reached outward consumers too:
   `runtime_config.rs` and the Studio UI type compatibility/config modules
   now import stable launch, artifact, and binding records from
   `xiuxian-wendao-core`
10. the next `M2` cutover ring has now reached helper-consumer modules:
    artifact resolution, transport-client assembly, and quantum rerank flow
    modules now consume stable contract records from `xiuxian-wendao-core`
11. the remaining selector/enum-only consumers under `plugin_runtime/` are
    also being cleaned up so that even focused helper/test seams stop reading
    stable contract records through the monolithic crate layer
12. `M3` physical extraction has now started too: the first
    `xiuxian-wendao-runtime` crate cut exists in the workspace and owns the
    transport-client construction slice, while `xiuxian-wendao` keeps a
    temporary re-export seam for compatibility
13. `M3` has now expanded to a second runtime-owned helper slice too:
    generic artifact render behavior lives in `xiuxian-wendao-runtime`, while
    the monolithic crate keeps only the runtime-config-backed resolver seam
14. `M3` now also owns the generic artifact resolve helper in
    `xiuxian-wendao-runtime`; the monolithic crate keeps only the
    runtime-state-backed resolver callback for the current Julia compatibility
    path
15. `M3` now owns the generic runtime-config settings helper seam too:
    `xiuxian-wendao-runtime/src/settings/` holds the override, TOML-merge,
    parse, access, and directory helpers, while
    `src/link_graph/runtime_config/settings/mod.rs` in `xiuxian-wendao`
    retains only the Wendao-embedded config wrapper and module-shaped
    re-export surface expected by the local resolve tree
16. `M3` now owns the first live runtime-config resolution slice as well:
    cache, related, coactivation, and index-scope records/constants/resolvers
    now live in `xiuxian-wendao-runtime/src/runtime_config/`, while
    `xiuxian-wendao` retains only the settings-backed wrapper layer that keeps
    the original module paths stable
17. `M3` now owns the agentic runtime subtree too:
    `xiuxian-wendao-runtime/src/runtime_config/models/agentic.rs` and
    `src/runtime_config/resolve/agentic/` hold the record/default/env/apply/
    finalize ownership, while `xiuxian-wendao` keeps only the
    `merged_wendao_settings()` wrapper boundary
18. `M3` now owns the generic retrieval semantic-ignition subtree too:
    `xiuxian-wendao-runtime/src/runtime_config/retrieval/semantic_ignition.rs`
    holds the record/default/env/settings-to-runtime resolver ownership, while
    `xiuxian-wendao` keeps only the existing model/resolver module paths as
    re-export seams
19. `M3` now owns the generic retrieval tuning/base slice as well:
    `xiuxian-wendao-runtime/src/runtime_config/retrieval/base.rs` resolves
    candidate multiplier, max sources, graph sufficiency thresholds, graph
    rows per source, and semantic-ignition integration, while
    `xiuxian-wendao` keeps only the `mode + julia_rerank` assembly wrapper in
    `resolve/policy/retrieval/base.rs`
20. `M2` now also owns the first analyzer contract extraction slice:
    `xiuxian-wendao-core/src/repo_intelligence/` holds repo-intelligence
    config records, plugin traits/context/output, record types, registry,
    `RepoIntelligenceError`, `ProjectionPageKind`, and the Julia Arrow
    analyzer transport column/schema contracts
21. that analyzer slice is now wired into both the monolithic host and the
    Julia package: the main-crate analyzer contract modules now re-export from
    `xiuxian-wendao-core`, and `xiuxian-julia-core` now imports those
    contracts from `core`
22. `M4` now owns the Julia link-graph launch/artifact compatibility slice:
    `xiuxian-julia-core/src/compatibility/link_graph/` now holds the Julia
    selector ids/helpers, `LinkGraphJuliaAnalyzerServiceDescriptor`,
    `LinkGraphJuliaAnalyzerLaunchManifest`,
    `LinkGraphJuliaDeploymentArtifact`, the Julia launch-option arg mapping,
    the default Julia analyzer launcher path, and the conversion boundary to
    and from Wendao core plugin contracts
23. the monolithic host now keeps
    `src/link_graph/runtime_config/models/retrieval/julia_rerank/{launch,artifact}.rs`
    only as compatibility re-export seams over that Julia-owned slice, while
    `runtime.rs` delegates Julia analyzer-launch arg encoding into the Julia
    crate
24. the remaining `M4` blockers were, until the previous slice, concentrated
    in still-hosted Julia runtime defaults and package-path semantics,
    especially `LinkGraphJuliaRerankRuntimeConfig` and package-path/default
    ownership
25. `M4` now owns the Julia package-path/default slice too:
    `xiuxian-julia-core/src/compatibility/link_graph/paths.rs` is now the
    physical owner of the default analyzer package dir, launcher path, and
    example-config path, while the host runtime/tests and integration fixtures
    consume those Julia-owned constants
26. `M4` now also owns the Julia rerank runtime record itself:
    `xiuxian-julia-core/src/compatibility/link_graph/runtime.rs` is now the
    physical owner of `LinkGraphJuliaRerankRuntimeConfig` and its
    provider-binding / launch / artifact normalization methods, while the host
    `runtime.rs` and `conversions.rs` files now behave as compatibility seams
27. as a result, the hard `M4` ownership blockers are now cleared and the
    next overall program move should be `M5` generic artifact cutover plus
    compatibility retirement sequencing
28. `M4` has now crossed the first dependency-rewrite milestone too:
    `xiuxian-julia-core` no longer depends on `xiuxian-wendao` directly and
    now builds against `xiuxian-wendao-core` plus `xiuxian-vector`
29. `M4` has now crossed the first host-integration milestone too:
    `src/analyzers/languages/mod.rs` no longer uses sibling-source inclusion
    for Julia and now loads `xiuxian-julia-core` through a normal crate
    dependency
30. `M6` has now landed its first additive plugin proof too:
    `xiuxian-wendao-modelica` now depends on
    `xiuxian-wendao-core::repo_intelligence` for production contracts, the
    host loads Modelica through a normal optional crate dependency instead of
    sibling-source inclusion, Modelica keeps `xiuxian-wendao` only as a
    dev-dependency for registry-aware integration-query validation, and the
    host `wendao-validation-gate` now carries a real builtin-registry
    Modelica repo-overview/module-search/example-search regression
31. that same `M6` proof is now two host consumers deep instead of one:
    the shared support topology under `tests/integration/support/` no longer
    compiles per-file repo helper copies, the historical
    `#[allow(dead_code)]` suppressions in `tests/support/repo_fixture.rs` and
    `tests/support/repo_intelligence.rs` are gone, and the builtin-registry
    Modelica path now has a second real host regression through
    `repo_symbol_search.rs`
32. `M6` has now reached a third host consumer too: the same external
    Modelica path now proves relation-graph output through
    `tests/integration/repo_relations.rs`, so the additive proof is no longer
    limited to overview/search-only consumers
33. `M6` has now reached projected-page lookup too: the external Modelica path
    now proves config-backed projected-page generation and page lookup through
    `tests/integration/repo_projected_page.rs`, so the additive proof has
    crossed from stage-one analysis/search consumers into stage-two docs
    projection
34. `M6` has now reached projected page-index trees too: the same external
    Modelica path now proves config-backed page-index tree generation and
    lookup through `tests/integration/repo_projected_page_index_tree.rs`, so
    the additive proof now covers parsed stage-two hierarchy output as well
35. `M6` has now reached projected page-index nodes too: the same external
    Modelica path now proves config-backed node lookup through
    `tests/integration/repo_projected_page_index_node.rs`, so the additive
    proof now covers stable subtree addressing inside parsed page hierarchies
36. `M6` has now reached page-centric navigation bundles too: the same
    external Modelica path now proves config-backed projected page navigation
    through `tests/integration/repo_projected_page_navigation.rs`, so the
    additive proof now covers assembled stage-two navigation around a real
    external plugin page
37. `M6` has now reached stage-two family context too: the same external
    Modelica path now proves config-backed projected page family context
    through `tests/integration/repo_projected_page_family_context.rs`, so the
    additive proof now covers grouped related-page families around a real
    external plugin page
38. `M6` has now reached singular family-cluster lookup too: the same
    external Modelica path now proves config-backed projected page family
    cluster lookup through `tests/integration/repo_projected_page_family_cluster.rs`,
    so the additive proof now covers direct family selection around a real
    external plugin page
39. `M6` has now reached search-driven family expansion too: the same
    external Modelica path now proves config-backed projected page family
    search through `tests/integration/repo_projected_page_family_search.rs`,
    so the additive proof now covers stable query-to-family expansion around a
    real external plugin page
40. `M6` has now reached search-driven navigation expansion too: the same
    external Modelica path now proves config-backed projected page navigation
    search through `tests/integration/repo_projected_page_navigation_search.rs`,
    so the additive proof now covers stable query-to-navigation bundle
    expansion around a real external plugin page
41. `M5` has now started with the first canonical generic outward artifact
    cutover: Studio routing and OpenAPI inventory now expose
    `/api/ui/plugins/{plugin_id}/artifacts/{artifact_id}` as the generic
    plugin-artifact endpoint family
42. the former Studio compat deployment-artifact route was initially narrowed
    into a wrapper over the generic plugin-artifact resolution/render path
    instead of owning primary outward implementation logic
43. `M5` has now expanded into Zhenfa too: the router now exposes
    `wendao.plugin_artifact` as the canonical generic selector-based
    tool/RPC surface
44. `M5` has now pushed the Studio UI payload seam further too:
    `UiPluginArtifact` is now the primary Studio artifact payload, while
    `UiJuliaDeploymentArtifact` stays under `types/compatibility/` and is
    built from the generic UI payload rather than directly from the core
45. `M5` has now retired the Julia-named Studio compatibility Rust symbols
    too: the compat route still preserves the legacy Julia-shaped JSON
    payload, but the remaining internal adapter is now compat-first rather
    than Julia-named
46. the canonical Studio schema-export seam now follows the same rule:
    `studio_type_collection()` and the `export_types` binary now register and
    compile only the generic artifact types, so the TypeScript-facing artifact
    schema path no longer needs the Julia DTO as a primary export
47. the remaining Julia UI DTO exposure has now been narrowed further too:
    `UiJuliaDeploymentArtifact` no longer rides through the compatibility
    namespace root and now survives only as route-local compat JSON
    adaptation in the deployment handler
48. the same `M5` cutover has now tightened the remaining Studio compatibility
    consumers too: router-level tests no longer deserialize
    `UiJuliaDeploymentArtifact` directly and instead assert the outward JSON
    payload through generic/value checks, leaving the legacy DTO shape
    coverage inside the compatibility leaf itself
49. the same `M5` cutover has now narrowed the compat handler seam further:
    the route layer no longer imports `UiJuliaDeploymentArtifact` directly and
    instead delegates legacy JSON shaping through a route-local wrapper over
    `UiPluginArtifact`
50. the same `M5` retirement path has now deleted the last test-only Studio
    Julia route/query shim too: `JuliaDeploymentArtifactQuery` and
    `get_julia_deployment_artifact` are gone, and legacy regression coverage
    now targets the compat handler directly
51. the same `M5` retirement path has now completed the OpenAPI Julia path
    alias removal too: `API_UI_JULIA_DEPLOYMENT_ARTIFACT_*` are gone from the
    codebase, and the route inventory now validates only the canonical plugin
    artifact path
52. the same `M5` retirement path has now completed the Zhenfa outward
    artifact retirement too: `wendao.julia_deployment_artifact` and
    `wendao.compat_deployment_artifact` are both gone from the live code
    path, so `wendao.plugin_artifact` is now the only Zhenfa artifact
    tool/RPC surface
53. the same `M5` retirement path has now completed the crate-root and
    `runtime_config` top-level Julia export retirement too: the Julia-named
    DTOs and deployment helpers no longer leak through flat crate-root or
    `runtime_config` root re-exports
54. the same `M5` retirement path had first retired the crate-root
    `src/compatibility/julia.rs` shim itself, temporarily narrowing the host
    compatibility surface down to `src/compatibility/link_graph.rs` before
    the final exit-review cut removed that last namespace as well
55. the same `M5` retirement path has now retired the last Julia-named Studio
    compatibility leaf path too: the dedicated Studio compatibility type
    module is gone, and the remaining legacy payload adapter is route-local in
    `src/gateway/studio/router/handlers/capabilities/deployment.rs`
56. that same route-local adapter has now narrowed one layer further too: the
    compat route no longer maintains a parallel Rust DTO and instead wraps the
    generic `UiPluginArtifact` into the legacy JSON shape at the serialization
    boundary
57. that same `M5` retirement path has now completed the Studio/OpenAPI UI
    artifact cutover too: the former `/api/ui/julia-deployment-artifact`
    compat route, query type, handler export, and OpenAPI inventory constants
    are gone from the live tree, so
    `/api/ui/plugins/{plugin_id}/artifacts/{artifact_id}` is now the only
    Studio UI artifact endpoint
58. the next overall program move no longer needs to chase outward artifact
    route/tool retirement; that work is complete on Studio/OpenAPI/Zhenfa
59. the remaining `M5` work is now limited to exit review, consumer cleanup,
    and package-owned Julia compatibility import cleanup
60. that same `M5` exit-review cut first retired the flat crate-root and
    `src/link_graph/mod.rs` compat-first re-export blocks
61. the final host crate-root compatibility namespace is now retired too:
    `src/compatibility/link_graph.rs`, `src/compatibility/mod.rs`, and the
    `pub mod compatibility;` mount in `src/lib.rs` are all gone, and the
    touched internal consumers now import Julia compatibility records from
    `xiuxian-julia-core::compatibility::link_graph::*`
62. the next phase transition has therefore already happened:
    `M6` additive plugin proof is now live in the Modelica path, not pending
    behind another host compatibility cycle
63. that same `M6` additive slice now reaches a docs-facing search consumer
    too: `tests/integration/docs_navigation_search.rs` proves
    config-backed `docs_navigation_search_from_config(...)` over the external
    Modelica path, so the additive proof now covers docs-facing
    query-to-navigation bundle expansion as well
64. that same `M6` additive slice now reaches the docs-facing family-search
    peer too: `tests/integration/docs_family_search.rs` proves
    config-backed `docs_family_search_from_config(...)` over the external
    Modelica path, so the additive proof now covers docs-facing
    query-to-family expansion as well
65. that same `M6` additive slice now reaches the docs-facing family-context
    peer too: `tests/integration/docs_family_context.rs` proves
    config-backed `docs_family_context_from_config(...)` over the external
    Modelica path, so the additive proof now covers docs-facing grouped
    family context around a stable external-plugin page as well
66. that same `M6` additive slice now reaches the docs-facing navigation
    lookup peer too: `tests/integration/docs_navigation.rs` proves
    config-backed `docs_navigation_from_config(...)` over the external
    Modelica path, so the additive proof now covers docs-facing deterministic
    navigation lookup with stable node context and family clustering as well
67. that same `M6` additive slice now reaches the docs-facing family-cluster
    lookup peer too: `tests/integration/docs_family_cluster.rs` proves
    config-backed `docs_family_cluster_from_config(...)` over the external
    Modelica path, so the additive proof now covers docs-facing deterministic
    family selection around a stable external-plugin reference page as well
68. that same `M6` additive slice now reaches the docs-facing page lookup
    peer too: `tests/integration/docs_page.rs` proves
    config-backed `docs_page_from_config(...)` over the external Modelica
    path, so the additive proof now covers docs-facing deterministic single
    page lookup over a stable external-plugin symbol page as well
69. that same `M6` additive slice now reaches the docs-facing page-index tree
    lookup peer too: `tests/integration/docs_page_index_tree.rs` proves
    config-backed `docs_page_index_tree_from_config(...)` over the external
    Modelica path, so the additive proof now covers docs-facing deterministic
    parsed page hierarchy lookup over a stable external-plugin symbol page as
    well
70. that same `M6` additive slice now reaches the docs-facing page-index node
    lookup peer too: `tests/integration/docs_page_index_node.rs` proves
    config-backed `docs_page_index_node_from_config(...)` over the external
    Modelica path, so the additive proof now covers docs-facing deterministic
    parsed page section lookup over a stable external-plugin symbol page as
    well
71. that same `M6` additive slice now reaches the docs-facing page-index tree
    search peer too: `tests/integration/docs_page_index_tree_search.rs`
    proves config-backed `docs_page_index_tree_search_from_config(...)` over
    the external Modelica path, so the additive proof now covers docs-facing
    deterministic parsed page hierarchy search over a stable external-plugin
    reference query as well
72. that same `M6` additive slice now reaches the docs-facing page-index
    trees peer too: `tests/integration/docs_page_index_trees.rs` proves
    config-backed `docs_page_index_trees_from_config(...)` over the external
    Modelica path, so the additive proof now covers docs-facing deterministic
    parsed page hierarchy listing over a stable external-plugin repository as
    well
73. that same `M6` additive slice now reaches the docs-facing page-index
    documents peer too: `tests/integration/docs_page_index_documents.rs`
    proves config-backed `docs_page_index_documents_from_config(...)` over
    the external Modelica path, so the additive proof now covers docs-facing
    parsed page-index-ready document generation over a stable external-plugin
    repository as well
74. that same `M6` additive slice now reaches the docs-facing markdown
    documents peer too: `tests/integration/docs_markdown_documents.rs`
    proves config-backed `docs_markdown_documents_from_config(...)` over the
    external Modelica path, so the additive proof now covers docs-facing
    projected markdown document generation over a stable external-plugin
    repository as well
75. that same `M6` additive slice now reaches the docs-facing search peer
    too: `tests/integration/docs_search.rs` proves config-backed
    `docs_search_from_config(...)` over the external Modelica path, so the
    additive proof now covers docs-facing projected page search over a stable
    external-plugin repository as well
76. that same `M6` additive slice now reaches the docs-facing retrieval peer
    too: `tests/integration/docs_retrieval.rs` proves config-backed
    `docs_retrieval_from_config(...)` over the external Modelica path, so the
    additive proof now covers docs-facing mixed projected retrieval over a
    stable external-plugin repository as well
77. that same `M6` additive slice now reaches the docs-facing retrieval-
    context peer too: `tests/integration/docs_retrieval_context.rs` proves
    config-backed `docs_retrieval_context_from_config(...)` over the external
    Modelica path, so the additive proof now covers docs-facing local
    projected retrieval context over a stable external-plugin repository as
    well
78. that same `M6` additive slice now reaches the docs-facing retrieval-hit
    peer too: `tests/integration/docs_retrieval_hit.rs` proves config-backed
    `docs_retrieval_hit_from_config(...)` over the external Modelica path, so
    the additive proof now covers docs-facing deterministic projected
    retrieval-hit reopening over a stable external-plugin repository as well
79. that same `M6` additive slice now reaches the docs-facing projected-gap
    report peer too: `tests/integration/docs_projected_gap_report.rs` proves
    config-backed `docs_projected_gap_report_from_config(...)` over the
    external Modelica path, so the additive proof now covers docs-facing
    projected gap reporting over a stable external-plugin repository as well
80. that same `M6` additive slice now reaches the docs-facing planner-queue
    peer too: `tests/integration/docs_planner_queue.rs` proves config-backed
    `docs_planner_queue_from_config(...)` over the external Modelica path, so
    the additive proof now covers docs-facing deterministic planner queue
    shaping over a stable external-plugin repository as well
81. that same `M6` additive slice now reaches the docs-facing planner-workset
    peer too: `tests/integration/docs_planner_workset.rs` proves config-backed
    `docs_planner_workset_from_config(...)` over the external Modelica path,
    so the additive proof now covers docs-facing deterministic planner
    workset shaping over a stable external-plugin repository as well
82. that same `M6` additive slice now reaches the docs-facing planner-rank
    peer too: `tests/integration/docs_planner_rank.rs` proves config-backed
    `docs_planner_rank_from_config(...)` over the external Modelica path, so
    the additive proof now covers docs-facing deterministic planner ranking
    over a stable external-plugin repository as well
83. that same `M6` additive slice now reaches the docs-facing planner-item
    peer too: `tests/integration/docs_planner_item.rs` proves config-backed
    `docs_planner_item_from_config(...)` over the external Modelica path, so
    the additive proof now covers docs-facing deterministic planner item
    reopening over a stable external-plugin repository as well
84. that same `M6` additive slice now reaches the docs-facing planner-search
    peer too: `tests/integration/docs_planner_search.rs` proves config-backed
    `docs_planner_search_from_config(...)` over the external Modelica path,
    so the additive proof now covers docs-facing deterministic planner search
    over a stable external-plugin repository as well
85. that same `M6` additive slice now reaches the Studio docs route layer
    too: the `studio_repo_sync_api` lib-test module now proves
    `/api/docs/planner-search` over the external Modelica plugin path, so the
    additive proof is no longer limited to analyzer-entry consumers; the same
    slice also remounts `tests/support/repo_fixture.rs` next to
    `repo_intelligence.rs` inside `src/analyzers/service/projection/tests.rs`
    so the shared lib-test projection fixture path keeps compiling after the
    test-support topology cleanup
86. that same `M6` additive slice now reaches a second Studio docs route peer
    too: the `studio_repo_sync_api` lib-test module now proves
    `/api/docs/planner-item` over the external Modelica plugin path by
    reopening a stable gap id sourced from the docs-facing projected-gap
    report, so the gateway-layer additive proof now covers deterministic
    planner-gap reopening as well as planner search
87. that same `M6` additive slice now reaches a third Studio docs route peer
    too: the `studio_repo_sync_api` lib-test module now proves
    `/api/docs/planner-workset` over the external Modelica plugin path by
    filtering the selection onto the injected `NoDocs` reference gap, so the
    gateway-layer additive proof now covers deterministic planner-workset
    shaping as well as planner search and planner-item reopening
88. that same `M6` additive slice now reaches a fourth Studio docs route peer
    too: the `studio_repo_sync_api` lib-test module now proves
    `/api/docs/planner-rank` over the external Modelica plugin path by
    filtering the selection onto the injected `NoDocs` reference gap, so the
    gateway-layer additive proof now covers deterministic planner ranking as
    well as planner search, planner-item reopening, and planner-workset
    shaping
89. that same `M6` additive slice now reaches a fifth Studio docs route peer
    too: the `studio_repo_sync_api` lib-test module now proves
    `/api/docs/planner-queue` over the external Modelica plugin path by
    filtering the selection onto the injected `NoDocs` reference gap, so the
    gateway-layer additive proof now covers deterministic planner queue
    shaping as well as planner search, planner-item reopening, planner-
    workset shaping, and planner ranking
90. that same `M6` additive slice now exits the Studio planner subtree too:
    the `studio_repo_sync_api` lib-test module now proves `/api/docs/search`
    over the external Modelica plugin path, so the gateway-layer additive
    proof now reaches the first non-planner docs-facing route family as well
91. that same `M6` additive slice now extends the non-planner Studio docs
    route family too: the `studio_repo_sync_api` lib-test module now proves
    `/api/docs/retrieval` over the external Modelica plugin path, so the
    gateway-layer additive proof now covers mixed docs-facing retrieval as
    well as plain docs search
92. that same `M6` additive slice now pushes one level deeper into the
    non-planner Studio docs route family: the `studio_repo_sync_api`
    lib-test module now proves `/api/docs/retrieval-context` over the
    external Modelica plugin path, so the gateway-layer additive proof now
    covers deterministic node-context reopening as well as mixed retrieval
    and plain docs search
93. that same `M6` additive slice now closes the sibling deterministic
    reopening peer too: the `studio_repo_sync_api` lib-test module now
    proves `/api/docs/retrieval-hit` over the external Modelica plugin
    path, so the gateway-layer additive proof now covers deterministic hit
    reopening as well as node-context reopening, mixed retrieval, and plain
    docs search
94. that same `M6` additive slice now closes the deterministic page-lookup
    peer too: the `studio_repo_sync_api` lib-test module now proves
    `/api/docs/page` over the external Modelica plugin path, so the
    gateway-layer additive proof now covers deterministic docs page lookup
    alongside hit reopening, node-context reopening, mixed retrieval, and
    plain docs search
95. that same `M6` additive slice now closes the deterministic family-context
    peer too: the `studio_repo_sync_api` lib-test module now proves
    `/api/docs/family-context` over the external Modelica plugin path, so
    the gateway-layer additive proof now covers grouped family-context
    reopening alongside page lookup, hit reopening, node-context reopening,
    mixed retrieval, and plain docs search
96. that same `M6` additive slice now closes the deterministic family-search
    peer too: the `studio_repo_sync_api` lib-test module now proves
    `/api/docs/family-search` over the external Modelica plugin path, so
    the gateway-layer additive proof now covers grouped family-search
    expansion alongside family-context reopening, page lookup, hit
    reopening, node-context reopening, mixed retrieval, and plain docs
    search
97. that same `M6` additive slice now closes the deterministic family-cluster
    peer too: the `studio_repo_sync_api` lib-test module now proves
    `/api/docs/family-cluster` over the external Modelica plugin path, so
    the gateway-layer additive proof now covers single-family reopening
    alongside family-search expansion, family-context reopening, page
    lookup, hit reopening, node-context reopening, mixed retrieval, and
    plain docs search
98. that same `M6` additive slice now closes the deterministic navigation
    peer too: the `studio_repo_sync_api` lib-test module now proves
    `/api/docs/navigation` over the external Modelica plugin path, so the
    gateway-layer additive proof now covers tree-context plus family-cluster
    reopening alongside single-family reopening, family-search expansion,
    family-context reopening, page lookup, hit reopening, node-context
    reopening, mixed retrieval, and plain docs search
99. that same `M6` additive slice now closes the deterministic navigation-
    search peer too: the `studio_repo_sync_api` lib-test module now proves
    `/api/docs/navigation-search` over the external Modelica plugin path, so
    the gateway-layer additive proof now covers grouped navigation-bundle
    expansion alongside deterministic navigation, single-family reopening,
    family-search expansion, family-context reopening, page lookup, hit
    reopening, node-context reopening, mixed retrieval, and plain docs
    search
100. that same `M6` additive slice now closes the docs projected-gap-report
     peer too: the `studio_repo_sync_api` lib-test module now proves
     `/api/docs/projected-gap-report` over the external Modelica plugin
     path, so the gateway-layer additive proof now covers docs-facing gap
     reporting alongside grouped navigation-bundle expansion, deterministic
     navigation, single-family reopening, family-search expansion,
     family-context reopening, page lookup, hit reopening, node-context
     reopening, mixed retrieval, and plain docs search
101. that same `M6` additive slice now exits the Studio docs route family
     and opens the sibling Studio repo route family too: the
     `studio_repo_sync_api` lib-test module now proves
     `/api/repo/overview` over the external Modelica plugin path, so the
     gateway-layer additive proof now covers repo-summary reopening at the
     same outward layer as well
102. that same `M6` additive slice now closes the sibling Studio repo
     module-search peer too: the `studio_repo_sync_api` lib-test module now
     proves `/api/repo/module-search` over the external Modelica plugin
     path, so the gateway-layer additive proof now covers deterministic
     module-search reopening at that same outward layer as well
103. that same `M6` additive slice now closes the sibling Studio repo
     symbol-search peer too: the `studio_repo_sync_api` lib-test module now
     proves `/api/repo/symbol-search` over the external Modelica plugin
     path, so the gateway-layer additive proof now covers deterministic
     symbol-search reopening at that same outward layer as well
104. that same `M6` additive slice now closes the sibling Studio repo
     example-search peer too: the `studio_repo_sync_api` lib-test module now
     proves `/api/repo/example-search` over the external Modelica plugin
     path, so the gateway-layer additive proof now covers deterministic
     example-search reopening at that same outward layer as well
105. that same `M6` additive slice now closes the sibling Studio repo
     doc-coverage peer too: the `studio_repo_sync_api` lib-test module now
     proves `/api/repo/doc-coverage` over the external Modelica plugin
     path, so the gateway-layer additive proof now covers deterministic
     module-scoped doc-coverage reopening at that same outward layer as well
106. that same `M6` additive slice now pushes through the sibling Studio repo
     lifecycle-and-projection peers as a batch: the
     `studio_repo_sync_api` lib-test module now proves `/api/repo/sync`,
     `/api/repo/projected-pages`, and `/api/repo/projected-gap-report` over
     the external Modelica plugin path, so the gateway-layer additive proof
     now covers repo status reopening, projected-page enumeration, and
     projected-gap reporting at that same outward layer as well
107. that same `M6` additive slice now pushes through the deterministic
     sibling Studio repo projected reopen family as a batch too: the
     `studio_repo_sync_api` lib-test module now proves
     `/api/repo/projected-page`, `/api/repo/projected-page-index-tree`,
     `/api/repo/projected-page-index-node`, `/api/repo/projected-retrieval-hit`,
     and `/api/repo/projected-retrieval-context` over the external Modelica
     plugin path, so the gateway-layer additive proof now covers stable
     symbol-page reopening, tree reopening, node reopening, deterministic hit
     reopening, and node-context reopening at that same outward layer as well
108. that same `M6` additive slice now closes the remaining sibling Studio
     repo projected query-and-navigation family as a batch too: the
     `studio_repo_sync_api` lib-test module now proves
     `/api/repo/projected-page-index-tree-search`,
     `/api/repo/projected-page-search`, `/api/repo/projected-retrieval`,
     `/api/repo/projected-page-family-context`,
     `/api/repo/projected-page-family-search`,
     `/api/repo/projected-page-family-cluster`,
     `/api/repo/projected-page-navigation`,
     `/api/repo/projected-page-navigation-search`, and
     `/api/repo/projected-page-index-trees` over the external Modelica
     plugin path, so the gateway-layer additive proof now also covers
     deterministic section search, projected page search, mixed projected
     retrieval, family-context reopening, family-search expansion,
     single-family reopening, navigation-bundle reopening,
     navigation-search expansion, and projected tree listing at that same
     outward layer as well
109. the current post-`Phase 8` bounded remediation lane is now
     `xiuxian-daochang` crate-health rather than another governance phase;
     the latest landed slice removed the first telegram/discord internal
     visibility and re-export drift bundle from the compile front, so the
     active next blockers now start in `llm/*`, `resolve.rs`,
     `runtime_agent_factory/*`, and `agent` test-support private-module
     drift
110. that same post-`Phase 8` remediation lane has now removed the stale
     `xiuxian-daochang` `llm/*` import and owner drift bundle too: the
     local chat implementation now binds to the live `client/mod.rs` owner,
     LiteLLM type imports now follow the current `chat/message/tools/context`
     split, and DeepSeek OCR helpers now follow the package-owned
     `vision::deepseek::*` surface; the active next blockers now start in
     `resolve.rs`, `runtime_agent_factory/*`, `session/*`,
     `test_support/*`, and root outward re-export drift in `lib.rs`
111. that same post-`Phase 8` remediation lane has now removed the
     `xiuxian-daochang` `runtime_agent_factory/*` owner drift bundle too:
     the duplicate memory-runtime owner was replaced with the live
     runtime-settings applicator, the embedding backend negotiation now
     handles the `mistral_sdk` branch explicitly, and the compile front no
     longer stops in `runtime_agent_factory/*`; the active next blockers now
     start in `agent/bootstrap/zhixing.rs`,
     `agent/turn_execution/react_loop/*`, and
     `session/bounded_store/window_ops/*`
112. that same post-`Phase 8` remediation lane has now removed the
     `xiuxian-daochang` `zhixing` reminder/bootstrap owner drift bundle too:
     reminder queue backfill, queue-aware reminder polling, and reminder
     notice rendering now live on `xiuxian-zhixing::ZhixingHeyi`, while
     `xiuxian-daochang` bootstrap only consumes that owner surface. The
     targeted `xiuxian-zhixing` reminder unit tests pass, and the
     `xiuxian-daochang --lib` compile front no longer stops in
     `agent/bootstrap/zhixing.rs`; the active next blockers now start in
     `agent/turn_execution/react_loop/*` and
     `session/bounded_store/window_ops/*`
113. that same post-`Phase 8` remediation lane has now removed the
     `xiuxian-daochang` `agent/turn_execution/react_loop/*` owner/import
     drift bundle too: the touched scope now uses explicit crate-owned
     imports instead of wildcard or deep relative imports, the live
     tool-dispatch call sites match current runtime signatures, and the
     `turn_store` owner seam is mounted again so the react loop can see the
     live append-turn path. The `xiuxian-daochang --lib` compile front no
     longer stops in `agent/turn_execution/react_loop/*`; the active next
     blockers now start in `agent/persistence/turn_store/*`,
     `agent/session_context/window_ops/*`, and
     `session/bounded_store/window_ops/*`
114. that same post-`Phase 8` remediation lane has now removed the next
     Discord/Telegram runtime owner-drift bundle too: the live Discord
     channel owner again carries
     `new_with_partition_and_control_command_policy(...)`, gateway/run loops
     now borrow the live join handle and emit the current admission snapshot
     shape, Telegram send/runtime now owns the missing send-rate gate and
     chunk-send helpers, `SessionGate` owns the shared acquire/drop seam,
     and the touched runtime/router leaves have been rebased from
     `super::super::...` to explicit crate-owned imports. The
     `xiuxian-daochang --lib` compile front no longer stops in Discord
     runtime gateway/run ownership or Telegram send/runtime drift; the
     active next blockers now start in `gateway/http/*`,
     `agent/injection/*`, `agent/native_tools/zhixing.rs`,
     `agent/zhenfa/bridge.rs`, Telegram ACL/settings, Telegram
     session-memory reply shaping, and test-support seams

## Scope Correction

The recent `xiuxian-daochang` crate-health work is workspace-adjacent, but it
is not authoritative Wendao-plugin-program phase progress unless it directly
unblocks a Wendao-owned phase gate, a Wendao package build, or a Wendao
outward contract.

## Post-Phase-8 Program Move

Authoritative next macro-phase target:

1. `Phase 9: Core and Runtime Consumer Cutover`

Intent:

1. the extracted `core` and `runtime` crates now exist, but the program still
   records incomplete consumer cutover
2. the next governed work must therefore identify and reduce the remaining
   live consumers that still depend on monolith-era `xiuxian-wendao` owner
   seams

Phase-9 staged push plan:

1. `Stage A: Consumer Reality Inventory Bundle`
   identify the live Wendao-owned consumers that still depend on monolith-era
   host contracts or runtime behavior instead of
   `xiuxian-wendao-core` / `xiuxian-wendao-runtime`
2. `Stage B: Bounded Consumer Cutover Bundle`
   move one bounded consumer family at a time onto the extracted owner seam
3. `Stage C: Compatibility Contraction and Gate Bundle`
   review which transitional host re-exports can now contract, then record an
   explicit `Phase 9` gate decision

Current staged position:

1. `Phase 9` is opened
2. `Stage A` is complete
3. `Stage B` is the next authoritative move

Phase-9 Stage-A inventory findings:

1. live monolith-era `xiuxian-wendao` direct dependencies still exist in
   `xiuxian-qianji`, `xiuxian-zhixing`, and `xiuxian-daochang`, with an
   optional monolith dependency still present in `xiuxian-qianhuan`
2. `xiuxian-wendao-modelica` already uses `xiuxian-wendao-core` for
   production code, but still retains a monolith dev-dependency for
   integration tests
3. no surveyed sibling consumer crate currently imports
   `xiuxian_wendao_core::...` or `xiuxian_wendao_runtime::...` directly in
   Rust source, which means the main consumer cutover is still pending
4. the most bounded first Stage-B cutover candidate is the resource/VFS
   family:
   - `SkillRuntimeResolver`
   - `WendaoResourceUri`
   - `embedded_resource_text_from_wendao_uri`
   - `WendaoResourceRegistry`

Stage-B starting boundary:

1. begin with the resource/VFS family because it is physically concentrated,
   still re-exported from the monolith crate root, and consumed by multiple
   sibling crates
2. the first landed `Stage B` slice now rebases source consumers in
   `xiuxian-qianhuan`, `xiuxian-qianji`, and `xiuxian-daochang` from crate
   root imports onto:
   - `xiuxian_wendao::skill_runtime::*`
   - `xiuxian_wendao::enhancer::WendaoResourceRegistry`
3. the follow-up test-level slice for the same family is also landed
4. root-qualified imports for this family are now cleared across the touched
   `src/` and `tests/` scope
5. the same family is now also cleared for Wendao's own internal unit-test
   consumer surface
6. the next bounded ingress/spider family slice is now also landed across
   `xiuxian-daochang` source and test consumers
7. those consumers now use the owner seam
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
13. bounded verification is clean on the seam:
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
    `ZhixingIndexSummary`, so the live `ZhixingWendaoIndexer` summary again
    matches downstream projection expectations
17. root-qualified imports for `ZhixingIndexSummary` and
    `ZhixingWendaoIndexer` are now cleared across the workspace `packages/**`
    Rust source and test scope
18. bounded verification is clean on the seam:
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
21. the cutover required one bounded visibility fix in the host crate:
    `xiuxian-wendao/src/lib.rs` now exports `pub mod types;` so the owner
    seam is physically reachable without falling back to crate-root
    re-exports
22. root-qualified imports for that family are now cleared across the
    touched `xiuxian-qianji` `src/` and `tests/` scope
23. bounded verification is clean on that seam:
    `xiuxian-qianji --lib`,
    `xiuxian-qianji --tests --no-run`, and
    `xiuxian-wendao --test wendao-validation-gate --no-run` pass
24. the next bounded graph-primitive slice is now also landed across the
    touched `xiuxian-qianji` and `xiuxian-zhixing` source/test consumers
25. those touched consumers now use the owner seams:
    `xiuxian_wendao::entity::{Entity, EntityType, Relation, RelationType}`
    and `xiuxian_wendao::graph::KnowledgeGraph`
26. the touched `xiuxian-zhixing/tests/test_strict_teacher.rs` seam now also
    matches the live APIs by using a local `ManifestationInterface` stub and
    the current `ZhixingHeyi::add_task(title, scheduled_at)` signature
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
38. `Stage B` remains open because the next bounded move is still the next
    consumer family cutover; crate-root re-export contraction stays in
    `Stage C`
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
44. `Stage B` remains open because the next move should still be another
    small bounded consumer family rather than a broad `LinkGraphIndex` cut
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
49. `Stage B` remains open because the next move should still be another
    small bounded consumer family that stays off a broad `LinkGraphIndex`
    cut
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
55. `Stage B` remains open because the next move should still be another
    small bounded consumer family that stays off a broad `LinkGraphIndex`
    cut
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
61. `Stage B` remains open because the next move should still be another
    small bounded consumer family that stays off a broad `LinkGraphIndex`
    cut
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
66. `Stage B` remains open because the next move should still be another
    small bounded consumer family that stays off a broad `LinkGraphIndex`
    cut
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
72. `Stage B` remains open, and this does not authorize a broad
    `LinkGraphIndex` migration across app/runtime surfaces
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
78. `Stage B` remains open, and the next move should still be another small
    bounded consumer family rather than a broad `LinkGraphIndex` cut
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
82. `Stage B` remains open, and the next move should still be another small
    bounded consumer family rather than a broad `LinkGraphIndex` cut
83. the same-port pure Flight search cut is now complete at the search
    business boundary:
    - `/api/search` is removed from the active router/OpenAPI surface
    - `/search/knowledge` is the only active knowledge-search business
      contract
    - the frontend source tree now relies on the Flight business routes rather
      than on `/api/search` or `/api/search/*/hits-arrow`
84. the same bounded cleanup also closes the touched warning front:
    - `SearchQuery` is test-only
    - dead references/symbols provider wrappers and batch-helper shims are
      removed
85. the runtime transport-server modularization slice is now complete:
    - retired `xiuxian-wendao-runtime/src/transport/server.rs`
    - added
      `xiuxian-wendao-runtime/src/transport/server/{mod,types,request_metadata,service,tests}.rs`
86. fresh same-port verification on a newly built `127.0.0.1:9519` gateway is
    clean:
    - `/api/health -> 200`
    - `FlightService/GetFlightInfo -> 400`
    - `/api/search -> 404`
    - frontend same-origin live Flight proof passes `4/4`
87. the same search-boundary cut is now complete for definition and
    autocomplete as well:
    - `/search/definition` and `/search/autocomplete` are active Flight
      business contracts in the runtime query contract and shared Flight
      snapshot
    - `/api/search/definition` and `/api/search/autocomplete` are removed from
      the router and bundled OpenAPI document
    - `.data/wendao-frontend` now routes both flows through same-origin
      Flight via `src/api/flightDocumentTransport.ts`
88. the governed next slice is therefore no longer search-definition cleanup;
    it is the first bounded `graph/vfs` Flight migration cut
89. that first bounded `graph/vfs` Flight migration cut is now landed:
    - `/vfs/resolve` is now a runtime-owned business contract and part of the
      shared workspace Flight snapshot
    - the Studio-backed provider seam lives in
      `src/gateway/studio/vfs/flight.rs` and is wired through
      `WendaoFlightService::new_with_route_providers(...)`
    - `/api/vfs/resolve` is removed from the outward router and bundled
      OpenAPI surface
    - `.data/wendao-frontend` now resolves Studio navigation targets through
      same-origin Flight via `src/api/flightWorkspaceTransport.ts`
90. that next bounded `graph/vfs` Flight migration cut is now landed too:
    - `/graph/neighbors` is now a runtime-owned business contract and part of
      the shared workspace Flight snapshot
    - the Studio-backed provider seam lives in
      `src/gateway/studio/router/handlers/graph/flight.rs` and is wired
      through `WendaoFlightService::new_with_route_providers(...)`
    - `/api/graph/neighbors/{id}` is removed from the outward router and
      bundled OpenAPI surface
    - `.data/wendao-frontend` now resolves graph-neighbor payloads through
      same-origin Flight via `src/api/flightGraphTransport.ts`
91. the governed next slice is therefore no longer graph-neighbor cutover; it
    is the remaining `graph/vfs` family utility surface, followed by the
    larger docs/repo families
92. the first remaining `graph/vfs` utility retirement slice is now landed:
    - dead legacy `/api/neighbors/{id}` is removed from the outward router and
      bundled OpenAPI document
    - backend `node_neighbors` handler/type/export residue is removed from the
      active Studio router surface
    - `.data/wendao-frontend` no longer exposes the unused `NodeNeighbors`
      client transport surface
    - canonical `/graph/neighbors` Flight behavior is unchanged, and
      `/api/topology/3d` plus `/api/vfs*` remain the next bounded utility
      audit surfaces

## Landed Slice: Modelica Docs Integration File Gating

Macro phase:

1. `M6` additive plugin proof follow-through

Gate:

1. plugin-specific integration-test execution hygiene

Ownership seam:

1. plugin-specific integration tests should disappear cleanly from non-plugin
   lanes instead of compiling as half-empty modules that emit warning noise

The landed changes are:

1. add file-level `feature = "modelica"` gating to the bounded
   `tests/integration/docs_*` family
2. keep the slice out of query/runtime implementation and out of the
   Julia-specific plugin-first lane
3. rerun a focused non-plugin `wendao-validation-gate` probe and confirm the
   `docs_*` warning front disappears while test semantics remain unchanged

## Landed Slice: Modelica Repo Integration Warning Closure

Macro phase:

1. `M6` additive plugin proof follow-through

Gate:

1. plugin-specific mixed integration-test warning closure

Ownership seam:

1. mixed repo integration files may keep generic Julia coverage visible in the
   non-plugin lane, but Modelica-only imports, helpers, and support fixtures
   must not stay unguarded once their proof cases are `feature = "modelica"`
   gated

The landed changes are:

1. gate Modelica-only imports and helper usage in the bounded
   `tests/integration/repo_*` files that still mix generic and Modelica test
   cases
2. gate `create_sample_modelica_repo` in the shared test-support helpers so
   non-plugin lanes no longer compile dead Modelica-only setup code
3. rerun a focused non-plugin `wendao-validation-gate` warning probe and confirm
   the remaining `repo_*` / `tests/support/repo_*` warning front narrows
   without changing plugin semantics

## Landed Slice: Builtin Plugin Self-Registration

Macro phase:

1. `M4` external plugin ownership follow-through

Gate:

1. builtin plugin bootstrap ownership

Ownership seam:

1. plugin crates should declare their own builtin registration surface, while
   `xiuxian-wendao` should only consume one generic registrar inventory instead
   of naming each plugin crate directly in host bootstrap code

The landed changes are:

1. add a generic builtin-plugin registrar contract to the common repo-intelligence
   surface
2. make Julia and Modelica submit their own builtin registrars from the plugin
   crates
3. reduce `xiuxian-wendao` bootstrap to a generic registrar iteration path so
   adding a new plugin dependency no longer requires editing host registration
   code

## Landed Slice: Host Plugin Facade Thinning

Macro phase:

1. `M4` external plugin ownership follow-through

Gate:

1. host public plugin-facade ownership

Ownership seam:

1. `xiuxian-wendao` should stop re-exporting plugin-owned registration and
   transport-heavy Julia or Modelica APIs from `analyzers/*`; the host should
   keep only generic bootstrap or domain-facing surfaces, while plugin crates
   remain the owner of plugin-specific thick APIs

The landed changes are:

1. remove plugin-registration and thick Julia transport facade re-exports from
   the bounded `src/analyzers/languages/`, `src/analyzers/service/`, and
   `src/analyzers/mod.rs` host seams
2. switch the remaining in-repo Julia graph-structural proof and the bounded
   host link-graph transport call sites to direct `xiuxian-julia-core`
   imports instead of routing through `xiuxian-wendao`
3. add a host-private builtin-plugin link anchor so Julia and Modelica
   inventory-submitted registrars still link into the host without reopening
   the public facade
4. rerun focused host bootstrap, Julia proof, and non-Julia compile probes to
   confirm the public facade cut lands without a new touched-scope warning
   front

## Landed Slice: Builtin Plugin Link Anchor Extraction

Macro phase:

1. `M4` external plugin ownership follow-through

Gate:

1. host registration-thin-layer clarity

Ownership seam:

1. the host-private builtin-plugin link anchor should live in a dedicated
   registration seam, not in `analyzers::languages`, so the host registration
   layer stays explicit and `languages` stops carrying plugin-bootstrap
   semantics

The landed changes are:

1. extract the builtin-plugin link anchor from `src/analyzers/languages/` into
   one dedicated host-private registration seam
2. make `bootstrap_builtin_registry()` depend on that dedicated seam instead
   of on the misleading `languages` namespace
3. rerun focused bootstrap and non-plugin probes to confirm the registration
   thin layer stays linked and warning-clean in the touched scope

## Landed Slice: Builtin Plugin Bundle Crate Extraction

Macro phase:

1. `M4` external plugin ownership follow-through

Gate:

1. builtin plugin registration independence

Ownership seam:

1. `xiuxian-wendao` should not remain the owner of builtin-plugin link
   anchoring and generic bootstrap assembly; that host-specific plugin bundle
   should move into one dedicated crate so the host consumes one registration
   package instead of directly naming builtin plugin crates

The landed changes are:

1. add `xiuxian-wendao-builtin` as one dedicated builtin-plugin bundle crate
   that owns the generic bootstrap loop plus private builtin-plugin linking
2. move the current host bootstrap implementation and link seam into that new
   crate, then delete the old `analyzers/builtin_plugins/` host seam
3. rewire `xiuxian-wendao` feature forwarding and
   `analyzers::service::bootstrap` to consume the bundle crate instead of
   carrying builtin plugin linkage locally
4. rerun default and feature-enabled bundle tests plus host bootstrap and
   non-Julia compile probes to confirm the ownership cut lands without a new
   touched-scope regression

## Landed Slice: Generic Plugin Arrow Exchange Runtime Extraction

Macro phase:

1. `M3` runtime extraction follow-through

Gate:

1. host direct plugin-import thinning

Ownership seam:

1. the generic `WendaoArrow` request or response batch helpers used by host
   rerank execution should not remain owned by `xiuxian-julia-core`; runtime
   should own those transport-contract helpers while the Julia plugin keeps
   repository config parsing and remote transport bootstrap

The landed changes are:

1. add one runtime-owned `transport/plugin_arrow_exchange/` feature folder that
   owns `PluginArrowRequestRow`, `PluginArrowScoreRow`, request-batch
   assembly, response decoding, cross-batch response validation, and focused
   transport-contract tests
2. rewire `xiuxian-wendao` production consumers in quantum-fusion and rerank
   execution to consume those runtime-owned helpers instead of importing the
   generic `WendaoArrow` helper set from `xiuxian-julia-core`
3. rewire `xiuxian-julia-core` to keep only Julia-specific repository
   config parsing, remote Flight bootstrap, and repository fetch ownership,
   while deleting the old crate-root and module-local `JuliaArrow*`,
   `build_julia_arrow_request_batch(...)`,
   `decode_julia_arrow_score_rows(...)`, and
   `validate_julia_arrow_response_batches(...)` facade exports
4. rerun runtime, plugin, host quantum, non-Julia compile, and diff checks to
   confirm the owner cut lands without a touched-scope regression

## Governance Rule

Any future implementation note, ExecPlan, or code slice that affects this
program should explicitly state:

1. macro phase
2. gate
3. ownership seam
4. compatibility impact

If it does not, it is not yet ready to be treated as migration-program work.

## Landed Slice: Builtin Plugin Artifact Dispatch Extraction

Macro phase:

1. `M4` Julia ownership externalization follow-through

Gate:

1. host direct plugin-import thinning

Ownership seam:

1. builtin plugin artifact selector dispatch for link-graph deployment
   artifacts should not remain owned by `xiuxian-wendao`; the builtin bundle
   crate should own builtin-plugin artifact resolution and render dispatch so
   the host stops importing Julia compatibility artifact helpers directly

The landed changes are:

1. add one builtin-owned `src/artifacts/` feature folder in
   `xiuxian-wendao-builtin` that owns builtin deployment-artifact selector
   dispatch plus focused bundle tests
2. move the current Julia deployment-artifact selector, resolve, and TOML
   render dispatch out of `xiuxian-wendao/src/link_graph/runtime_config/artifacts.rs`
   so the host production artifact seam now calls into the builtin bundle
3. rewire gateway and Zhenfa deployment consumers to keep calling the same
   host runtime-config surface while that surface delegates builtin-plugin
   artifact dispatch to `xiuxian-wendao-builtin`
4. keep Julia-specific deployment-artifact payload shaping inside
   `xiuxian-julia-core`, and confirm the non-Julia host compile probe still
   passes after the new builtin artifact surface is feature-gated to Julia

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Host Plugin Cargo Dependency Retirement

Macro phase:

1. `M4` Julia and Modelica host-thinning follow-through

Gate:

1. host manifest ownership thinning

Ownership seam:

1. `xiuxian-wendao/Cargo.toml` should not keep direct builtin plugin
   dependency ownership once the host crate no longer imports Julia or
   Modelica plugin crates directly; the builtin bundle should own that
   forwarding seam, including the bounded Modelica `tree-sitter` feature
   bridge

The landed changes are:

1. add one builtin-owned feature-forwarding bridge for the remaining
   Modelica `tree-sitter` seam
2. remove the host manifest's direct `dep:xiuxian-julia-core` and
   `dep:xiuxian-wendao-modelica` wiring while keeping the host-facing
   `julia`, `modelica`, and `tree-sitter` feature names stable
3. re-validate plugin-enabled and non-default host compile probes and confirm
   the direct plugin dependency declarations disappear from the host manifest

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Quantum Fusion Plugin Request Wrapper Ownership Cutover

Macro phase:

1. `M4` host transport-thinning follow-through

Gate:

1. host plugin-wrapper thinning

Ownership seam:

1. `link_graph/index/search/plan/payload/quantum/rerank.rs` should not retain
   backend-specific request wrapper logic for OpenAI-compatible and vector-store
   semantic ignition; those wrappers belong with the ignition owners in
   `quantum_fusion/*_ignition.rs`
2. the legacy `build_julia_rerank_request_batch` compatibility shims in the
   ignition adapters no longer have live callers and should retire

The landed changes are:

1. move backend-specific request-wrapper ownership from host `rerank.rs` into
   `openai_ignition.rs` and `vector_ignition.rs`
2. delete the unused Julia-named compatibility request-builder shims
3. keep host `rerank.rs` focused on domain orchestration, transport roundtrip,
   telemetry, and score application
4. close the touched-scope warning surfaced during validation by removing the
   stale unused `RecordBatch` import from host `rerank.rs`

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Runtime Plugin Arrow Provider-Aware Trace Id Cutover

Macro phase:

1. `M4` host transport-thinning follow-through

Gate:

1. plugin transport ownership thinning

Ownership seam:

1. `xiuxian-wendao-runtime` owns the plugin arrow request metadata seam, but
   `plugin_arrow_request_trace_id(...)` still hardcodes the Julia-specific
   `julia-rerank:*` prefix
2. the runtime seam should build one provider-aware generic trace id and let
   host ignition adapters thread the negotiated provider id through, instead
   of baking plugin-specific identity into runtime

The landed changes are:

1. change the runtime request trace-id helper so it accepts one provider id and
   emits one generic provider-aware trace id
2. thread the provider id through `openai_ignition.rs`, `vector_ignition.rs`,
   and host `quantum/rerank.rs`
3. update focused runtime, quantum-fusion, and integration proof so they assert
   the provider-aware trace id instead of the Julia-specific prefix

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Runtime Plugin Arrow Metadata-Aware Request Shaping Extraction

Macro phase:

1. `M4` host transport-thinning follow-through

Gate:

1. plugin transport ownership thinning

Ownership seam:

1. `quantum_fusion/openai_ignition.rs` and
   `quantum_fusion/vector_ignition.rs` still duplicate the same
   runtime-oriented request shaping sequence:
   build one `WendaoArrow` request batch from scored candidates and fetched
   embeddings, then attach provider-aware request metadata
2. that composition belongs in `xiuxian-wendao-runtime`, while the ignition
   owners should stay focused on query-vector resolution and embedding fetch

The landed changes are:

1. add one runtime helper that builds a plugin arrow request batch from scored
   candidates plus embeddings and attaches provider-aware metadata
2. rewire both ignition owners to consume that runtime helper
3. keep ignition-local logic limited to query-vector resolution, embedding
   lookup, and owner-specific error mapping
4. close the touched-scope warning surfaced during validation by restoring the
   public method docs after the private embedding helper insertion

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Quantum Plugin Rerank Host Orchestration Deduplication

Macro phase:

1. `M4` host transport-thinning follow-through

Gate:

1. host rerank flow thinning

Ownership seam:

1. `quantum/rerank.rs` still duplicates the same host-local
   `QuantumContext -> QuantumAnchorHit` staging, request-batch failure
   telemetry, negotiated roundtrip, and score-apply orchestration across the
   vector-store and openai-compatible plugin rerank paths
2. that duplication belongs in the host rerank flow itself, not in runtime,
   because it depends on `QuantumContext`, host telemetry shaping, and the
   domain rerank orchestration boundary

Landed changes:

1. add one host-local helper that stages `QuantumContext` into
   `QuantumAnchorHit`
2. add one host-local helper that runs the shared request-batch
   failure/roundtrip/telemetry orchestration
3. keep the vector-store and openai-compatible entrypoints focused on their
   real behavioral differences only
4. keep the blocker recovery inside this same lane by moving the canonical
   semantic-lineage helper back under `link_graph/index/lookup.rs`, removing
   the parser-owned `LinkGraphIndex` impl from
   `parsers/markdown/links/normalize.rs`, fixing the `PageIndexNode`
   namespace import in `link_graph/index/page_indices.rs`, and adding one
   focused page-index lineage regression in `tests/unit/link_graph/index.rs`

Validation:

1. `direnv exec . cargo fmt -p xiuxian-wendao`
2. `direnv exec . cargo test -p xiuxian-wendao parsers::markdown:: --lib`
3. `direnv exec . cargo test -p xiuxian-wendao --lib link_graph::index::tests::test_page_index_lineage_helpers_follow_canonical_owner_surface`
4. `direnv exec . cargo test -p xiuxian-wendao --lib link_graph::index::search::plan::payload::quantum`
5. `direnv exec . cargo test -p xiuxian-wendao --lib link_graph::index::search::quantum_fusion`
6. `direnv exec . cargo check -p xiuxian-wendao --no-default-features --features julia,zhenfa-router`
7. `direnv exec . git diff --check`

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Runtime Plugin Arrow Vector-Store Request Row Preparation Extraction

Macro phase:

1. `M4` host transport-thinning follow-through

Gate:

1. plugin transport ownership thinning

Ownership seam:

1. `quantum_fusion/openai_ignition.rs` and
   `quantum_fusion/vector_ignition.rs` still duplicate the same generic
   request-row preparation from doc-score pairs into fetched embeddings plus
   owned `PluginArrowRequestRow` values
2. that preparation is runtime-owned host wiring around `xiuxian-vector`, while
   the ignition owners should stay focused on query-vector resolution and
   owner-specific error mapping

The landed changes are:

1. add one runtime helper that fetches candidate embeddings and materializes
   `PluginArrowRequestRow` values from doc-score pairs
2. rewire both ignition owners to consume that helper
3. keep ignition-local logic limited to query-vector resolution and
   owner-specific error mapping
4. move the direct `fetch_embeddings_by_ids(...)` call out of host
   `quantum_fusion` and into one runtime-owned `plugin_arrow_exchange/prepare.rs`
   seam

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Runtime Plugin Arrow Candidate Projection Extraction

Macro phase:

1. `M4` host transport-thinning follow-through

Gate:

1. plugin transport ownership thinning

Ownership seam:

1. `quantum_fusion/openai_ignition.rs` and
   `quantum_fusion/vector_ignition.rs` still duplicate the same generic
   projection from owner-local anchor hits into `doc_ids` plus
   `PluginArrowScoredCandidate`
2. that projection is transport-side request shaping and belongs in
   `xiuxian-wendao-runtime`, while the ignition owners should stay focused on
   query-vector resolution and embedding fetch

The landed changes are:

1. add one runtime helper that projects one generic doc-id list and scored
   candidate list from doc-score pairs
2. rewire both ignition owners to consume that helper
3. keep ignition-local logic limited to query-vector resolution, embedding
   lookup, and owner-specific error mapping
4. close the touched-scope issues surfaced during validation by adding explicit
   anchor-tied lifetimes to the private helper returns and removing one stale
   unused import

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Runtime Plugin Arrow Request Metadata Extraction

Macro phase:

1. `M4` host transport-thinning follow-through

Gate:

1. plugin transport ownership thinning

Ownership seam:

1. `link_graph/index/search/plan/payload/quantum/rerank.rs` should not keep
   request-batch metadata attachment once `xiuxian-wendao-runtime` already owns
   the transport metadata keys and the `WendaoArrow` request-batch surface

The landed changes are:

1. add one runtime-owned helper for plugin rerank request `trace_id` shaping
   and batch metadata attachment
2. rewire host rerank wrappers to consume that helper instead of calling
   `attach_record_batch_metadata` locally
3. move metadata-shaping proof out of host quantum tests and into runtime
   transport tests

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Runtime Plugin Arrow Request-Batch Builder Extraction

Macro phase:

1. `M4` host transport-thinning follow-through

Gate:

1. plugin transport ownership thinning

Ownership seam:

1. `openai_ignition.rs` and `vector_ignition.rs` should not keep assembling
   `PluginArrowRequestRow` values and bridging them into one `WendaoArrow`
   request batch once `xiuxian-wendao-runtime` already owns the request-row
   contract and Arrow request-batch materialization

The landed changes are:

1. add one runtime-owned helper that converts anchor hits plus fetched
   embeddings into one `WendaoArrow` request batch
2. rewire the host quantum-fusion ignition adapters to consume that helper
   while keeping query-vector resolution and vector-store retrieval semantics
   host-owned
3. split proof ownership so runtime covers request shaping and host covers
   semantic ignition semantics

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Runtime Plugin Arrow Rerank Roundtrip Extraction

Macro phase:

1. `M4` host transport-thinning follow-through

Gate:

1. plugin transport ownership thinning

Ownership seam:

1. `quantum/rerank.rs` should not keep transport negotiation, Arrow batch
   roundtrip, and typed response decode as host-local logic once
   `xiuxian-wendao-runtime` already owns the underlying transport contract,
   negotiation client, and request/response typing

The landed changes are:

1. add one runtime-owned `plugin_arrow_exchange` helper surface for one
   negotiated rerank batch roundtrip plus typed response decode
2. rewire `quantum/rerank.rs` to consume the runtime helper instead of owning
   transport negotiation and response decode locally
3. keep host ownership only for request-batch generation, metadata attachment,
   telemetry shaping, and score application

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Builtin Retrieval Policy Julia Runtime Extraction

Macro phase:

1. `M4` Julia ownership externalization follow-through

Gate:

1. host direct plugin-import thinning

Ownership seam:

1. the host retrieval-policy runtime model should not retain a
   `LinkGraphJuliaRerankRuntimeConfig` field or resolve that Julia-specific
   runtime directly; builtin or plugin-owned helpers should project the
   generic rerank binding, score weights, schema version, and artifact-dispatch
   inputs so `xiuxian-wendao` keeps only generic retrieval-policy ownership

The landed changes are:

1. add one builtin-owned retrieval-policy helper surface that resolves
   Julia-backed rerank binding, rerank score weights, rerank schema version,
   and artifact-dispatch input from merged settings
2. rewire `xiuxian-wendao/src/link_graph/runtime_config/models/retrieval/`
   and `resolve/policy/retrieval/` so the host runtime model stores only
   generic rerank outputs instead of a Julia-specific runtime struct
3. rewire host runtime-config and artifact-dispatch helpers to consume that
   builtin-owned projection surface while keeping non-Julia host probes green
4. keep Julia-specific runtime parsing and deployment-artifact payload shaping
   inside `xiuxian-julia-core`, and rerun builtin, host runtime-config,
   host quantum, and non-Julia compile probes to confirm the owner cut lands
   without a touched-scope regression

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Builtin OpenAPI Julia Example Assertion Extraction

Macro phase:

1. `M4` Julia ownership externalization follow-through

Gate:

1. host direct plugin-import thinning

Ownership seam:

1. the host bundled `OpenAPI` assertion surface under `gateway/openapi/`
   should not import Julia example helpers directly from the plugin crate; the
   builtin bundle should own that example-consumption seam so host assertion
   code depends only on the builtin package

The landed changes are:

1. add one builtin-owned `artifacts/openapi.rs` helper surface for the current
   Julia-backed bundled `OpenAPI` examples used by the host assertion layer
2. rewire `xiuxian-wendao/src/gateway/openapi/document.rs` tests to consume
   those builtin-owned helpers instead of importing Julia example helpers
   directly from `xiuxian-julia-core`
3. keep the actual example source of truth inside `xiuxian-julia-core`
   while confirming the host bundled `OpenAPI` assertion seam no longer
   imports the plugin crate directly and the non-Julia host compile probe
   stays green

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Builtin Studio Gateway Artifact Fixture Assertion Extraction

Macro phase:

1. `M4` Julia ownership externalization follow-through

Gate:

1. host direct plugin-import thinning

Ownership seam:

1. the host `gateway/studio` artifact fixture assertion seam should not import
   Julia fixture helpers directly from the plugin crate; the builtin bundle
   should own the linked fixture-consumption seam so host gateway assertions
   depend only on the builtin package

The landed changes were:

1. added builtin-owned linked gateway fixture helpers under
   `xiuxian-wendao-builtin/src/artifacts/gateway.rs` and re-exported them
   through the builtin crate root
2. rewired
   `gateway/studio/router/handlers/capabilities/deployment.rs`,
   `gateway/studio/types/config.rs`, and
   `gateway/studio/router/tests/config.rs` to consume builtin-owned fixture
   helpers instead of importing Julia helpers directly
3. kept the concrete fixture source of truth inside `xiuxian-julia-core`
   while confirming the touched host `gateway/studio` assertion seam no longer
   imports the plugin crate directly
4. added builtin regression coverage for the linked gateway helper surface and
   kept the non-Julia host compile probe green

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Builtin Host Julia Src-Test Helper Extraction

Macro phase:

1. `M4` Julia ownership externalization follow-through

Gate:

1. host direct plugin-import thinning

Ownership seam:

1. the host internal `src/` test seams under `link_graph/runtime_config/` and
   `link_graph/index/search/plan/payload/quantum/` should not import Julia
   compatibility helpers directly from the plugin crate; the builtin bundle
   should own the linked helper surface for those host internal tests

The landed changes were:

1. added one builtin-owned `test_support/` linked helper surface for the
   current Julia selectors, constants, and rerank binding fixture used by host
   internal `src/` tests
2. rewire
   `link_graph/runtime_config/tests.rs` and
   `link_graph/index/search/plan/payload/quantum/tests.rs`
   to consume builtin-owned linked helpers instead of importing Julia helpers
   directly
3. keep the concrete Julia compatibility source of truth inside
   `xiuxian-julia-core` while confirming the touched host internal test seam
   no longer imports the plugin crate directly
4. added builtin regression coverage for the new linked host-test helper seam
   and kept the non-Julia host compile probe green

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Builtin Zhenfa Router Julia Fixture Extraction

Macro phase:

1. `M4` Julia ownership externalization follow-through

Gate:

1. host direct plugin-import thinning

Ownership seam:

1. the host `zhenfa_router` unit-test seams should not import Julia gateway
   fixture helpers directly from the plugin crate; the builtin bundle should
   own the linked fixture-consumption seam for those host unit tests

The landed changes were:

1. extend the builtin-owned linked gateway fixture helpers for the JSON-RPC
   params and expected JSON fragment fixtures used by host `zhenfa_router`
   tests
2. rewired
   `tests/unit/zhenfa_router/native/deployment.rs` and
   `tests/unit/zhenfa_router/rpc.rs`
   to consume builtin-owned fixture helpers instead of importing Julia helpers
   directly
3. kept the concrete gateway fixture source of truth inside
   `xiuxian-julia-core` while confirming the touched host `zhenfa_router`
   unit-test seam no longer imports the plugin crate directly
4. added builtin regression coverage for the extended linked gateway fixture
   seam and kept the non-Julia host compile probe green

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Builtin Planned Search Julia Integration Fixture Extraction

Macro phase:

1. `M4` Julia ownership externalization follow-through

Gate:

1. host direct plugin-import thinning

Ownership seam:

1. the host planned-search integration tests that only consume plugin
   integration-support fixtures should not import those helpers directly from
   the plugin crate; the builtin bundle should own the linked fixture
   consumption seam for those integration tests

The landed changes are:

1. add one builtin-owned linked planned-search helper surface under
   `xiuxian-wendao-builtin/src/test_support/planned_search/` for the current
   runtime-config and test-server helpers used by the touched host
   `planned_search_*` integration tests
2. rewire
   `planned_search_julia_rerank.rs`,
   `planned_search_julia_rerank_vector_store.rs`,
   `planned_search_julia_rerank_metadata_example.rs`,
   `planned_search_julia_rerank_official_example.rs`, and
   `planned_search_wendaoanalyzer_linear_blend.rs`
   to consume builtin-owned helpers instead of importing plugin helpers
   directly
3. keep the concrete planned-search fixture source of truth inside
   `xiuxian-julia-core` while confirming the touched host integration seam
   no longer imports the plugin crate directly and the non-Julia host compile
   probe remains green

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Builtin Planned Search Similarity-Only Analyzer Artifact Extraction

Macro phase:

1. `M4` Julia ownership externalization follow-through

Gate:

1. host direct plugin-import thinning

Ownership seam:

1. the remaining host planned-search similarity-only integration test should
   not import Julia compatibility runtime types, analyzer artifact materialization,
   or artifact-backed service spawn helpers directly from the plugin crate;
   the builtin bundle should own that linked analyzer-artifact seam

The landed changes are:

1. add one builtin-owned linked helper surface for the similarity-only
   planned-search analyzer-artifact path, covering runtime-config rendering
   and artifact-backed analyzer service spawn while keeping the thick
   implementation in `xiuxian-julia-core::integration_support::planned_search`
2. rewire
   `planned_search_wendaoanalyzer_similarity_only.rs`
   to consume builtin-owned helpers instead of importing plugin compatibility
   or integration-support helpers directly
3. keep the concrete analyzer runtime and deployment artifact source of truth
   inside `xiuxian-julia-core` while confirming the touched host integration
   seam no longer imports the plugin crate directly and the non-Julia host
   compile probe remains green

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---

## Landed Slice: Builtin Graph-Structural Host Test Helper Extraction

Macro phase:

1. `M4` Julia ownership externalization follow-through

Gate:

1. host direct plugin-import thinning

Ownership seam:

1. the remaining host `link_graph_agentic` unit tests should not import
   graph-structural constants or helper functions directly from the Julia
   plugin crate; the builtin bundle should own that linked test-support seam

The landed changes are:

1. add one builtin-owned linked graph-structural test-support helper surface
   for the bounded constants and helper functions used by
   `tests/unit/link_graph_agentic/expansion.rs` without moving concrete
   graph-structural implementation out of `xiuxian-julia-core`
2. rewire
   `tests/unit/link_graph_agentic/expansion.rs`
   to consume builtin-owned helpers instead of importing those graph-structural
   helpers directly from `xiuxian-julia-core`
3. keep the concrete graph-structural implementation source of truth inside
   `xiuxian-julia-core` while confirming the touched host test seam no
   longer imports the plugin crate directly and the non-Julia host test-compile
   probe still completes

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---
