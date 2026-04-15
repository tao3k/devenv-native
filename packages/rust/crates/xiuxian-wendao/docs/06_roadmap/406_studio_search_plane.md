# 406 Studio Search Plane

## Goal

Replace Studio request-path search hot spots with a background-built search plane backed by Lance tables, Arrow batch reranking, and Valkey coordination.

## Active Scope

- unify Lance and Arrow dependency ownership under `xiuxian-vector`
- add reusable columnar table APIs in `xiuxian-vector`
- consolidate the search-plane runtime under `xiuxian_wendao::search`
- expose Studio search index lifecycle through `/api/search/index/status`
- migrate Studio search handlers away from direct AST cache construction

## Constraints

- `LinkGraphIndex` stays responsible for graph traversal and topology APIs
- search payload contracts remain stable during the migration
- search-plane Valkey manifests and query-cache semantics stay owned by
  `xiuxian-wendao`; `xiuxian-vector` remains the Lance/Arrow/vector kernel
  rather than the root owner of Valkey-backed search state
- the strategic semantic-addressing blueprint referenced by repository policy
  is present in this checkout and should be treated as the active guide for
  this roadmap item, while the exact hidden-path reference stays in the active
  tracking record

## Current Slice

- foundation for corpus status, epoch publication, and single-flight builds is landed
- `/api/analysis/markdown` and `/api/analysis/code-ast` are no longer active
  HTTP business surfaces; the bundled gateway route surface now removes them
  outright and keeps `/analysis/markdown` plus `/analysis/code-ast` as
  Flight-only business contracts
- `analyzers/config.rs` is now split into `analyzers/config/` with dedicated types, TOML schema, parse, load, and tests modules, while the repo-intelligence config surface remains unchanged; the next bounded target is `search/tantivy/index.rs`
- `search/tantivy/index.rs` is now split into `search/tantivy/index/` with dedicated core, exact, prefix, fuzzy, and helper modules, while the shared Tantivy search surface remains unchanged; the next bounded target is `analyzers/service/helpers/tests.rs`
- `analyzers/service/helpers/tests.rs` is now split into `analyzers/service/helpers/tests/` with dedicated fixture and themed assertion modules, while the helpers test surface remains unchanged; the next bounded target is `search/local_symbol/query/shared.rs`
- `search/reference_occurrence/query.rs` is now split into `search/reference_occurrence/query/` with dedicated search, candidates, decode, and tests modules, while the reference-occurrence search surface remains unchanged; the next bounded target is `search/repo_entity/build/tests.rs`
- `search/service/core/construction.rs` is now split into `search/service/core/construction/` with dedicated runtime, paths, concurrency, and tests modules, while the public `SearchPlaneService` construction surface remains unchanged; the next bounded target is `gateway/studio/router/handlers/repo/analysis/search.rs`
- `gateway/studio/router/handlers/repo/analysis/search.rs` is now split into `gateway/studio/router/handlers/repo/analysis/search/` with dedicated cache, publication, module, symbol, example, and tests modules, while the repo-analysis handler surface remains unchanged; the next bounded target is `analyzers/query/docs/planner.rs`
- the old `zhenfa_router/native/section_create/` helper home has now been retired in favor of the parser-owned `xiuxian-wendao-parsers::section_create` surface plus the Wendao-local adapter `src/parsers/markdown/section_create/`; the semantic-edit surface remains unchanged while parser ownership no longer lives under `zhenfa_router`
- `gateway/studio/router/code_ast.rs` is now split into `gateway/studio/router/code_ast/` with dedicated response, resolve, blocks, and atoms modules, while the public code-AST router surface remains unchanged; the next bounded target is `gateway/studio/search/handlers/knowledge/intent.rs`
- `gateway/studio/search/definition.rs` is now split into `gateway/studio/search/definition/` with dedicated resolve, filters, and tests modules, while the public definition resolution surface remains unchanged; the next bounded target is `analyzers/service/projection/planner/api.rs`
- `link_graph/index/build/assemble.rs` is now split into `link_graph/index/build/assemble/` with dedicated inputs, notes, edges, virtual-nodes, finalize, and api modules, while the public link-graph index build surface remains unchanged; the next bounded target is `gateway/studio/search/handlers/code_search/search.rs`
- `gateway/studio/search/handlers/code_search/search.rs` is now split into `gateway/studio/search/handlers/code_search/search/` with dedicated response, repo-search, buffered, and task modules, while the public code-search handler surface remains unchanged; the next bounded target is `link_graph/context_snapshot.rs`
- `link_graph/context_snapshot.rs` is now split into `link_graph/context_snapshot/` with dedicated types, id, runtime, store, and tests modules, while the public quantum-context snapshot surface remains unchanged; the next bounded target is `gateway/studio/router/config.rs`
- `link_graph/index/ppr/kernel.rs` is now split into `link_graph/index/ppr/kernel/` with dedicated types, adjacency, iteration, runtime, and tests modules, while preserving the related-PPR kernel surface; the next bounded target is `gateway/studio/router/config.rs`
- `local_symbol` backs `search_ast`, `search_autocomplete`, and `search_definition`
- `reference_occurrence` now backs `search_references`
- `attachment` now backs `search_attachments`
- `repo_content_chunk` now backs code-search file fallback and removes long-lived source blob storage from `RepoIndexSnapshot`
- `repo_entity` now materializes repo analyzer modules, symbols, and examples into per-repository Lance tables
- repo-backed incremental planning now uses semantic equivalence instead of raw whole-file metadata equality: `repo_entity` changed-path detection reuses analyzer-row semantic hashes, `repo_content_chunk` fingerprints hash document contents, and shared repo staging keeps metadata-only refreshes on the existing Parquet table instead of rotating table identity
- `knowledge_section` now backs `search_knowledge` and non-code `search_intent`, with note body and section text materialized into Lance `search_text`
- non-code `search_intent` now merges `knowledge_section`, `local_symbol`, and repo-content hits into a single hybrid response path instead of treating intent as a pure knowledge lookup
- code-biased Studio search now queries `repo_entity` before repo-content fallback, and hybrid intent merges repo-entity hits into the same ranked response path
- `search::cache` now fronts repeat autocomplete, knowledge, non-repo intent, repo-scoped code search, and code-biased hybrid intent requests with corpus-aware Valkey keys and silent fallback to direct Lance reads when Valkey is unavailable
- TOML-first search cache precedence is now collapsed into
  `xiuxian-config-core`. `search/cache/config.rs` and
  `search/cache/runtime.rs` consume the shared `toml_first_env!` helper
  instead of carrying local precedence parsing, while the strict gateway
  startup-health target still treats malformed TOML as a boot-blocking
  dependency failure instead of silently downgrading to env
- the owner-path retirement is now landed too: `src/search/` is the sole
  implementation root for the search plane runtime, and the old
  `src/search_plane/` module has been removed
- backend-issued markdown display-math atoms now flow through `gateway/studio/analysis/markdown/compile.rs` into `math:block` retrieval atoms, and the markdown waterfall math-slot path is green
- `gateway/studio/types/search_index.rs` is now split into `gateway/studio/types/search_index/` with dedicated definitions, conversions, status rollups, and a split `tests/` tree (`counts.rs`, `reason.rs`, `mapping.rs`, `summary.rs`), while the public Studio search-index DTO façade remains unchanged
- `search_plane/service/core/status.rs` is now split into `search_plane/service/core/status/` with dedicated runtime, compaction, repo-status synthesis, and tests, while the public `SearchPlaneService` surface remains unchanged; the next bounded target is `search_plane/service/core/maintenance.rs`
- `search_plane/service/core/repo_runtime.rs` is now split into `search_plane/service/core/repo_runtime/` with dedicated helpers, reads, sync, and tests, while the public `SearchPlaneService` repo-runtime surface remains unchanged; the next bounded target is `search_plane/service/core/maintenance.rs`
- `search_plane/cache.rs` is now split into `search_plane/cache/` with dedicated config, construction, key generation, reads, writes, and tests modules, while the public `SearchPlaneCache` surface remains unchanged; the next bounded target is `search_plane/service/core/maintenance.rs`
- `search_plane/service/helpers.rs` is now split into `search_plane/service/helpers/` with dedicated paths, status, and cache helpers, while the public `search_plane::service::helpers` surface remains unchanged; the next bounded target is `search_plane/reference_occurrence/build.rs`
- `search_plane/knowledge_section/build.rs` is now split into `search_plane/knowledge_section/build/` with dedicated orchestration, paths, rows, types, write, and tests modules, while the public knowledge-section build surface remains unchanged; the next bounded target is `search_plane/reference_occurrence/build.rs`
- `gateway/studio/search/handlers/tests.rs` is now split into `gateway/studio/search/handlers/tests/` with dedicated helper setup, query, repo-content, code-search, and intent test modules, while the handler test surface remains unchanged; the next bounded target is `search_plane/reference_occurrence/build.rs`
- `search_plane/local_symbol/query.rs` is now split into `search_plane/local_symbol/query/` with dedicated shared, search, autocomplete, and tests modules, while the public local-symbol query surface remains unchanged; the next bounded target is `search_plane/reference_occurrence/build.rs`
- `search_plane/reference_occurrence/build.rs` is now split into `search_plane/reference_occurrence/build/` with dedicated orchestration, plan, extract, write, types, and tests modules, while the public reference-occurrence build surface remains unchanged; the next bounded target is `search_plane/local_symbol/build.rs`
- `search_plane/local_symbol/build.rs` is now split into `search_plane/local_symbol/build/` with dedicated orchestration, partitions, plan, write, types, and tests modules, while the public local-symbol build surface remains unchanged; the next bounded target is `search_plane/repo_entity/build.rs`
- `search_plane/repo_entity/build.rs` is now split into `search_plane/repo_entity/build/` with dedicated orchestration, plan, write, types, and tests modules, while the public repo-entity build surface remains unchanged; the next bounded target is `search_plane/attachment/build.rs`
- `search_plane/attachment/build.rs` is now split into `search_plane/attachment/build/` with dedicated orchestration, plan, extract, write, types, and tests modules, while the public attachment build surface remains unchanged; the next bounded target is `search_plane/repo_entity/schema.rs`
- `search_plane/repo_entity/schema.rs` is now split into `search_plane/repo_entity/schema/` with dedicated definitions, columns, helpers, rows, batches, and tests modules, while the public repo-entity schema surface remains unchanged; the next bounded target is `search_plane/coordinator.rs`
- `search_plane/coordinator.rs` is now split into `search_plane/coordinator/` with dedicated state, build, maintenance, types, and tests modules, while the public coordinator surface remains unchanged; the next bounded target is `gateway/studio/search/handlers/code_search.rs`
- `gateway/studio/repo_index/state/coordinator/runtime.rs` is now split into `gateway/studio/repo_index/state/coordinator/runtime/` with dedicated scheduler, task, repository, and state modules, while the public coordinator runtime surface remains unchanged; the next bounded target is `analyzers/service/projection/planner/workset.rs`
- `zhenfa_router/native/semantic_check/docs_governance/rendering.rs` is now split into `zhenfa_router/native/semantic_check/docs_governance/rendering/` with dedicated index, landing, footer, links, planning, and shared modules, while the public docs-governance rendering surface remains unchanged; the next bounded target is `skill_vfs/zhixing/indexer/resource_graph.rs`
- `skill_vfs/zhixing/indexer/resource_graph.rs` is now split into `skill_vfs/zhixing/indexer/resource_graph/` with dedicated helpers, references, and skills modules, while the public zhixing indexer surface remains unchanged; the next bounded target is `search_plane/knowledge_section/query.rs`
- `search_plane/knowledge_section/query.rs` is now split into `search_plane/knowledge_section/query/` with dedicated errors, ranking, candidates, search, and tests modules, while the public knowledge-section query surface remains unchanged; since then the gateway command slice has also landed, and the next bounded target is `search_plane/service/core/maintenance.rs`
- `bin/wendao/execute/gateway.rs` is now split into `bin/wendao/execute/gateway/` with `mod.rs`, `command.rs`, `config.rs`, `health.rs`, `registry.rs`, `shared.rs`, and `status.rs`, while preserving the CLI gateway entrypoint and gateway test coverage; the next bounded target is `search_plane/service/core/maintenance.rs`
- `gateway/studio/router/handlers/repo/analysis.rs` is now split into `gateway/studio/router/handlers/repo/analysis/` with `mod.rs`, `overview.rs`, `search.rs`, `doc_coverage.rs`, `sync.rs`, and `tests.rs`, while preserving the repo overview, repo search, doc coverage, and sync endpoint surfaces; the next bounded target is `search_plane/service/core/construction.rs`
- `analyzers/service/projection/planner.rs` is now split into `analyzers/service/projection/planner/` with dedicated planner API, scoring, workset, and tests modules, while the public planner surface remains unchanged
- `analyzers/service/projection/planner/api.rs` is now split into `analyzers/service/projection/planner/api/` with dedicated item, search, queue, and rank modules, while the public planner API surface remains unchanged; the next bounded target is `link_graph/index/build/assemble.rs`
- `analyzers/service/projection/planner/workset.rs` is now split into `analyzers/service/projection/planner/workset/` with dedicated orchestration, groups, balance, strategy, and math modules, while the public planner workset surface remains unchanged; the next bounded target is `search_plane/repo_entity/query/hydrate.rs`
- `search_plane/repo_entity/query.rs` is now split into `search_plane/repo_entity/query/` with dedicated execution, hydrate, prepare, search, and types modules, while the public repo-entity query surface remains unchanged; the next bounded target is `gateway/studio/search/handlers/tests/code_search.rs`
- `search_plane/service/tests/status.rs` is now split into `search_plane/service/tests/status/` with dedicated repo-content, maintenance, and issue tests plus shared helpers, while the status test surface remains unchanged; the next bounded target is `search_plane/service/core/maintenance.rs`
- the former `git/checkout/tests/` split is now retired entirely because
  repo-substrate contract coverage moved into `xiuxian-git-repo`, while
  the remaining Wendao-side caller bridge now lives under
  `analyzers/repo_source.rs` and `xiuxian-wendao/src/git/` no longer exists;
  the next bounded target is
  `zhenfa_router/native/semantic_check/docs_governance/rendering.rs`
- `zhenfa_router/native/semantic_check/checks.rs` is now split into `zhenfa_router/native/semantic_check/checks/` with dedicated contracts, identity, links, observations, and structure modules, while the public semantic-check surface remains unchanged; the next bounded target is `zhenfa_router/native/semantic_check/docs_governance/rendering.rs`
- search-plane Valkey client resolution now uses the Wendao-local thin helper layer, so the first shared transport primitive is centralized without moving cache keyspace or manifest semantics out of the search-plane domain
- the whole-search-plane DataFusion refactor plan is now tightened around a mandatory two-stage query rule: narrow filter/ranking columns are scanned first, and wide payload columns are hydrated only for bounded Top-K identities
- that same DataFusion plan now treats statistics-aware pruning as selective rather than universal; Parquet writers should preserve statistics on contract-safe scalar quality columns, but cross-corpus join views and DataFusion runtime-priority scheduling remain deferred until after single-corpus parity is landed
- repo-backed query keys now derive from local corpus state plus repo-index status fragments for `repo_entity` and `repo_content_chunk`, so repo-aware caching no longer has to bypass Valkey just because the response depends on repo publication state
- `repo_entity` and `repo_content_chunk` now emit explicit publication records after successful table writes, so the search plane can distinguish "published rows that remain readable" from transient repo-index phase churn
- repo publication manifests now also carry `source_revision`, and repo indexing threads `sync_result.revision` into both repo-backed publish paths so published tables are pinned to the exact source revision that produced them
- repo-backed publication planning was already incremental before the latest repo-index audit. `repo_entity` and `repo_content_chunk` both compute per-file fingerprints and reuse prior publications through staged mutation (`noop`, revision-only refresh, or clone-and-mutate) instead of blindly rebuilding every row on every refresh
- practical repo-index incrementality is now pushed one layer earlier too. `RepoIndexCoordinator::process_task(...)` now skips analyzer execution and code-document collection for managed remotes when both repo-backed corpora already expose readable Parquet publications for the synced revision, so steady-state remote refreshes no longer pay the full post-sync indexing cost just to rediscover an unchanged publication
- Julia repo-index incrementality now also uses parser-summary semantics instead of raw source churn for safe leaf-file edits. `prepare_incremental_analysis(...)` reads the previous revision blob through `xiuxian-git-repo`, compares previous/current Julia parser-summary file fingerprints, and reuses cached analysis when those semantic fingerprints match, so comment-only or formatting-only leaf edits stop before incremental analyzer execution
- single-plugin Modelica repo-index incrementality now also reuses cached analysis for AST-equivalent `.mo` churn. `prepare_incremental_analysis(...)` compares previous/current Modelica semantic fingerprints and short-circuits to cached analysis reuse when they match; those fingerprints now include both parser-summary payloads and file-local doc-topology markers so large `package.mo` and UsersGuide `.mo` files can reuse safely without misclassifying doc-surface edits as no-ops, while real Modelica semantic edits still fall back to full analysis until Modelica incremental overlay merge grows first-class ownership for imports and repository-level module reconstruction
- supported mixed `julia+modelica` repositories now share that same repo-index semantic-reuse fast path for AST-equivalent churn. When every analysis-affecting modified `.jl` or `.mo` file keeps the same parser-summary fingerprint across revisions, `prepare_incremental_analysis(...)` reuses cached analysis directly without widening into mixed-plugin overlay merge; unknown-plugin mixes still stay on the conservative fallback path
- Julia analyzer cache identity now uses that same semantic owner for `src/*.jl` too. `RepositoryAnalysisCacheKey.analysis_identity` now prefers Julia parser-summary file fingerprints over raw file bytes for single-plugin Julia source files, while keeping a raw-contents fallback when parser-summary is unavailable so the cache-key path does not become a mandatory parser-summary bootstrap edge
- Modelica analyzer cache identity now follows the same bounded semantic-owner rule for single-plugin `.mo` repositories. `RepositoryAnalysisCacheKey.analysis_identity` prefers Modelica parser-summary plus doc-surface semantic fingerprints over raw contents for AST-equivalent source churn, but preserves the raw-contents fallback when parser-summary is unavailable so cache-key lookup does not require a mandatory Modelica parser-summary bootstrap
- the next bounded mixed-plugin owner is now closed too: repositories that only register `julia` and `modelica` keep parser-summary semantic identity for `.jl` and `.mo` files instead of dropping back to raw contents just because the repository is mixed, while unknown-plugin mixes still stay on the conservative raw-contents path until a broader capability owner exists
- analyzer-cache identity now also has one generic AST structural owner instead of only language-specific special cases. `xiuxian-ast` provides a generic semantic fingerprint helper for supported structural languages, and `RepositoryAnalysisCacheKey.analysis_identity` now dispatches per file: parser-summary owners still win for Julia and Modelica, but every remaining AST-supported file can reuse structural fingerprints by file language alone without requiring a matching repo-intelligence plugin id, while unowned paths still stay on the conservative raw-contents fallback
- note-based markdown incrementality now also has one parser-owned semantic owner instead of corpus-local row or hit hashing. `xiuxian_wendao_parsers::note::fingerprint_markdown_note` computes one semantic fingerprint from `MarkdownNote`, `search::markdown_snapshot` caches that parser fingerprint beside the adapted note parse, and `knowledge_section` plus `attachment` now reuse it before row or hit materialization so metadata-only note edits stop before rebuilding Wendao-owned payloads
- the markdown branch of `local_symbol` now follows the same parser-owned boundary instead of hashing Wendao-owned markdown `AstSearchHit` payloads. `xiuxian_wendao_parsers::note::fingerprint_markdown_symbol_surface` computes one parser-owned symbol fingerprint from `comrak` structural traversal plus parser-owned section metadata, `search::markdown_snapshot` caches that symbol fingerprint and lazily materializes markdown AST hits, and `local_symbol` now compares the parser-owned symbol fingerprint before AST-hit materialization so metadata-only Markdown edits skip rebuild while heading/task/property/observation changes still invalidate
- repo-index incremental preflight now reuses that same semantic-owner dispatcher for known semantic-owner plugin sets such as `rust`. When every analysis-affecting changed path remains semantically equivalent, the coordinator reloads the previous cached analysis directly instead of widening into a new language-specific incremental branch; modified directory diff entries recurse into child semantic-owner files before the reuse decision is made, and unknown plugin mixes still stay on the conservative fallback
- the next bounded mixed shape is now code-proven too: repositories that only register `modelica` plus one generic AST-supported owner such as `rust` keep the Modelica parser-summary owner on `.mo` files and the generic structural owner on `.rs` files, and both analyzer-cache identity and repo-index incremental preflight reuse can stay active on AST-equivalent churn instead of falling back to raw bytes or full analysis for the entire repository
- the mixed `modelica+rust` shape now has bundle-level analyzer-service proof too: `analyze_registered_repository_bundle(...)` keeps reusing the cached full repository analysis when either the generic Rust path or the specialized Modelica path changes in an AST-equivalent way, so the mixed semantic-owner rule is now closed across cache-key selection, preflight reuse, and bundle caching
- the conservative unknown-mix rule is now code-backed too: semantic-owner dispatch now keys off all configured plugin ids rather than only repo-intelligence ids, so bounded unknown mixes such as `rust+ast-grep` and `modelica+ast-grep` stay on the conservative raw-contents or full-analysis fallback path across analyzer-cache identity, bundle caching, and repo-index incremental preflight
- repo-index coordinator fingerprinting now follows that same all-configured-plugin contract, so adding or removing a search-only or otherwise unknown plugin such as `ast-grep` changes queue/status identity and re-enqueues the repository instead of being treated as a fingerprint-identical sync
- that all-configured-plugin contract is also normalized now at the proof level: order-only churn and duplicate plugin declarations do not re-enqueue the repository, so repo-index identity is keyed to the effective plugin set rather than config ordering noise
- the same normalized-plugin contract now also covers repository analysis cache keys and repo query cache keys: reordered or duplicate-only declarations yield the same `RepositoryAnalysisCacheKey`, and downstream repo query keys stay stable because they reuse that normalized analysis identity
- gateway repo-search callers now have the same proof too: both the repo-search helper path and the projected-page search caller build handler cache keys from the cached normalized `RepositoryAnalysisCacheKey` rather than any hand-built plugin fragment, so reorder-only or duplicate plugin churn does not perturb the gateway hot-query surface
- the shared repo-aware hot-query builder now also has an exact repo-set normalization proof: reordered or duplicate-only `repo_ids` yield the same cache key, so higher gateway callers such as intent hybrid and code search inherit repo-set stability instead of config-order noise
- intent hybrid now also has a direct caller proof for that inherited contract: `build_intent_cache_key(...)` yields the same code-biased cache key when the same repo set is declared in a different studio config order
- code search now also has a direct caller proof for that inherited contract: `build_code_search_response(...)` yields the same repo-aware code-search cache key when the same repo set is declared in a different studio config order without an explicit repository hint
- repo-backed corpus status synthesis now also has an exact higher-level normalization proof: `SearchPlaneService::synthesize_repo_table_status(...)` yields the same ready published status when the same repo publication/runtime rows are iterated in a different order
- repo index status diagnostics transport now also has an explicit boundary proof: reordering `repos` rows leaves the recomputed diagnostics summary and injected summary fields unchanged when `active_repo_ids` stays fixed, while the raw `repos` payload still preserves incoming order
- repo-backed runtime hydration now also has an exact higher-level proof: `SearchPlaneService::status_with_repo_runtime(...)` yields the same repo entity and repo content status rows when the same ready published repo rows are reordered before synchronization
- Studio `search_index_status(...)` transport now also has an exact direct handler proof: the same reordered ready published repo rows still yield the same repo-backed status fields and corpus rows after serialization
- repo-index restart warm-start is now more selective. `RepoIndexCoordinator::sync_repositories(...)` hydrates persisted repo-backed state plus config fingerprints before enqueue for local checkouts, and it now also allows a bounded managed-remote fast path when local `Status` metadata still proves the published revision is current: `manual` remotes may reuse healthy in-sync managed assets directly; `fetch` remotes may do so when the local mirror fetch timestamp is still `Fresh` or `Aging`; the same `fetch` remotes may also reuse healthy same-revision assets after `last_fetched_at` has gone `Stale` when a recent persisted upstream probe marker still matches that revision; retryable probe failures no longer destroy that recent success proof, so startup hydration may still reuse a healthy same-revision readable remote while the preserved last-success marker remains within the accepted freshness window; and commit-pinned `fetch` remotes may likewise reuse healthy same-revision assets even after the local fetch timestamp has gone `Stale` because upstream drift no longer changes the selected revision
- managed-remote `Ensure` also does less redundant local Git work now. Fetch-policy remotes first probe upstream ref advertisements and only fall through to a full mirror fetch when the advertised target revision actually changed or cannot be determined, so unchanged branch/default/tag remotes stop downloading a packfile on every startup cycle; the no-fetch probe-success path now also persists a small mirror-side probe marker so startup hydration can reuse that proof later without redefining `last_fetched_at`, and the retryable-failure path preserves the most recent matching success marker instead of replacing it with weaker outage-only state; when a mirror fetch does still happen, Wendao also skips the follow-up checkout-side fetch whenever the desired revision from the mirror already matches the current checkout revision. Commit-pinned `fetch` remotes sit on an even stronger immutable fast path and skip that upstream mirror fetch entirely once the local mirror already contains the pinned commit
- `code_search` and code-biased hybrid `intent` now keep serving published repo-backed tables while a repo refresh is in flight, instead of collapsing to snapshot-miss pending state as soon as repo indexing starts
- repo-aware hot-query keys now preserve stable publication identity for steady-state ready reads, but append repo phase plus current/published revision fragments while refresh or ready-state drift is present, so cache hits no longer hide refresh-state or revision-mismatch responses
- repo-aware hot-query caches now survive restart one layer better too. The
  shared repo-search query cache still checks its in-process map first, but it
  now falls through to Wendao-owned Valkey using full
  `RepositorySearchQueryCacheKey` identity and writes hits back into memory, so
  hot repo `module`, `symbol`, `example`, `import`, and `projected-page` reads
  can recover after gateway restart without changing the published-table trust
  boundary
- repo-backed status synthesis now preserves published row counts, fragment counts, fingerprints, and publish timestamps while refresh work is active, which gives `/api/search/index/status` a stable read-availability view even before repo corpora have first-class coordinator epochs
- repo-backed status synthesis now also surfaces ready-state revision drift explicitly. A repo reported as `ready` but backed by a manifest for a different revision is treated as a manifest consistency error instead of being silently accepted as current
- search-plane lifecycle now includes a first-class `degraded` phase for readable-but-inconsistent corpora, and repo-backed status synthesis upgrades to `degraded` whenever published rows still serve reads but manifest drift or partial repo failures are present
- Studio `/api/search/index/status` now exposes `degraded` in both aggregate counters and per-corpus phases, so clients can distinguish fully ready corpora from stale/partial repo-backed availability without parsing human error strings
- repo-backed status now also emits machine-readable `issues` metadata. Each issue carries a stable code plus repo/revision/readability context, so clients can branch on manifest-missing, revision-missing, revision-mismatch, and repo-index-failed conditions without scraping `last_error`
- repo-backed status now also emits `issueSummary`, which compresses the raw issue list into `family`, `primaryCode`, `issueCount`, and `readableIssueCount` so UI consumers can render a stable summary without reimplementing issue bucketing logic
- each corpus status row now also emits `statusReason`, which projects lifecycle phase plus issue state into one direct decision surface: `code`, `severity`, `action`, and `readable`. Initial indexing reports `warming_up`, refresh indexing reports `refreshing`, failed rebuilds report `build_failed`, and repo-backed consistency issues map directly to repo resync or repo-sync inspection actions
- `statusReason` now also absorbs maintenance for healthy readable corpora. A ready corpus with queued background compaction reports `compaction_pending`, so status consumers no longer need to independently join `phase` and `maintenance.compaction_pending` to detect search-plane optimization work
- maintenance runtime telemetry is now explicit. Corpus maintenance rows expose `compaction_running`, healthy readable corpora report `statusReason = compacting` while a compaction task is actually in flight, and the aggregate `/api/search/index/status` reason priority now prefers `compacting` over `compaction_pending`
- repo-backed corpora now synthesize stable epoch semantics from publication manifests and live repo-index activity. Published repo tables fold into a stable synthetic `active_epoch`, and queued/checking/syncing/indexing repo work folds into a synthetic `staging_epoch`, so repo-backed status rows now participate in the same epoch vocabulary as local corpora without losing the identity of the readable publication during refresh
- repo publication manifests now persist `active_epoch` explicitly. `SearchRepoManifestRecord` stores a stable epoch derived from `publication_id`, repo-backed status synthesis now aggregates those persisted epoch values, and legacy manifests fall back to deterministic epoch derivation so cached publication state remains backward-compatible
- repo-backed status is no longer purely overlay-based. `status_with_repo_content` now synchronizes synthesized `RepoEntity` and `RepoContentChunk` rows back into `SearchPlaneCoordinator`, so subsequent `service.status()` snapshots reuse the last repo-backed `active_epoch`, `staging_epoch`, fingerprint, and row-count state without recomputing a one-shot overlay
- `/api/search/index/status` now also emits a response-level aggregate `statusReason`. The aggregate selects the dominant corpus reason by `severity -> code` priority and reports `affectedCorpusCount`, `readableCorpusCount`, and `blockingCorpusCount`, which gives clients a stable top-level banner surface without rescanning every corpus row
- the old production dependency on `RepoIndexSnapshot` has been removed; repo snapshots are now only a test shim and no longer determine whether repo-backed search paths can read data
- repo-backed publication records now carry explicit `publication_id` values and can be persisted into Valkey. Request-path repo search and repo-aware cache keys can therefore hydrate publication state from Valkey after process restarts instead of relying only on in-memory state
- publication reads are now unified by trust boundary: search, cache invalidation, and repo-backed status synthesis all use in-memory publication state plus Valkey manifests only. Disk tables are no longer treated as an implicit publication fallback when the manifest is missing
- corpus publish now schedules background `xiuxian-vector` compaction when maintenance thresholds trip, and the recorded fragment count is refreshed after compaction completes
- same-fingerprint corpus requests no longer short-circuit across schema changes; schema version now participates in build identity so a schema bump forces a fresh staging epoch
- repo-aware handler control flow is now also centralized on search-plane semantics. `StudioState::synchronize_repo_search_runtime(...)` is the only handler bridge to repo-index status, `SearchPlaneService::repo_search_publication_state(...)` owns `entity/content/searchable/pending/skipped` classification, and `code_search` plus code-biased `intent` no longer open-code repo phase decisions
- repo runtime synchronization is now pushed from the producer side instead of sampled from the request path. `RepoIndexCoordinator::refresh_status_snapshot()` now sends the authoritative aggregate snapshot into `SearchPlaneService::synchronize_repo_runtime(...)`, runtime synchronization replaces the full repo-runtime map on every refresh, and the Studio search handlers no longer call `repo_index.status_response(...)` just to warm repo runtime before cache-key generation or pending/skipped classification
- repo runtime persistence is now folded entirely into the combined repo-corpus model. `SearchPlaneService::synchronize_repo_runtime(...)` persists refreshed `SearchRepoCorpusRecord` rows plus the combined `SearchRepoCorpusSnapshotRecord`, rather than dual-writing standalone runtime keys
- repo-aware read paths now hydrate runtime from persisted combined repo-corpus rows on memory miss. `repo_search_publication_state(...)`, repo-backed status synthesis, and repo-aware cache-key generation no longer read standalone runtime rows
- the old `SearchRepoRuntimeSnapshotRecord` path is gone. `SearchPlaneService::status_with_repo_runtime()` now rebuilds repo-backed corpus rows only from the search-plane-owned combined repo-corpus snapshot plus publication manifests, with no separate runtime snapshot key
- Studio `/api/search/index/status` now reads repo-backed corpus state from `SearchPlaneService::status_with_repo_runtime()` rather than calling back into `repo_index.status_response(...)`. The status surface, repo-aware cache identity, and repo availability checks now consume the same search-plane-owned runtime source
- producer-side repo snapshot refresh now also warms repo-backed corpus coordinator rows. `RepoIndexCoordinator::refresh_status_snapshot()` still pushes the authoritative runtime snapshot into `SearchPlaneService`, and that push now fans out into repo-backed corpus status replacement as well as runtime persistence
- repo runtime and publication are now also materialized into combined repo-corpus rows. `SearchRepoCorpusRecord` stores `runtime + publication` for one repo/corpus pair, and `SearchRepoCorpusSnapshotRecord` stores the full search-plane-owned repo-backed view for restart-stable hydration
- repo-aware hot-query identity, repo-backed availability, and repo-backed status synthesis now read combined repo-corpus rows first instead of rebuilding state from runtime snapshots plus publication manifests as two primary inputs
- combined repo-corpus reads now reconcile against the freshest in-memory runtime/publication before serving the hot path, which removes the async-persistence lag that previously let a repo refresh reuse a stale cache identity until background cache writes completed
- the process-local repo-backed truth has now collapsed to one structure. `SearchPlaneService` no longer carries separate in-memory `repo_runtime` or `repo_publications` maps; combined `repo_corpus_records` now hold the only in-process repo-backed state
- legacy standalone `repo_runtime` Valkey keys and the old runtime snapshot key have now been removed. Persisted combined repo-corpus rows are the only runtime recovery surface after restart; per-corpus publication manifests remain a narrow publication-only fallback beneath that combined model
- that remaining publication-only fallback is now removed from repo-backed read recovery too. `repo_search_publication_state(...)`, repo-backed status synthesis, and repo-aware cache-key generation now recover published state only from combined repo-corpus record/snapshot data, not from manifest-only Valkey rows
- repo publication manifests are now removed entirely. Repo-backed publication metadata is embedded only in combined `SearchRepoCorpusRecord` / `SearchRepoCorpusSnapshotRecord` state as `SearchRepoPublicationRecord`, which eliminates the last repo-specific manifest key/API and the remaining dual-write drift risk
- the repo-intelligence cache gap upstream of those combined repo-corpus records is now partially closed too. `ValkeyAnalysisCache` is no longer a placeholder: normalized analyzer output is persisted as versioned JSON under a repository/revision/plugin-scoped Valkey key, and analyzer service load paths now warm the in-process cache from that persisted snapshot when available
- that analyzer cache intentionally remains separate from the repo-backed search-plane publication state. Repo-corpus readiness still comes from combined repo-corpus rows, while repo-analysis search fallback and cached-only repo gateway lanes can now recover the analyzer snapshot itself after process restart
- repo-backed human-readable issue/status text now says "published state" instead of "published manifest", while machine-readable issue/status codes remain unchanged for client compatibility
- `xiuxian-vector` now exposes streaming columnar scan and FTS consumers. `repo_entity` and `repo_content_chunk` query lanes now process Lance batches incrementally and trim rerank working sets to bounded candidate/path windows instead of materializing every recalled `RecordBatch` before scoring
- `knowledge_section` and `attachment` now follow the same bounded streaming rerank discipline. Both lanes consume Lance batches incrementally, and `knowledge_section` now also trims its path-dedup working set so note-heavy searches do not scale with full recall volume
- `search_plane::ranking` now centralizes the retained-window policy shared by `repo_entity`, `repo_content_chunk`, `knowledge_section`, and `attachment`, so the large corpus lanes no longer open-code duplicate trim/sort/window helpers
- recent bounded-rerank telemetry is now first-class runtime state. `SearchPlaneService` records recent per-corpus query telemetry, the larger streaming query lanes report scanned rows, matched rows, working-set budget, trim threshold, peak working set, and dropped rows, and `/api/search/index/status` now exposes that data as `lastQueryTelemetry` without changing any search response payloads
- `/api/search/index/status` now also emits a response-level `queryTelemetrySummary`, so Studio can render one aggregate view of recent search pressure across corpora instead of re-summing `lastQueryTelemetry` client-side
- `queryTelemetrySummary` is now tied to the formal performance lane. The `tests/performance/gateway_search.rs` target has a dedicated `search_index_status` warm-cache case that first seeds telemetry through code-search, then requires `/api/search/index/status` to return the summary inside an explicit latency/throughput budget
- the gateway warm-cache perf lane is now owned exclusively by the formal `xiuxian-testing-gate` target in `tests/performance/gateway_search.rs`. `repo_module_search`, `repo_symbol_search`, `repo_example_search`, `repo_projected_page_search`, `studio_code_search`, and `search_index_status` all validate there as isolated per-case gates under `serial_test::file_serial`
- `xiuxian-wendao-runtime::WendaoFlightService` now keeps a bounded reusable route-payload cache instead of a one-shot get/do_get handoff only. Repeated identical `GetFlightInfo` requests can reuse the already materialized batch/app-metadata payload instead of re-entering the provider, and focused runtime server tests now lock that behavior for both repeated `GetFlightInfo` and the existing `GetFlightInfo -> DoGet` handoff
- the same cached payload path now also lazily owns encoded `DoGet` frames, so repeated identical `DoGet` requests can reuse the already encoded Flight payload instead of rerunning batch-to-Flight encoding work after the first request
- the frontend live Flight perf harness now also normalizes loopback origins,
  avoids redundant runtime config sync retries, and waits for
  `/api/search/index/status` plus `/api/repo/index/status` to settle before
  measured Flight runs begin. That turns the suite into a steady-state perf
  proof instead of a restart-noise detector only
- the latest steady-state same-port live proof on the default `127.0.0.1:9517` gateway after the `DoGet` encoded-frame reuse slice reported `2.15ms` overall average, `3.31ms` P95, with `GetFlightInfo` averaging `0.86ms` and `DoGet` averaging `0.79ms` across the checked-in 179-repository config
- the production-line restart audit also exposed an operator wiring issue outside the request path: `process.nix` had been recording the wrapper-shell PID in `wendao.pid`, while readiness compared that pidfile against `x-wendao-process-id`. The gateway process entry now records the actual child PID so local operator restarts can validate against the real listener process instead
- the next fresh restart proof exposed the remaining blocker more precisely: `repo_entity` / `repo_content_chunk` could reenter a transient runtime-only in-memory state after restart, which made `/api/search/index/status` report `published_manifest_missing` and kept the live perf harness waiting for steady state even though persisted repo-corpus publication records already existed
- read-time repo-runtime recovery now repairs that restart gap. `repo_corpus_record_for_reads()` and `repo_corpus_snapshot_for_reads()` now rehydrate publication/maintenance from persisted repo-corpus record cache/local state even when an in-memory runtime-only record already exists, so restart-time runtime sync no longer masks persisted repo-backed publication state. The manifest-only fallback rule remains unchanged
- a follow-up live audit on the restarted `9517` gateway narrowed the remaining
  drift further: the repo-corpus record on disk could already be correct while
  `/api/search/index/status` still surfaced stale `published_manifest_missing`
  issues. The culprit was producer-side, not reader-side: repeated repo-index
  phase updates spawned asynchronous runtime-refresh tasks without any
  generation fence, so an older task could finish after a newer snapshot and
  overwrite the latest repo-backed status synthesis
- `search_plane/service/core/repo_runtime/sync.rs` now carries an explicit
  repo-runtime generation fence. Only the newest async runtime-refresh task
  may delete repo runtime rows, rewrite combined repo-corpus cache/snapshot
  records, or recompute repo-backed corpus status, which removes the
  restart-time stale-snapshot overwrite path without changing the formal
  Flight-vs-JSON surface boundary
- the last production-line blocker turned out to be the frontend perf harness
  gate, not another gateway defect. `liveFlightSearchPerf.test.ts` had still
  been waiting for repo/background indexing to drain completely after restart;
  it now accepts the readable recovery window instead: zero blocking
  repo-backed corpus issues, readable `repo_entity` / `repo_content_chunk`
  publications, and visible repo-index progress are sufficient to start
  measuring
- with that gate repaired, the restarted local `9517` gateway now has an
  explicit post-restart live proof too: `2.93ms` overall average, `4.52ms`
  P95, `GetFlightInfo` average `1.24ms`, and `DoGet` average `0.98ms` while
  the gateway was serving from readable repo-backed corpora during recovery;
  later in the same restart cycle repo-index convergence reached
  `ready=152`, `failed=9`, `unsupported=16`, `queued=0`, `active=0`
- repo analyzer artifact fast paths are repaired and reconnected. `analyzers::service::search::artifacts` is now the single artifact builder/cache entry point, projected-page artifact search is re-exported from the current projection module surface, and the stale duplicate artifact helper was removed from `documents.rs`
- producer-side runtime refresh now follows the same rule. `refresh_repo_corpus_records(...)` no longer reattaches publication from persisted manifest rows after a memory miss; if the combined repo-corpus cache is gone, runtime refresh restores runtime only and leaves publication absent until a fresh publish or combined snapshot recovery happens
- restoring the runtime-push slice to a clean verification state also required a handful of small modularization repairs outside the hot path: parser section test helpers now have matching visibility, `semantic_check` test re-exports now route through `test_api`, stale parser/link-graph import drift has been aligned, and the unused `LinkGraphPromotedOverlayTelemetry` import has been removed
- bounded verification for the runtime-snapshot source slice also required one small `skill_vfs/zhixing/resources` repair: the stray `text.rs` test-module declaration was removed and stale imports were trimmed so the crate compiled cleanly again under `cargo nextest`
- the repo-search control-flow slice also closed its local warning debt: the temporary unused `RepoSearchPublicationState` re-export is gone, and stale unused analyzer-service helper re-exports were removed so this lane validates cleanly again
- the follow-up repair lane is now closed as well: repo-publication clear synchronously drops the test-only cache shadow before async cleanup runs, the VFS relative-doc navigation test now establishes explicit project/config roots, and the temporary `repo_phase` assertion helper now lives entirely in the tests module, so both `cargo test -p xiuxian-wendao --lib` and `cargo check -p xiuxian-wendao` are clean for this slice again
- full lib-test verification also required calibrating the Studio bootstrap performance guard. `StudioState::new()` now uses the same 150ms cold-start threshold that survived the final Tier-3 gate, which removes bootstrap jitter from test lanes while leaving tighter request-path latency guards unchanged
- Studio now exposes `/api/search/index/status` for corpus lifecycle visibility without changing existing search payload contracts
- `/api/search/index/status` now synthesizes `repo_content_chunk` readiness from live repo-index phases plus published per-repo Lance table metadata, so repo content no longer appears as a permanently idle corpus
- `/api/search/index/status` now also synthesizes `repo_entity` readiness from live repo-index phases plus published per-repo Lance table metadata, so both repo-backed corpora report real readiness and failure state
- search snapshots now assert non-empty knowledge hits from published corpora rather than empty request-path placeholders
- the formal verification lane for the repo-publication epoch slice is back on `cargo nextest`; the remaining blockers were unrelated `semantic_check` test-visibility drift and env-mutation in `valkey_common` tests, both now repaired
- the unified repo-corpus single-source slice is now stricter: combined repo-corpus data is the only repo-backed publication and runtime recovery surface, and the larger active corpora now all stream batches instead of collecting them all first
- shared rerank-window helpers are now extracted into `search_plane::ranking`, and the follow-up priority is explicit recall-window / memory-budget telemetry rather than one more round of helper deduplication
- the Tier-3 burn-down is now closed for `xiuxian-wendao`. The remaining production query-module lint debt in `attachment`, `knowledge_section`, `repo_content_chunk`, and `repo_entity` was split or normalized away, the test-only `expect/unwrap` front in search-plane and analyzer support was burned down, and `cargo clippy -p xiuxian-wendao --all-targets --all-features -- -D warnings` is green
- the final gate closure also tightened shared test support. Search-plane service tests now centralize bounded panic-on-failure helpers and repo publication setup, analyzer projection/search tests no longer depend on repeated `expect` chains in the touched scope, and the larger `xiuxian-wendao --lib` lane is back to a stable warning-free baseline for this feature
- the formal verification lane is now fully restored: `cargo nextest run -p xiuxian-wendao` is green again after the last cold-start performance repair, and the search-plane/runtime/status slices are no longer depending on ad hoc narrow test filters to stay landable
- full-gate verification also required a final calibration of the Studio bootstrap guard. `StudioState::new()` now uses a 150ms cold-start threshold in the VFS performance test while leaving the tighter API latency guard unchanged, which removes bootstrap jitter from the industrial gate without weakening request-path budgets
- the broader gateway perf lane is now consolidated on the formal `xiuxian-testing-gate` performance target. The six isolated warm-cache cases run there under recalibrated per-case budgets through `src/gateway/studio/perf_support.rs` and `tests/performance/gateway_search.rs`
- the execution entrypoints are now explicit too. `just rust-wendao-performance-gate` expands into `rust-wendao-performance-quick` and `rust-wendao-performance-gateway-formal`, `.github/workflows/xiuxian-wendao-performance-gates.yaml` mirrors that split so CI reports the quick target and the formal gateway proof separately, and both `just` recipes now consume one shared `xiuxian_wendao_gateway_formal_filter` source instead of repeating the six-case inventory
- the formal gateway proof now carries the live runner-aware budget policy through `tests/performance/support/gateway.rs`: Linux keeps the stricter baseline, local/other runs use looser defaults, the local `repo_symbol_search` p95 baseline is now `1.35ms`, and per-case env overrides remain available through `XIUXIAN_WENDAO_GATEWAY_PERF_*`
- `tests/performance/support/` is now split by concern. `link_graph.rs` keeps the related-search fixture helpers, while `gateway.rs` owns formal gateway budget/profile parsing and keeps `gateway_search.rs` focused on endpoint assertions
- the old lib-only gateway perf calibration lane is now removed, and the follow-up runtime-stability fix is landed too: `RepoIndexCoordinator` now has an explicit stop path, `StudioState::stop_background_services()` stops both repo and symbol coordinators, the formal perf fixture calls that unified shutdown on drop, and the six-case formal gateway bundle is back to a clean `6 passed, 0 leaky`
- `local_symbol` and `reference_occurrence` are now aligned with the larger corpora on the read path too: both search queries stream projected Lance batches through bounded rerank windows, both record query telemetry into search-plane status snapshots, and `local_symbol` now explicitly returns no hits for an empty query instead of treating the empty string as a universal prefix
- the remaining `local_symbol` autocomplete scan is now aligned too: uncached autocomplete suggestions stream through the same bounded retained-window discipline, and `local_symbol` status telemetry now distinguishes `scope=search` from `scope=autocomplete`
- the compile-graph thinning lane is now active too: Arrow/DataFusion-heavy SQL
  query adapters are explicitly gated behind `search-runtime`, and the
  Arrow-backed link-graph quantum-fusion scorer/export surface is explicitly
  gated behind `vector-store`, so `cargo check -p xiuxian-wendao --lib
--no-default-features` stops paying for those default-disabled surfaces by
  construction
- that compile-thinning lane now has its next bounded owner cut too:
  `graphql-parser` and `xiuxian-vector` are explicit `search-runtime`
  dependencies, `notify` plus `notify-debouncer-full` are explicit
  `zhenfa-router` dependencies, and `reqwest` plus `xiuxian-zhenfa` now sit
  behind their real `vector-store` / `zhenfa-router` / `studio` owners, so the
  remaining direct `--no-default-features` edges are down to `tantivy`,
  `xiuxian-ast`, and `xiuxian-wendao-runtime`
- the repo-index parser-summary incremental lane now has its first true
  Modelica semantic overlay cut. Single-plugin safe leaf API `.mo` files now
  incremental-merge real declaration changes on top of the previous analysis
  instead of falling back immediately to a full rebuild, while `package.mo`,
  import-bearing files, documentation-annotation files, and broader
  repository-shape Modelica edits remain on the conservative fallback path
- that same Modelica overlay lane now also owns import-bearing leaf files.
  Plugin file overlays can emit import rows, repo-analysis import rows now keep
  repository-relative source paths, and changed-path incremental merge replaces
  stale imports by path together with symbols. Documentation-annotation files
  and broader package-shape Modelica edits remain the next conservative
  fallback boundary
- that same bounded overlay lane now also owns one safe root `package.mo`
  file at a time. Root-package incremental admission now uses a lightweight
  lexical owner for package identity, import extraction, and semantic
  fingerprinting, so very large root `package.mo` files no longer need to
  re-enter Modelica parser-summary just to prove AST-equivalent churn or apply
  bounded root-local import/documentation overlays; root package renames,
  UsersGuide/support `package.mo`, and broader package-shape edits still stay
  on the conservative full-analysis path
- that same safe root-package owner now also applies during full repository
  analysis. When the repository root `package.mo` stays inside the bounded
  no-nested-declarations contract, repository analysis reuses lexical root
  package metadata for imports and skips root-package symbol parser-summary
  work entirely, so large root `package.mo` files no longer need Modelica
  parser-summary just to materialize root-local import/documentation rows
  during full analysis; nested declarations, UsersGuide/support `package.mo`,
  and broader package-shape files still stay on the conservative fallback path
- that same lexical package owner now also extends to safe API-surface nested
  `package.mo` files during full repository analysis and repo-owned
  `analyze_file(...)`. When a nested package file keeps package identity aligned
  with its directory path and stays inside the bounded import/doc-only
  contract, Wendao skips package-file parser-summary symbol work and reuses
  lexical import metadata there too; package-name/path mismatch, nested
  declarations, and UsersGuide/support `package.mo` files still stay on the
  conservative parser-summary fallback path
- that same bounded nested-package owner now also extends into repo-index
  incremental preparation. One changed safe API-surface nested `package.mo`
  file can now reuse the incremental overlay lane, and package-file semantic
  fingerprints ignore import line-number drift so comment-only churn keeps
  cached analysis; nested declarations, package-name/path mismatch,
  UsersGuide/support `package.mo`, and root package rename semantics still stay
  on the conservative fallback path
- the same Modelica package owner now also removes repeated scan/read churn
  within one repository-analysis pass. Full repository analysis and repo-owned
  `analyze_file(...)` now build one shared repository snapshot, reuse one
  sorted file inventory, and reuse preloaded `.mo` contents plus parsed
  `package.order` rows across module/import/doc/example collection, so large
  `package.mo` files stop being reread multiple times inside the same pass
  without widening parser-summary fallback or the existing bounded lexical and
  incremental admission rules
- `xiuxian-ast` is now pulled only when one of its real owners is enabled too:
  `search-runtime`, `studio`, and `zhenfa-router` each declare the dependency
  explicitly, and `cargo tree -p xiuxian-wendao --no-default-features -e normal`
  now shows only `tantivy` plus `xiuxian-wendao-runtime` as remaining direct
  edges in the minimal build
- `tantivy` is now pulled only behind the new `repo-lexical-index` owner too:
  the indexed fast path remains enabled for default and Studio builds, while
  the no-feature lane falls back to the existing lexical / heuristic ranking
  helpers, and `cargo tree -p xiuxian-wendao --no-default-features -e normal`
  now bottoms out at `xiuxian-wendao-runtime` as the only remaining direct edge
- the response-level telemetry surface now also groups by raw scope hint. `/api/search/index/status` `queryTelemetrySummary` keeps the global totals and now adds `scopes[]` buckets with per-scope source counts, latest capture time, scan/match/result totals, and bounded-rerank trim/drop totals so clients can see which lane or repo last drove pressure without re-summing corpus rows
- the formal `search_index_status` warm-cache gate now consumes those scope buckets too. The repo-scoped code-search warmup must leave a `gateway-sync` bucket in `queryTelemetrySummary.scopes`, and the performance support layer now formats scope pressure into compact diagnostics so a failing gate names the hot scope instead of only saying that the summary was malformed
- that formal status gate no longer warms telemetry through the full HTTP `intent` path. `GatewayPerfFixture::warm_repo_scope_query(...)` now exercises `SearchPlaneService` directly for repo-scoped telemetry seeding, which preserves the same `gateway-sync` scope bucket while eliminating the slow nextest wall-clock inflation from unnecessary handler-path setup
- `local_symbol` build identity is now file-sensitive. Instead of hashing only project/config topology, `fingerprint_projects(...)` now walks the same searchable file set used by AST extraction and folds normalized path + file size + mtime into the build fingerprint. This is still coarse-grained rebuilds, not true per-file incremental Lance mutations, but it closes the immediate correctness hole where editing a local source file could leave the old epoch permanently “already ready”
- that metadata-based build identity is now shared across the local-file corpora too. `search_plane::project_fingerprint` centralizes the searchable-file walk for `local_symbol`, `reference_occurrence`, `attachment`, and `knowledge_section`, so all four corpora now react to scanned file metadata changes with consistent path normalization, skip-directory handling, and note/source inclusion rules
- `local_symbol` is now the first local-file corpus with true file-level incremental extraction. The search plane persists per-file `SearchFileFingerprint` state, computes changed and deleted paths per build, reparses only changed files through `build_ast_hits_for_file(...)`, and carries unchanged rows forward from the previously published epoch by streaming old Lance batches into the new staging epoch
- that first incremental slice keeps the epoch/publish contract intact while reducing parse work sharply. Readers still see only published epochs, but staging assembly is now “stream unchanged rows forward + append changed-file rows” instead of “reparse the whole local-symbol corpus after any file edit”
- `reference_occurrence` now follows the same staged incremental model. It persists per-file fingerprints, rescans only changed source files for token occurrences, streams unchanged rows forward from the previous published epoch, and no longer depends on a project-wide `build_ast_index(...)` prepass plus full-table replacement on every edit
- `knowledge_section` now follows the same staged incremental model for notes. It persists per-note fingerprints, reparses only changed note files into doc/section rows, streams unchanged rows forward from the previous published epoch, and no longer rebuilds the entire note corpus through a fresh `WalkDir` + full-table replace on every note edit
- `attachment` now follows the same staged incremental model for notes. It persists per-note fingerprints, reparses only changed note files for attachment targets, streams unchanged rows forward from the previous published epoch, and no longer rebuilds the entire note corpus through a fresh `WalkDir` + full-table replace on every note edit
- the first local-file incremental architecture is now complete: `local_symbol`, `reference_occurrence`, `knowledge_section`, and `attachment` all keep epoch isolation for readers while reusing unchanged published rows during staging and reparsing only changed files
- `xiuxian-vector` now also exposes `clone_table(...)`, which gives the search plane a direct way to fork a published Lance epoch into a staging table before path-scoped mutation
- `local_symbol` is the first corpus to move from “stream unchanged rows forward” to direct staged mutation. Its staging writer now clones the previous published epoch, deletes changed/deleted `path` rows, and `merge_insert`s changed-file batches keyed by `id`
- unchanged `local_symbol` rows therefore stay inside Lance-level table cloning instead of being scanned back through Rust and appended into staging, making this the first search-plane corpus on the `clone + delete + merge_insert` path
- `reference_occurrence` now follows the same direct staged mutation path. Its staging writer clones the previous published epoch, deletes changed/deleted `path` rows, and `merge_insert`s changed-file batches keyed by `id`
- unchanged `reference_occurrence` rows therefore also stay inside Lance-level table cloning instead of being streamed back through Rust during epoch assembly, making this the second local-file corpus on the `clone + delete + merge_insert` path
- `knowledge_section` now follows the same direct staged mutation path. Its staging writer clones the previous published epoch, deletes changed/deleted `path` rows, and `merge_insert`s changed-note batches keyed by `id`
- unchanged `knowledge_section` rows therefore also stay inside Lance-level table cloning instead of being streamed back through Rust during epoch assembly, making this the third local-file corpus on the `clone + delete + merge_insert` path
- `attachment` now follows the same direct staged mutation path. Its staging writer clones the previous published epoch, deletes changed/deleted note `source_path` rows, and `merge_insert`s changed-note batches keyed by `id`
- unchanged `attachment` rows therefore also stay inside Lance-level table cloning instead of being streamed back through Rust during epoch assembly, making this the fourth local-file corpus on the `clone + delete + merge_insert` path
- the local-file migration is now complete: `local_symbol`, `reference_occurrence`, `knowledge_section`, and `attachment` all keep epoch isolation while using staged `clone + delete + merge_insert` instead of Rust-level unchanged-row copy-forward
- `repo_content_chunk` has now crossed into honest repo-backed direct mutation. Repo indexing publishes the full current document set with stable per-file metadata (`size_bytes` and `modified_unix_ms`), which is enough for search-plane to persist repo-scoped `SearchFileFingerprint`s, infer changed and deleted repo paths, and stop full-replacing repo-content tables on every refresh
- repo-backed `repo_content_chunk` staging now matches the local direct-mutation model: first publish writes a versioned table, revision-only publication updates reuse the already published table, and content changes clone the previously published table, delete changed/deleted `path` rows, and `merge_insert` changed-document batches keyed by `id`
- repo-content reads now resolve the published `table_name` from combined repo-corpus state before querying Lance. The old fixed `repo_content_chunk_table_name(repo_id)` path survives only as a legacy fallback when no publication record exists
- `repo_entity` now follows the same repo-backed direct-mutation path. Search-plane persists repo-scoped `SearchFileFingerprint`s for analyzer-emitted entity paths, using repo-document `size + mtime` when code-document metadata exists and a row-hash fallback when an analyzer path is not present in the collected repo-document slice
- repo-backed `repo_entity` staging now matches `repo_content_chunk`: first publish writes a versioned table, revision-only publication updates reuse the already published table, and entity changes clone the previously published table, delete changed/deleted `path` rows, and `merge_insert` changed entity rows keyed by `id`
- repo-entity reads now also resolve the published `table_name` from combined repo-corpus state before querying Lance. The old fixed `repo_entity_table_name(repo_id)` path survives only as a legacy fallback when no publication record exists
- the repo-backed request-serving corpus set is now fully on staged direct mutation. Both `repo_content_chunk` and `repo_entity` preserve publication isolation while avoiding full-table replacement during ordinary repo refreshes
- maintenance now goes one step deeper for the local direct-mutated corpora: `local_symbol`, `reference_occurrence`, `knowledge_section`, and `attachment` all run a bounded projected scan against the freshly written staging epoch before publish, which prewarms table metadata and the first hot data pages before the epoch flips to readable
- that prewarm is now observable too. Search-plane maintenance state records `last_prewarmed_at` and `last_prewarmed_epoch`, and Studio `/api/search/index/status` maps those fields through its maintenance payload without changing existing search response contracts
- prewarm is now also a first-class status-reason lane. An unreadable indexing corpus whose current `staging_epoch` has already been prewarmed now reports `prewarming`, while unreadable non-prewarmed first builds still report `warming_up` and readable refresh builds still report `refreshing`
- local-file corpus prewarm now exposes active lifecycle too. `prewarm_epoch_table(...)` marks coordinator maintenance as `prewarm_running` while the staging-table scan is in flight, clears it on failure, and clears-plus-records `last_prewarmed_epoch` on success, so local first-build indexing can emit `prewarming` before the completion marker lands instead of only after completion
- repo-backed prewarm now follows the same contract. `repo_entity` and `repo_content_chunk` both run a bounded projected scan against the freshly written versioned table before publication, persist repo-backed maintenance into combined `SearchRepoCorpusRecord` rows, and preserve that maintenance across the subsequent publication write
- repo-backed status synthesis now merges per-repo maintenance before deriving lifecycle reason, so unreadable repo-backed corpora can emit `prewarming` when their current synthetic `staging_epoch` has already been prewarmed
- repo-backed direct-mutation planning is now partially de-duplicated. A shared `search_plane::repo_staging` helper owns the common staged-mutation action model, revision-only refresh decision, changed/deleted path replacement plan, shared path-delete filter, and versioned table naming, while `repo_entity` and `repo_content_chunk` keep only their corpus-specific fingerprint and payload derivation
- repo-backed compaction now has a dedicated maintenance-task path. Repo publication computes the next combined-row `SearchMaintenanceStatus`, persists `publish_count_since_compaction` plus `last_compacted_row_count`, schedules per-repo compaction tasks keyed by `(corpus, repo_id, publication_id)`, and writes running/completed compaction metadata back into the combined repo-corpus record once Lance compaction finishes
- repo-backed `compaction_running` now survives status annotation because local task-set state is ORed onto synthesized maintenance instead of overwriting it
- repo-backed prewarm now also runs through the shared repo-maintenance bookkeeping instead of a bare inline helper call. `repo_entity` and `repo_content_chunk` still wait for prewarm completion before publication flips readable, but prewarm now claims and releases the same per-repo maintenance slot family used by repo compaction, which closes the maintenance-slot leak risk and unifies repo maintenance execution semantics
- the repo-backed maintenance model is therefore split cleanly: prewarm is now a repo-maintenance task with synchronous publication gating, while compaction is a repo-maintenance task that continues asynchronously after publication
- repo-backed prewarm also no longer silently degrades if an identical prewarm slot is already occupied. In that duplicate in-flight case, Wendao now falls back to a direct inline prewarm instead of returning early, which preserves the stronger publication guarantee that a repo-backed table is actually prewarmed before it becomes the readable publication
- repo-backed maintenance now also tracks active prewarm directly. `SearchMaintenanceStatus` and the Studio status DTO expose `prewarm_running`, repo prewarm toggles that flag around maintenance-task execution, and unreadable repo-backed indexing corpora can emit `prewarming` as soon as active prewarm begins instead of only after `last_prewarmed_epoch` has been recorded
- repo-backed prewarm and compaction now also share a single internal `RepoMaintenanceTask` envelope and dispatch path. External behavior stays the same for now, but the duplicated task-lifecycle branches are gone, which reduces the surface area for the eventual queue-backed repo maintenance worker
- repo-backed maintenance is now queue-backed too. Search-plane keeps a repo-maintenance runtime with `in_flight`, `waiters`, `queue`, and `active_task`; `prewarm_repo_table(...)` enqueues work and waits on oneshot completion, duplicate callers join the same queued/running prewarm instead of duplicating work, and compaction uses the same queue with fire-and-forget semantics
- that queue worker is drain-based rather than permanent: it starts on demand, drains queued repo maintenance tasks, notifies waiters, and exits once the queue is empty. This preserves the current publication contract that prewarm must finish before readable publication flips, without introducing a permanently running service loop
- the repo-compaction slice is now green under `cargo check -p xiuxian-wendao`, targeted `cargo nextest -p xiuxian-wendao ...`, and `git diff --check`. The transient `xiuxian-llm` DeepSeek vision build failures did not reproduce under isolated `cargo check -p xiuxian-llm --tests`
- staged delete hardening is now shared too. `search_plane::staged_mutation` batches path-scoped deletes for `local_symbol`, `reference_occurrence`, `knowledge_section`, `attachment`, `repo_content_chunk`, and `repo_entity`, which avoids oversized `DELETE ... IN (...)` predicates and removes duplicated per-corpus filter builders
- the staged-delete predicate ceiling is now deliberately conservative at `100` paths per batch, trading a few extra delete calls for lower DataFusion SQL parsing pressure during large branch switches or repo refresh bursts
- the proposed pure `merge_insert` rewrite was explicitly rejected for the current incremental model. Deleted-path correctness still depends on reusing the last published table as the mutation base, so the production-safe path remains `clone + batched delete + merge_insert`
- version cleanup is now closer to epoch reality too. `xiuxian-vector::VectorStore::compact(...)` no longer relies on a coarse seven-day cleanup window; it compacts first and then asks Lance to retain only the most recent two table versions, which bounds post-mutation version drift without pretending that old epoch-local history is still a live reader requirement
- gateway realism now has a dedicated benchmark-and-audit module. `scripts/benchmark_wendao_gateway_openapi.ts` remains the thin CLI entrypoint, while `scripts/wendao_gateway_openapi_benchmark.ts` validates the bundled OpenAPI control surface, benchmarks repo-scale `/api/repo/index/status`, `/api/repo/sync?mode=status`, and `/api/search/index/status` traffic against real repo ids from `.data/wendao-frontend/wendao.toml`, smoke-probes the remaining safe GET control routes with discovered repo/page/node/gap/VFS seeds, runs sustained high-concurrency hot-path stress suites, and writes timestamped TOML reports under `.data/wendao-frontend/.benchmark/`
- Studio shutdown now explicitly tears down the queue-backed repo maintenance runtime too. `SearchPlaneService::stop_repo_maintenance()` rejects new repo maintenance work, clears queued and in-flight waiter state, aborts any active repo-maintenance worker, and is called from `StudioState::stop_background_services()` next to `repo_index.stop()` and `symbol_index_coordinator.stop()`
- shutdown semantics are now locked by targeted regressions: one test proves shutdown clears waiters and releases the worker handle, and another proves repo prewarm can no longer fall back to inline execution after shutdown has started
- local corpus maintenance now has the same shutdown boundary. `SearchPlaneService::stop_local_maintenance()` marks local maintenance as shutting down, aborts in-flight local compaction handles, clears the running-compaction runtime, and `stop_background_maintenance()` now shuts down both local and repo maintenance from the Studio stop path
- local prewarm now also rejects work once shutdown starts. `prewarm_epoch_table(...)` checks the local-maintenance shutdown flag before and during its bounded scan, so local publish/prewarm cannot silently continue after the Studio stop boundary has been crossed
- local compaction is now queue-backed too. Search-plane keeps a local-maintenance runtime with `running_compactions`, `compaction_queue`, `worker_running`, `worker_handle`, and `active_compaction`; scheduled local compactions drain through a single on-demand worker instead of one detached handle per corpus, while local prewarm remains synchronous and still gates publish
- local maintenance status now exposes queue telemetry too. `compaction_running` is derived only from the current `active_compaction`, while `compaction_queue_depth` and `compaction_queue_position` are surfaced through `SearchMaintenanceStatus` and the Studio `/api/search/index/status` DTO so queued local compactions are no longer reported as if they were already compacting
- repo-backed maintenance status now mirrors that queue telemetry. Repo compaction is no longer marked `compaction_running` at enqueue time; the flag now reflects only the active repo-maintenance worker task, while `compaction_queue_depth` and `compaction_queue_position` expose queued repo backlog for `repo_entity` and `repo_content_chunk`
- repo-backed prewarm now mirrors that queue telemetry too. `prewarm_running` is no longer treated as an enqueue-time flag; it now reflects only the active repo-maintenance worker task, while `prewarm_queue_depth` and `prewarm_queue_position` expose queued repo prewarm backlog for `repo_entity` and `repo_content_chunk`
- repo-backed maintenance enqueue order now also reflects publication priority. `prewarm` is inserted before the first queued `compaction`, while compaction remains tail-appended, so publish-gating work cannot sit behind background cleanup and `queue_position` still matches actual execution order
- queued repo compaction now also coalesces per `(corpus, repo_id)`. When a newer publication schedules compaction for the same repo-backed corpus, the older queued compaction task is dropped before enqueue so stale publication cleanup does not keep occupying the repo-maintenance queue
- local queued compaction now coalesces per corpus as well. When a newer compaction plan is derived for the same local corpus while the older plan is still queued, the queued task is replaced instead of being left to compact a stale epoch
- local compaction queue now also has an explicit ordering rule. `PublishThreshold` tasks outrank `RowDeltaRatio`, and within the same reason smaller `row_count` is enqueued ahead of larger work, so background maintenance can clear smaller urgent compactions sooner without lying about `queue_position`
- both local and repo compaction queues now also apply a narrow enqueue-time aging guard. An old `RowDeltaRatio` task eventually stops being displaced by every new `PublishThreshold` task, but the guard still preserves truthful `queue_position` because it changes ordering only when new work is inserted
- the status surface now projects that fairness state too. `SearchMaintenanceStatus` and Studio `/api/search/index/status` expose `compaction_queue_aged`, which marks that a queued compaction has already crossed the enqueue-time aging guard rather than merely waiting in ordinary backlog
- `/api/search/index/status` now also emits a response-level `maintenanceSummary`, so clients can read one aggregate view of prewarm running, queued maintenance backlog, pending compactions, and aged queued compactions without rescanning every corpus row
- the formal `search_index_status` performance gate now reads that same `maintenanceSummary`, rejects internally inconsistent aggregate maintenance pressure when it appears, and includes formatted maintenance pressure in perf failure diagnostics alongside query-telemetry scope pressure
- gateway perf support now also wraps budget-failure output with the latest status diagnostics, so a `search_index_status` perf regression still reports `maintenanceSummary` and scope-pressure context even when the failure comes from `assert_perf_budget(...)` rather than the endpoint assertions
- that budget-failure wrapper now also covers the other formal gateway warm-cache gates. Repo/module/symbol/example/projected-page/code-search perf misses now append the request URI plus compact `/api/search/index/status` and `/api/repo/index/status` pressure snapshots instead of failing with only the raw perf report
- the same compact diagnostics now also persist on successful formal gateway samples. Saved perf JSON reports carry `gateway_uri`, compact search-index pressure, compact repo-index pressure, and any case-specific extras such as `statusGatePressure`, so offline perf triage no longer depends on reproducing the failure path
- that same success-path metadata now also covers the ignored real-workspace samples. Large-workspace `repo_index_status` and `code_search` reports persist the same compact pressure digests plus sample-specific extras such as `minRepos` and `workspaceQuery`
- the next architecture track is now formalized in the active local-corpus partitioned search-plane ExecPlan. That tracked plan narrows the future refactor to local corpora only, keeps repo-backed corpora on their existing per-repo publication model, and makes a unified multi-table scan primitive in `xiuxian-vector` the prerequisite before any local dataset partition flip
- the plan also records one remaining hard constraint from the current
  checkout: the inspected Lance `3.0.1` dependency surface did not expose a
  usable `replace_where(...)` primitive for a first-step migration
- the intended local partition boundary is therefore `project_id` or `package_id`, not `path_hash`, because the missing capability is logical fanout-and-rerank across multiple local datasets rather than one more predicate column inside the existing single-table epoch layout
- that prerequisite read abstraction is now partially landed. `xiuxian-vector::VectorStore` exposes multi-table batch scans for sync and async consumers, and the first implementation keeps one shared global limit budget while preserving table-order fanout
- `local_symbol` is now the first Wendao consumer of that abstraction. Runtime behavior is intentionally unchanged for now because it still supplies a singleton table list derived from the published active epoch, but the query and autocomplete paths are no longer hard-wired to one `table_name` parameter internally
- a targeted Wendao regression now covers true multi-table local-symbol fanout and rerank, which makes the remaining migration boundary explicit: the next step is local partition discovery and publication metadata, not one more scan API rewrite
- local project-scope scanning now materializes stable partition ids plus `project_name` / `root_label` metadata, and those partition ids are persisted into local `SearchFileFingerprint` rows so deleted-path incremental routing can survive a partitioned local publication model
- `SearchPlaneService` now enumerates local epoch partition tables with a legacy single-table fallback, and local-symbol query, prewarm, and compaction all consume that discovered table set instead of assuming one published table per epoch
- `local_symbol` is now the first local corpus on true partitioned publication. One table is written per configured project scope, the first partitioned rollout rebuilds from source when the previous active epoch is still a legacy singleton, and later refreshes resume per-partition `clone + delete + merge_insert`
- local-symbol hits now persist `project_name` and `root_label` directly from the configured scope that produced the file, so partitioned reads preserve Studio navigation metadata without a request-path re-enrichment pass
- repo compaction queue now also has an explicit ordering rule. After same-repo stale replacement, `PublishThreshold` outranks `RowDeltaRatio`, and within the same reason smaller `row_count` is inserted ahead of larger work so repo maintenance can clear smaller urgent compactions sooner without lying about `queue_position`
- checkout lock pressure tolerance is now configurable. Managed checkout lock acquisition resolves `XIUXIAN_WENDAO_CHECKOUT_LOCK_MAX_WAIT_SECS`, defaults to `20s`, and keeps the existing stale-lock reclamation path, which raises the concurrency ceiling for repo-intelligence requests without changing lock ownership semantics
- the bundled gateway OpenAPI contract now omits the retired `/api/analysis/markdown`, `/api/analysis/code-ast`, and `/api/search/ast` HTTP business paths entirely, so the checked-in document matches the pure Flight boundary instead of preserving compatibility shims
- `search_definition` now inherits the resolver defaults again instead of pinning `ExactOnly` and disabling Markdown hits in the handler, so the endpoint can fall back to fuzzy symbol matches and Markdown heading resolution when exact symbol lookup misses
- the gateway benchmark-and-audit posture is now aligned with the pure Flight boundary: retired HTTP business routes under `/api/analysis/*` and `/api/search/ast` are no longer part of the outward gateway contract and must be validated through Flight-native coverage instead of HTTP smoke requests
- the gateway benchmark transport is now hardened for local loopback pressure runs on this host. The CLI no longer depends on Undici `fetch`, and it explicitly binds `localAddress` for loopback targets when issuing `node:http` / `node:https` requests so repeated benchmark traffic against `127.0.0.1` does not fail with local `EADDRNOTAVAIL`
- with that transport fix in place, the refreshed `96`-concurrency / `60s` pressure report cleared the old markdown contract failure entirely. The remaining benchmark failures are now all honest `INDEX_NOT_READY` responses for unpublished local corpora rather than malformed smoke requests
- with the later definition-seed correction in place, the refreshed steady-state `96`-concurrency / `60s` report at `.data/wendao-frontend/.benchmark/wendao_gateway_openapi_2026_03_25T06_21_30_057Z.toml` now records `62 passed / 0 failed / 2 skipped`, which removes the last benchmark-only false negative from the bundled OpenAPI smoke surface
- explicit repo gateway search endpoints now also have repo-aware hot-query caching. `repo/module-search`, `repo/symbol-search`, and `repo/example-search` wrap their typed Search Plane / cached-analyzer result path in a repo publication/runtime keyed `SearchPlaneCacheTtl::HotQuery` cache, so repeated mixed-hotset `(repo, query, limit)` traffic can return without re-reading Lance or rebuilding analyzer artifacts
- that cache layer only stores successful typed payloads and keeps existing failure semantics unchanged, which means `REPO_INDEX_PENDING` and other gateway errors still bypass caching while steady-state hot queries reuse the cached payload directly
- local corpus request serving now closes that cold-start gap too. `StudioState` waits for the first successful publish of `local_symbol`, `knowledge_section`, `attachment`, and `reference_occurrence` before AST search, autocomplete, definition resolve, knowledge search, attachment search, and reference search continue, so a fresh gateway restart no longer surfaces transient `INDEX_NOT_READY` errors to the first caller when the build is still in progress
- targeted gateway tests now pin that contract explicitly. Six cold-start regressions call the handlers without pre-publishing their local corpora and assert success once the background build completes, which protects the exact OpenAPI-facing failure mode that the pressure benchmark exposed
- the pure-Flight search cut now also covers `definition` and `autocomplete`.
  The active business contracts are `/search/definition` and
  `/search/autocomplete`; the old `/api/search/definition` and
  `/api/search/autocomplete` HTTP business routes are removed from the router
  and bundled OpenAPI surface
- the Studio-owned definition/autocomplete builders are now wired through the
  same `WendaoFlightService::new_with_route_providers(...)` aggregate seam as
  the rest of the semantic search family, and the checked-in Flight snapshot
  now pins those two routes directly instead of relying on HTTP wrapper tests
- `.data/wendao-frontend` now routes definition resolution and autocomplete
  through same-origin Flight in `src/api/flightDocumentTransport.ts`, so the
  browser no longer uses standalone HTTP document-search business endpoints for
  those two flows
- the first bounded `graph/vfs` Flight cut is now landed too:
  `/api/vfs/resolve` is removed from the outward router and bundled OpenAPI
  surface, the canonical business contract is now `/vfs/resolve`, the Studio
  VFS provider is wired through `WendaoFlightService::new_with_route_providers(...)`,
  and `.data/wendao-frontend` now resolves navigation targets through
  same-origin Flight in `src/api/flightWorkspaceTransport.ts`
- the next bounded `graph/vfs` Flight cut is now landed too:
  `/api/graph/neighbors/{id}` is removed from the outward router and bundled
  OpenAPI surface, the canonical business contract is now `/graph/neighbors`,
  the Studio graph-neighbors provider lives in
  `src/gateway/studio/router/handlers/graph/flight.rs`, the shared workspace
  Flight snapshot now locks `/graph/neighbors` alongside `/vfs/resolve`, and
  `.data/wendao-frontend` now resolves graph-neighbor payloads through
  same-origin Flight in `src/api/flightGraphTransport.ts`
- the next bounded repo-search pressure slice is now formalized in the active repo-content query-pressure hardening ExecPlan. It stays intentionally narrow: repo-content query filtering, read-side backpressure, and FTS eligibility only
- `xiuxian-vector` now exports `string_contains_mask(...)`, a reusable Arrow-backed substring mask helper built on `arrow-string`, so search-plane consumers can prefilter UTF-8 batches with one boolean mask instead of open-coding row-by-row folded-string probes
- `repo_content_chunk` query filtering now uses that helper against `line_text_folded`, which preserves the exact-match boost logic but only evaluates `line_text.contains(raw_needle)` for rows that already matched the folded batch mask
- repo-content request-path reads are now also wired to a shared `SearchPlaneService` semaphore with env override `XIUXIAN_WENDAO_REPO_SEARCH_READ_CONCURRENCY`, closing the old gap where concurrent repo code-search scans had no search-plane-owned backpressure surface
- repo-content FTS eligibility is now widened for common code-search punctuation such as `@`, `.`, `/`, `:`, and `()`, which keeps more real code queries on the cheaper FTS-first lane before scan fallback is attempted
- that repo-content pressure slice is now fully validated end to end: the new `xiuxian-vector` helper tests pass, targeted Wendao query/config regressions pass, `cargo check -p xiuxian-vector` passes, and `cargo check -p xiuxian-wendao` passes after the temporary maintenance-module conflict in the shared worktree was reconciled
- the next live pressure report showed the remaining bottleneck had moved from local corpora to repo-backed FD exhaustion, so the repo-read gate has now been tightened again. `XIUXIAN_WENDAO_REPO_SEARCH_READ_CONCURRENCY` now defaults from `available_parallelism() / 8`, clamps to `1..4`, and falls back to `2`, while request fanout still reuses that exact stored budget
- `repo_entity` wide-row persistence has now landed its first stage. Repo-entity rows and batches persist structured metadata such as `module_id`, line range, verification fields, `attributes_json`, and `projection_page_ids`; repo-entity reranking hydrates `hit_json` only for final Top-K ids; and repo-entity incremental fingerprints no longer rely on a single giant string-concatenation token
- repo-backed pressure recovery is now also explicit on the repo-index side: retry classification accepts descriptor-pressure and resolver transport failures from both `AnalysisFailed` and `InvalidRepositoryPath`, which keeps one bounded retry available when managed git setup fails under OS pressure instead of treating every wrapped error as terminal
- managed checkout bootstrap now also treats descriptor pressure as transient in two places: lock acquisition retries through `EMFILE` inside the existing checkout-lock wait policy, and managed git `open_bare(...)` / `open(...)` calls retry a short bounded number of times when libgit surfaces `Too many open files`
- persisted gateway diagnostics now also have a workflow-facing summary layer. `scripts/render_wendao_gateway_perf_summary.py` renders the newest saved report for each of the six formal gateway cases into `gateway_perf_summary.json` and `gateway_perf_summary.md`, explicitly ignoring helper/sample artifacts that share the same report directory but are not part of the formal gate surface
- `just wendao-gateway-perf-summary` is now the stable local/CI entrypoint for that render step, and `.github/workflows/xiuxian-wendao-performance-gates.yaml` appends the generated markdown summary directly to `GITHUB_STEP_SUMMARY` after the formal gateway lane
- that same summary surface now also folds in the ignored real-workspace gateway samples when they exist. The JSON/Markdown payload keeps formal warm-cache cases and real-workspace samples in separate sections, keeps formal missing cases as the only hard gate, and leaves the real-workspace section advisory/optional so CI and manual large-workspace runs can share one artifact shape
- the summary renderer now also supports a mirrored output directory. The manual `rust-wendao-performance-gateway-real-workspace` lane uses that to drop the same unified summary into both the canonical `perf-gateway` root and the `perf-gateway-real-workspace` root, so large-workspace sampling leaves behind a self-contained summary surface without a second manual render step
- repo-backed state reads are now snapshotted per request instead of polled per repository. `code_search` and repo-backed intent merge batch `repo_search_publication_states(...)` once for the full repo set, then partition searchable / pending / skipped repositories from that in-memory snapshot
- repo-backed hit collection is now bounded fan-out instead of one serial `await` per repository. Both the code-search and repo-intent merge paths dispatch searchable repos through a `JoinSet` with parallelism capped by local `available_parallelism()`, while preserving the original repo ordering in `pending_repos` and `skipped_repos`
- the repo IO gate now actually covers both repo-backed corpora. `repo_entity` joins `repo_content_chunk` on the shared `repo_search_read_permits` semaphore, and both acquire the permit before `open_store(...)` so search-plane can throttle store-open plus scan pressure instead of only content-scan pressure
- targeted regressions now lock those two fixes explicitly: one service test proves batched publication-state hydration works after a runtime-memory miss, one handler test proves repo partitioning preserves repo order and availability semantics, and two service tests prove repo-entity and repo-content queries both stall when all repo read permits are exhausted
- repo-analysis query-search regression coverage now keeps snapshot baselines at
  two levels: `tests/snapshots/wendao/` for typed query-core and repo-entity
  result shapes, and `tests/snapshots/gateway/studio/` for the shared
  repo-entity fast-path payloads used by module, symbol, and example gateway
  search flows
- the active Phase-1 execution model is now milestone-based, and the unified
  repo-query surface milestone is landed. Repo-analysis fast paths no longer
  depend on a gateway-local Search Plane helper seam; module, symbol, and
  example routes now delegate their typed repo-entity fast path into
  `query_core::service`, which aligns them with the already-landed repo
  code-search query-core path
- Phase 2 has now started with one bounded orchestration slice: repo-analysis
  search handlers no longer each duplicate the same cache + fast-path +
  analyzer-fallback control flow. That shared orchestration now lives in
  `gateway/studio/router/handlers/repo/analysis/search/service.rs`, and the
  shared service now exposes typed `run_repo_module_search(...)`,
  `run_repo_symbol_search(...)`, and `run_repo_example_search(...)`
  entrypoints, keeping the route files focused on request decoding and result
  typing
- that Phase 2 slice has now landed its next milestone as well: the typed
  repo-analysis entrypoints are fully owned by the shared service, and
  `module.rs`, `symbol.rs`, and `example.rs` are now thin route adapters with
  no route-local fallback-builder closures
- the same Phase 2 service now also owns a lower-level typed contract for
  repo-analysis query construction and artifact-backed fallback binding, so the
  module, symbol, and example flows differ by typed contract metadata rather
  than three near-identical builder closures
- that fallback contract has now been pushed beneath the gateway layer too.
  `analyzers::service::search` owns the module/symbol/example fallback
  contracts, while the shared repo-analysis gateway service only composes those
  contracts with query-core fast-path dispatch and gateway error mapping
- query-core now owns the repo-analysis fast-path contract layer too.
  `query_core::service` exposes typed repo-entity fast-path contracts for
  module, symbol, and example result surfaces, and the shared repo-analysis
  gateway service composes those contracts instead of carrying three dedicated
  fast-path helpers of its own
- the same shared Phase-2 repo-analysis surface now also exposes
  `repo/import-search`. OpenAPI inventory and Studio routes include the import
  endpoint, analyzer search owns the import fallback contract, gateway request
  validation now pins `MISSING_REPO` and `MISSING_IMPORT_FILTER`, and accepted
  snapshot baselines now exist for both analyzer import results and the
  gateway-facing import payload
- that import lane has now also landed its cache-identity hardening slice.
  Canonical import query text now lives inside the analyzer-owned fallback
  contract and preserves both `package` and `module`, so the shared Phase-2
  service no longer needs a separate ad-hoc import cache-key helper and
  combined import filters no longer alias onto one cached query identity
- that same import lane is now publication-aware too. Repo-entity publication
  materializes import rows, repo-entity query can reconstruct typed
  `ImportSearchResult` payloads, and the shared Phase-2 repo-analysis service
  now prefers a query-core-backed import fast path before falling back to the
  analyzer-owned import contract
- the same consolidation style now also extends beyond search endpoints.
  `repo/overview` and `repo/doc-coverage` no longer keep their own inline
  `with_repo_analysis(...)` orchestration; both now sit behind one shared
  non-search repo-analysis service seam
- the projected retrieval cluster now follows that same pattern too. One shared
  projected-service seam owns the typed orchestration for projected retrieval
  hit/context, projected retrieval, projected page-index tree search, and
  cached projected page search, leaving `retrieval.rs` as a thin route adapter
- repo request fan-out now reuses that same repo-read budget instead of computing a second independent cap from host parallelism. `SearchPlaneService` persists `repo_search_read_concurrency_limit`, and request-path `repo_search_parallelism(...)` now reads that stored budget directly
- the repo request path therefore now has one shared concurrency contract: one budget controls both how wide `code_search` / repo-intent merge may fan out at once and how many repo-backed Lance store opens / scans may proceed concurrently
- repo-wide `code_search` now also has a bounded server-side completion budget. When an all-repo code query exceeds that request budget, Wendao aborts the remaining repo fanout tasks and returns `partial = true` with `indexing_state = partial` instead of letting the client sit until the outer `30s` timeout expires
- explicit repo-hint code search is intentionally excluded from that new budget, so targeted single-repo debugging keeps the full query path while broad all-repo search degrades gracefully under pressure
- targeted handler regressions now pin both sides of that contract: one unit test locks the default budget to all-repo search only, and one async test drains the shared repo-read permits and proves that a repo-wide search now returns a successful partial response when the budget expires instead of surfacing a transport timeout
- the status surface now exposes that repo-read contract directly. `SearchPlaneStatusSnapshot` and Studio `/api/search/index/status` carry top-level `repoReadPressure`, which reports the shared repo-read budget, current in-flight permit count, and the last observed repo dispatch (`requestedRepoCount`, `searchableRepoCount`, `parallelism`, `fanoutCapped`, `capturedAt`)
- request-path repo fan-out now records those dispatch observations at the moment `code_search` and repo-backed intent merge partition their repo sets, so pressure triage can distinguish “budget is small” from “this specific request was actually capped by the budget”
- formal gateway perf diagnostics now consume that same surface. The compact `/api/search/index/status` digest embedded into generic gateway perf reports includes `repoReadPressure`, so success-path metadata and budget-failure messages can show whether the shared repo-read gate was actually saturated during the run
- the dedicated `search_index_status` formal gate now also validates `repoReadPressure` consistency directly: `budget` must stay positive, `inFlight` cannot exceed it, `parallelism` cannot exceed the same budget, and `fanoutCapped` must agree with `searchableRepoCount > parallelism`
- the case-local `statusGatePressure` extra therefore now rolls three signals into one line: aggregate maintenance pressure, repo-read gate pressure, and query-telemetry scope pressure
- real-workspace gateway samples now persist the same repo-read gate digest as a first-class case-local extra. Both advisory large-workspace samples append `repoReadPressure` alongside their existing `workspaceQuery` / `minRepos` metadata, so offline comparison between formal and large-workspace reports no longer depends on re-parsing the larger compact status string
- the gateway perf summary renderer now surfaces that digest explicitly. Every summary case entry carries `repo_read_pressure`, preferring the explicit extra when present and otherwise extracting the same value from the compact search-index status string
- the rendered markdown summary now includes dedicated repo-read sections for both formal and real-workspace cases, so offline pressure triage can read repo gate saturation directly instead of digging through the larger diagnostics blob
- repo gateway cached reads are now aligned with that same bounded-read
  contract. The shared `with_repo_cached_analysis_bundle(...)` helper uses
  cached-only analyzer loading and no longer falls through to request-path
  `Ensure`
- that change removes managed-remote sync permit acquisition from warm-cache
  repo gateway search endpoints, which means `repo/module-search`,
  `repo/symbol-search`, `repo/example-search`, and the projected-page cached
  lane no longer inject remote-sync blocking into the mixed-hotset benchmark
- the gateway test suite now also pins the cold-cache outcome explicitly:
  without a prewarmed repo analysis cache, cached repo gateway search endpoints
  return fast `409 REPO_INDEX_PENDING` instead of hanging into on-demand
  analysis
- `repo_entity` wide-row persistence has now landed its typed-query stage.
  Search Plane reconstructs `ModuleSearchResult`, `SymbolSearchResult`, and
  `ExampleSearchResult` directly from persisted structured repo-entity columns
  and Top-K row hydration, so published repos no longer need analyzer-cache
  payload reconstruction for these three repo gateway endpoints
- repo gateway handlers now prefer published `repo_entity` tables for
  `module-search`, `symbol-search`, and `example-search`, while preserving
  cached-analyzer fallback when repo-entity publication is absent
- repo-entity FTS execution now defensively falls back to ordinary scan when a
  Lance FTS batch omits one of the structured projected columns, which closes
  the typed-query `missing string column entity_kind` failure without relaxing
  ordinary schema checks
- the stage-2 repo-entity slice is green under targeted `cargo nextest`,
  `cargo check -p xiuxian-wendao`, and `git diff --check`
- repo-wide `code_search` now also trims per-repo work before global reranking.
  All-repo fanout uses a dedicated local result cap of `min(limit, 12)` for
  each repo-local entity/content query, while explicit repo-hint searches still
  use the full requested limit
- the shared repo-read budget is now slightly less conservative by default.
  `XIUXIAN_WENDAO_REPO_SEARCH_READ_CONCURRENCY` still honors positive env
  overrides and the existing clamp, but the fallback heuristic now derives from
  `available_parallelism() / 5` instead of `/ 8`, which raises the current host
  class from `budget=2` to `budget=3`
- targeted regressions now pin both of those tail-latency controls, and the
  slice is green under targeted `cargo nextest`, `cargo check -p xiuxian-wendao`,
  and `git diff --check`
- repo-wide `code_search` now also distinguishes between the lighter
  `repo_entity` path and the heavier `repo_content_chunk` fallback. All-repo
  fanout keeps the entity cap at `min(limit, 12)` but lowers repo-content
  fallback to `min(limit, 4)`, while explicit `repo:` searches still use the
  full requested limit for both paths
- handler regressions now also pin that entity-first contract explicitly: when a
  repo has both published `repo_entity` and `repo_content_chunk` data for the
  same query, repo-wide code search returns the symbol hit and does not append a
  redundant file fallback hit from the same repo
- gateway runtime observability is now less fragile during outage triage. The
  supervised `wendao-gateway` process in `nix/modules/process.nix` still keeps
  its process-compose-owned `/tmp` logs, but stdout/stderr are now mirrored into
  `.run/logs/wendao-gateway.stdout.log` and
  `.run/logs/wendao-gateway.stderr.log`, which restores stable workspace-local
  evidence when the browser reports a transport-level `NetworkError`
- the first post-restart stderr sweep on that mirrored log was materially
  cleaner than before: no panic/error signatures were present, and the main
  remaining noise source was Lance `_score` autoprojection warnings from
  explicit FTS projections
- `xiuxian-vector` now disables Lance scoring autoprojection in the shared FTS
  streaming wrapper, so Wendao's explicit projected-column searches no longer
  spam stderr with `_score` deprecation warnings just because they do not read
  the score column
- repo-wide `code_search` now also recognizes exact repo-name seed queries
  before dispatch. When a query such as `SciMLBase` uniquely normalizes to a
  configured repo id like `SciMLBase.jl`, the handler upgrades it into an
  effective repo hint and skips all-repo fanout, which removes unrelated
  pending/skipped repos from the response and targets the remaining steady-state
  long-tail timeouts without changing explicit `repo:` search semantics
- the first Wendao RFC query-core callers are now live behind the unchanged
  Studio API surface. Repo-scoped `repo_content` and `repo_entity` code search
  both route through the internal `query_core` service facade before decoding
  back into legacy `SearchHit` payloads, so the search-plane-backed code search
  surface now shares one typed internal operator path instead of two separate
  handler-local search entrypoints
- graph query-core integration is now live too. `graph_neighbors` and
  `node_neighbors` both route through `query_core::query_graph_neighbors_projection(...)`.
  The new `query_core::graph` module now owns graph-neighbor relation decoding,
  unique-node projection, and internal link assembly, so Studio graph handlers
  keep the old response shapes without parsing Arrow columns directly
- the graph handler cluster is now also closed behind one shared service seam.
  `graph/neighbors.rs` and `graph/topology.rs` are thin route adapters over
  `gateway/studio/router/handlers/graph/service.rs`, which now owns
  query-core graph projection execution, center-node resolution, explain
  summary logging, stable `NodeNeighbors` assembly, and `topology_3d`
  construction
- the remaining graph shared-state surface is now split by responsibility too.
  `graph/shared/query.rs` owns graph-neighbor query parsing and normalization,
  `graph/shared/render.rs` owns graph-node/path/topology helpers, and
  `graph/shared/mod.rs` is now interface-only
- the graph test-support surface now follows the same pattern. `graph/tests/support/`
  separates fixture runtime, request/response helpers, assertions, and
  snapshot shaping, so graph regression support no longer depends on one flat
  `fixtures.rs`
- the top-level router export seam is now grouped too. `graph_exports.rs`
  owns the outward-facing graph handler exports, so `handlers/mod.rs` no
  longer has to flat-export graph routes directly
- the repo handler cluster now follows the same outward-facing export rule.
  `repo_exports.rs` owns the outward-facing repo handler exports, so
  `handlers/mod.rs` no longer has to flat-export the repo route surface
- analysis handlers now follow that same top-level export rule too.
  `analysis_exports.rs` owns the outward-facing analysis handler exports, so
  `handlers/mod.rs` no longer has to flat-export the analysis route surface
- the remaining top-level handler seams are now closed too.
  `capabilities_exports.rs`, `ui_config_exports.rs`, and `vfs_exports.rs`
  own the outward-facing exports for those clusters, so `handlers/mod.rs`
  no longer has to flat-export any of the remaining simple handler groups
- the flat analysis handler body is now closed behind a feature folder too.
  `analysis/types.rs`, `analysis/service.rs`, `analysis/markdown.rs`, and
  `analysis/code_ast.rs` now separate query types, shared loader logic, and
  route entrypoints, while `analysis/mod.rs` stays interface-only
- the flat capabilities handler body is now closed behind a feature folder too.
  `capabilities/types.rs`, `capabilities/service.rs`, and
  `capabilities/deployment.rs` now separate query types and the two outward
  route surfaces, while `capabilities/mod.rs` stays interface-only
- query-core explain is now partially operational in production code, not only
  tests. Repo code-search lanes record query-core-derived scan telemetry back
  into Search Plane status, and graph handlers emit a compact explain summary to
  internal debug logging so later Qianji or Zhenfa integration can consume one
  stable summary shape instead of bespoke handler-local diagnostics
- repo-analysis repo-entity fast paths are now one layer tighter too.
  `repo/module-search`, `repo/symbol-search`, and `repo/example-search` route
  through one shared typed repo-analysis service boundary. Repo-entity
  publication gating, typed error mapping, cache orchestration, and the shared
  fast-path/fallback envelope no longer live in three separate handlers
- query-search result auditing now has its first snapshot lane too. The
  Wendao test suite now writes stable JSON snapshots for query-core repo-code
  routing, query-core graph projections, and typed repo-entity query results,
  which gives the new internal query surfaces a higher-signal review diff than
  field-by-field assertions alone
- repo-wide `code_search` fanout is now one layer cleaner too. The buffered
  join-set task scheduler no longer open-codes entity-first/content-fallback
  behavior; that policy now lives in one repo-scoped `search_repo_code_hits(...)`
  entrypoint, which keeps the scheduling layer focused on parallelism, timeout,
  and result collection
- the whole-search-plane DataFusion refactor now has its first real corpus on
  the new engine. `SearchPlaneService` owns one shared `SearchEngineContext`,
  repo-entity publication writes export a Parquet artifact beside the Lance
  table and record `storage_format = parquet`, and repo-entity query plus
  typed hydration now execute as a two-stage DataFusion read: narrow projected
  scan first, then Top-K payload hydration by `id`
- the second corpus cutover now lands the text-heavy `repo_content_chunk`
  path on the same engine boundary: publication writes export Parquet beside
  the Lance table, repo-content publications record `storage_format = parquet`,
  and repo-content search now reads narrow projected columns through
  DataFusion instead of Wendao-side Lance FTS/scan orchestration
- the third corpus cutover now moves `knowledge_section` onto the same
  DataFusion execution model. Local epoch writes export a Parquet artifact
  beside the Lance table, knowledge search requires that Parquet artifact for
  active-epoch reads, and the query path now does narrow ranking-column scans
  first before hydrating `hit_json` by `id`
- the fourth corpus cutover now moves `attachment` onto the same local-epoch
  DataFusion model. Attachment writes export a Parquet artifact beside the
  Lance table, attachment search now executes as a narrow ranking-column scan
  plus `hit_json` hydration by `id`, and the internal bounded-rerank source
  enum is reduced to the scan path that the refactor still emits
- the fifth corpus cutover now moves `reference_occurrence` onto that same
  local-epoch DataFusion model. Reference-occurrence writes export a Parquet
  artifact beside the Lance table, stage-1 query scans now project only `id`
  plus the exact-match ranking columns, and the query path hydrates `hit_json`
  by `id` from the active epoch's Parquet-backed engine table instead of
  scanning the active Lance table directly
- the targeted regression slice for `reference_occurrence` is green under
  `cargo check -p xiuxian-wendao --lib`, focused `cargo nextest`, and
  `git diff --check`, and build coverage now also proves the epoch-level
  Parquet export exists after a successful incremental refresh
- the sixth corpus cutover now moves `local_symbol` onto the same partition-
  aware DataFusion model. Each local-symbol partition table now exports a
  sibling parquet artifact, active reads register each partition parquet into
  the shared search engine, and both local-symbol search and autocomplete now
  execute as narrow stage-1 scans across active partitions followed by payload
  hydration keyed by `(table_name, id)`
- `local_symbol` schema version is now `3`, so old Lance-only active epochs are
  treated as stale and rebuilt instead of being read by the new engine-backed
  path
- the targeted regression slice for `local_symbol` is green under
  `cargo check -p xiuxian-wendao --lib`, focused `cargo nextest`, and
  `git diff --check`, and build coverage now proves parquet export exists for
  every active local-symbol partition after incremental refresh
- direct runtime triage also confirmed a separate transport-layer fault under
  gateway pressure: `127.0.0.1:9517` could remain in `LISTEN` while lightweight
  health probes intermittently failed with immediate connect refusal, even
  though the gateway process stayed alive and mirrored stderr showed no panic
- the current gateway entrypoint is now hardened at the accept boundary. It
  binds through `tokio::net::TcpSocket` with an env-configurable listen backlog
  and wraps the merged studio router in load shedding plus a global concurrency
  cap, while leaving `/api/health`, `/api/stats`, and `/api/notify` outside the
  overload layer so supervisor probes retain a lightweight path during studio
  saturation
- targeted gateway execution regressions now pin both env parsers and the bind
  path for that hardening slice under focused `cargo nextest`
- direct runtime triage now also ties a large part of the post-restart startup
  pressure to legacy Lance secondary-index builds that survived the read-path
  DataFusion cutover. All six cutover corpora now declare that they no longer
  require those legacy inverted/scalar indices, and their publication writes
  stop after table mutation plus parquet export instead of also building unused
  Lance read-side indices
- the targeted regression slice for that cleanup is green under
  `cargo check -p xiuxian-wendao --lib`, focused `cargo nextest`, and
  `git diff --check`, which closes the write-path mismatch between the
  DataFusion read architecture and the previous Lance-index publication cost
- the repo-backed half of the cutover now also drops the remaining Lance table
  creation itself. `repo_entity` and `repo_content_chunk` publish directly to
  parquet, refresh publication metadata from parquet inspection, prewarm via
  parquet registration, and skip repo compaction once the publication format is
  parquet. This removes the last repo-backed dependence on Lance mutation and
  compact from the normal gateway startup path
- repo-backed regression coverage now proves the published parquet artifacts are
  enough for repo gateway search and code-search hydration, and that fresh repo
  publications no longer create sibling `.lance` tables
- direct post-restart health probes still reproduced intermittent same-process
  connect refusal even after the repo-backed parquet-only cutover, but the new
  stderr slice showed no fresh repo-backed Lance publication writes. The next
  hardening slice therefore tightened gateway-side backpressure itself instead
  of search-plane persistence again
- the gateway now caps studio concurrency much more aggressively by default
  (`4 x available_parallelism`, clamped to `32..128`) and applies a studio-only
  request timeout (`XIUXIAN_WENDAO_GATEWAY_STUDIO_REQUEST_TIMEOUT_SECS`,
  default `15s`, clamped to `5..60s`). Heavy studio routes now shed as
  `503 GATEWAY_OVERLOADED` or `504 GATEWAY_TIMEOUT` earlier, while health,
  stats, and notify endpoints remain outside that envelope
- the next startup-pressure hotfix moved one layer earlier than load shedding:
  bootstrap `wendao.toml` apply no longer eagerly starts repo/symbol
  background indexing by default, and the remaining bootstrap policy is now
  exposed through `/api/notify`, `/api/search/index/status`, and
  `/api/repo/index/status`, plus the UI-facing `/api/ui/capabilities`, as
  `studioBootstrapBackgroundIndexing*` telemetry
- that same bootstrap-indexing telemetry now also records the first deferred
  activation boundary as `studioBootstrapBackgroundIndexingDeferredActivation*`.
  `/api/notify`, `/api/search/index/status`, and `/api/repo/index/status` now
  all surface that state. The current real lazy-start source is
  `symbol_index_status`; repo-status surfaces only mirror that gateway-level
  state and do not imply repo sync is implicitly started on read
- the three status surfaces now assemble that bootstrap-indexing telemetry from
  one shared `StudioState` snapshot helper instead of hand-reading the same
  fields in three separate handlers, which keeps notify/search/repo status
  payloads aligned without changing any external JSON keys
- that shared helper now also emits a coarse summary flag,
  `studioBootstrapBackgroundIndexingDeferredActivationObserved`, and the
  UI-facing `/api/ui/capabilities` surface exposes the same boolean so clients
  no longer need to infer "has lazy activation happened yet" from nullable
  timestamp/source fields
- focused gateway execution regressions now cover both the tightened
  concurrency clamp and the new studio timeout parser, alongside the existing
  bind/listener checks
- handler regressions now pin both sides of that routing rule: unique normalized
  repo seeds resolve to one repo, while ambiguous normalized seeds continue to
  avoid inference and stay on the all-repo path
- the DataFusion cutover now has a validated `xiuxian-vector` foundation slice.
  `src/search_engine/` provides a project-scoped `SearchEngineContext`,
  Parquet registration/collection helpers, Arrow-57 <-> Arrow-58 IPC batch
  conversion, and Parquet export helpers for existing Lance-backed tables
- the first bridge integration proves the intended migration shape directly:
  a Lance/Arrow-57 batch can be converted into an Arrow-58 engine batch, then
  exported to Parquet and queried back through DataFusion without changing the
  Wendao-side typed hydration layer yet
- Arrow/DataFusion uplift exposed one concrete mixed-version fault line in the
  existing code: `ops/string_match.rs` was implicitly using workspace
  `arrow-string`. That helper now explicitly binds to Arrow-57 compatibility
  crates so the legacy Lance query path remains buildable while the new
  DataFusion engine advances separately
- the query-core caller boundary is now one step tighter for repo code search.
  Repo publication gating plus entity-first/content-fallback policy can now be
  expressed through one query-core service entrypoint, so the buffered repo-wide
  scheduler no longer needs to duplicate repo-lane selection semantics outside
  query-core-adjacent code
- the Phase-1 retrieval path is now materially exercised end-to-end for real
  repo-scoped callers, not only unit tests. Repo content and repo entity
  relation helpers now run through `vector_search -> column_mask -> payload_fetch`
  before gateway decoding, so the narrow-phase and payload-phase operators are
  now part of the live internal query path
- the Phase-2 projected retrieval seam is now expanded into a broader
  projected-service boundary. `retrieval.rs` and `pages.rs` both delegate
  typed endpoint work into one shared service module, so projected retrieval
  and projected page lookups no longer own repeated route-local
  `with_repo_analysis(...)` orchestration
- that same projected-service boundary now also covers `family.rs`, so the
  projected family-context, family-search, family-cluster, navigation, and
  navigation-search endpoints all reuse one shared typed service seam instead
  of binding route-local repository-analysis orchestration
- the repo handler closure pass now also covers command-style routes.
  `index.rs` and `refine.rs` both delegate through one shared repo-command
  service, so repo-index mutation/status and analysis refine-doc entrypoints no
  longer bind route-local command orchestration
- the repo shared-helper audit is now underway as the next structural cleanup
  slice. `repo/shared.rs` has been split into a `repo/shared/` feature folder
  with separate repository and execution modules, and the split is now covered
  by one focused shared-helper unit test plus representative repo handler
  regressions across analysis, projected retrieval, and sync execution seams
- the same route-thinning pattern now also covers the docs handler cluster.
  `gateway/studio/router/handlers/docs/service.rs` now owns typed entrypoints
  for docs search, retrieval, page, family, and navigation flows, so
  `search.rs`, `retrieval.rs`, `page.rs`, `family.rs`, and `navigation.rs`
  stop binding repeated `with_repo_analysis(...)` orchestration directly
- that same docs-service seam now also covers the remaining docs planner/gap
  endpoints. `projected_gap.rs` and `planner.rs` now delegate through
  `docs/service.rs`, so the docs handler cluster is fully closed over one
  shared typed service boundary
- the docs shared-state audit is now landed too. The old flat `docs/types.rs`
  surface has been split into `docs/types/` with separate planner and
  projected-gap modules, while `types/mod.rs` remains interface-only
- the docs handler closure audit is now landed too. The flat docs route files
  now resolve through `docs/projection/` and `docs/planner/`, while
  `docs/mod.rs` stays interface-only and the router-level re-export surface
  remains stable
- the docs service audit is now landed too. The old `docs/service.rs` surface
  now resolves through `docs/service/{projection,planner,runtime}.rs`, so the
  service layer matches the route and shared-state feature boundaries
- the docs router export audit is now landed too. Docs handler re-exports now
  resolve through `handlers/docs_exports.rs`, so the outward-facing docs symbol
  boundary is grouped instead of repeated inline in `handlers/mod.rs`
- deferred repo bootstrap is now corrected too. The eager config-apply path no
  longer drops repo or symbol indexing when an unchanged sanitized config is
  explicitly re-applied, and `/api/repo/index/status` now seeds deferred repo
  indexing on first access instead of remaining stuck at `total = 0` with a
  populated repo-project config
- Studio startup now treats config ownership as backend-loaded only. The
  runtime still reads legacy `wendao.studio.overlay.toml` composition during
  migration, but `/api/ui/config` no longer exists as a live persistence
  surface for rewriting `wendao.toml`
  removing it on the next successful write. Gateway bootstrap plus Studio
  bootstrap therefore keep old overlay-backed state bootable without
  continuing to generate overlay files.
- gateway boot now also enforces a fail-fast startup dependency gate before it
  spawns background workers or binds the listener. `wendao gateway start`
  probes the built-in plugin registry plus the TOML-owned search-cache and
  link-graph Valkey surfaces, logs stable `connected` / `failed` status lines,
  and aborts startup when any required dependency is missing, invalid, or
  unreachable
