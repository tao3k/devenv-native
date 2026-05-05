# Large Rust File Modularization

:PROPERTIES:
:ID: wendao-large-rust-modularization
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, refactor, modularization, rust, gateway, link-graph
:STATUS: PLANNED
:END:

## Mission

This roadmap note tracks a lossless modularization pass over oversized Rust source files in `xiuxian-wendao`.

Active migration references:

- `[[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md]]`
- `[[06_roadmap/409_core_runtime_plugin_surface_inventory]]`

This modularization lane is now part of the active `core` / `runtime` / plugin-package migration discipline. It should no longer be treated as optional cleanup work that can lag behind boundary extraction.

Inventory date: `2026-03-21`

Inventory scope:

- `packages/rust/crates/xiuxian-wendao/src/**/*.rs`
- threshold: files larger than 400 lines
- current count: 35 files

The canonical per-file execution plan is maintained in the active tracking
record rather than linked from this persistent roadmap note.

## Why This Slice Exists

- Several files are serving as DTO warehouses or transport façades and have accumulated unrelated responsibilities.
- `gateway/studio/router/mod.rs` and `analyzers/service/mod.rs` currently violate the repository rule that `mod.rs` should be interface-only.
- `semantic_check`, `sentinel`, and `link_graph` now have enough internal phases that file-level boundaries hide the real ownership model.
- A lossless split is a prerequisite for later behavior work because it reduces blast radius and makes targeted testing feasible.

## Delivery Waves

Priority override:

- `gateway/studio/search/handlers.rs` is the first implementation target and should be split before the numbered waves.

### Wave 1: Contracts and Helpers

- `gateway/studio/types.rs`
- `analyzers/query.rs`
- `analyzers/service/helpers.rs`
- `enhancer/markdown_config.rs`
- `link_graph/addressing/mod.rs`
- `link_graph/models/records/markdown_block.rs`

### Wave 2: Search and Projection

- `search/fuzzy.rs`
- `search/tantivy.rs`
- `analyzers/projection/builder.rs`
- `analyzers/projection/search.rs`
- `analyzers/service/search.rs`
- `analyzers/service/projection.rs`
- `gateway/studio/search/source_index.rs`

### Wave 3: Gateway and Router

- `gateway/studio/search/handlers.rs`
- `gateway/studio/router/handlers/repo.rs`
- `gateway/studio/router/mod.rs`
- `gateway/studio/vfs.rs`
- `gateway/studio/repo_index/state.rs`
- `gateway/openapi/paths.rs`

### Wave 4: Governance and Audit

- `zhenfa_router/native.rs`
- `zhenfa_router/native/sentinel.rs`
- `zhenfa_router/native/semantic_check/docs_governance/collection.rs`
- `zhenfa_router/native/semantic_check/docs_governance/tests.rs`
- `zhenfa_router/native/audit/fuzzy_suggest.rs`
- `zhenfa_router/native/audit/fix.rs`

### Wave 5: Link-Graph and Remaining Infrastructure

- `link_graph/addressing/mod.rs`
- `link_graph/parser/sections.rs`
- `link_graph/saliency/store/write.rs`
- `ingress/spider.rs`
- `skill_runtime/zhixing/resources.rs`

## Current Status

- [PLANNED] Inventory completed for all 35 oversized files.
- [PLANNED] Each file now has a proposed feature-folder or leaf-module split in the canonical execplan.
- [DONE] `gateway/studio/search/handlers.rs` is split into `gateway/studio/search/handlers/` with interface-only `mod.rs`, preserved public exports, and a green `cargo test -p xiuxian-wendao gateway::studio::search:: --lib` gate.
- [DONE] `gateway/studio/search/handlers/knowledge.rs` is further split into `gateway/studio/search/handlers/knowledge/` with interface-only `mod.rs`, separate `search.rs`, `intent.rs`, `merge.rs`, and `helpers.rs` leaf modules, preserved handler exports, and the existing test-only `build_intent_search_response` entrypoint.
- [DONE] `search_plane/service/core.rs` is split into `search_plane/service/core/` with interface-only `mod.rs`, shared `types.rs`, and a retained `core_impl.rs` implementation leaf, preserving `SearchPlaneService` and the crate-visible repo publication/runtime state helpers.
- [DONE] `gateway/studio/types.rs` is split into `gateway/studio/types/` with interface-only `mod.rs`, grouped DTO leaf modules, preserved public type names, and the same `studio_type_collection()` façade.
- [DONE] `analyzers/query.rs` is split into `analyzers/query/` with interface-only `mod.rs`, query-family leaf modules, and preserved `crate::analyzers::query::*` re-exports.
- [DONE] `analyzers/query/docs.rs` is split into `analyzers/query/docs/` with interface-only `mod.rs`, plus `coverage.rs`, `planner.rs`, and `search.rs`, while preserving the docs-facing query and response contracts.
- [DONE] `search/fuzzy.rs` is split into `search/fuzzy/` with interface-only `mod.rs`, focused helper leaf modules, preserved `search::fuzzy::*` exports, and the crate-visible scoring bridge retained for Tantivy integration.
- [DONE] `search/tantivy.rs` is split into `search/tantivy/` with interface-only `mod.rs`, focused document/index/matcher helper modules, preserved `search::tantivy::*` exports, and unchanged analyzer-facing shared search contracts.
- [DONE] `analyzers/service/mod.rs` is now interface-only, with orchestration logic moved into focused leaf modules while preserving `crate::analyzers::service::*` exports and sibling `super::*` call sites.
- [DONE] The stale tracked `analyzers/service/mod.rs.bak2` monolith is removed after confirming the live `analyzers/service/` leaf modules cover the split, so the service folder no longer carries a shadow copy of the pre-modularization implementation.
- [DONE] `gateway/studio/router/mod.rs` is now interface-only, with Studio state, configured-repository derivation, API error mapping, route assembly, and router-local tests moved into focused leaf modules while preserving `crate::gateway::studio::router::*` exports and the existing `code_ast`, `config`, `handlers`, and `sanitization` child modules.
- [DONE] `cargo check -p xiuxian-wendao --lib --keep-going` remains green in the current worktree after the first six modularization slices.
- [DONE] `cargo test -p xiuxian-wendao analyzers::service:: --lib`, `cargo test -p xiuxian-wendao repo_sync_endpoint_returns_repo_status_payload --lib`, and `cargo test -p xiuxian-wendao --bin wendao test_build_plugin_registry_bootstraps_builtin_plugins` are green after the `analyzers/service/mod.rs` split.
- [DONE] `cargo check -p xiuxian-wendao`, `cargo test -p xiuxian-wendao gateway::studio::router:: --lib`, `cargo test -p xiuxian-wendao --lib studio_repo_sync_api -- --nocapture`, and `cargo test -p xiuxian-wendao --bin wendao test_gateway_server_bind -- --nocapture` are green after the `gateway/studio/router/mod.rs` split.
- [DONE] `gateway/studio/repo_index/state.rs` is now a feature folder with interface-only `state/mod.rs`, while task-queue control, coordination logic, fingerprinting, code-document collection, status filtering, language inference, and repo-index tests live in focused leaf modules without changing the `RepoIndexCoordinator` surface.
- [DONE] `gateway/studio/repo_index/state/tests.rs` is split into `gateway/studio/repo_index/state/tests/` with interface-only `mod.rs`, plus `helpers.rs`, `sync.rs`, `status.rs`, `concurrency.rs`, `runtime.rs`, and `collect.rs`, while preserving the repo-index state test coverage.
- [DONE] `cargo fmt -p xiuxian-wendao`, `cargo check -p xiuxian-wendao --lib --keep-going`, `cargo test -p xiuxian-wendao gateway::studio::repo_index:: --lib`, `cargo test -p xiuxian-wendao gateway::studio::search:: --lib`, and `cargo test -p xiuxian-wendao repo_sync_endpoint_returns_repo_status_payload --lib` are green after the `gateway/studio/repo_index/state.rs` split.
- [DONE] `gateway/studio/router/state.rs` is now a feature folder with interface-only `state/mod.rs`, while lifecycle, UI config, graph accessors, search accessors, helper utilities, and router-state tests live in focused leaf modules without changing the `GatewayState` / `StudioState` surface.
- [DONE] `cargo fmt -p xiuxian-wendao`, `cargo check -p xiuxian-wendao --lib --keep-going`, `cargo test -p xiuxian-wendao gateway::studio::router::state::tests:: --lib`, and `cargo test -p xiuxian-wendao gateway::studio::router::tests:: --lib` are green after the `gateway/studio/router/state.rs` split.
- [DONE] `gateway/studio/router/handlers/repo.rs` is now a feature folder with interface-only `repo/mod.rs`, while `query.rs`, `parse.rs`, `shared.rs`, `analysis.rs`, `index.rs`, `pages.rs`, `retrieval.rs`, `family.rs`, and `refine.rs` carry the handler families without changing the existing repo handler names.
- [DONE] `cargo fmt -p xiuxian-wendao`, `cargo check -p xiuxian-wendao --lib --keep-going`, `cargo test -p xiuxian-wendao gateway::studio::studio_repo_sync_api_tests:: --lib`, `cargo test -p xiuxian-wendao gateway::studio::router:: --lib`, and `cargo test -p xiuxian-wendao gateway::studio::repo_index:: --lib` are green after the `gateway/studio/router/handlers/repo.rs` split.
- [DONE] `gateway/studio/vfs.rs` is now a feature folder with interface-only `vfs/mod.rs`, while `filters.rs`, `roots.rs`, `scan.rs`, `navigation.rs`, `content.rs`, `categories.rs`, and `tests.rs` carry VFS scanning, root resolution, navigation, and content reads without changing the active Studio VFS entry points.
- [DONE] `cargo fmt -p xiuxian-wendao`, `cargo check -p xiuxian-wendao --lib --keep-going`, and `cargo test -p xiuxian-wendao gateway::studio:: --lib` are green after the `gateway/studio/vfs.rs` split.
- [DONE] `gateway/openapi/paths.rs` is now a feature folder with interface-only `paths/mod.rs`, while `analysis.rs`, `graph.rs`, `repo.rs`, `search.rs`, `shared.rs`, `ui.rs`, and `vfs.rs` carry the route constants and route inventory without changing `crate::gateway::openapi::paths::*`, `RouteContract`, or `WENDAO_GATEWAY_ROUTE_CONTRACTS`.
- [DONE] Public constant-surface parity against the pre-split index version, route-inventory count parity, file-local `rustfmt`, standalone `rustc` re-export compilation, and `git diff --check` are green for the `gateway/openapi/paths.rs` slice.
- [DONE] `gateway/studio/search/source_index.rs` is now a feature folder with interface-only `source_index/mod.rs`, while `ast.rs`, `symbols.rs`, `markdown.rs`, `navigation.rs`, and `filters.rs` carry the index build, markdown-hit synthesis, navigation payload assembly, and scan filtering logic without changing the existing `build_ast_index` / `build_symbol_index` call path.
- [DONE] `cargo fmt -p xiuxian-wendao`, `cargo check -p xiuxian-wendao --lib --keep-going`, `cargo test -p xiuxian-wendao gateway::studio::search:: --lib`, and `git diff --check` are green after the `gateway/studio/search/source_index.rs` split.
- [DONE] `analyzers/service/search.rs` is now a feature folder with interface-only `search/mod.rs`, while `documents.rs`, `legacy.rs`, `indexed_exact.rs`, `indexed_fuzzy.rs`, `ranking.rs`, `overview.rs`, `module.rs`, `symbol.rs`, `example.rs`, `imports.rs`, `coverage.rs`, and `tests.rs` carry the repo overview, ranked search, import search, and doc-coverage logic without changing the existing `crate::analyzers::service::*` search APIs.
- [DONE] `analyzers/service/projection.rs` is now a feature folder with interface-only `projection/mod.rs`, while `gap.rs`, `pages.rs`, `page.rs`, `family.rs`, `navigation.rs`, `search.rs`, `index_tree.rs`, `retrieval.rs`, `registry.rs`, and `tests.rs` carry the projected-page, family, navigation, tree, retrieval, and gap-report wrappers without changing the existing `crate::analyzers::service::*` projection APIs.
- [DONE] `analyzers/service/helpers.rs` is now a feature folder with interface-only `helpers/mod.rs`, while `uri.rs`, `ecosystem.rs`, `path.rs`, `ranking.rs`, `backlinks.rs`, `projection_lookup.rs`, `example_relations.rs`, `scope.rs`, and `tests.rs` carry the URI, ecosystem, path, ranking, backlink, projection lookup, example relation, and scope helpers without changing the existing analyzer-service call sites.
- [DONE] `enhancer/markdown_config.rs` is now a feature folder with interface-only `markdown_config/mod.rs`, while `types.rs`, `index.rs`, `blocks.rs`, `links.rs`, and `tests.rs` carry markdown config block parsing, link-target extraction, and memory-index helpers without changing the `resource_registry` and enhancer API surface.
- [DONE] `skill_runtime/manifest.rs` is now a feature folder with interface-only `manifest/mod.rs`, while `types.rs`, `load.rs`, `authority.rs`, and `tests.rs` carry skill manifest parsing, authority resolution, and workflow-type helpers without changing `skill_runtime::manifest::*`.
- [DONE] `link_graph/addressing/mod.rs` is now a feature folder with interface-only `mod.rs`, while `address.rs`, `resolve.rs`, `indices.rs`, `node_lookup.rs`, `mutation.rs`, `errors.rs`, and `tests.rs` preserve the registry, topology, skeleton-rerank, and structural-transaction child modules.
- [DONE] `link_graph/index.rs` is now a feature folder with interface-only `mod.rs`, while `constants.rs`, `types.rs`, `lookup.rs`, `symbol_cache.rs`, and the preserved `build/`, `search/`, `ids.rs`, `page_indices.rs`, `passages.rs`, `ppr.rs`, `rank.rs`, `scoring.rs`, `semantic_documents.rs`, `shared.rs`, and `traversal.rs` child namespaces keep the public `link_graph::index::*` surface stable.
- [DONE] `link_graph/parser/code_observation.rs` is now a feature folder with interface-only `mod.rs`, while `types.rs`, `glob.rs`, `extract.rs`, `format.rs`, and the preserved tests fixture carry code-observation parsing, glob matching, formatting, and extraction logic without changing `link_graph::parser::code_observation::*`.
- [DONE] `cargo fmt -p xiuxian-wendao`, `cargo check -p xiuxian-wendao --lib --keep-going`, `cargo test -p xiuxian-wendao skill_runtime::manifest::tests:: --lib`, `cargo test -p xiuxian-wendao link_graph::addressing:: --lib`, `cargo test -p xiuxian-wendao link_graph::index:: --lib`, `cargo test -p xiuxian-wendao link_graph::parser::code_observation:: --lib`, and `git diff --check` are green after the `skill_runtime/manifest.rs`, `link_graph/addressing/mod.rs`, `link_graph/index.rs`, and `link_graph/parser/code_observation.rs` splits.
- [DONE] `link_graph/parser/sections.rs` is split into `link_graph/parser/sections/` with interface-only `mod.rs`, property-drawer parsing, logbook parsing, and section extraction leaf modules, plus the preserved tests fixture.
- [DONE] `link_graph/saliency/store/write.rs` is split into `link_graph/saliency/store/write/` with interface-only `mod.rs`, `types.rs`, `edge_updates.rs`, `coactivation.rs`, `touch.rs`, `valkey.rs`, `time.rs`, and `tests.rs`, while preserving the saliency write API surface and the coactivation propagation behavior.
- [DONE] `cargo fmt -p xiuxian-wendao`, `cargo check -p xiuxian-wendao --lib --keep-going`, `cargo test -p xiuxian-wendao link_graph::parser::sections:: --lib`, `cargo test -p xiuxian-wendao link_graph::saliency::store::write::tests:: --lib`, `cargo test -p xiuxian-wendao --test wendao-validation-gate saliency`, `cargo test -p xiuxian-wendao --test wendao-validation-gate coactivation`, and `git diff --check` are green after the `link_graph/parser/sections.rs` and `link_graph/saliency/store/write.rs` splits.
- [DONE] the canonical markdown parser stack now lives under `src/parsers/markdown/`, `src/parsers/mod.rs` is the crate-root parser namespace, crate-internal parser consumers now import `crate::parsers::markdown::*`, and the parser unit fixtures moved under `tests/unit/parsers/markdown/` to match the new ownership shape.
- [DONE] parser namespace baseline recovery aligned the remaining `src/parsers/markdown/*` helpers to public `crate::link_graph::*` re-exports and reattached the canonical `LinkGraphIndex::extract_lineage` helper under `link_graph/index/lookup.rs`, which cleared the crate-level baseline that had been blocking the quantum rerank proof lane.
- [DONE] markdown semantic frontmatter parsing now lives under `src/parsers/markdown/frontmatter/`, the duplicate `enhancer/frontmatter.rs` implementation is retired, and enhancement plus `skill_runtime` discovery flows now consume `crate::parsers::markdown::{parse_frontmatter, NoteFrontmatter}` with parser-owned unit coverage.
- [DONE] link-graph search-query parsing now lives under `src/parsers/link_graph/query/`, the former `link_graph::query` implementation tree is retired, link-graph search planning now consumes `crate::parsers::link_graph::query::{parse_search_query, ParsedLinkGraphQuery}`, and parser-owned query coverage lives under `tests/unit/parsers/link_graph/query.rs`.
- [DONE] zhixing task-line parsing now lives under `src/parsers/zhixing/tasks/`, the former `skill_runtime::zhixing::indexer::parse/` subtree is retired, and zhixing task ingestion plus agenda-status stats now consume `crate::parsers::zhixing::tasks::{parse_task_projection, normalize_identity_token, TaskLineProjection}` with parser-owned unit coverage.
- [DONE] Cargo.toml dependency parsing now lives under `src/parsers/cargo/dependencies/`, the former `dependency_indexer/cargo/` subtree is retired without a compatibility shim, `dependency_indexer` no longer owns Cargo parser exports, and parser-owned coverage now lives under `tests/unit/parsers/cargo/dependencies.rs`.
- [DONE] repo-code search query parsing now lives under `src/parsers/search/repo_code_query/`, the former `search/repo_search/query.rs` module is retired without a compatibility shim, repo-search flows now consume `crate::parsers::search::repo_code_query::parse_repo_code_search_query`, and parser-owned coverage now lives under `tests/unit/parsers/search/repo_code_query.rs`.
- [DONE] graph persistence dict parsing now lives under `src/parsers/graph/persistence/`, the former `graph/persistence/parse.rs` module is retired without a compatibility shim, graph save/load now consumes `crate::parsers::graph::persistence::{entity_from_dict, relation_from_dict}`, and parser-owned coverage now lives under `tests/unit/parsers/graph/persistence.rs`.
- [DONE] the stable parser documentation home now lives under `docs/02_parser/`, with `index.md` and `architecture.md` capturing the canonical parser-family matrix, parser-vs-helper boundary, and the structural parsing rule that prefers explicit tokens such as wiki links before loose keyword matching.
- [DONE] parser-core namespace closeout audit found no remaining domain-core parser families outside `src/parsers/`; the remaining `parse.rs`, `query.rs`, and `document.rs` helpers are adapter-local or subsystem-local by design, and graph persistence relation parsing now structurally decodes wiki-link relation references before enum mapping.
- [DONE] the first relation-semantics correction slice now treats ordinary `[[...]]` links as structural graph edges instead of auto-promoting them to semantic relation labels, narrows graph persistence and Python relation parsers to exact internal token decoding, and records the rule that typed relation semantics must come from explicit metadata or structural suffix signals rather than hardcoded wiki-link strings.
- [DONE] markdown property-drawer relation parsing now lives under `src/parsers/markdown/relations/`, `PROPERTIES` relation targets are parsed as a dedicated scoped grammar distinct from global body `[[...]]` links, enhancer outputs now preserve source and target scope metadata, and the local `link_graph` property-drawer builder is reduced to a graph adapter over parser-owned relation outputs.
- [DONE] parser docs under `docs/02_parser/` now include `relation_semantics.md`, which explains the difference between global topology links, explicit section-scoped property relations, and scalar property metadata, plus the current explicit-`ID` resolution boundary for stable cross-document section targets.
- [DONE] ordinary body wiki links now align with the Obsidian structural-address model, so `[[note#Heading]]` and `[[note#^block-id]]` are parsed as note plus address targets instead of semantic type hints, while explicit semantic categories stay on metadata surfaces such as section tags and property drawers.
- [DONE] ordinary Markdown reference parsing now lives under `src/parsers/markdown/references/`, so `[...](...)` and `[[...]]` share one parser-owned extraction contract and `SKILL.md` no longer owns a separate link-parsing split.
- [DONE] ordinary body wikilink extraction now lives under `src/parsers/markdown/wikilinks/` as a comrak-backed parser-owned module with snapshot coverage, and `link_graph_refs` now consumes that parser instead of owning its own regex grammar.
- [DONE] docs-governance relation-line and index-body wikilink collection now consume `src/parsers/markdown/wikilinks/` instead of duplicating an ordinary wikilink scanner, while hidden-path byte-range extraction remains adapter-local.
- [DONE] `skill_runtime/manifest/authority.rs` now consumes `src/parsers/markdown/references/` for ordinary `SKILL.md` manifest intents, so Markdown links and wikilinks follow the same parser-owned contract instead of a local mixed scanner.
- [DONE] the local parser/enhancer PyO3 wrappers were retired from `src/pybindings/` so this parser lane no longer preserves a duplicate Python contract for `enhancer` and `link_graph_refs`.
- [DONE] `zhenfa_router/native/semantic_check.rs` is split into `zhenfa_router/native/semantic_check/` with interface-only `mod.rs`, `checks.rs`, `core.rs`, `parsing.rs`, `report.rs`, `types.rs`, and `test_api.rs`, while preserving the existing `docs_governance/` child folder.
- [DONE] `zhenfa_router/native/audit/audit_bridge.rs` is split into `zhenfa_router/native/audit/audit_bridge/` with interface-only `mod.rs`, `types.rs`, `helpers.rs`, `batch_fix.rs`, `bridge.rs`, and `surgical.rs`, while preserving the existing unit-test fixture and the public audit bridge surface.
- [DONE] `zhenfa_router/native/semantic_check/docs_governance/collection.rs` is split into `zhenfa_router/native/semantic_check/docs_governance/collection/` with interface-only `mod.rs`, `workspace.rs`, `footer.rs`, `relations.rs`, and `package_docs.rs`, while preserving the existing docs-governance issue collectors and the external test coverage.
- [DONE] `zhenfa_router/native.rs` is split into `zhenfa_router/native/` with interface-only `mod.rs`, `context.rs`, `search.rs`, `semantic_read.rs`, `semantic_edit.rs`, and the preserved `agentic_nav`, `audit`, `forwarder`, `remediation`, `section_create`, `semantic_check`, `sentinel`, and `xml_lite` child modules while preserving `zhenfa_router::native::*`.
- [DONE] `zhenfa_router/native/audit/fuzzy_suggest.rs` is split into `zhenfa_router/native/audit/fuzzy_suggest/` with interface-only `mod.rs`, `cache.rs`, `format.rs`, `pattern.rs`, `search.rs`, `similarity.rs`, `sources.rs`, and `types.rs`, while preserving the external unit-test fixture and the public fuzzy-suggestion API surface.
- [DONE] `zhenfa_router/native/audit/fix.rs` is split into `zhenfa_router/native/audit/fix/` with interface-only `mod.rs`, `batch.rs`, `format.rs`, `hashing.rs`, `preview.rs`, `report.rs`, and `tests.rs`, while preserving `AtomicFixBatch`, `FixPreview`, `FixReport`, `format_fix_preview`, and the audit-fix write-back semantics.
- [DONE] `zhenfa_router/native/semantic_check/docs_governance/tests.rs` is split into `zhenfa_router/native/semantic_check/docs_governance/tests/` with interface-only `mod.rs`, `doc_identity.rs`, `fixes.rs`, `index_links.rs`, `package_docs.rs`, `scope.rs`, `standard_sections.rs`, and `workspace.rs`, while preserving the docs-governance test coverage and the existing `run_audit_core` / surgical-fix assertions.
- [DONE] `zhenfa_router/native/semantic_check/docs_governance/tests/index_links.rs` is split into `zhenfa_router/native/semantic_check/docs_governance/tests/index_links/` with interface-only `mod.rs`, plus `section_links.rs`, `relations.rs`, and `footer.rs`, while preserving the docs-governance index-link coverage and the existing `run_audit_core` assertions.
- [DONE] `skill_runtime/zhixing/resources.rs` is split into `skill_runtime/zhixing/resources/` with interface-only `mod.rs`, `paths.rs`, `mounts.rs`, `text.rs`, `registry.rs`, `discovery.rs`, and `tests.rs`, preserving the embedded skill-resource registry and discovery surface. The embedded `zhixing` tree is vendored under `packages/rust/crates/xiuxian-wendao/resources/zhixing` and injected through `build.rs`, so downstream builds no longer depend on the sibling `xiuxian-zhixing` crate path.
- [DONE] `analyzers/projection/builder.rs` is split into `analyzers/projection/builder/` with interface-only `mod.rs`, `anchors.rs`, `sources.rs`, `kinds.rs`, `assemble.rs`, `helpers.rs`, and `tests.rs`, preserving `build_projection_inputs` and the builder test surface.
- [DONE] `analyzers/projection/search.rs` is split into `analyzers/projection/search/` with interface-only `mod.rs`, `indexed.rs`, `heuristic.rs`, `lexical.rs`, `mapping.rs`, `ranking.rs`, `sort.rs`, `options.rs`, and `tests.rs`, preserving the projected-page search API and `scored_projected_page_matches`.
- [DONE] `search_plane/repo_entity/query.rs` is split into `search_plane/repo_entity/query/` with interface-only `mod.rs`, `types.rs`, `scan.rs`, `scoring.rs`, `search.rs`, and `tests.rs`, preserving the repo-entity query, ranking, hydration, and test surface.
- [DONE] `search_plane/attachment/query.rs` is split into `search_plane/attachment/query/` with interface-only `mod.rs`, `types.rs`, `scan.rs`, `scoring.rs`, `search.rs`, and `tests.rs`, preserving the attachment query, ranking, hydration, and test surface.
- [DONE] `search_plane/repo_content_chunk/build.rs` is split into `search_plane/repo_content_chunk/build/` with interface-only `mod.rs`, plus `orchestration.rs`, `plan.rs`, `types.rs`, `write.rs`, and `tests.rs`, while preserving `publish_repo_content_chunks` and the repo-content publication semantics.
- [DONE] Wendao configuration ownership is now import-overlay-aware: `xiuxian-config-core` owns recursive `imports` plus import-path environment expansion, the checked-in workspace boot config lives at `$PRJ_ROOT/wendao.toml`, `src/settings/` no longer carries a hand-written deep-merge path, `src/bin/wendao/execute/gateway/config.rs` no longer scans TOML text for gateway port/webhook settings, `nix/modules/process.nix` now reads the gateway port through a bounded TOML-aware helper instead of `awk`, and Studio UI persistence now collapses any legacy `wendao.studio.overlay.toml` state back into the base `wendao.toml` instead of generating a new overlay.
- [DONE] `search_plane/service/tests/status/repo_content.rs` is split into `search_plane/service/tests/status/repo_content/` with interface-only `mod.rs`, plus `helpers.rs`, `ready.rs`, `snapshot.rs`, `publication.rs`, and `failure.rs`, while preserving the repo-content status coverage.
- [DONE] `search_plane/manifest.rs` is split into `search_plane/manifest/` with interface-only `mod.rs`, plus `keyspace.rs`, `fingerprint.rs`, `records.rs`, and `tests.rs`, while preserving the manifest keyspace and publication record semantics.
- [DONE] `analyzers/service/search/ranking.rs` is split into `analyzers/service/search/ranking/` with interface-only `mod.rs`, plus `shared.rs`, `module.rs`, `symbol.rs`, `example.rs`, and `tests.rs`, while preserving the ranked module/symbol/example search surfaces and the existing sibling-module call sites.
- [PLANNED] `gateway/studio/repo_index/state/coordinator/runtime.rs` is the next active oversized slice on deck.

## Local Constraints

- The governing execution-plan policy lives at `.agent/PLANS.md`.
- The workspace now contains the semantic-addressing blueprint again.
- The refactor should therefore follow that blueprint together with the
  current code topology and stable public API paths, while the exact
  hidden-path reference stays in the active tracking record.

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/402_repo_intelligence_mvp|Repo Intelligence MVP]], [[06_roadmap/403_document_projection_and_retrieval_enhancement|Document Projection and Retrieval Enhancement]], [[06_roadmap/404_repo_intelligence_for_sciml_and_msl|Repo Intelligence for SciML and MSL]]
:END:

---

:FOOTER:
:STANDARDS: v2.0
:LAST_SYNC: 2026-04-05
:END:
