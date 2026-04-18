# Document Projection and Retrieval Enhancement

:PROPERTIES:
:ID: wendao-document-projection-retrieval
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, docs, page-index, ppr, diataxis
:STATUS: PLANNED
:END:

## Purpose

This roadmap note defines the second-stage architecture that sits on top of Repo Intelligence MVP.

Stage 1 establishes repository records and stable repository queries:

- `repo.overview`
- `module.search`
- `symbol.search`
- `example.search`
- `doc.coverage`

Stage 2 projects those indexed records into a documentation layer and enhances retrieval using existing Wendao kernels such as page index, graph diffusion, weighted fusion, and agentic retrieval.

This roadmap note is scoped to the current target scenarios:

- Julia SciML repositories
- MSL (Modelica Standard Library)

## Position in the Architecture

The intended layering is:

```text
Stage 1: Repo Intelligence
  git mirror -> sync -> normalized records -> repo graph -> stable repo queries

Stage 2: Document Projection and Retrieval Enhancement
  repo records -> document pages -> page index -> graph-enhanced retrieval -> deep wiki surfaces
```

This stage does not replace Repo Intelligence. It consumes Repo Intelligence outputs and turns them into documentation objects and richer retrieval behavior.

## Core Principle

Wendao should not generate documentation directly from raw search results.

Instead, the system should:

1. index repository structure into stable records
2. project those records into documentation pages
3. index page structure with existing Wendao page kernels
4. run retrieval over both repository records and projected documentation pages

The key shift is from "search raw chunks and ask the LLM to improvise" to "project stable pages from indexed truth, then retrieve over those pages."

## Projection Model

### Target Page Types

The projection layer should build four first-class page families aligned with Diataxis:

- `ReferencePage`
- `HowToPage`
- `TutorialPage`
- `ExplanationPage`

### Projection Inputs

The projection engine consumes Stage 1 records:

- `RepositoryRecord`
- `ModuleRecord`
- `SymbolRecord`
- `ExampleRecord`
- `DocRecord`
- `RelationRecord`

The first landed contract for this boundary should stay minimal: a deterministic
`ProjectionInputBundle` emitted directly from `RepositoryAnalysisOutput`, with
stable `ProjectionPageSeed` rows grouped by `Reference`, `HowTo`, `Tutorial`,
and `Explanation`.

The next deterministic refinement should stay graph-native rather than
LLM-driven:

- module reference seeds should aggregate child-symbol docs and examples
- example how-to seeds should carry direct target-side docs and format hints
- doc seeds should carry direct target-side example anchors
- doc page-family classification should treat symbol-targeted docs and
  explicitly reference-like formats as `Reference` instead of collapsing all
  non-tutorial docs into `Explanation`

The next landed contract after seeds should stay equally deterministic:

- `ProjectedPageRecord`
- `ProjectedPageSection`

These records should preserve page family, source anchors, source paths, and a
small stable section set such as `Overview`, `Anchors`, `Sources`, and
`Examples`, without depending on prose generation or Markdown rendering.

The next parser-facing refinement should also stay deterministic:

- `ProjectedMarkdownDocument`
- `ProjectedPageIndexDocument`

These records should render projected pages into stable virtual markdown paths,
then reuse the existing markdown parser so Stage 2 can hand page-index-ready
sections downstream without persisting synthetic files first.

The next landed refinement after that should connect those parsed sections to the
real page-index builder:

- `ProjectedPageIndexTree`
- `ProjectedPageIndexNode`

These records should preserve the actual builder semantics, including heading
hierarchy, structural paths, token counts, and thinning state, while staying
serializable for snapshot and gateway-facing inspection.

The next external-consumption refinement should expose those deterministic
projection contracts through the existing Studio gateway surface:

- `GET /api/repo/projected-pages?repo=<id>`
- `GET /api/repo/projected-page?repo=<id>&page_id=<stable-id>`
- `GET /api/repo/projected-page-index-tree?repo=<id>&page_id=<stable-id>`
- `GET /api/repo/projected-page-index-trees?repo=<id>`

These routes should stay read-only and should return the same deterministic
Stage-2 payloads already pinned in the library snapshot lane instead of adding
gateway-only projection shapes.

The next retrieval refinement on top of those inspection routes should stay
deterministic as well:

- `GET /api/repo/projected-page-index-tree-search?repo=<id>&query=<text>&kind=<family>&limit=<n>`
- `GET /api/repo/projected-page-search?repo=<id>&query=<text>&kind=<family>&limit=<n>`
- `GET /api/repo/projected-retrieval?repo=<id>&query=<text>&kind=<family>&limit=<n>`

These routes should search existing deterministic projected pages and
builder-native projected page-index trees by title, path, format hint,
structural-path, and section-title evidence, with an optional page-family
filter, instead of introducing LLM-ranked retrieval at this stage. The mixed
retrieval route should merge page-level hits and builder-native section hits
under one deterministic contract so downstream Stage-2 consumers do not need to
manually fan out across the separate page and tree-search surfaces.

The next lookup refinement on top of those search and inspection routes should
also stay deterministic:

- `GET /api/repo/projected-retrieval-hit?repo=<id>&page_id=<stable-id>&node_id=<stable-node-id?>`
- `GET /api/repo/projected-retrieval-context?repo=<id>&page_id=<stable-id>&node_id=<stable-node-id?>&related_limit=<n>`
- `GET /api/repo/projected-page-family-context?repo=<id>&page_id=<stable-id>&per_kind_limit=<n>`
- `GET /api/repo/projected-page-family-cluster?repo=<id>&page_id=<stable-id>&kind=<family>&limit=<n>`
- `GET /api/repo/projected-page-index-node?repo=<id>&page_id=<stable-id>&node_id=<stable-node-id>`
- `GET /api/repo/projected-page-index-tree?repo=<id>&page_id=<stable-id>`

These routes should resolve one mixed retrieval hit, one builder-native
projected page-index node, one local mixed-hit context bundle, one grouped
page-family context bundle, one singular page-family cluster, or one full
projected page-index tree directly by stable identifiers instead of forcing
downstream consumers to fetch the full search result set or projected page-index
tree set before opening one specific page, tree, anchor, or family cluster.

The page-family context route should stay deterministic and graph-native:

- resolve one center `ProjectedPageRecord` by stable `page_id`
- rank related projected pages by shared module/symbol/example/doc anchors
- group those related pages by `ProjectionPageKind`
- preserve the shared-anchor score for each related page instead of flattening
  family context into an unannotated list

The next family-discovery refinement should stay equally deterministic:

- `GET /api/repo/projected-page-family-search?repo=<id>&query=<text>&kind=<family>&limit=<n>&per_kind_limit=<n>`

This route should search stable center projected pages first, then expand each
matched center page into grouped family clusters using the same shared-anchor
evidence as `projected-page-family-context`, so downstream Stage-2 consumers
can discover local family neighborhoods without already knowing a stable
`page_id`.

The next singular family-opening refinement should stay equally deterministic:

- `GET /api/repo/projected-page-family-cluster?repo=<id>&page_id=<stable-id>&kind=<family>&limit=<n>`

This route should resolve one stable center `ProjectedPageRecord`, require one
specific `ProjectionPageKind`, and return exactly one grouped family cluster
ranked by the same shared-anchor evidence as the broader family-context and
family-search lanes, so downstream Stage-2 consumers can reopen one family
cluster directly without fetching all families first.

The next page-centric navigation refinement should compose those singular
lookups into one deterministic bundle:

- `GET /api/repo/projected-page-navigation?repo=<id>&page_id=<stable-id>&node_id=<stable-node-id?>&family_kind=<family?>&related_limit=<n>&family_limit=<n>`

This route should return one stable center mixed hit, the full projected
page-index tree for the page, the optional local node neighborhood, related
projected pages ranked by shared anchors, and an optional singular family
cluster when a `family_kind` is requested, so downstream Stage-2 consumers can
open one projected page with its immediate deterministic navigation context in
one call instead of stitching together retrieval-context, tree, and
family-cluster routes manually.

The next navigation-discovery refinement should stay deterministic as well:

- `GET /api/repo/projected-page-navigation-search?repo=<id>&query=<text>&kind=<family>&family_kind=<family?>&limit=<n>&related_limit=<n>&family_limit=<n>`

This route should search stable center projected pages first, then expand each
matched page into the same page-centric navigation bundle used by
`projected-page-navigation`, optionally keeping only hits that can also resolve
the requested `family_kind`. That gives downstream Stage-2 consumers one search
surface that opens page-level context, page-index trees, related pages, and an
optional family cluster without a second round trip per hit.

The first deep-wiki planning surface above those Stage-2 page kernels should
also stay deterministic:

- `GET /api/repo/projected-gap-report?repo=<id>`

This route should expose the analyzer-native `RepoProjectedGapReportResult`
through the existing repo inspection surface so downstream consumers can plan
documentation work from stable coverage gaps before any docs namespace,
materialized wiki corpus, or Qianji-backed generation loop exists.

The initial gap kinds should stay planner-facing and grounded in indexed truth:

- module reference without documentation
- symbol reference without documentation
- symbol reference marked `unverified`
- example how-to without anchors
- documentation page without anchors

The next docs-facing refinement above that repo inspection lane should also stay
deterministic:

- `GET /api/docs/projected-gap-report?repo=<id>`

This route should reuse the same `RepoProjectedGapReportResult` payload instead
of inventing a second gap schema, so the first docs namespace surface starts as
semantic re-framing, not contract duplication.

The next docs-facing discovery refinement above that initial gap surface should
stay equally deterministic:

- `GET /api/docs/search?repo=<id>&query=<text>&kind=<family>&limit=<n>`

This route should reuse the same `RepoProjectedPageSearchResult` payload as the
repo inspection lane instead of creating a second docs-search schema. The docs
namespace should therefore grow by re-framing stable Stage-2 contracts for
planning and navigation, not by forking search behavior before a materialized
wiki corpus exists.

The next docs-facing mixed-retrieval refinement above docs search should also
stay deterministic:

- `GET /api/docs/retrieval?repo=<id>&query=<text>&kind=<family>&limit=<n>`

This route should reuse the same `RepoProjectedRetrievalResult` payload as the
repo inspection lane instead of creating a second docs-retrieval schema. The
docs namespace should therefore be able to search both projected pages and
builder-native projected page-index node hits without leaving the stable
Stage-2 contract family.

The next docs-facing mixed-retrieval context refinement above docs retrieval
should also stay deterministic:

- `GET /api/docs/retrieval-context?repo=<id>&page_id=<stable-id>&node_id=<stable-node-id?>&related_limit=<n>`

This route should reuse the same `RepoProjectedRetrievalContextResult` payload
as the repo inspection lane instead of creating a second
docs-retrieval-context schema. The docs namespace should therefore be able to
reopen one mixed hit with its related projected pages and optional builder-native
node neighborhood without leaving the stable Stage-2 contract family.

The next docs-facing singular mixed-hit refinement above docs retrieval-context
should also stay deterministic:

- `GET /api/docs/retrieval-hit?repo=<id>&page_id=<stable-id>&node_id=<stable-node-id?>`

This route should reuse the same `RepoProjectedRetrievalHitResult` payload as
the repo inspection lane instead of creating a second docs-retrieval-hit
schema. The docs namespace should therefore be able to reopen one stable mixed
hit directly before expanding into page, family, or retrieval-context flows.

The next docs-facing planner refinement above docs retrieval-hit should also
stay deterministic:

- `GET /api/docs/planner-item?repo=<id>&gap_id=<stable-gap-id>&family_kind=<family?>&related_limit=<n>&family_limit=<n>`

This route should compose the existing deterministic `RepoProjectedGapReport`,
`RepoProjectedRetrievalHitResult`, and `RepoProjectedPageNavigationResult`
contracts into one docs-facing work-item opener instead of creating a second
planner-only schema. The docs namespace should therefore be able to reopen one
stable projected gap into its concrete page hit and navigation neighborhood
before any materialized wiki corpus or Qianji-backed generation loop exists.

The next docs-facing planner discovery refinement above planner-item should
also stay deterministic:

- `GET /api/docs/planner-search?repo=<id>&query=<text>&gap_kind=<gap?>&page_kind=<family?>&limit=<n>`

This route should reuse the same projected gap records already emitted by the
deterministic gap report and rank them by explicit title/path/entity/kind
evidence instead of creating a second planner backlog schema. The docs
namespace should therefore be able to discover candidate deep-wiki work items
before opening one concrete gap through `planner-item`.

The next docs-facing planner backlog refinement above planner-search should
also stay deterministic:

- `GET /api/docs/planner-queue?repo=<id>&gap_kind=<gap?>&page_kind=<family?>&per_kind_limit=<n>`

This route should keep reusing the same projected gap records already emitted
by the deterministic gap report, but group them into stable backlog lanes by
projected gap kind instead of inventing a second planner queue entity model.
The docs namespace should therefore be able to shape candidate deep-wiki work
items into deterministic queue groups before opening one concrete gap through
`planner-item`.

The next docs-facing planner ranking refinement above planner-queue should
also stay deterministic:

- `GET /api/docs/planner-rank?repo=<id>&gap_kind=<gap?>&page_kind=<family?>&limit=<n>`

This route should keep reusing the same projected gap records already emitted
by the deterministic gap report, but order them by explicit gap-kind,
page-family, and anchor-density evidence instead of inventing a second planner
ranking entity model. The docs namespace should therefore be able to rank
candidate deep-wiki work items before opening a concrete gap through
`planner-item` or a bounded batch through `planner-workset`.

The next explanation refinement on top of that same deterministic ranking lane
should still avoid schema sprawl:

- `GET /api/docs/planner-rank?repo=<id>&gap_kind=<gap?>&page_kind=<family?>&limit=<n>`

This route should keep the same ranked gap shape, but carry machine-readable
priority reasons alongside the stable score so planners and UIs can explain
why one work item outranks another without recreating hidden ranking logic.

The next docs-facing planner batch-opening refinement above planner-rank
should also stay deterministic:

- `GET /api/docs/planner-workset?repo=<id>&gap_kind=<gap?>&page_kind=<family?>&per_kind_limit=<n>&limit=<n>&family_kind=<family?>&related_limit=<n>&family_limit=<n>`

This route should compose the deterministic planner queue, deterministic
planner-rank selection, and the existing `planner-item` opener instead of
inventing a second batch-work schema. The docs namespace should therefore be
able to keep a filtered queue preview, expose the ranked gap selection chosen
for the workset, group that ranked selection by stable projected gap kind, and
nest those grouped selections by projected page family before reopening the
first stable `N` ranked gaps as concrete planner bundles before any
materialized wiki corpus or Qianji-backed generation loop exists.

The next balancing refinement on top of that same deterministic workset lane
should still avoid schema sprawl:

- `GET /api/docs/planner-workset?repo=<id>&gap_kind=<gap?>&page_kind=<family?>&per_kind_limit=<n>&limit=<n>&family_kind=<family?>&related_limit=<n>&family_limit=<n>`

This route should keep the same queue, rank, group, family, and planner-item
shapes, but add deterministic quota-band evidence for populated gap-kind groups
and populated page-family groups. The docs namespace should therefore be able
to explain batch balance through stable floor/ceiling target counts and
per-group `within_target_band` markers instead of introducing a separate
planner-balancing surface.

The next grouped-quota refinement on top of that same deterministic workset
lane should still avoid schema sprawl:

- `GET /api/docs/planner-workset?repo=<id>&gap_kind=<gap?>&page_kind=<family?>&per_kind_limit=<n>&limit=<n>&family_kind=<family?>&related_limit=<n>&family_limit=<n>`

This route should keep the same queue, rank, balance, family, and planner-item
shapes, but carry explicit `quota` hints on each grouped gap-kind lane and
each nested page-family lane. The docs namespace should therefore be able to
show per-group quota expectations directly where grouped execution happens
instead of forcing planner consumers to reconstruct those hints from the
top-level balance summary.

The next docs-facing opening refinement above docs search should also stay
deterministic:

- `GET /api/docs/page?repo=<id>&page_id=<stable-id>`

This route should reuse the same `RepoProjectedPageResult` payload as the repo
inspection lane, so docs search hits can open one stable projected page without
introducing a second docs-page schema or depending on repo-prefixed
navigation-only consumers.

The next docs-facing structure refinement above docs page should also stay
deterministic:

- `GET /api/docs/page-index-tree?repo=<id>&page_id=<stable-id>`

This route should reuse the same `RepoProjectedPageIndexTreeResult` payload as
the repo inspection lane, so docs consumers can inspect one stable page-index
tree directly without dropping back to repo-prefixed tree routes or inflating
the page payload with an always-on structural tree.

The next docs-facing family refinement above docs page should also stay
deterministic:

- `GET /api/docs/family-context?repo=<id>&page_id=<stable-id>&per_kind_limit=<n>`

This route should reuse the same `RepoProjectedPageFamilyContextResult` payload
as the repo inspection lane, so the docs namespace can open one stable page and
inspect all related page families grouped by shared-anchor evidence without
introducing a docs-only family-context schema.

The next docs-facing family discovery refinement above docs family context
should also stay deterministic:

- `GET /api/docs/family-search?repo=<id>&query=<text>&kind=<family>&limit=<n>&per_kind_limit=<n>`

This route should reuse the same `RepoProjectedPageFamilySearchResult` payload
as the repo inspection lane, so the docs namespace can search stable center
pages and receive grouped family clusters without introducing a docs-only
family-search schema.

The next docs-facing family opening refinement above docs family search should
also stay deterministic:

- `GET /api/docs/family-cluster?repo=<id>&page_id=<stable-id>&kind=<family>&limit=<n>`

This route should reuse the same `RepoProjectedPageFamilyClusterResult` payload
as the repo inspection lane, so the docs namespace can reopen one stable center
page with one required family cluster without introducing a docs-only
family-cluster schema.

The next docs-facing context refinement above docs page should also stay
deterministic:

- `GET /api/docs/navigation?repo=<id>&page_id=<stable-id>&node_id=<stable-node-id?>&family_kind=<family?>&related_limit=<n>&family_limit=<n>`

This route should reuse the same `RepoProjectedPageNavigationResult` payload as
the repo inspection lane, so the docs namespace can open one stable page with
its related pages, projected page-index tree, optional node context, and
optional family cluster without introducing a docs-only navigation schema.

The next docs-facing discovery refinement above docs navigation should also stay
deterministic:

- `GET /api/docs/navigation-search?repo=<id>&query=<text>&kind=<family>&family_kind=<family?>&limit=<n>&related_limit=<n>&family_limit=<n>`

This route should reuse the same `RepoProjectedPageNavigationSearchResult`
payload as the repo inspection lane, so the docs namespace can search stable
center pages and receive full page-centric navigation bundles without
introducing a docs-only navigation-search schema.

The in-process Rust ownership split should stay equally explicit:

- shared query access belongs to `SearchQueryService` under `src/search/queries/`
- docs/page-index capability access belongs to `DocsToolService` under
  `src/analyzers/service/projection/`

That keeps SQL, FlightSQL, GraphQL, REST, and CLI `wendao query ...` scoped to
the shared query system, while `wendao docs ...` and future planner/tool
callers consume the docs capability surface directly instead of SQL-ifying
page-index access or routing same-process calls through gateway adapters.

The same owner path now also exposes repository-scoped markdown TOC/page-index
documents without widening gateway ownership:

- `DocsToolService::get_toc_documents()`
- `wendao docs toc --repo <repo>`
- `wendao.docs.get_toc_documents`

The same owner path now also exposes one stable page-index node opener without
widening gateway ownership:

- `DocsToolService::get_document_node(page_id, node_id)`
- `wendao docs node --repo <repo> --page-id <page-id> --node-id <node-id>`
- `wendao.docs.get_document_node`

The same owner path now also exposes one deterministic structure-search
capability without widening gateway ownership:

- `DocsToolService::search_document_structure(query, kind, limit)`
- `wendao docs search-structure --repo <repo> --query <query> [--kind <kind>] [--limit <n>]`
- `wendao.docs.search_document_structure`

The same owner path now also exposes one lightweight structure opener without
widening gateway ownership:

- `DocsToolService::get_document_structure_outline(page_id)`
- `wendao docs tree-outline --repo <repo> --page-id <page-id>`
- `wendao.docs.get_document_structure_outline`

The same owner path now also exposes one repo-scoped lightweight structure
catalog without widening gateway ownership:

- `DocsToolService::get_document_structure_catalog()`
- `wendao docs structure-catalog --repo <repo>`
- `wendao.docs.get_document_structure_catalog`

The same owner path now also exposes one precise document-segment opener
without widening gateway ownership:

- `DocsToolService::get_document_segment(page_id, line_start, line_end)`
- `wendao docs segment --repo <repo> --page-id <page-id> --line-start <n> --line-end <n>`
- `wendao.docs.get_document_segment`

These pageindex-only capabilities intentionally remain crate-local in this
slice. The gateway still exposes only the 4 core page-opening routes below.
The underlying Markdown TOC parsing owner is now
`xiuxian_wendao_parsers::parse_markdown_toc`; `DocsToolService` remains the
repo-scoped opener over projected page-index documents, nodes, bounded
structure-search candidate generation, token-thinned outline reopening, and
repo-scoped lightweight structure catalog reopening. The new segment opener
also stays on that owner path by reusing projected markdown plus stable
`line_range` coordinates rather than introducing a new parser or gateway
owner.

The first gateway adapter cutover of that rule is now bounded to the same core
4 capability calls:

- `GET /api/docs/page`
- `GET /api/docs/page-index-tree`
- `GET /api/docs/navigation`
- `GET /api/docs/retrieval-context`

These routes now reuse `DocsToolService` while still preserving Studio-owned
registered repository resolution, managed-remote sync permits, plugin-registry
reuse, and in-memory repo state. The gateway adapter therefore does not need a
persisted `wendao.toml` file just to reopen those core docs capability calls
after Studio has already booted.

The crate-local CLI stays aligned to the same bounded surface. The canonical
HTTP to CLI mapping for the 4 core docs openers now lives in
[`207_gateway_openapi_contract_surface.md`](../03_features/207_gateway_openapi_contract_surface.md#core-docs-mapping):

- `GET /api/docs/page` <-> `wendao docs page --repo <repo> --page-id <page-id>`
- `GET /api/docs/page-index-tree` <-> `wendao docs tree --repo <repo> --page-id <page-id>`
- `GET /api/docs/navigation` <-> `wendao docs navigation --repo <repo> --page-id <page-id> [--node-id <node-id>] [--family-kind <kind>] [--related-limit <n>] [--family-limit <n>]`
- `GET /api/docs/retrieval-context` <-> `wendao docs context --repo <repo> --page-id <page-id> [--node-id <node-id>] [--related-limit <n>]`

The same owner path now also serves planner-facing native tools. Wendao ships
`wendao.docs.get_document`, `wendao.docs.get_document_structure`,
`wendao.docs.get_document_segment`,
`wendao.docs.get_document_structure_catalog`,
`wendao.docs.get_document_structure_outline`,
`wendao.docs.get_document_node`, `wendao.docs.get_navigation`,
`wendao.docs.get_retrieval_context`, `wendao.docs.get_toc_documents`, and
`wendao.docs.search_document_structure` as native zhenfa tools backed by the
same crate-local docs runtime surface, with `DocsToolService` as the default
injected owner, so future planner or MCP-style adapters can reuse the docs
capability surface without going through gateway HTTP or folding page-index
access into the shared query system.

The proof surface is now intentionally split:

- `zhenfa_router::native::docs::registry::tests::get_document_tool_returns_serialized_page_payload`
  is a fast wrapper proof that injects a fake docs runtime and validates
  argument handling plus serialized payload shape.
- `docs_tool_service::docs_tool_service_opens_page_tree_search_navigation_node_and_context`
  remains the real docs capability integration proof over the owner service.

The integration fixture now also reuses the shared linked parser-summary test
helper. In that mode, `RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST=1` keeps the
owner-service proof on the real docs capability path while allowing hot reruns
to reuse the process-managed parser-summary service instead of spawning a fresh
linked transport in every cargo process.

### Projection Rules

The initial projection rules should stay deterministic:

- `ReferencePage`
  - sourced from exported modules, symbols, signatures, and direct documentation
- `HowToPage`
  - sourced from minimal runnable examples and task-oriented example clusters
- `TutorialPage`
  - sourced from ordered learning paths, long-form guides, and example sequences
- `ExplanationPage`
  - sourced from conceptual docs, architecture notes, UsersGuide sections, and semantic relation clusters

The first implementation should prefer explicit rules and graph evidence over LLM classification.

## Integration with Existing Wendao Kernels

This is the main architectural purpose of the stage: connect the existing Wendao kernels to the repository documentation layer.

### Page Index

`page index` should become the primary indexing kernel for projected documentation pages.

Responsibilities:

- build hierarchical page structure
- preserve section ancestry
- support section-level navigation and injection
- provide stable page and section identities for projected documentation

In this stage, `page index` is not the source of repository truth. It is the structural index over projected documentation truth.

### Link Graph and PPR

`link_graph` and `PPR` should operate over a richer mixed graph containing:

- repository records
- projected documentation pages
- page sections
- example-to-symbol links
- concept-to-module links

This allows PPR to surface not only structurally central code entities, but also structurally central documentation pages and explanation hubs.

### Weighted Fusion

Existing weighted fusion should become a shared retrieval layer across:

- repository entities
- projected pages
- page sections
- examples
- conceptual explanation nodes

The retrieval target is no longer "document chunks only." It becomes a multi-object retrieval surface.

### Agentic Retrieval

`agentic retrieval` should be positioned as an advanced retrieval mode for:

- cross-page learning path expansion
- architecture walkthroughs
- multi-hop explanation generation
- documentation gap discovery

It should not be required for the baseline projection pipeline.

## SciML and MSL Projection Strategy

### SciML

SciML repositories often split truth across:

- root package metadata
- exported Julia APIs
- external docs sites
- tutorial repositories
- benchmark/example repositories

For SciML, the projection layer should emphasize:

- API-centric `ReferencePage`
- example-driven `HowToPage`
- ecosystem-linked `ExplanationPage`

`TutorialPage` should remain optional until cross-repository tutorial aggregation is stable.

### MSL

MSL already exposes strong documentation conventions:

- `package.mo`
- `UsersGuide`
- `Examples`
- package hierarchy

For MSL, the projection layer should emphasize:

- `ReferencePage` from package and class definitions
- `TutorialPage` and `ExplanationPage` from `UsersGuide`
- `HowToPage` from `Examples`

MSL is therefore a strong early proving ground for deterministic projection.

## Retrieval Enhancement Modes

The stage-2 retrieval layer should expose at least four high-level modes:

- `reference`
- `tutorial`
- `howto`
- `explanation`

These are retrieval filters over projected pages, not ad-hoc prompt labels.

Example CLI shape:

```bash
wendao docs search --repo sciml-diffeq --mode reference --query "solve options"
wendao docs search --repo msl --mode tutorial --query "fluid heat exchanger"
wendao docs page --repo msl --page "Modelica.Fluid.UsersGuide"
```

## Gap Detection and Coverage

Stage 2 should also compute documentation gaps from the difference between Stage 1 truth and projected documentation.

Examples:

- module has exported symbols but no projected reference page
- symbol appears in examples but has no documentation link
- example exists but is not attached to any how-to path
- UsersGuide concept node has no explanation projection

This coverage signal should become a first-class input for future wiki expansion and auditing.

The first deterministic gap slice is now landed inside the analyzer kernel through
`RepoProjectedGapReportQuery -> RepoProjectedGapReportResult`.

The current landed gap kinds are intentionally narrow and planner-facing:

- module reference page without documentation evidence
- symbol reference page without documentation evidence
- symbol reference page marked `unverified`
- example how-to page without stable module/symbol anchors
- documentation-backed projected page without stable module/symbol anchors

This slice stays inside `xiuxian-wendao::analyzers`:

- no new gateway route yet
- no LLM/Qianji refinement
- no materialized wiki corpus

That gives Stage 2 its first deterministic expansion signal without coupling
gap planning to generation.

## Qianji Boundary

Qianji should remain outside the deterministic projection kernel.

Its role in Stage 2 is optional and bounded:

- page refinement
- page classification review
- prose cleanup
- audit and contradiction checks

Qianji must not become the only source of page structure or page truth.

## Execution Phases

1. Build projected page records from Repo Intelligence outputs.
2. Attach `page index` to projected pages and sections.
3. Extend mixed-graph retrieval to include pages and sections.
4. Add mode-aware docs retrieval (`reference`, `tutorial`, `howto`, `explanation`).
5. Add gap detection and coverage reporting.
6. Add optional Qianji-assisted refinement and audit.

## Expected Outcome

After this stage, Wendao should support:

- fast repository understanding through Stage 1 records
- MATLAB-like documentation surfaces through Stage 2 projection
- retrieval that understands both repository structure and page structure
- a clean path toward deep wiki generation without depending on raw LLM improvisation

## Current Validation Status

The deterministic Stage-2 projection and retrieval slice is now operating on a
green Tier-3 lane for the active Wendao plus external Modelica scope:

- `cargo clippy -p xiuxian-wendao -p xiuxian-wendao-modelica --all-targets --all-features -- -D warnings`
- `cargo nextest run -p xiuxian-wendao -p xiuxian-wendao-modelica --no-fail-fast`

That shifts the next bounded step away from more surface expansion and toward
post-gate hygiene and deep-wiki planning:

- remove clearly disposable backup/debris files from `xiuxian-wendao/src/`
- the stale tracked `src/analyzers/service/mod.rs.bak2` monolith is now gone,
  so projection work no longer carries that service-layer refactor artifact
- keep package docs and execution records aligned with the live analyzer and
  Stage-2 projection contracts
- extend the new deterministic docs gap surface into broader docs navigation,
  page opening, or search routes only after the planner-level contract proves
  stable

## Why This Split Matters

Separating this stage from Repo Intelligence MVP prevents three common failures:

- confusing repository truth with projected documentation truth
- overloading the first-stage common core with page-generation concerns
- burying existing Wendao kernels such as page index and PPR under a repo-schema-only narrative

This split keeps Stage 1 focused on repository truth and Stage 2 focused on projected knowledge surfaces and retrieval enhancement.
