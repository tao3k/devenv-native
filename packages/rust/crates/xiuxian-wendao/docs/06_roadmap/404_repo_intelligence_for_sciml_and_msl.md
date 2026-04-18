# Repo Intelligence for SciML and MSL

:PROPERTIES:
:ID: wendao-repo-intelligence-sciml-msl
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, repo-intelligence, julia, modelica, plugins, git
:STATUS: IN-PROGRESS
:END:

## Active References

- `[[docs/rfcs/2026-03-27-wendao-arrow-plugin-flight-rfc.md]]`
- `[[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md]]`
- `[[06_roadmap/405_large_rust_modularization]]`
- `[[06_roadmap/409_core_runtime_plugin_surface_inventory]]`

The architecture notes in this roadmap remain useful context, but the current
migration direction for `core`, `runtime`, and independently published
Arrow-native plugin packages is now governed by the RFC set above. The active
task-scoped blueprint remains tracked in the paired ExecPlan rather than this
persistent roadmap note. The large-file modularization program is no longer a
parallel cleanup lane; it is now part of the same architectural discipline,
and future migration phases must satisfy both ownership and structural
modularization rules.

The `P0 / Mapping Gate` inventory is now tracked separately in
`[[06_roadmap/409_core_runtime_plugin_surface_inventory]]` so current Julia
host leaks, target ownership, and destination feature folders stay explicit
while implementation proceeds.

## Core Vision

This roadmap note defines a Wendao-native **Repository Intelligence** architecture for two target ecosystems:

- Julia SciML repositories
- MSL (Modelica Standard Library)

The goal is to move beyond fuzzy search and give agents a stable, pre-indexed understanding of repositories. The first milestone is a Repo Intelligence MVP that answers repository overview, module, symbol, example, and documentation coverage queries without repeated repository exploration. Deep wiki generation remains a downstream phase built on top of the indexed repository graph.

## Implementation Update: Hot-Path Removal and Visible Progress

Current 2026-03-21 checkpoint:

- Studio now has a dedicated `gateway/studio/repo_index/` feature folder for repository-intelligence background indexing.
- Studio now also has a dedicated `gateway/studio/symbol_index/` feature folder for local project symbol indexing.
- The coordinator owns:
  - a host-derived adaptive bounded worker window
  - per-repo phase tracking
  - snapshot storage
  - interactive-priority queue promotion for user-targeted repos
  - aggregate `/api/repo/index/status` reporting
- `code_search` no longer performs request-time repository analysis or checkout-wide source scanning. It reads repo-index snapshots and returns explicit pending metadata through the unified search contract:
  - `partial`
  - `indexingState`
  - `pendingRepos`
  - `skippedRepos`
- `search_symbols` no longer performs first-hit request-path index construction. It now triggers background local symbol indexing and returns explicit pending metadata through `SymbolSearchResponse`:
  - `partial`
  - `indexingState`
  - `indexError`
- Repo-intelligence read endpoints now use cached-only analysis lookup so missing snapshots surface as `REPO_INDEX_PENDING` instead of silently triggering cold analysis.
- Julia repository analysis now performs a cheap preflight before heavy indexing so unsupported layouts settle into `unsupported` state instead of re-triggering hot-path failures.
- Repo-index terminal status writes now avoid `RwLock` re-entry deadlocks. Ready/failed status transitions compute the next attempt count before taking the `statuses` write guard, so one completed repo can no longer freeze the entire single-worker queue in `indexing`.
- Repo-index scheduling now derives its hard concurrency ceiling from host parallelism and adjusts the live worker window from recent repo-task latency, so the queue is no longer intentionally serialized behind a fixed worker count.
- The default gateway runtime now also carries the builtin `modelica` analyzer by compiling the external `xiuxian-wendao-modelica` plugin sources into the core crate under the `modelica` feature, closing the previous `mcl` live failure mode where the repo indexer could see the repository but not the registered analyzer.
- Repo indexing now also performs generation-aware stale-task checks before result publication and during code-document collection, preventing obsolete repo tasks from continuing all the way to snapshot commit after a fingerprint/config change.
- Adaptive repo scheduling now uses an efficiency-gradient signal (`concurrency / EMA latency`) instead of simple success streak alone, allowing the worker window to shrink when marginal concurrency stops improving throughput and likely indicates IO contention.
- A bounded analysis timeout remains as a secondary safety net so a genuinely stuck repository analysis is marked `failed` and the queue can continue.
- The frontend now separates `repoIndexStatus` from `vfsStatus`, polls `/api/repo/index/status` independently, keeps workspace boot gated only on health/config sync, surfaces indexing progress in the bottom status bar, and exposes unsupported/failed repo issue details directly from the status payload so parser/layout gaps can be triaged from live Studio.
- SearchBar `lang:` autocomplete now consumes the gateway-reported supported-language list from `/api/ui/capabilities` rather than deriving languages from frontend `wendao.toml` plugin inference. The gateway registry is now the source of truth for available code-language suggestions.

This closes the main architectural gap that previously allowed large multi-repo Julia configs to collapse Studio responsiveness under repeated request-time analysis.

## Architecture Mapping

This note maps the proposed system directly onto the current Wendao architecture instead of using conceptual overlays.

### Current Module Boundaries

- `xiuxian-wendao`
  - owns the common Repo Intelligence core
  - owns the plugin authoring interface
  - owns the Julia analyzer bridge and normalization flow
- `xiuxian-ast`
  - owns Julia syntax parsing primitives used by Repo Intelligence
- `xiuxian-wendao-modelica`
  - is an external Rust extension crate
  - implements Modelica and MSL-specific semantic analysis against Wendao's plugin interface

In concrete terms, the target structure is:

```text
xiuxian-wendao
  - analyzers common core
  - analyzers plugin interface
  - analyzers julia bridge/orchestration

xiuxian-ast
  - julia syntax parser and source-summary extraction

xiuxian-wendao-modelica
  - external Modelica/MSL extension crate
```

### Terminology Mapping

To avoid architectural ambiguity, the following terms are mapped to concrete implementation targets:

| Conceptual term    | Concrete implementation meaning                                                                                                       |
| :----------------- | :------------------------------------------------------------------------------------------------------------------------------------ |
| `Prospector`       | Repo analysis logic in `xiuxian-wendao::analyzers::{languages, service}` and external plugin crates such as `xiuxian-wendao-modelica` |
| `HippoRAG`         | Wendao graph, retrieval, and fusion capabilities                                                                                      |
| `Annotator`        | Later-stage document projection or classification flow, not part of the Repo Intelligence MVP indexing path                           |
| `Trinity / Qianji` | Higher-level orchestration for documentation workflows, not a required dependency for core repository indexing                        |
| `Skeptic`          | Post-generation or post-projection verifier that checks generated content against indexed repository records                          |

## Common Core and Extension Boundary

### Common Core in `xiuxian-wendao`

The common core should absorb everything that is high-performance, repeated, and repository-agnostic:

- repository registry from `link_graph.projects.<id>` in `wendao.toml`
- dedicated `xiuxian-git-repo` ownership for git mirror management, ghq-style
  layout, checkout locking, revision sync, and backend-neutral substrate
  contracts
- local checkout validation and canonical git-root discovery
- managed checkout materialization from upstream git URLs
- mirror-backed checkout refresh policy
- registry-aware library entry points so external crates can reuse the same configured query surface with custom plugin registries
- lifecycle health summarization for source status and drift diagnostics
- lifecycle freshness timestamps for sync observation and last local fetch visibility
- lifecycle staleness classification for mirror freshness reporting
- grouped status-summary projection for agent-facing sync inspection
- incremental discovery and invalidation
- normalized record storage
- graph persistence
- shared query contracts
- plugin registration and diagnostics

Update 2026-04-05:

- the repo substrate boundary is now being extracted into the dedicated crate
  `xiuxian-git-repo`
- `xiuxian-wendao/src/git/` is now fully retired; the only Wendao-local repo
  bridge that remains is `src/analyzers/repo_source.rs`, which maps
  `RegisteredRepository` config and domain errors onto
  `xiuxian-git-repo` contracts without duplicating substrate enums or structs
- `xiuxian-git-repo` has now removed its runtime `git2` dependency and uses
  `gix` for repository open/probe/revision/drift handling without
  re-expanding the public API around backend-specific handles
- `xiuxian-git-repo` now also persists managed mirror and checkout remote
  alignment through `gix` local-config writes plus repository reload instead
  of shelling out to `git remote add/set-url`
- `xiuxian-git-repo` managed remote target probing now also stays inside the
  backend substrate by using `gix` remote ref-map inspection with explicit
  HEAD/branch/tag probe refspecs instead of `git ls-remote`
- `xiuxian-git-repo` now also performs managed bare clone, checkout clone,
  and origin fetch through `gix`, while preserving mirror refspec parity for
  bare repos instead of falling back to native `git clone` or `git fetch`
- annotated tag probe results now align with peeled checkout revisions, so
  fetch-policy reuse no longer treats tag-pinned repositories as stale when
  the tag target is unchanged
- local path remotes are now normalized through canonical filesystem paths
  before drift comparison, which keeps tmp-backed mirrors and checkouts from
  re-fetching solely due to path aliases such as `/var` versus `/private/var`
- `xiuxian-wendao` itself no longer depends on `git2`; the remaining
  Wendao-owned repo fixtures now use bounded CLI git helpers, and the
  repo-intelligence snapshots redact concrete revision hashes so the contract
  stays backend-neutral
- `xiuxian-git-repo` now also performs detached checkout through `gix`
  index/worktree/reference mutation, including tracked-path pruning for
  stale-file parity when a managed checkout moves backward across revisions
- detached checkout stale-path cleanup now also refuses recursive directory
  removal when a previously tracked file path is unexpectedly backed by a
  directory, preserving unrelated untracked contents instead of deleting them
  during collision handling
- the internal `gix` backend has been split into
  `packages/rust/crates/xiuxian-git-repo/src/backend/gix/` so retry/open/
  probe/remote/clone/fetch/checkout logic evolve as separate responsibilities
  instead of accreting inside one oversized backend file
- no production native-`git` bridge remains in `xiuxian-git-repo`; the repo
  substrate is now end-to-end `gix` backed behind the crate-owned API surface
- the immediate Wendao done-gate follow-up is now also closed: repo gateway
  cache tests use unique keyspaces, repo sync/overview snapshots normalize
  revision hashes, publication-state hydration tests use a deterministic
  awaitable runtime sync helper, and
  `cargo test -p xiuxian-wendao --lib` plus
  `cargo clippy -p xiuxian-wendao --lib --tests -- -D warnings` are green
  again for the migrated lane

Legacy `[[repo_intelligence.repos]]` configuration should not remain part of the active runtime contract. Older files may still carry that table during migration, but the runtime loader now ignores it and derives registrations only from project-scoped entries.

### Plugin Interface Inside `xiuxian-wendao`

The plugin authoring interface is integrated into `xiuxian-wendao`, not split into a separate package. A plugin or native analyzer should operate against Wendao-owned records and relations.

The first interface should stay narrow:

1. detect whether a repository or file set is supported
2. analyze files into normalized records
3. enrich cross-file and cross-module relations
4. optionally expand or rerank query results

### Native Julia Support

Julia repository intelligence is a Wendao-native path, but the syntax parser itself should stay in `xiuxian-ast`.

The Julia path should therefore split responsibilities:

- `xiuxian-ast`
  - parses Julia source files
  - extracts root modules, literal include edges, exports, imports, conservative symbol summaries, and source docstring attachments
- `xiuxian-wendao::analyzers::languages::julia` plus `xiuxian-wendao::analyzers::service`
  - register the built-in Julia analyzer
  - load repository metadata
  - resolve local git checkout metadata
  - consume managed checkouts prepared by the common core when only `url/ref` is configured
  - discover doc and example assets
  - walk the root-file include graph
  - normalize Julia records and source-derived docs into Wendao-owned repository records
  - emit relation edges such as `Uses` and `Documents`

The Julia bridge should focus on:

- `Project.toml` awareness
- module and include structure
- exported names and reexport surfaces
- documentation asset discovery and linking
- conservative method and signature extraction

Julia support should be described as a Wendao-native analyzer flow backed by `xiuxian-ast`, not as a standalone external plugin crate.

### External Modelica Support

Modelica support should live in an external crate:

- crate name: `xiuxian-wendao-modelica`
- role: external Rust extension crate for Modelica and MSL analysis

This crate should use the plugin interface integrated into `xiuxian-wendao` and return normalized records rather than mutating Wendao internals directly.

Current conservative implementation status:

- the workspace crate boundary is established
- `register_into(...)` integrates a `modelica` plugin into an existing Wendao registry
- the first implementation indexes `package.mo` hierarchies, lightweight `.mo` declarations, `Examples`, `UsersGuide`, and conservative `annotation(Documentation(...))` docs
- repository walking now skips hidden/VCS paths, preventing `.git` internals from leaking into documentation inventory
- the external crate itself is now split into a feature folder so plugin entry, discovery, parsing, and relation construction evolve independently instead of accreting in a monolithic `lib.rs`
- `package.order` is now consumed for canonical module ordering, and the common-core module query path preserves analyzer order for equal-score matches so Modelica package semantics can survive the query layer
- `package.order` also now feeds canonical example ordering, and the common-core example query path preserves analyzer order for equal-score matches so example catalogs can keep Modelica-authored sequence semantics
- Modelica discovery now classifies repository paths into API, example, documentation, and support surfaces before projection, keeping runnable `Examples/` models in the example surface while treating `Examples/ExampleUtilities` as support-only and `UsersGuide/` models as documentation assets, preventing tutorial/doc/support models from inflating library symbol counts or polluting default search surfaces
- `UsersGuide` file docs and `UsersGuide` annotation docs now also emit `Documents` links to their owning functional modules and visible `UsersGuide` hierarchy modules instead of only linking the root package, allowing module-scoped documentation coverage to surface nested guide pages and inline guide annotations in the relevant subsystem
- external Modelica discovery now also emits semantic `DocRecord.format` hints for `UsersGuide` assets, distinguishing generic guide pages from `Tutorial`, `ReleaseNotes`, `References/Literature`, `Overview`, `Contact`, `Glossar/Glossary`, `Concept/*Concept`, and `Parameters/Parameterization` content while preserving `_annotation` variants for inline `annotation(Documentation(...))` payloads
- external Modelica discovery now also preserves `package.order` semantics in projected `UsersGuide` doc ordering, keeps `package.mo` and inline annotation payloads in a stable relative position, and excludes non-doc control files such as `package.order` from the doc graph
- external Modelica discovery now also normalizes file-backed doc titles to page titles instead of raw filenames, improving downstream projection and agent-facing readability without changing the common-core schema
- external Modelica queries already reuse the same common-core `repo.overview`, `module.search`, `symbol.search`, `example.search`, and `doc.coverage` entry points through registry-aware helpers
- package-local documentation tracking for the external extension now lives under `packages/rust/crates/xiuxian-wendao-modelica/docs/`, mirroring the `01_core` / `03_features` / `05_research` / `06_roadmap` section layout used by `xiuxian-wendao/docs`

## Repository Intelligence MVP

### MVP Query Surface

The first delivery target is a deterministic repository query layer, not deep wiki generation.

The MVP should answer:

- `repo.overview`
- `module.search`
- `symbol.search`
- `example.search`
- `doc.coverage`

These five queries are the stable contract that downstream agents and later documentation generation should consume.

`doc.coverage` should be defined as a repository-graph aggregation query, not a query-time guessing routine. The correct pipeline is:

1. discover `DocRecord` inventory during repository analysis
2. emit explicit `RelationKind::Documents` edges during Julia/Modelica link phases
3. aggregate documentation coverage from those relations at query time

The query should not infer coverage from ad-hoc path or title matching inside the service layer.

### Record Model

The initial normalized record model should include:

- `RepositoryRecord`
- `ModuleRecord`
- `SymbolRecord`
- `ExampleRecord`
- `DocRecord`
- `RelationRecord`
- `DiagnosticRecord`

This record model must be language-neutral so both SciML and MSL can flow through the same Wendao core.

## Retrieval and Ranking

Wendao should continue to use weighted fusion across vector, keyword, and graph signals. However, this note treats retrieval fusion as a shared Wendao concern, not a Julia-only architecture.

### Weighted RRF

For each candidate document or entity $d$, the fused score is:

$$Score(d) = \sum_{i \in \{Vector, Keyword, Graph\}} \frac{W_i}{K + rank_i(d)}$$

Where:

- $rank_i(d)$ is the rank of $d$ in the $i$-th retrieval engine
- $W_i$ is the engine weight
- $K$ is a smoothing constant, defaulting to 60

### Shared Signal Sources

1. **Vector (LanceDB)** for semantic similarity recall
2. **Keyword (Tantivy/BM25)** for exact token matching
3. **Graph (Wendao graph/PPR)** for structural saliency and topology-aware relevance

### Intent-Driven Weighting

Query-intent-driven weighting remains a Wendao retrieval concern and should be shared by Julia and Modelica repository intelligence flows.

## Proposed API and Configuration

### Configuration Shape

The repository registry should live in `wendao.toml`. Example:

```toml
[link_graph.projects.sciml-diffeq]
url = "https://github.com/SciML/DifferentialEquations.jl.git"
ref = "main"
refresh = "fetch"
plugins = ["julia"]
dirs = ["docs"]

[link_graph.projects.msl]
url = "https://github.com/modelica/ModelicaStandardLibrary.git"
ref = "main"
refresh = "fetch"
plugins = ["modelica"]
dirs = ["Modelica/UsersGuide"]
```

### Search and Query Modes

The first public API should reflect the MVP query surface instead of introducing Julia-specific URI schemes as the primary contract.

Examples:

```bash
wendao repo sync --repo sciml-diffeq
wendao repo sync --repo sciml-diffeq --mode refresh
wendao repo sync --repo sciml-diffeq --mode status
wendao repo overview --repo sciml-diffeq
wendao repo module-search --repo sciml-diffeq --query OrdinaryDiffEq
wendao repo symbol-search --repo sciml-diffeq --query solve
wendao repo example-search --repo msl --query HeatExchanger
wendao repo doc-coverage --repo msl --module Modelica.Fluid
```

For remote repositories, the common core now uses a two-stage source lifecycle:

1. prepare or refresh a cache-local bare mirror under `PRJ_CACHE_HOME`
2. materialize or refresh a managed checkout from that mirror

This keeps remote fetch behavior inside the common core and prevents language plugins from owning repository transport details.

Current implementation checkpoint for this transport layer:

- repository transport is now owned by `xiuxian-wendao::git::checkout` instead of `analyzers`
- managed checkouts now derive ghq-style physical paths from the configured remote URL
- if a configured repo keeps the same logical id but changes URL in `wendao.toml`, the managed mirror remote is overwritten from config and the checkout is synchronized against that new source of truth
- managed checkout mutation now uses a dedicated lock path under `PRJ_DATA_HOME/.data/xiuxian-wendao/repo-intelligence/locks/...`
- `checkout/mod.rs` is now interface-only, with checkout data types in `types.rs` and checkout locking in `lock.rs`
- the external Modelica analyzer now accepts repositories whose dominant root package lives under a top-level package directory (for example `Modelica/package.mo` in `ModelicaStandardLibrary`) and rewrites indexed record paths back to repository-relative paths so Studio VFS navigation remains clickable
- repo-index status payloads now include per-repository queue position so Studio can surface which repo is next after interactive promotion instead of forcing operators to infer queue order from aggregate counters alone

The common core now exposes this lifecycle explicitly through `wendao repo sync --repo <id>`, which reports the resolved source kind, requested sync mode, configured refresh policy, mirror/check-out lifecycle states, mirror revision, drift state, checkout path, optional mirror path, upstream URL, and checkout revision before any language-specific analysis runs. `--mode refresh` can force a remote refresh even when the repository is configured with `refresh = "manual"`. `--mode status` is read-only: it inspects the current cache state and returns `missing` lifecycle states when mirror or checkout assets do not exist yet.

For external extensions, the common core now also exposes registry-aware library entry points so a crate such as `xiuxian-wendao-modelica` can answer the same repository query families without depending on CLI wrappers or gateway-only glue.

## Deep Wiki as a Later Phase

Deep wiki generation remains in scope, but not as a prerequisite for Repo Intelligence MVP.

The intended flow is:

1. build repository intelligence records and relations
2. expose stable repository query APIs
3. project documentation views such as tutorial, how-to, explanation, and reference
4. optionally use Qianji for classification, refinement, and audit

## Execution Phases

1. **Phase 1 (Wendao Core)**: add `repo_intelligence` common core plus native Julia support inside `xiuxian-wendao`
2. **Phase 2 (External Extension Validation)**: build `xiuxian-wendao-modelica` as the first external extension crate
3. **Phase 3 (Documentation Projection)**: implement Diataxis-oriented deep wiki projection on top of the indexed repository graph

Phase 2 is now started and minimally landed through the first conservative `xiuxian-wendao-modelica` crate plus registry-aware query validation.

### Gateway-Reported Capability Surface

The `lang:` autocomplete surface is now meant to consume the gateway-reported supported-language list from `/api/ui/capabilities` rather than deriving language candidates from frontend TOML plugin inference. The gateway plugin registry is the source of truth for active code-language suggestions, which keeps the autocomplete surface aligned with the runtime analyzer set.

## Why Better

- **Vs. BM25 only**: topology, records, and semantic relations are available to retrieval and navigation
- **Vs. ad-hoc agent exploration**: agents consume stable repository queries instead of repeating repository scans
- **Vs. repo-specific one-off tooling**: SciML and MSL share one Wendao core while preserving language-specific precision at the correct boundary
