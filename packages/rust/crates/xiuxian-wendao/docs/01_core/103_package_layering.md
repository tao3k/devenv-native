# Wendao Package Layering

:PROPERTIES:
:ID: wendao-package-layering
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: architecture, core, runtime, plugins, layering
:STATUS: ACTIVE
:END:

## Purpose

Define the architectural ownership boundary among:

1. `xiuxian-wendao-core`
2. `xiuxian-wendao-runtime`
3. `xiuxian-wendao`

This note is the architectural rule for new code placement. It is not a claim
that the current tree has already completed the migration.

## Layer Definitions

### `xiuxian-wendao-core`

`core` is the stable contract kernel.

It owns:

1. ids and selectors
2. stable request or response record shapes
3. capability, artifact, and transport descriptors
4. plugin-facing traits and contract enums
5. schema and route constants that are contract, not execution

It must not own:

1. filesystem or env resolution
2. Flight/DataFusion client or server execution
3. graph or retrieval algorithms
4. parser implementation
5. storage-facade-backed execution logic

### `xiuxian-wendao-runtime`

`runtime` is the host execution kernel.

It owns:

1. config and settings resolution
2. transport negotiation
3. Arrow Flight client and server wiring
4. DataFusion session bootstrap and runtime query execution glue
5. request metadata decoding and contract materialization
6. plugin registry, loading, and host-side orchestration

It must not own:

1. stable contract ownership that plugins consume directly
2. Wendao graph semantics
3. Wendao retrieval semantics
4. plugin-specific thick implementation

`xiuxian-config-core` is the canonical owner of recursive config imports,
import-path environment expansion, merge precedence, and shared project-root
path resolution primitives such as `PRJ_ROOT` and `PRJ_CONFIG_HOME` joining,
plus generic trimmed env lookup and scalar parse helpers for numeric and bool
runtime overrides. `runtime` and `wendao` may project typed runtime settings
from merged TOML, but they must not reintroduce bespoke TOML line scanners,
duplicate merge logic, or crate-local copies of generic `PRJ_*` path
normalization and env-parse hygiene. The checked-in workspace boot config now
lives at `$PRJ_ROOT/wendao.toml`, while crate-owned embedded defaults remain
under `resources/config/wendao.toml`. That same shared ownership now also
includes precedence-ordered named lookup hygiene for runtime endpoints such as
Valkey URLs: `xiuxian-config-core` owns the generic "first non-empty key +
trimmed value" helper, while Wendao owners keep the domain decision about
whether a missing or invalid target is optional, startup-blocking, or
feature-disabling. Wendao-owned dotted key names stay with the owning
subsystem, for example `analyzers.cache.*`, `graph.persistence.*`, and
`storage.knowledge.*`. Runtime-owned dotted keys such as `link_graph.cache.*`
follow the same rule: the typed projection stays in `xiuxian-wendao-runtime`,
but TOML-first named lookup and generic env hygiene belong in
`xiuxian-config-core`. These owners should resolve through config-core helpers
instead of bespoke local precedence code.

### `xiuxian-wendao`

`wendao` is the domain kernel.

It owns:

1. `link_graph`
2. graph algorithms, traversal, PPR, saliency, and relation semantics
3. parser implementation for general Wendao document or code understanding
4. search, retrieval, fusion, and storage semantics
5. domain retrieval behavior backed by the storage facade
6. business-domain services and transitional compatibility seams

It must not become the long-term owner of:

1. new stable plugin contracts
2. new generic runtime helpers
3. plugin-specific thick implementation that can live in its own crate

## Core Rule

Do not classify code by how important it feels.

Classify it by which kind of ownership it requires:

1. stable contract ownership -> `core`
2. host runtime ownership -> `runtime`
3. Wendao domain ownership -> `wendao`

## Data Layer Interpretation

The same data-plane stack splits across layers.

### Arrow Flight

- Flight contract records and route constants -> `core`
- Flight server or client execution and negotiation -> `runtime`
- Flight-backed business semantics -> `wendao` or a plugin crate

### DataFusion

- query contract shape -> `core`
- session bootstrap and query execution glue -> `runtime`
- Wendao query semantics and business planning -> `wendao`

### Lance Vector-Store Facade

If a component depends on Lance vector-store execution semantics, it is no
longer a pure contract. Active Wendao callers should use `xiuxian-db-store` as
the storage facade instead of depending on the retiring `xiuxian-vector` crate
directly.

That code belongs in:

1. `wendao` when it is domain retrieval logic
2. `runtime` when it is generic host wiring

It does not belong in `core`.

## Link Graph And Parser Rule

`link_graph` and the general Wendao parser stack are domain core, not contract
core.

They belong in `xiuxian-wendao` because they define how Wendao understands and
retrieves knowledge.

The canonical implementation home for parser families is the crate-root
`src/parsers/{cargo,graph,markdown,link_graph,search,zhixing,...}` namespace.
`link_graph`, `dependency_indexer`, `skill_runtime`, and other subsystems may
consume parser services, but they do not own parallel parser namespaces.
That canonical parser stack also owns semantic markdown frontmatter parsing,
the `NoteFrontmatter` contract consumed by enhancement and skill-discovery
flows, and the link-graph search-query parser now implemented under
`src/parsers/link_graph/query/`, repo-code search query parsing now
implemented under `src/parsers/search/repo_code_query/`, graph persistence
dict parsing now implemented under `src/parsers/graph/persistence/`, plus
Cargo.toml dependency parsing now implemented under
`src/parsers/cargo/dependencies/`. That same parser stack now also owns
explicit markdown property-drawer relation parsing under
`src/parsers/markdown/relations/`, where section-scoped relation targets are
parsed separately from ordinary global wiki links.

Remaining parse-like helpers outside `src/parsers/` stay local by design.
`search/queries/graphql/document.rs` is adapter-local GraphQL request parsing,
`gateway/studio/router/handlers/repo/parse.rs` is gateway-local request
validation, and helpers such as analyzer config parsing, search-plane hydrate
decode, and pybinding refresh JSON parsing remain subsystem-local utilities
rather than standalone parser families. Likewise, `entity/query.rs`,
`storage/query.rs`, and similar `query.rs` modules are query models or
execution surfaces, not parser ownership gaps. See the dedicated parser docs
under [../02_parser/index.md](../02_parser/index.md).

Within that parser stack, ordinary `[[...]]` links establish graph topology
first. Typed relation semantics should come from explicit metadata owners such
as property drawers or subsystem-owned metadata, not from parser-side
hardcoded string matches on wiki-link text. Property-drawer relation values
and scalar values are separate contracts: scoped relation fields add explicit
semantic edges, while numeric or other scalar properties stay local metadata
unless a subsystem explicitly owns them. See the parser docs under
[../02_parser/index.md](../02_parser/index.md) and
[../02_parser/relation_semantics.md](../02_parser/relation_semantics.md).

For ordinary body references, Wendao now follows one shared Markdown reference
contract: `[label](path)`, `[label](path#Heading)`, `[[note]]`,
`[[note#Heading]]`, and `[[#Heading]]` are structural targets plus optional
addresses, not semantic type suffixes. The parser-owned implementation for
this surface now lives under `src/parsers/markdown/references/`. The narrower
ordinary-wikilink subset still lives under `src/parsers/markdown/wikilinks/`
for consumers that only need `[[...]]` topology links. `link_graph_refs` and
docs-governance consume that wikilink subset instead of owning local scanners,
while `skill_runtime::manifest::authority` now consumes the shared
reference parser so `SKILL.md` ordinary Markdown links and ordinary wikilinks
follow the same parser-owned contract. The parser and enhancer PyO3 wrappers
for these surfaces were retired so the Rust contract can evolve directly
without duplicate binding compatibility work.

`skill_runtime` is now tightening around a clearer inventory/resolver split.
Parser-owned code discovers canonical `SKILL.md` documents, while the
runtime-side inventory layer consumes that discovery surface and preloads
semantic mounts. Its runtime path stays intentionally lenient: missing
frontmatter still falls back to the skill directory name, while invalid YAML
fails resolver bootstrap. The stricter lint-only `metadata:` requirement
remains a separate parser-owned authoring contract and is not imposed on
runtime resolver indexing. The older `skill_runtime::index` naming is now treated
as a compatibility alias, not the architectural target.

The skill manifest data model is now also Wendao-owned for the local runtime
path: manifest loading, workflow-type parsing, manifest scans,
authority reporting, native-alias compilation, and schema resources all live
under Wendao-owned `skill_runtime` / `resources` surfaces. Daochang consumes the
compiled native-alias contract from Wendao rather than importing a separate
skills crate.

Only their stable plugin-facing contracts should move to `core`.

## Gateway Rule

`gateway` is an adapter layer, not the primary home of domain behavior.

Its long-term role is:

1. decode protocol input
2. validate contract metadata
3. dispatch into runtime or domain services
4. encode protocol output

Therefore:

- thin Arrow Flight/DataFusion contract dispatch is acceptable at the gateway
  boundary
- thick search, graph, parser, and plugin business logic should live below the
  gateway boundary

## Plugin Rule

A plugin crate should own as much plugin-specific implementation as possible.

The host should prefer:

1. add dependency
2. register capability
3. compile and load plugin

The host should avoid:

1. adding new plugin-specific business modules to `xiuxian-wendao`
2. hard-coding plugin-specific parser or launch behavior in the host crate

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/412_core_runtime_plugin_program|Wendao Core Runtime Plugin Program]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/417_wendao_package_boundary_matrix|Wendao Package Boundary Matrix]]
:END:

---

:FOOTER:
:AUDITOR: codex
:END:
