# Repo Intelligence MVP

:PROPERTIES:
:ID: wendao-repo-intelligence-mvp
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, repo-intelligence, plugins, git
:STATUS: ACTIVE
:END:

## Goal

Land a Wendao-native Repo Intelligence MVP that lets agents answer repository questions from pre-indexed structure instead of repeating `grep`, `ls`, and ad-hoc exploration on every request.

## Scope

The MVP surface is limited to five query families:

- `repo.overview`
- `module.search`
- `symbol.search`
- `example.search`
- `doc.coverage`

The common core owns repository mirroring, incremental discovery, normalized record storage, graph persistence, and shared query contracts. Language-specific or ecosystem-specific semantics are delegated to Rust plugins selected in `wendao.toml`, for example `plugins = ["julia-code-parser"]` or `plugins = ["modelica"]`.

## Repository Findings

### DifferentialEquations.jl

- Root shape is compact: `Project.toml`, `README.md`, `src/`, `test/`, and assets.
- The entry module is thin and primarily reexports upstream packages:
  - `SciMLBase`
  - `OrdinaryDiffEq`
- Effective intelligence for this repository depends on understanding package metadata, `@reexport` surfaces, and ecosystem links to external docs/tutorial packages.

### Modelica Standard Library

- Root shape is library-first: `Modelica/`, `ModelicaReference/`, `ModelicaServices/`, `ModelicaTest/`, plus top-level package files.
- `Modelica/package.mo` exposes rich structured metadata through `annotation(Documentation(...))`.
- `Examples` and `UsersGuide` subtrees are widespread and regular, making them strong candidates for first-class `ExampleRecord` and `DocRecord` extraction.

## Common-Core Boundary

The Wendao common core should absorb everything that is expensive, repeated, or storage-sensitive:

- git mirror management and refresh policies
- repository registry from `wendao.toml`
- incremental file discovery and invalidation
- file classification and normalized record ingestion
- graph persistence and shared retrieval APIs
- plugin registry, scheduling, and diagnostics

Plugins should only provide semantic enrichment, not take over the runtime.

## Plugin API Boundary

The first plugin API should stay narrow:

1. Detect whether the plugin applies to a repository or file set.
2. Analyze files into normalized records.
3. Enrich cross-file or cross-module relations after base ingestion.
4. Optionally expand or rerank query results at query time.

Plugins should return normalized records and relations, not mutate Wendao storage internals directly.

## Immediate Next Steps

1. Extend the explicit `wendao repo sync --repo <id>` control surface beyond the current `ensure`/`refresh`/`status` modes with richer sync policies and remote lifecycle diagnostics instead of keeping all source preparation implicit behind analysis queries.
2. Replace the current conservative Julia-only doc linker with richer repository-graph linking for docstrings and structured docs.
3. Deepen the external `xiuxian-wendao-modelica` implementation from conservative package-layout indexing toward richer MSL-aware semantics.
4. Consolidate fuzzy retrieval into shared Wendao search primitives so lexical edit-distance scoring, fuzzy option contracts, and Tantivy-backed fuzzy querying stop drifting across isolated search call sites.

## Current Status

- The Repo Intelligence implementation namespace now lives under `xiuxian-wendao::analyzers`; this roadmap keeps "Repo Intelligence" as the product concept, but code references should now point at `src/analyzers/` rather than the retired `src/repo_intelligence/` path.
- Initial contracts now exist for:
  - repository registration metadata
  - normalized records
  - MVP query request/response types
  - plugin trait boundaries
  - plugin registry behavior
- All five Repo Intelligence query slices are now wired end to end:
  - `wendao.toml` now derives repo-intelligence registrations from `link_graph.projects.<id>` instead of maintaining a parallel `[[repo_intelligence.repos]]` registry
  - legacy `[[repo_intelligence.repos]]` entries are now ignored by the runtime loader instead of being merged with project-derived registrations
  - project-scoped repo sources use `root = "..."` for local checkouts and `url = "..."` with optional `ref = "..."` for managed git materialization, while `plugins = ["julia-code-parser" | "modelica"]` acts as the repo-intelligence opt-in on that same project entry
  - relative project roots resolve against the active `wendao.toml` directory
  - the common core now validates that configured local paths point at git checkout roots instead of arbitrary directories
  - repository records now derive `revision` and fallback `url` metadata from the local git checkout when configuration does not provide them
  - managed checkout refresh behavior is now explicit through `refresh = "fetch" | "manual"` instead of being hardcoded in the service layer
  - managed checkouts now clone from cache-local mirrors instead of cloning directly from upstream URLs every time
  - `repo.overview` now again merges plugin-provided repository metadata, post-analysis relation enrichment, checkout metadata hydration, and skeptic verification-state backfill before snapshotting or returning analysis results
  - `wendao repo sync --repo <id>` now exposes the common-core source lifecycle directly and returns the resolved source kind, requested sync mode, refresh policy, mirror/check-out lifecycle states, observation time (`checked_at`), last local mirror fetch time (`last_fetched_at`), mirror revision, tracking revision, drift state, high-level `health_state`, freshness-oriented `staleness_state`, a grouped `status_summary`, checkout path, optional mirror path, upstream URL, and active revision without requiring a full analysis pass
  - repo configuration now honors explicit `ref = "branch:<name>" | "tag:<name>" | "commit:<sha>"` prefixes instead of interpreting every non-empty `ref` string as a branch name
  - `wendao repo sync --repo <id> --mode status` now inspects the current managed-source cache state without creating mirrors, creating working checkouts, or triggering network refresh
  - managed source `status` mode is now again read-only for existing checkouts, while `ensure`/`refresh` correctly advance bare-mirror branch heads before materializing or fast-forwarding managed working copies
  - `repo sync` now also exposes a compact health summary so callers can distinguish `healthy`, `missing_assets`, `needs_refresh`, `has_local_commits`, `diverged`, and `unknown` without reinterpreting the lower-level lifecycle fields themselves
  - `repo sync` now also classifies mirror freshness into `fresh`, `aging`, and `stale` buckets, with `not_applicable` for local checkouts and `unknown` when managed metadata is missing
  - `repo sync` now also groups lifecycle, freshness, and revision state into a nested `status_summary` so agent-side consumers can read one structured object instead of reconstructing those relationships from flat fields
  - the Studio repo-index background lane now isolates managed remote sync in `spawn_blocking`, caps concurrent remote sync pressure through `XIUXIAN_WENDAO_REPO_INDEX_SYNC_CONCURRENCY` (default `2`), retries transient managed mirror transport failures with bounded backoff, and requeues one batch-level retry for retryable sync failures instead of immediately surfacing them as terminal `Failed` rows
  - bounded real-workspace sampling against the current `.data/wendao-frontend` SciML config now shows the repo-index lane progressing with `0 failed` during the first minute under `XIUXIAN_WENDAO_REPO_INDEX_SYNC_CONCURRENCY=1`, while `ready` rises steadily instead of collapsing into the earlier mass `failed to connect to github.com: Can't assign requested address` burst
  - a direct `XIUXIAN_WENDAO_REPO_INDEX_SYNC_CONCURRENCY=1` vs `2` first-minute A/B sample now shows no material throughput regression or failure burst at the default `2` ceiling: `sync=1` reached `46 ready / 3 unsupported / 0 failed` at `t+60s`, while `sync=2` reached `45 ready / 3 unsupported / 0 failed` over the same window
  - later gateway pressure work exposed one more gap in that model: the request-path repo analysis and repo sync endpoints were still bypassing the repo-index remote-sync semaphore, so managed-remote overview/module/symbol/example/page requests could add extra upstream fetch pressure on top of the background lane
  - the request path now shares the same remote-sync semaphore through `RepoIndexCoordinator`, so managed-remote repo overview/search/sync traffic and the background repo-index lane are capped by one common concurrency budget instead of two unrelated ones
  - the default remote-sync ceiling was temporarily reduced to `1` while transport pressure and stuck-sync recovery were still unsettled, but `XIUXIAN_WENDAO_REPO_INDEX_SYNC_CONCURRENCY` continued to override that budget for explicit operator tuning
  - repo-index implementation ownership now lives at `src/repo_index/`, and
    the transport layer no longer owns coordinator startup either:
    `GatewayState` and performance helpers now consume a crate-level
    repo-index bootstrap seam instead of calling
    `RepoIndexCoordinator::new(...).start()` directly under `src/gateway/studio/`
  - the next live `96`-concurrency gateway pressure run shifted the dominant failure mode again: local-corpus cold starts were largely gone, but repo sync and managed git access started failing with `Too many open files`, `failed to resolve address`, and upstream socket exhaustion instead
  - repo-index retry classification now treats descriptor pressure and resolver transport failures as retryable even when they arrive through `InvalidRepositoryPath` wrappers, which keeps one bounded retry path available for `Too many open files` and DNS-resolution spikes instead of immediately pinning the repo in `Failed`
  - managed checkout lock acquisition now treats `EMFILE` / `Too many open files` as transient pressure, waiting within the existing bounded lock window instead of failing immediately when the process briefly exhausts descriptors
  - managed mirror/opened-checkout bootstrap now also retries `Repository::open_bare(...)` and `Repository::open(...)` for descriptor-pressure failures with a short bounded backoff, which hardens the exact repo-intelligence path that the pressure benchmark surfaced
  - the remaining three `unsupported` rows (`StokesDiffEq.jl`, `SundialsBuilder`, `TensorFlowDiffEq.jl`) now consistently classify as expected Julia-layout misses with `missing Project.toml`, not transient sync/network failures
  - the `177` live repo-index total against `.data/wendao-frontend/wendao.toml` is now explained: the config contains `179` `link_graph.projects.*` entries, but `kernel` and `main` are link-graph-only local projects with `plugins = []`, so they are intentionally excluded from repo-index registration
  - the new real-workspace live audit surface now confirms the current short-window bottleneck is no longer an immediate transport-failure burst: with both `XIUXIAN_WENDAO_REPO_INDEX_SYNC_CONCURRENCY=1` and `=2`, the first `15s` sample window stayed at `failed=0` while all `177` audited repos classified as `managed_remote`
  - that same audit also exposed why throughput was collapsing: startup fanout briefly reached multi-repo activity, but `targetConcurrency` dropped back to `1` within the first `5s` because unsupported Julia-layout repos were still feeding the same adaptive failure path as true runtime pressure
  - the repo-index task lane now routes scheduler success feedback through a dedicated sync/admission `control_elapsed` metric and ignores structural repo failures such as `UnsupportedRepositoryLayout` when adjusting adaptive concurrency, so long indexing tasks and expected `missing Project.toml` rows no longer count as transport pressure
  - the corresponding real-workspace follow-up audit now holds `targetConcurrency=4`, `active=4`, and `indexing=4` by `t+10s` on `.data/wendao-frontend`, draining `queued` from `174` to `157` instead of collapsing back to one long-lived `Sundials.jl` worker
  - the performance audit helper now also supports a bounded full-workspace mode with per-sample live logs for `completed`, `deltaCompleted`, queue phase counts, current repo IDs, and unsupported/failed reason buckets, so large live runs no longer fail as opaque long-running tests
  - the first full-workspace proof target `gateway_perf_audits_real_workspace_repo_index_full_run_live` completed the current `.data/wendao-frontend` `177`-repo inventory in `101s` with the then-default `syncLimit=1`, reaching terminal state with `160 ready`, `17 unsupported`, `0 failed`, and `timedOut=false`
  - the next full-run A/B proof then showed the dedicated remote-sync budget had become the real ceiling: re-running the same `177` repos with `syncLimit=2` dropped terminal time to `61-65s` while preserving the same final result (`160 ready`, `17 unsupported`, `0 failed`)
  - the next scaling slice then exposed the remaining admission bug: higher budgets such as `8` were not losing to `gix` transport directly, but to a Wendao-local adaptive controller heuristic that compared higher-concurrency syncs against the fastest low-concurrency baseline and misclassified ordinary large-repo variance as `io_pressure`
  - repo-index performance telemetry now records controller-side `reference_limit`, `io_pressure_streak`, last adjustment reason, and control-timing snapshots in the live audit output, so large ignored live tests now explain whether throughput loss comes from true transport pressure or Wendao-local admission logic
  - the adaptive controller now resets its baseline when it moves to a new concurrency tier and requires sustained I/O-pressure observations before halving concurrency, which removes the earlier false `contracted_io_pressure` collapse under `syncLimit=8`
  - the new scaling matrix on the real `.data/wendao-frontend` `177`-repo inventory is now:
    `syncLimit=1 -> 101s`, `syncLimit=2 -> 61-65s`,
    `syncLimit=8 -> 55-56s`, `syncLimit=16 -> 66s`
  - the repo-index default sync ceiling is therefore now machine-aware rather than a frozen literal: Wendao derives `XIUXIAN_WENDAO_REPO_INDEX_SYNC_CONCURRENCY` from `available_parallelism()` with a bounded policy, which still yields the empirically best `8` on the current `12`-core host while preserving operator override control through the same env var
  - the repo-index sync worker timeout is now machine-aware under the same policy surface instead of staying pinned to one static `120s` default, so higher-parallelism hosts start from a wider remote-sync budget without giving up the explicit `XIUXIAN_WENDAO_REPO_INDEX_SYNC_TIMEOUT_SECS` override
  - the remaining repo-index policy knobs are now machine-aware too: analysis timeout no longer stays pinned to a fixed `45s`, and retryable sync failures no longer stop after one frozen requeue by default; the current `12`-core host now resolves to `analysisTimeout=60s` and `syncRetryBudget=2`, while `XIUXIAN_WENDAO_REPO_INDEX_ANALYSIS_TIMEOUT_SECS` and `XIUXIAN_WENDAO_REPO_INDEX_SYNC_REQUEUE_ATTEMPTS` remain explicit operator overrides
  - the full real-workspace proof stays healthy after closing those last static defaults: with the current default machine-aware policy surface, the `177` managed repos in `.data/wendao-frontend` now again reach terminal state in `66s` with `160 ready`, `17 unsupported`, `0 failed`, and the live audit logs now print `analysisTimeout=60s`, `syncTimeout=120s`, and `syncRetryBudget=2` at run start and completion
  - repo-index phase reporting now marks repositories as `Syncing` only after a sync permit is acquired, so `/api/repo/index/status` no longer overstates concurrent remote sync pressure while tasks are still waiting in the coordinator
  - `xiuxian-git-repo` managed remote clone/fetch/probe paths now run through a dedicated `gix` interrupt watchdog, and the default watchdog budget also scales with host parallelism instead of staying pinned to one static `45s` literal; transport stalls still surface as deterministic `timed out` sync failures, and `XIUXIAN_GIT_REPO_REMOTE_OPERATION_TIMEOUT_SECS` remains the highest-precedence operator override
  - the remaining fixed substrate retry/open defaults are now machine-aware too: `xiuxian-git-repo` derives managed remote retry attempts, git-open retry attempts, git-open retry delay, and checkout-lock retry delay from host parallelism instead of frozen literals, while preserving explicit operator control through `XIUXIAN_GIT_REPO_MANAGED_REMOTE_RETRY_ATTEMPTS`, `XIUXIAN_GIT_REPO_MANAGED_GIT_OPEN_RETRY_ATTEMPTS`, `XIUXIAN_GIT_REPO_MANAGED_GIT_OPEN_RETRY_DELAY_MS`, and `XIUXIAN_GIT_REPO_CHECKOUT_LOCK_RETRY_DELAY_MS`
  - the current `12`-core host therefore keeps the already-proven substrate defaults (`remoteRetries=3`, `gitOpenRetries=5`, `gitOpenRetryDelay=100ms`, `checkoutLockRetryDelay=100ms`) without baking those numbers back into code, while smaller and larger hosts now scale to lighter or heavier retry/open defaults through the same policy seam
  - the Studio repo-index runtime now also applies `XIUXIAN_WENDAO_REPO_INDEX_SYNC_TIMEOUT_SECS` (default `120`) as a final worker-level guard and releases active repo ownership when a sync worker times out or panics, so one wedged managed remote no longer leaves the queue frozen at one long-lived `Syncing` repo with the remaining inventory stuck in `Queued`
  - the bounded-recovery lane now also has fast unit coverage for the normal success path on both helpers, proving the watchdog and sync-completion guards return well before their timeout budgets instead of quietly adding steady-state latency
  - repo-index restart warm-start is now backed by persisted per-repo repo-corpus records instead of in-process memory only: bootstrap recovery now checks in-memory rows first, then batch-loads Valkey per-repo records for the requested repo IDs, then local JSON per-repo records, and only falls back to re-enqueue if those record paths cannot restore a readable status
  - managed-remote restart hydration is now publication-first when the persisted repo-backed rows are still readable: if both corpora already expose readable Parquet publications, repo-index restores `Ready` without forcing an immediate requeue just because the current mirror or checkout assets are missing or stale
  - repo-search/query cache reuse is already Valkey-backed through `SearchPlaneCache`, and that runtime now resolves `search.cache.*` from merged `wendao.toml` before it falls back to `XIUXIAN_WENDAO_SEARCH_PLANE_VALKEY_URL`, `XIUXIAN_WENDAO_KNOWLEDGE_VALKEY_URL`, `VALKEY_URL`, or `REDIS_URL`; the real `.data/wendao-frontend/wendao.toml` now declares `search.cache.valkey_url`, so live gateway proofs no longer depend on ad-hoc shell env wiring just to enable cross-restart cache hits
  - repo-index ownership no longer sits under `src/gateway/studio/`: the implementation tree now lives under `src/repo_index/`, while Studio router/state code consumes the crate-owned module instead of declaring itself the owner of queue coordination, sync orchestration, and repo status synthesis
  - `/api/repo/index/status` now also exposes `syncConcurrencyLimit`, so Studio and live debugging can distinguish the dedicated remote-sync semaphore from the broader adaptive `targetConcurrency` used for indexing work
  - the bundled gateway route inventory and `OpenAPI` artifact now explicitly document both `POST /api/repo/index` and `GET /api/repo/index/status`, so repo-index status payload changes no longer sit outside the checked-in contract surface
  - the coordinator lifecycle now composes `tokio_util::sync::CancellationToken` and `tokio_util::task::TaskTracker` instead of hand-rolled `AtomicBool + JoinHandle` shutdown ownership, keeping queue semantics stable while moving cancellation/task tracking onto mature runtime primitives
  - the repo-index and performance-support targets stay green after that lifecycle cut, and the latest two full live proofs on the real `.data/wendao-frontend` `177`-repo inventory still terminate cleanly in `63s` and `67s` with `160 ready`, `17 unsupported`, and `0 failed`
  - that newer evidence sharpened the next performance target, and the first bounded follow-up is now landed: search-plane cache persists revision-scoped repo publication rows keyed by `(corpus, repo, source_revision)`, and repo-index currentness now consults that revision-scoped Valkey path before deciding a managed remote must be reindexed
  - the new revision-scoped reuse proof now covers the case where the latest repo/corpus row has already advanced from `rev-1` to `rev-2`: a later managed-remote sync at `rev-1` can still reuse the older readable publication instead of being forced into a full fresh index just because the latest pointer moved forward
  - that revision-keyed cache seam is now bounded instead of append-only: search-plane keeps a retained revision index per `(corpus, repo)`, trims older revision-scoped publication rows with `XIUXIAN_WENDAO_SEARCH_PLANE_REPO_REVISION_RETENTION` (default `32`), and clears retained revision entries through the same repo-publication cache maintenance path
  - the new retention regressions now prove both the cache and repo-index sides of that maintenance path: revision-scoped entries disappear after retention overflow, and repo-index currentness no longer reuses a revision once it has been evicted from the retained cache window
  - repo-index currentness now also prefers the latest readable repo-corpus record before falling back to retained revision-scoped publication rows, so cold revision-cache state can still short-circuit directly on the persisted Parquet publication that already matches the synced git revision
  - the newest focused regressions cover the full fast-path seam: one repo-index runtime test proves managed remotes reuse the latest persisted publication even after the revision cache is cleared, one publication/cache test proves readable publication lookup can recover from the latest record alone, and one cache-level test proves deleting revision-scoped entries does not wipe the latest repo-corpus record
  - analyzer cache identity is now also narrower than raw git revision identity: `RepositoryAnalysisCacheKey` keeps checkout/mirror/tracking revisions as observational metadata, but cache equality and Valkey lookup now hinge on a conservative `analysis_identity` fingerprint for analysis-affecting inputs
  - that fingerprint is mode-aware instead of hashing the whole checkout blindly: Julia `Project.toml` and `src/**/*.jl` participate by contents, Julia `README*`, `docs/**/*.md`, `examples/**/*.jl`, and `test/**/*.jl` participate by path only; Modelica `.mo` and `package.order` participate by contents, while `README*` and UsersGuide text docs participate by path only
  - the new cache/service regressions now prove the intended incremental shape: non-affecting revision churn can reuse cached analysis across commits, while Julia source changes still invalidate the analyzer cache and trigger a fresh analysis pass
  - repo-index now also consumes real git revision diffs through `xiuxian-git-repo` instead of treating every new revision as full-reindex work: non-code churn can advance repo-backed publication revisions without reanalysis, non-analysis-affecting code/doc churn can reuse the previous cached analysis snapshot, and only conflict-like diffs still force a full reindex
  - the first bounded partial-analysis path is intentionally conservative and Julia-only: safe leaf edits under `src/**/*.jl` now merge into the previous analysis base, while deleted analysis-affecting paths, missing cached bases, mixed-plugin repos, and import/include/module-shape edits still fall back to full reindex
  - focused runtime/publication proofs now pin those three outcomes directly: `RefreshOnly` for non-code churn, cached-analysis reuse for example churn, and partial Julia merge for safe leaf-source changes
  - the next bounded repo-content data-plane follow-up is landed too: when repo-index already has a readable repo-content publication plus previous file fingerprints, code-document collection now rereads only changed supported checkout files instead of rescanning the whole checkout on every diff-driven reindex
  - repo-content publication now accepts that merged fingerprint snapshot and changed-file payload directly, so unchanged supported files stay on the existing parquet clone/mutate path while deleted paths are removed without requiring a full `WalkDir + read_to_string` rebuild
  - the latest focused probe on a synthetic `2048`-file Julia-shaped checkout measured roughly `102ms` for the old full collector versus `0.07ms` for the incremental changed-file collector on `3` changed and `1` deleted file, or about `1456x` faster on that bounded hot path
  - the repo performance surface now includes a dedicated `incremental_repo_code_documents` Criterion benchmark plus a `performance`-gated unit probe so future slices can keep watching this collector seam without waiting for live multi-repo gateway audits
  - the next `10000+` state-scaling slice is now landed too: repo publication/runtime hot paths no longer rewrite one global `repo_corpus_snapshot`, batch repo-search/publication reads now hydrate from per-repo records directly, and full-inventory surfaces rebuild from the local per-repo record directory instead of a snapshot file
  - the new focused proofs pin the new boundary directly: repo-index bootstrap restores `Ready` from per-repo records without a snapshot file, failed runtime hydration also recovers from per-repo records alone, batched publication-state reads stay green without snapshot state, and runtime removal now deletes local per-repo record files so full inventory scans do not retain stale repos
  - the first synthetic `10000`-repo bootstrap probe on this new path restored `10000` repo statuses in about `862.881ms` with no snapshot file present, which turns the next scaling bottleneck away from metadata snapshot churn and toward remaining Parquet rewrite cost on large repo mutations
  - that next data-plane audit slice is now measured directly too: the search-plane now has a dedicated synthetic repo-content Parquet clone-and-mutate benchmark fixture plus Criterion hook, and the focused correctness proof confirms the fixture preserves row counts as well as added/deleted query readability across a copied base publication state
  - the first bounded clone-and-mutate probe now reads `1000 docs -> 101.736ms` and `10000 docs -> 519.417ms` while mutating only `3` changed files and `1` deleted path, a roughly `5.11x` increase from a `10x` base-document increase; the write path is therefore measurably better than naive linear blowup on this local seam, but it still scales materially with base publication size instead of with changed-file count alone
  - the first bounded `10000+` storage-boundary follow-up is now landed for `repo_content_chunk`: repo-content file fingerprints carry deterministic partition IDs, repo-content publications write one logical parquet root containing partition files, and incremental mutation rewrites only touched partitions once the base publication is already partitioned
  - the repo-content read boundary stayed stable through that cut: query and inspection paths still address one logical publication name, `duckdb` directory reads now glob `*.parquet`, and legacy single-file repo-content publications migrate forward on the next incremental mutation instead of forcing a manual rebuild
  - perf validation also exposed a new bounded correctness seam at the same owner boundary: `DataFusion` reloads string columns as `Utf8View`, while generated changed payloads stay `Utf8`, so repo-content writes now normalize every batch to one canonical Arrow schema before writing to keep directory-level reads stable across mixed partitions
  - the rerun synthetic repo-content mutation probe now reads `1000 docs -> 63.328ms` and `10000 docs -> 431.292ms` while mutating `3` changed files and `1` deleted path, improving the earlier full-rewrite baseline (`101.736ms` / `519.417ms`) in absolute latency even though the new `6.81x` growth still shows meaningful dependence on base publication size
  - the next bounded repo-content follow-up is now landed too: partitioned repo-content publications persist `_stats.json` sidecars with per-partition row and fragment counts, and repo-content inspection now prefers that sidecar before falling back to a full Parquet directory scan
  - the rerun synthetic repo-content mutation probe now reads `1000 docs -> 47.648ms` and `10000 docs -> 291.493ms` while mutating `3` changed files and `1` deleted path, improving the first partition prototype (`63.328ms` / `431.292ms`) in absolute time and trimming the ratio from `6.81x` to `6.12x`
  - the next bounded repo-content follow-up also tested untouched-partition materialization directly: unchanged partition files now prefer hard links into the next logical publication root with byte-copy fallback, and a Unix-focused test proves the shared-inode fast path after incremental mutation
  - that rerun synthetic mutation probe reads `1000 docs -> 48.662ms` and `10000 docs -> 292.863ms`; the ratio improved slightly from `6.12x` to `6.02x`, but absolute time stayed within noise, which shows untouched-partition copying was not the dominant remaining local cost on this same-filesystem path
  - the next bounded repo-content follow-up then removed request-scoped DataFusion registration from touched-partition reload by reading single partition files directly through a Parquet reader, keeping the same logical publication contract and focused correctness proofs green
  - that rerun synthetic mutation probe reads `1000 docs -> 41.795ms` and `10000 docs -> 305.287ms`; the smaller case improved relative to the untouched-link slice, but the larger case regressed and the ratio worsened to `7.30x`, which shows reader setup was not the dominant remaining local cost on this seam
  - the next bounded repo-content follow-up then profiled the touched partition distribution directly: the synthetic mutation benchmark now reports touched partition count and touched base-document mass beside elapsed time
  - that current synthetic profile touches `4` partitions at both scales, but those partitions already hold `255` base docs at `1000` total docs and `2595` base docs at `10000` total docs; with the current fixed `16`-bucket partition fanout, one mutation is therefore already rewriting roughly one quarter of the base repo-content document mass
  - the latest rerun with that profile reads `1000 docs -> 41.252ms` and `10000 docs -> 263.742ms`, which is consistent with the touched base-document mass growing by about `10.18x` while touched partition count stays flat
  - the next bounded storage follow-up then raised repo-content fixed partition fanout from `16` to `64` buckets and reran the same synthetic mutation seam
  - that rerun still touches `4` partitions, but touched base-document mass drops to `62` at `1000` total docs and `685` at `10000` total docs instead of `255` and `2595`
  - the latest rerun with finer fanout reads `1000 docs -> 65.849ms` and `10000 docs -> 210.378ms`; this is a real `10000+` win relative to the `16`-bucket profile (`263.742ms`), but it regresses the smaller `1000`-doc case relative to `41.252ms`
  - the next `10000+` storage-boundary follow-up then landed that narrower cut: repo-content partition fanout is now repo-size-aware instead of one global fixed bucket count, keeping `16` buckets below `4_096` docs and `64` buckets at or above that threshold
  - that rerun synthetic mutation seam reads `1000 docs -> touched_base_docs=255, elapsed_ms=40.767` and `10000 docs -> touched_base_docs=685, elapsed_ms=148.875`, with the same `4` touched partitions at both scales and a ratio of `3.65x`
  - the adaptive cutoff therefore preserves the small-repo cost shape from the old `16`-bucket layout while improving the larger `10000`-doc case beyond both prior fixed layouts (`210.378ms` with fixed `64`, `263.742ms` with fixed `16`)
  - the next bounded experiment then tested one more extra-large `128`-bucket tier for repos at or above `8_192` docs; that cut halved the `10000`-doc touched base-document mass (`685 -> 340`) but did not show a stable latency gain, with the experiment at `146.332ms` and the reverted stable `16 / 64` rerun at `140.911ms`
  - the next bounded follow-up then instrumented the remaining incremental mutation path directly: the synthetic benchmark now reports phase timings and untouched partition counts beside the touched partition mass
  - that current profile reads `1000 docs -> 40.588ms` with `write_touched 15.445ms`, `prewarm 8.240ms`, `load_touched 5.502ms`, `copy_untouched 4.377ms`, and `plan 3.044ms`
  - the same profile reads `10000 docs -> 139.443ms` with `write_touched 42.674ms`, `plan 33.473ms`, `copy_untouched 19.296ms`, `prewarm 19.495ms`, and `load_touched 11.948ms`
  - the next `10000+` repo-content follow-up therefore should not keep increasing bucket count blindly; it should first attack the proven non-partition-mass mutation overhead before widening the same pattern to `repo_entity`
  - the next bounded `100000` query-path slice is now landed too: `search::perf_support` and the Criterion bench surface now include a synthetic repo-backed query fixture that measures three read-path seams separately on the same publication: the first cold query on a fresh `SearchPlaneService`, the next hot query on the warmed service, and the local Flight-facing `search_repo_content_batch(...)` materialization path
  - that slice also clarified the architecture boundary from code instead of naming: steady-state repo-backed search is not a literal `Valkey -> Arrow Flight -> DuckDB` chain. Valkey only participates in persisted repo-corpus metadata recovery, while the hot query path resolves the readable Parquet publication, executes local SQL through `ParquetQueryEngine` and `DuckDB` `query_arrow([])` in `duckdb` builds, then lets Rust consume and rerank the returned Arrow batches; the Flight-facing `LanceRecordBatch` surface sits above that query path
  - the first synthetic `100000`-document proof on that clarified boundary measured `cold=1014.785ms`, `hot=975.154ms`, and `flight_batch=980.371ms` on a `1_200_000`-row repo-content publication with `metadata_backend=local_json_only`
  - the next bounded read-path follow-up then pushed the folded repo-content query term into stage-1 SQL and narrowed the Arrow projection once SQL already guaranteed the substring match; the rerun on the same synthetic publication dropped to `cold=143.317ms`, `hot=121.676ms`, and `flight_batch=123.636ms`, while the existing repo-query telemetry reported `rows_scanned=1` for all three samples
  - one more bounded warmed-path follow-up then reused DuckDB parquet view registrations for the same `table_name -> path` pair inside the service-scoped `ParquetQueryEngine`; the next rerun held the same `rows_scanned=1` outcome and improved the warmed paths again to `cold=145.870ms`, `hot=74.682ms`, and `flight_batch=78.131ms`
  - the next bounded DuckDB object-cache experiment was rejected instead of being carried as speculative debt: enabling `enable_object_cache=true` regressed the same point-lookup seam to `cold=244.589ms`, `hot=76.039ms`, and `flight_batch=84.451ms`, so the branch kept the prior SQL-pushdown plus parquet-view-reuse baseline
  - the next kept read-path slice then targeted broad repo-content queries directly by pushing one-best-row selection per repo path into stage-1 SQL. Repo-content lookup now computes `exact_match`, ranks rows with `ROW_NUMBER() OVER (PARTITION BY path ...)`, and emits one best row per file before Arrow batches cross back into Rust
  - the focused 256-document broad-query proof now reports `rows_scanned=256` and `matched_rows=256` for query token `value`, proving that the high-hit seam collapsed from line-level to file-level cardinality while preserving the limit-5 result contract
  - the standalone `100000` broad-query proof now reports `hot=1238.003ms`, `flight_batch=1244.868ms`, `hot_rows_scanned=100000`, and `flight_rows_scanned=100000`; the remaining broad-query floor is therefore still dominated by local DuckDB execution plus Arrow materialization, but now on one row per path instead of one row per matching line
  - the next kept read-path follow-up then split repo-content lookup into a light stage-1 scan plus bounded detail hydration for the final retained candidates, so `line_text` no longer crosses the hot stage-1 Arrow boundary for every candidate row
  - that same slice also made the folded text predicate length-aware instead of global: short tokens stay on `LIKE '%...%'` while longer tokens switch to `strpos(...) > 0`, because the synthetic `100000`-document seam showed the `strpos` form is materially faster for long sparse tokens but regresses the short broad-query path
  - the rerun `100000` point-query proof after that change now reads `cold=122.817ms`, `hot=40.950ms`, and `flight_batch=41.701ms` on the same `1_200_000`-row publication, with `rows_scanned=1` still preserved for all three samples
  - the rerun `100000` broad-query proof stays in the same local scan-bound band at roughly `hot=1.25s` with `hot_rows_scanned=100000`, which confirms the length-aware predicate is a kept point-lookup win rather than a general broad-query breakthrough
  - the next bounded broad-query follow-up then reintroduced SQL-side global top-K only for short sparse-insensitive query tokens and only when no tag filters force post-SQL filtering; longer point-lookups keep the prior `strpos(...)` path without that extra stage-1 sort/limit
  - that same slice also hardened the synthetic benchmark fixture so each process uses a unique temporary publication root, preventing concurrent `duckdb` benchmark runs from colliding on the same Parquet directory when point and broad seams execute side by side
  - the rerun `100000` point-query proof after that kept follow-up reads `cold=126.252ms`, `hot=38.609ms`, and `flight_batch=42.213ms`, so the short-token broad-query cut preserves the existing long-token point-query win within normal noise while keeping `rows_scanned=1`
  - the rerun `100000` broad-query proof now reads `hot=1036.397ms`, `flight_batch=1163.720ms`, `hot_rows_scanned=256`, and `flight_rows_scanned=256`; this is a real improvement over the earlier `~1.25s / 100000 rows_scanned` broad-query seam, but it also proves the remaining floor is still dominated by DuckDB's internal scan and sort work rather than by post-SQL Arrow row materialization alone
  - that evidence now narrows the remaining `100000+` read-path hotspot more precisely: repeated DuckDB parquet view registration was a real warmed-path cost, object cache was not, and the current floor is still dominated by local DuckDB query execution plus Arrow result materialization. The current telemetry proves post-SQL output-row reduction rather than underlying Parquet bytes scanned, so the next deep read-path slice should measure and reduce the remaining local scan/prepare and Arrow decode cost instead of revisiting Valkey or Flight transport assumptions
  - the next DuckDB official-config slice also normalized the connection baseline against the current DuckDB manual instead of keeping speculative Wendao-local defaults: the runtime now exposes `preserve_insertion_order`, `parquet_metadata_cache`, `memory_limit`, and `max_temp_directory_size`, but the kept defaults match DuckDB's documented baseline rather than forcing repo-local tuning
  - the new serial `100000` profile sweep on one shared publication then measured `official_defaults -> point_hot=60.575ms / broad_hot=1286.476ms`, `no_insertion_order -> 58.569ms / 1326.915ms`, `metadata_cache_only -> 60.185ms / 1332.158ms`, and `combined_tuning -> 55.859ms / 1356.704ms`; that evidence keeps `preserve_insertion_order=false` and `parquet_metadata_cache=true` as explicit opt-in knobs instead of default-on behavior, because the broader repo-search seam regressed under every tuned profile
  - the same slice now also lands that baseline in the embedded system config under `resources/config/wendao.toml`: Wendao's default `search.duckdb` profile now explicitly enables the bounded DuckDB lane with `:memory:` storage, `.cache/duckdb/tmp` spill space, `preserve_insertion_order=true`, and `parquet_metadata_cache=false`, while leaving unproven resource knobs such as `memory_limit` and `max_temp_directory_size` unset until a workload-specific benchmark justifies them
  - that same embedded config surface is now also phrased to avoid future memory-layer confusion: the system config no longer spells out DuckDB's `:memory:` marker directly, and the runtime/config docs now clarify that DuckDB's in-memory catalog mode is an execution detail unrelated to `xiuxian-memory-engine`
  - analyzer-side Valkey access now also follows one scope-oriented facade: repo-analysis current/revision lookups and repo-search-query payloads route through typed analyzer cache scopes instead of leaving `analyzers/service`, query-cache helpers, and repo-index incremental reuse coupled to direct Valkey operation methods
  - the same `repo sync` payload is now exposed through the studio gateway at `GET /api/repo/sync?repo=<id>&mode=<ensure|refresh|status>`, and the bundled OpenAPI artifact now documents that route for downstream consumers
  - `repo overview` is now also exposed through the studio gateway at `GET /api/repo/overview?repo=<id>`, so external agent callers can consume the normalized overview counts without shelling out to the CLI
  - `repo module-search` is now also exposed through the studio gateway at `GET /api/repo/module-search?repo=<id>&query=<text>&limit=<n>`, returning normalized module rows from the existing Repo Intelligence service path
  - `repo symbol-search` is now also exposed through the studio gateway at `GET /api/repo/symbol-search?repo=<id>&query=<text>&limit=<n>`, returning normalized symbol rows from the existing Repo Intelligence service path
  - `repo example-search` is now also exposed through the studio gateway at `GET /api/repo/example-search?repo=<id>&query=<text>&limit=<n>`, returning normalized example rows from the existing Repo Intelligence service path
  - `repo doc-coverage` is now also exposed through the studio gateway at `GET /api/repo/doc-coverage?repo=<id>&module=<qualified-name>`, returning normalized doc rows plus covered and uncovered symbol counts from the existing Repo Intelligence service path
  - the common core now also exposes registry-aware library entry points for `repo.overview`, `module.search`, `symbol.search`, `example.search`, and `doc.coverage`, so external crates can reuse the same configured query surface with custom plugin registries
  - `xiuxian-wendao` bootstraps the built-in `julia` plugin automatically for this slice
  - Julia syntax extraction now lives in `xiuxian-ast` behind its `julia` dependency feature, and the built-in Julia analyzer now registers through `xiuxian-wendao::analyzers::languages::julia` while the query/runtime orchestration lives under `xiuxian-wendao::analyzers::service`
  - the Julia AST layer now extracts conservative symbol docstrings and literal `include("...")` edges, and the Wendao Julia bridge now walks the root-file include graph before normalizing `DocRecord` inventory plus explicit `RelationKind::Documents` edges
  - the analyzer implementation is now split across `analyzers/` feature folders with interface-only `mod.rs` boundaries instead of keeping the old `repo_intelligence/` path as the live implementation root
  - `wendao repo overview --repo <id>` returns a real `RepoOverviewResult` through the existing `--output json|pretty` surface
  - `wendao repo module-search --repo <id> --query <text>` returns a real `ModuleSearchResult` through the same output surface
  - `wendao repo symbol-search --repo <id> --query <text>` returns a real `SymbolSearchResult` through the same output surface
  - `wendao repo example-search --repo <id> --query <text>` returns a real `ExampleSearchResult` through the same output surface and now uses explicit `RelationKind::ExampleOf` edges instead of relying only on example file names
  - `wendao repo doc-coverage --repo <id> [--module <module>]` now aggregates explicit `RelationKind::Documents` edges emitted during the Julia link phase instead of performing query-time path/title guessing
  - structural graph edges now exist for `Contains`, `Declares`, `Uses`, `Documents`, and `ExampleOf` in the Julia MVP slice
  - the first external extension validation slice is now landed as workspace crate `xiuxian-wendao-modelica`, which registers `plugins = ["modelica"]` and conservatively indexes `package.mo`, lightweight `.mo` declarations, `Examples`, `UsersGuide`, and inline `annotation(Documentation(...))` docs through the same common-core query surface
  - the external Modelica walker now skips hidden/VCS paths such as `.git`, so documentation inventory no longer picks up repository internals as false-positive docs
  - `xiuxian-wendao-modelica` is now realigned to the live `xiuxian-wendao::analyzers` record and import contracts again, so `cargo check -p xiuxian-wendao -p xiuxian-wendao-modelica` and `cargo test -p xiuxian-wendao-modelica` are both green instead of drifting behind stale common-core schemas
  - `cargo test -p xiuxian-wendao --lib` is now green again after prewarming the repo-analysis cache inside the gateway repo test fixture and splitting the brittle `StudioState::new()` bootstrap threshold from the stricter cached-router latency threshold
  - the external Modelica crate now follows a feature-folder module split, with `lib.rs` reduced to public re-exports and internal responsibilities separated across `plugin/entry.rs`, `plugin/analysis.rs`, `plugin/discovery.rs`, `plugin/relations.rs`, and `plugin/parsing.rs`
  - `module.search` now preserves analyzer order for equal-score matches, allowing language plugins such as `xiuxian-wendao-modelica` to project canonical `package.order` semantics into query results instead of having common-core alphabetical tiebreaks overwrite them
  - `example.search` now also preserves analyzer order for equal-score matches, allowing `xiuxian-wendao-modelica` to project canonical example ordering from `package.order` instead of falling back to title/path alphabetical ordering
  - the external Modelica bridge now classifies repository paths into API, example, documentation, and support surfaces before record projection, keeping runnable `Examples/` models in the example surface while treating `Examples/ExampleUtilities` as support-only and `UsersGuide/` as documentation so `symbol.search`, `example.search`, and repository counts stay focused on library/API entities
  - the external Modelica relation layer now links both `UsersGuide` file docs and `UsersGuide` annotation docs to the owning functional module as well as the visible `UsersGuide` module hierarchy, so module-scoped `doc.coverage` can surface nested guide pages and their inline annotation payloads without falling back to root-only linkage
  - the external Modelica discovery layer now also projects semantic `DocRecord.format` hints for `UsersGuide` assets, distinguishing generic guide pages from `Tutorial`, `ReleaseNotes`, `References/Literature`, `Overview`, `Contact`, `Glossar/Glossary`, `Concept/*Concept`, and `Parameters/Parameterization` content while preserving separate `_annotation` variants for inline documentation payloads
  - the external Modelica discovery layer now also orders `UsersGuide` docs with `package.order` semantics plus stable `package.mo`/annotation positioning, while excluding non-doc control files such as `package.order` from `DocRecord` inventory so `doc.coverage` stays focused on actual documentation assets
  - the external Modelica discovery layer now also normalizes file-backed doc titles to page titles instead of raw filenames, so projected docs read `ReleaseNotes`, `Concept`, or `Overview` rather than `ReleaseNotes.mo`, `Concept.mo`, or `Overview.mo`
  - Repo Intelligence now also exposes a deterministic Stage-2 handoff contract through `build_projection_inputs(...)`, emitting `ProjectionInputBundle` seeds so external analyzers such as `xiuxian-wendao-modelica` can verify that `format`, hierarchy, and attached relations survive into projection-ready page families without going through LLM classification
  - the external Modelica package now also maintains its own `docs/` tree with the same section layout as `xiuxian-wendao/docs`, so Modelica-specific architecture, feature notes, research notes, and roadmap progress can be tracked locally instead of only inside Wendao-wide roadmap files
- Focused verification passed:
  - `cargo check -p xiuxian-wendao -p xiuxian-wendao-modelica`
  - `cargo test -p xiuxian-wendao --test repo_example_search`
  - `cargo test -p xiuxian-wendao --test repo_doc_coverage`
  - `cargo test -p xiuxian-wendao --test repo_module_search`
  - `cargo test -p xiuxian-wendao --test repo_symbol_search`
  - `cargo test -p xiuxian-wendao --test repo_overview`
  - `cargo test -p xiuxian-wendao --test repo_sync`
  - `cargo test -p xiuxian-wendao --test repo_relations`
  - `cargo test -p xiuxian-wendao --test repo_intelligence_registry`
  - `cargo test -p xiuxian-wendao-modelica`
  - `cargo test -p xiuxian-ast --features julia --lib`
- Tier-3 verification is now green for the current Repo Intelligence and external Modelica slice:
  - `cargo clippy -p xiuxian-wendao -p xiuxian-wendao-modelica --all-targets --all-features -- -D warnings`
  - `cargo nextest run -p xiuxian-wendao -p xiuxian-wendao-modelica --no-fail-fast`
- The current post-gate cleanup priority is no longer endpoint expansion; the stale tracked `src/analyzers/service/mod.rs.bak2` monolith has now been removed after confirming the live `analyzers/service/` leaf modules fully cover that split, so the next cleanup focus can move to the remaining modularization wave instead of backup-file hygiene.

## Search Primitive Follow-Up

The next bounded refactor slice should establish a crate-local common fuzzy-search layer inside `xiuxian-wendao` before any wider workspace rollout:

- extract reusable lexical distance and normalized fuzzy-scoring helpers into a shared search module
- define shared fuzzy option contracts for edit distance, prefix length, and transposition behavior
- adapt existing Tantivy-backed lookup paths so fuzzy querying is exposed through a reusable matcher boundary instead of feature-local query construction
- migrate touched callers in Wendao to the common primitive layer first, leaving cross-crate search unification as a later decision

Initial bounded progress for that slice is now landed:

- `xiuxian-wendao` now exposes a shared `search` module with reusable lexical and Tantivy-backed fuzzy matchers
- `crate::search::tantivy` now also owns a shared `SearchDocumentIndex` and shared search-document schema so new search backends stop rebuilding Tantivy field layouts ad hoc
- the shared fuzzy hot path now uses a three-row scratch-buffer edit-distance implementation and avoids repeated query lowercasing inside the lexical/Tantivy matcher loops, reducing per-candidate allocation churn
- the shared fuzzy hot path now also reuses query/candidate char buffers and distance scratch space across lexical/Tantivy matcher loops, while public scoring helpers (`edit_distance`, `normalized_score`, `score_candidate`) now borrow thread-local buffers instead of allocating fresh `Vec<char>` / `Vec<usize>` scratch state on every call
- Tantivy best-fragment rescoring now also walks borrowed fragment slices directly instead of first materializing a temporary `Vec<String>` candidate list for each stored title, and only allocates the final `matched_text` once the winning fragment is known
- Tantivy identifier-boundary discovery now also uses a single-pass `Peekable<char_indices()>` state machine instead of collecting every `(byte_idx, char)` pair into a temporary `Vec`, and the splitter semantics are now pinned by direct unit tests for CamelCase, acronym-to-word, and alpha-digit transitions
- `LexicalMatcher::search` now also uses the shared thread-local fuzzy buffers directly instead of constructing fresh `query_chars`, `candidate_chars`, and `scratch` vectors on every search call, and focused tests now verify consecutive lexical searches clear that reused state correctly
- prefix gating now also runs inside the shared candidate-lowercasing pass instead of scanning each candidate once for prefix validation and a second time for lowercase collection, and the prefix comparison now treats non-ASCII case pairs through `char::to_lowercase()` equality rather than ASCII-only case folding
- `FuzzySearchOptions` now also exposes a preparatory `camel_case_symbol()` profile so future Julia/Modelica symbol callers can opt into relaxed prefix gating for CamelCase-style abbreviations without changing the default symbol profile
- `UnifiedSymbolIndex` now supports option-aware fuzzy lookup through the shared matcher layer and now reuses the shared search-document schema instead of maintaining a feature-local Tantivy schema
- Repo Intelligence `module.search`, `symbol.search`, and `example.search` now also build shared `SearchDocumentIndex` instances and use an `exact Tantivy -> fuzzy Tantivy -> legacy lexical fallback` ranking pipeline, so the studio repo handlers inherit the shared search primitives without re-scanning every module/symbol/example row first
- deterministic projected-page search now also uses `build_projected_pages(...)` plus the shared search-document index for exact and fuzzy retrieval before falling back to the existing keyword/path heuristics
- projected-page family search, navigation search, retrieval context, and projected page-index tree lookup now also resolve against the shared projected-page and projected page-index tree builders instead of re-deriving partial views from raw docs
- `build_projected_retrieval_hit(...)` now also resolves stable projected `page_id` values through the shared projected-page lookup instead of misreading them as raw stage-one `doc_id` values
- Tantivy best-fragment rescoring now also expands CamelCase and alpha-digit identifier spans, so symbol-like names get higher-quality secondary matches even before a full custom tokenizer lands
- LinkGraph topology discovery now has a typo-tolerant lexical title fallback backed by the same shared fuzzy primitives
- Studio definition resolution, semantic-auditor fuzzy scoring, and graph dedup edit-distance scoring now reuse the shared primitives instead of carrying isolated edit-distance implementations
- dedicated projection integration targets now validate the shared projected-page search/navigation/retrieval slice through a stable in-memory analysis fixture, avoiding the currently broken built-in Julia plugin bootstrap path while keeping the search-contract assertions in place
- the `repo_projected_` slice of `wendao-validation-gate` is now back to green after updating the stale projection fixtures to the current contracts and accepting the deterministic snapshot drift
- the `repo_example_search` slice of `wendao-validation-gate` now also passes with shared Tantivy-backed typo handling for example-title queries, and the stale CLI JSON snapshot baseline has been refreshed to the current payload shape
- the filtered `repo_overview` and `repo_sync` slices of `wendao-validation-gate` are now green again after restoring overview aggregation semantics and managed-source drift/freshness classification, and the affected overview snapshots have been refreshed to the current symbol/diagnostic payload shape
- focused lib tests now validate typo-tolerant Repo Intelligence module/symbol retrieval through `analyzers::service::search::tests::*`, which stays runnable even while the broader `wendao-validation-gate` target is blocked by unrelated compile failures
- projected doc-kind inference now also honors the shared doc-format contract for standalone `reference` docs while still upgrading symbol-anchored explanation docs to `Reference`, which unblocked the shared projected-page lib tests and removed one source of repo-sync payload drift
- the bundled Wendao gateway OpenAPI artifact now also covers `/api/analysis/code-ast`, keeping the route inventory test aligned with the runtime gateway surface
- `cargo test -p xiuxian-wendao --lib` is now green again after refreshing the affected studio Markdown-analysis and repo-sync snapshot baselines to the current response contracts

## Gateway-Driven Tantivy Performance Landing

The next bounded repo-intelligence performance slice is now tracked in the
active performance-landing ExecPlan rather than this persistent roadmap note.

Its execution contract is:

- keep repo gateway search on the current analyzer path for this slice instead of widening the owner-path migration
- add reusable analyzer-derived `RepositorySearchArtifacts` and per-endpoint query caches so `/api/repo/module-search`, `/api/repo/symbol-search`, `/api/repo/example-search`, and `/api/repo/projected-page-search` stop rebuilding Tantivy indexes per request
- upgrade the shared Tantivy search layer toward multi-field exact/prefix/fuzzy recall, code-aware tokenization, lightweight hit rehydration, and bounded rescoring
- preserve the current repo gateway HTTP contracts while aligning semantic search assumptions with the canonical Flight contract `/search/intent`
- finish with a gateway-level async performance suite that exercises the four repo-search endpoints plus Studio code search in warm-cache steady state

Initial execution for that slice is now landed:

- shared Tantivy search documents now split `title/path/namespace` into
  `*_exact` and `*_text` fields, and `terms` now participates in the shared
  full-text/fuzzy query layer instead of leaving `title` as the only fuzzy
  recall field
- the shared Tantivy layer now registers a code-aware tokenizer for
  camelCase, snake_case, acronym, and alpha-digit boundaries, and the search
  API now returns lightweight hit records that are rehydrated through local
  lookup maps instead of eagerly materializing full `SearchDocument` payloads
- repo gateway `module.search`, `symbol.search`, `example.search`, and
  `projected-page.search` now build immutable analyzer-derived
  `RepositorySearchArtifacts` once per cached analysis identity and then reuse
  those search indexes plus a second-layer query-result cache for repeated
  requests
- the semantic search lane stayed on the current `search` runtime path for that
  slice, but the long-term contract is now the Flight route `/search/intent`
  rather than a Studio HTTP search endpoint
- the blocking modularity regressions in
  `src/search/service/tests/mod.rs` and
  `src/zhenfa_router/native/semantic_check/docs_governance/tests/mod.rs` were
  cleaned up by moving helper logic into dedicated `support.rs` modules and
  keeping `mod.rs` interface-only
- the owner-path cut is now also landed: `src/search/` is the sole search
  implementation root, `src/search_plane/` is gone, and repo-index/query-core/
  gateway callers now import `crate::search`
- the gateway perf suite now uses runner-aware warm-cache budgets instead of
  placeholder thresholds. The current workstation-safe local profile is:
  - `repo_module_search`: `p95 <= 1.25ms`, `qps >= 500`
  - `repo_symbol_search`: `p95 <= 1.25ms`, `qps >= 700`
  - `repo_example_search`: `p95 <= 1.5ms`, `qps >= 600`
  - `repo_projected_page_search`: `p95 <= 1.5ms`, `qps >= 700`
  - `studio_code_search`: `p95 <= 10.0ms`, `qps >= 100`
  - `search_index_status`: `p95 <= 0.48ms`, `qps >= 1250`
- the stable gateway warm-cache suite is now also formalized under the
  `performance` feature. `src/gateway/studio/perf_support.rs` exposes a narrow
  gateway fixture surface, and `tests/performance/gateway_search.rs` mounts six
  serialized formal cases for `repo_module_search`, `repo_symbol_search`,
  `repo_example_search`, `repo_projected_page_search`, `studio_code_search`,
  and `search_index_status`
- the formal gateway lane now owns the calibration knobs directly. It resolves
  runner-specific defaults through `RUNNER_OS` and accepts explicit
  `XIUXIAN_WENDAO_GATEWAY_PERF_<CASE>_<METRIC>` overrides without reviving a
  duplicate in-crate calibration suite
- the local workstation can now also exercise a real large-corpus gateway lane
  against `.data/wendao-frontend` via
  `just rust-wendao-performance-gateway-real-workspace`; that ignored sample
  now reuses a single fixture, bootstraps the `179` configured repositories
  until the workspace is query-ready, and then records cross-repo
  `code_search` plus `repo/index/status` latency without turning that heavy
  scenario into the default blocking gate
  - the latest local sample reports `studio_code_search_real_workspace_sample`
    at `p95 = 142.719ms` and `repo_index_status_real_workspace_sample` at
    `p95 = 1.331ms`
- the execution entrypoints are now explicit too. `just
rust-wendao-performance-gate` expands into
  `rust-wendao-performance-quick` and
  `rust-wendao-performance-gateway-formal`. The quick lane stays on `nextest`
  but explicitly excludes the six `gateway_search` cases, while the formal
  gateway proof runs through
  `just rust-wendao-performance-gateway-formal`, which now drives the same
  focused `cargo nextest ... -E <formal-filter>` bundle used by direct
  validation instead of a separate `cargo test formal_gate` process
- the old lib-only gateway perf calibration lane is now removed, so the quick
  perf entrypoint no longer depends on a duplicate in-crate gateway suite to
  keep `nextest` and `clippy` green
- focused verification now covers the full default Wendao lib surface, the
  `wendao-validation-gate` contract target, and the full default feature-gated
  gateway perf suite:
  - `cargo test -p xiuxian-wendao --lib`
  - `cargo test -p xiuxian-wendao --test wendao-validation-gate`
  - `cargo check -p xiuxian-wendao --features performance --tests`
  - `cargo test -p xiuxian-wendao --features performance --test wendao-validation-gate -- --list`
  - `cargo nextest run -p xiuxian-wendao --features performance --test wendao-validation-gate -E "not (test(performance::gateway_search::repo_module_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::repo_symbol_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::repo_example_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::repo_projected_page_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::studio_code_search_perf_gate_reports_warm_cache_latency_formal_gate) | test(performance::gateway_search::search_index_status_perf_gate_reports_query_telemetry_summary_formal_gate))"`
  - `just rust-wendao-performance-gateway-formal`
  - `just rust-wendao-performance-gate`
  - `cargo nextest run -p xiuxian-wendao`
- Tier-3 closure is now green for the touched Wendao scope:
  - `cargo clippy -p xiuxian-wendao --all-targets --all-features -- -D warnings`
- the cached repo gateway search surface is now stricter too. `module.search`,
  `symbol.search`, `example.search`, and the shared projected-page cached lane
  now load ready cached analysis only instead of silently falling through to
  request-path `Ensure`
- those cached repo gateway endpoints therefore no longer acquire the
  managed-remote sync permit on the happy path, which removes request-path
  remote-sync contention from the same mixed-hotset pressure lane that was
  timing out under steady-state load
- when a repo has no ready analysis cache yet, those cached repo gateway
  endpoints now fail fast with `409 REPO_INDEX_PENDING` instead of stalling
  behind on-demand analysis or remote materialization work
- the ready analysis cache is no longer in-process only. `ValkeyAnalysisCache`
  now persists normalized `RepositoryAnalysisOutput` snapshots under a
  repository/revision/plugin-scoped key, so cached repo gateway paths can
  recover a ready analyzer snapshot after process restart when stable revision
  identity is available
- that same Wendao-owned Valkey boundary now also persists repo-search
  query-result payloads under full `RepositorySearchQueryCacheKey` identity,
  including endpoint, query, filter, fuzzy-search options, and limit. Hot
  `module`, `symbol`, `example`, `import`, and `projected page` reads can
  therefore recover after process restart without first rebuilding the same
  ranked response from scratch when the analysis snapshot identity has not
  changed
- that Valkey runtime is now explicitly Wendao-owned and sync-bound:
  `XIUXIAN_WENDAO_ANALYZER_VALKEY_URL` is the canonical repo-analysis cache
  endpoint, `VALKEY_URL` / `REDIS_URL` remain generic fallback inputs, and
  optional key-prefix / TTL controls live alongside the analyzer cache rather
  than inside `xiuxian-vector`
- the residual repo-gateway verification caveat is no longer `clippy`; it is
  now the need to keep the formal gateway six-case proof stable under one
  shared nextest filter and decide later whether the current Linux/local budget
  split should become a broader shared helper
- `tests/unit/studio_vfs_performance.rs::studio_state_creation_is_fast` now
  measures a warmed best-of-five `StudioState::new()` sample window instead of
  a single wall-clock sample, which keeps `cargo test --lib` and
  `cargo nextest` stable under normal concurrent test scheduling while still
  enforcing the bootstrap budget
- real-workspace repo-index perf support now also owns a mixed gateway-load
  proof under `tests/performance/support/repo_index_audit/`, so the `177+`
  repo workspace can be audited while concurrent repo-backed gateway queries
  are in flight instead of treating repo-index status as the whole system
- that mixed-load proof first probes candidate repo-backed URIs and only
  replays the ones that return `200 OK`, while logging skipped probes
  explicitly. This prevents fixed `409 Conflict` capability mismatches from
  being misclassified as transport or queue regressions during the stress run
- the latest real `.data/wendao-frontend` mixed-load proof completed in `62s`
  with `177 total`, `160 ready`, `17 unsupported`, `0 failed`, while
  `6` query workers at `6ms` pause issued `291` successful gateway requests
  against the accepted query mix
- the same proof also exposed one real product-shape constraint: the bootstrap
  probe for
  `/api/repo/projected-page-search?repo=TestPackage&query=solve&kind=reference&limit=5`
  returned `409 Conflict`, so the route is now logged as a skipped mixed-load
  candidate instead of polluting the gateway query failure count
- per-sample live logs now include gateway-side query totals, `p95`/max
  latency, busiest URI, and last error alongside repo-index queue progress, so
  the next optimization slice can distinguish transport pressure from query
  latency pressure without another ad hoc probe pass

## Open Constraint

The repository-level semantic-addressing blueprint is now present again in the
workspace. The Repo Intelligence MVP should treat that blueprint plus this
roadmap note and the paired ExecPlan as the active execution guide, while the
exact hidden-path reference remains in the tracking record.
