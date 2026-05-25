# M4 Julia Externalization Package List

:PROPERTIES:
:ID: wendao-m4-julia-externalization-package-list
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, migration, plugins, julia, m4
:STATUS: ACTIVE
:END:

## Purpose

This note is the first concrete `M4` deliverable for the Wendao
core/runtime/plugin migration program.

It defines the package list for externalizing Julia ownership into
`xiuxian-julia-core`.

Primary references:

- `[[06_roadmap/412_core_runtime_plugin_program]]`
- `[[06_roadmap/413_m2_core_extraction_package_list]]`
- `[[06_roadmap/414_m3_runtime_extraction_package_list]]`
- `[[06_roadmap/409_core_runtime_plugin_surface_inventory]]`
- `[[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md]]`

The active task-scoped blueprint for this migration remains recorded in the
paired ExecPlan rather than this persistent package-list note.

## M4 Goal

Make `xiuxian-julia-core` the physical owner of Julia-specific plugin
behavior.

At the end of `M4`, the host should consume Julia through:

1. stable contracts from `core`
2. orchestration from `runtime`
3. Julia-owned plugin declarations and launch/artifact semantics from
   `xiuxian-julia-core`

## Externalization Rule

A Julia-related boundary belongs in `xiuxian-julia-core` if it is true that:

1. it expresses Julia-specific capability behavior
2. it encodes Julia-specific launch or CLI semantics
3. it owns Julia-specific deployment artifact meaning
4. it exists only to bridge Wendao contracts into Julia package behavior

If a boundary is only a stable contract record, it should move to `core`.
If it is only host orchestration, it should move to `runtime`.

## Current Blocking Reality

The first dependency blocker has now been removed:

1. `xiuxian-julia-core` no longer depends on `xiuxian-wendao` directly
2. repo-intelligence contracts and the Julia Arrow analyzer
   column/schema-contract surface now come from `xiuxian-wendao-core`
3. the former `M4` blockers are no longer Cargo-edge blockers; they have been
   reduced and then externalized as Julia-owned compatibility seams for launch
   semantics, deployment artifact meaning, package-path/default ownership, and
   the Julia rerank runtime record
4. the host now consumes Julia through a normal crate dependency instead of
   sibling-source inclusion under `src/analyzers/languages/mod.rs`

The remaining Julia work has now shifted from ownership externalization to
compatibility retirement and generic-surface cutover.

## Candidate Package List

### Package A: Julia Capability Declarations

Current source boundary:

- Julia plugin entry and capability declaration code
- Julia capability metadata that is currently bridged through the host crate

Target `xiuxian-julia-core` ownership:

1. Julia capability identifiers and declarations
2. Julia provider metadata
3. Julia-owned capability registration surfaces

Do not keep in host:

1. Julia-specific capability declaration assembly
2. Julia plugin entrypoint wiring as host-owned implementation

### Package B: Julia Launch and Manifest Semantics

Current source boundary:

- Julia launch-manifest semantics currently represented by
  `LinkGraphJuliaAnalyzerLaunchManifest`
- Julia service descriptor assembly
- Julia CLI argument meaning

Target `xiuxian-julia-core` ownership:

1. Julia launch manifest schema interpretation
2. Julia CLI ordering and package-owned defaults
3. Julia launcher path and package-owned startup semantics

Do not keep in host:

1. Julia-specific ordered CLI argument assembly as primary ownership
2. Julia service descriptor semantics as host-owned meaning

### Package C: Julia Deployment Artifact Semantics

Current source boundary:

- Julia deployment artifact meaning currently surfaced through
  `LinkGraphJuliaDeploymentArtifact`
- Julia-specific TOML/JSON deployment contract meaning

Target `xiuxian-julia-core` ownership:

1. Julia deployment artifact schema interpretation
2. Julia artifact metadata fields and package-owned defaults
3. Julia package-facing export contract

Host may still own:

1. generic artifact payload record shape
2. generic artifact resolution/routing
3. compatibility shims during migration

### Package D: Julia Runtime Defaults and Package Paths

Current source boundary:

- Julia-specific env-var defaults
- `.data/WendaoArrow.jl` and `.data/WendaoAnalyzer.jl` package path conventions
- Julia launch defaults that are currently host-side

Target `xiuxian-julia-core` ownership:

1. Julia package path conventions
2. Julia default launcher and script locations
3. Julia package-owned transport defaults where not part of generic contracts

Do not keep in host:

1. package-owned Julia script names as primary host constants
2. Julia package path knowledge as long-term host ownership

### Package E: Julia Compatibility Surface

Current source boundary:

- legacy Julia outward shims in gateway and Zhenfa compatibility seams
- historical note: the temporary crate-root shim `src/compatibility/julia.rs`
  was used during the `M4 -> M5` bridge and is now retired from the live tree

Target ownership after `M4`:

1. host keeps only thin compatibility wrappers at the explicit crate-root
   compatibility namespace, which is now `src/compatibility/link_graph.rs`
2. Julia package becomes the physical owner of Julia-specific meaning
3. all compatibility wrappers delegate into `core`/`runtime` + Julia plugin
   ownership

## Explicit Non-Julia-Package List

These boundaries must remain out of `xiuxian-julia-core`:

### Core-Owned

1. plugin ids and selector record shapes
2. capability and artifact descriptor records
3. generic launch-spec and artifact-payload records

### Runtime-Owned

1. host config discovery and filesystem override resolution
2. transport client construction and fallback orchestration
3. gateway assembly
4. Zhenfa router execution
5. registry/bootstrap orchestration

## Dependency Rewrite Target

The desired dependency shape after `M4` is:

```text
xiuxian-julia-core
  -> xiuxian-wendao-core
  -> optional narrow runtime integration seam if unavoidable

xiuxian-wendao-runtime
  -> xiuxian-wendao-core
  -> xiuxian-julia-core

xiuxian-wendao
  -> facade / compatibility / transitional assembly
```

The key rule is:

`xiuxian-julia-core` must stop depending on the monolithic host crate as its
primary dependency surface.

## First Physical Externalization Cut

The first physical `M4` cut should aim to move:

1. Julia launch-manifest meaning
2. Julia deployment artifact meaning
3. Julia package path/default ownership

It should not attempt to remove every Julia compatibility shim in one landing.

Current implementation status:

1. `xiuxian-julia-core` now has a direct dependency on
   `xiuxian-wendao-core`
2. repo-intelligence contract imports in the Julia plugin entry, discovery,
   linking, project, sources, and transport modules now source stable records
   and traits from `xiuxian-wendao-core::repo_intelligence`
3. the monolithic host analyzer contract modules now re-export that same
   repo-intelligence slice from `xiuxian-wendao-core`, so the Julia package no
   longer depends on `xiuxian-wendao` for those stable contracts
4. the Julia Arrow analyzer column/schema contract also now lives in
   `xiuxian-wendao-core::repo_intelligence`, which removes the last direct
   `xiuxian-wendao` Cargo dependency from the Julia package
5. `xiuxian-wendao` now loads `xiuxian-julia-core` through a normal crate
   dependency instead of `#[path]` source inclusion, so Julia publication is
   no longer blocked by host-side source embedding
6. `xiuxian-julia-core::compatibility::link_graph` now owns the Julia
   plugin selector ids/helpers, `LinkGraphJuliaAnalyzerServiceDescriptor`,
   `LinkGraphJuliaAnalyzerLaunchManifest`,
   `LinkGraphJuliaDeploymentArtifact`, the Julia CLI-arg mapping for analyzer
   launch, and the conversion boundary between those Julia DTOs and
   `PluginLaunchSpec` / `PluginArtifactPayload`
7. the monolithic host now keeps `launch.rs` and `artifact.rs` only as
   compatibility re-export seams for those Julia-owned DTOs, while
   `runtime.rs` now delegates Julia analyzer-launch arg encoding back into the
   Julia crate
8. `xiuxian-julia-core::compatibility::link_graph` now also owns the Julia
   analyzer package-dir/default path slice through `paths.rs`, including the
   default analyzer launcher path and the default analyzer example-config path,
   so the monolithic host no longer carries those package-owned defaults in
   `runtime_config/constants.rs`
9. the host runtime/tests and integration fixtures now consume those
   Julia-owned path defaults instead of embedding raw
   `.data/WendaoAnalyzer.jl/...` or `.data/WendaoArrow.jl/...` literals
   across the touched `M4` seams
10. `xiuxian-julia-core::compatibility::link_graph` now also owns
    `LinkGraphJuliaRerankRuntimeConfig` and its provider-binding / launch /
    artifact normalization methods through `runtime.rs`
11. the host `runtime.rs` and `conversions.rs` files now behave as
    compatibility seams over that Julia-owned runtime record, so the hard
    ownership blockers for `M4` are now cleared
12. the staged mixed-graph structural plugin contract now also follows the
    same ownership rule: Julia-specific graph-structural route names, draft
    schema-version defaults, request or response column inventories, and Arrow
    batch validation live in `xiuxian-julia-core`, while
    `xiuxian-wendao-runtime` stays limited to reusable Flight client and route
    normalization helpers
13. the next graph-search dispatch layer now follows that same rule too:
    Julia-specific repository option parsing for `graph_structural_transport`,
    graph-structural route-kind defaults, and request or response dispatch
    helpers live in `xiuxian-julia-core` instead of being reintroduced into
    `xiuxian-wendao-runtime`
14. the typed graph-search exchange surface now follows that rule as well:
    request-row structs, response-row structs, Arrow batch builders, Arrow
    batch decoders, and repository-scoped fetch helpers for structural rerank
    or constraint filter live in `xiuxian-julia-core`, while
    `xiuxian-wendao` keeps at most a thin plugin consumption seam
15. the semantic projection layer above those row types now also follows the
    same rule: normalized query-anchor DTOs, candidate-subgraph DTOs, rerank
    signal DTOs, and filter-constraint DTOs for graph-structural requests live
    in `xiuxian-julia-core` so the host does not have to manually align
    request-list columns
16. the current host-side proof also follows that rule: a real
    `LinkGraphIndex` agentic-expansion pair is projected into
    Julia-owned graph-structural DTOs and a validated request batch from a
    test-only consumption seam, without adding a second production adapter to
    `xiuxian-wendao`
17. the pair-specific projection helpers above that proof now also follow the
    same rule: stable pair candidate id normalization, pair candidate-subgraph
    construction, and pair-to-request-row projection live in
    `xiuxian-julia-core`, so the host no longer rebuilds two-node candidate
    ids or candidate-subgraph wrappers by hand
18. the next simple request-semantics layer also follows that same rule:
    keyword-or-tag query-context builders and binary keyword-or-tag rerank
    signal builders live in `xiuxian-julia-core`, so the host no longer
    manually constructs those anchors or maps boolean plane matches to staged
    score columns
19. the convenience layer above those helpers now also follows that same rule:
    combined keyword-or-tag pair-rerank request-row builders live in
    `xiuxian-julia-core`, so the host no longer manually composes
    `query context -> rerank signals -> pair rerank row` in sequence
20. the shared-tag overlap discovery step now also follows that same rule:
    normalized shared-tag anchor extraction and overlap-aware combined
    pair-rerank helpers live in `xiuxian-julia-core`, so the host no longer
    computes tag overlap before calling the plugin-owned request builder
21. the next metadata projection seam now also follows that same rule:
    plugin-owned node-metadata input bundles and a metadata-aware overlap
    helper live in `xiuxian-julia-core`, so the host no longer threads raw
    tag vectors directly into the staged request-row builder
22. the next row-to-batch assembly seam now also follows that same rule:
    scored metadata-aware rerank input bundles and a metadata-aware rerank
    batch helper live in `xiuxian-julia-core`, so the host no longer builds
    `Vec<GraphStructuralRerankRequestRow>` before Arrow batch materialization
23. the next higher-level candidate-input seam now also follows that same
    rule: single-bundle keyword-overlap request inputs and a candidate-input
    batch helper live in `xiuxian-julia-core`, so the host no longer
    composes query-input, metadata-input, pair-input, and scored-rerank-input
    bundles by hand for each pair
24. the next shared-query and candidate-bundle seam now also follows that
    same rule: one shared keyword-overlap query bundle, one plugin-owned
    per-pair candidate bundle, and the query-plus-candidate batch helper live
    in
    `xiuxian-julia-core`, so the host no longer constructs higher-level
    request-input bundles by hand before staging the Arrow batch
25. the next repository-fetch seam now also follows that same rule: the
    query-plus-candidate rerank fetch helper lives in
    `xiuxian-julia-core`, so a host caller with those plugin-owned DTOs no
    longer needs to materialize Arrow batches before dispatching the
    repository-configured structural-rerank request, and the host only
    re-exports that helper through its thin language seam; the bounded host
    proof now calls that public helper directly instead of stopping at batch
    projection, and it imports the graph-structural surface through
    `xiuxian_wendao::analyzers::languages`
26. the next raw-to-candidate staging seam now also follows that same rule:
    `build_graph_structural_keyword_overlap_candidate_inputs(...)` lives in
    `xiuxian-julia-core`, so the host no longer manually constructs
    `GraphStructuralNodeMetadataInputs`,
    `GraphStructuralKeywordOverlapCandidateInputs` before calling the
    plugin-owned request-batch or repository-fetch helpers
27. the next raw-to-query staging seam now also follows that same rule:
    `build_graph_structural_keyword_overlap_query_inputs(...)` lives in
    `xiuxian-julia-core`, so the host no longer manually constructs
    `GraphStructuralKeywordOverlapQueryInputs` before calling the plugin-owned
    request-batch or repository-fetch helpers
28. the next raw-to-pair staging seam now also follows that same rule:
    `build_graph_structural_pair_candidate_inputs(...)` lives in
    `xiuxian-julia-core`, so the host no longer manually constructs
    `GraphStructuralPairCandidateInputs` before calling the plugin-owned
    request-batch or repository-fetch helpers
29. the next raw pair-metadata-to-candidate staging seam now also follows
    that same rule:
    `build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw(...)`
    lives in `xiuxian-julia-core`, so the host no longer manually composes
    `build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(...)`
    and `build_graph_structural_keyword_overlap_candidate_inputs(...)` before
    calling the plugin-owned request-batch or repository-fetch helpers
30. the next raw-candidate collection batch or fetch seam now also follows
    that same rule:
    `GraphStructuralKeywordOverlapRawCandidateInputs`,
    `build_graph_structural_keyword_overlap_raw_candidate_inputs(...)`,
    `build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates(...)`,
    and
    `fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates(...)`
    live in `xiuxian-julia-core`, so the host no longer manually normalizes
    each raw candidate before calling the plugin-owned request-batch or
    repository-fetch helpers
31. the Julia plugin capability-discovery seam now also follows that same
    rule:
    the Rust host keeps only static plugin identity registration, while
    Julia-specific capability discovery, manifest decoding, manifest-to-binding
    conversion, and repository transport interpretation for a
    `/plugin/capabilities` Arrow route now live in `xiuxian-julia-core`
    instead of growing a second host-owned registration adapter
32. the bounded host live-smoke slice now also follows that same rule:
    `xiuxian-wendao` consumes the existing Julia-owned graph-structural fetch
    helper through `xiuxian_wendao::analyzers::languages` against the live
    `WendaoSearch.jl --mode solver_demo` service, both for explicit transport
    config and for manifest-discovery fallback, rather than adding a second
    host-local structural transport path
33. the bounded host generic-topology proof now also keeps planner-aware
    connected-pair collection extraction inside dedicated host test-support,
    so `tests/unit/link_graph_agentic/expansion.rs` no longer carries that
    collection-selection algorithm inline while the live solver-demo downcall
    contract remains unchanged
34. the next host generic-topology live-harness layer now also keeps
    manifest-discovery repository setup, shared query-context setup, and
    baseline solver-demo row assertions inside dedicated host test-support, so
    `tests/unit/link_graph_agentic/expansion.rs` keeps only test intent plus
    pin-specific assertions while the Julia-owned fetch seam remains unchanged
35. the next host mixed-graph promotion step now also derives one seed-centered
    generic-topology batch from a real `LinkGraphAgenticExpansionPlan`, so the
    host proof now exercises one more realistic plan-derived candidate batch
    above connected-pair collections without changing the Julia-owned live
    contract
36. that same host live lane now also derives one worker-partition
    generic-topology batch from real `LinkGraphAgenticWorkerPlan` partitions,
    so the current solver-demo route now covers one more planner-shaped
    candidate batch above seed-centered groups while accepting mixed feasible
    and infeasible solver rows inside the same returned batch
37. the next bounded host proof now also derives one batch-level
    generic-topology query context from the real expansion-plan query plus
    selected worker seed metadata, so the manifest-discovered solver-demo
    downcall no longer hard-codes `"alpha"` or `"related"` inside the host
    helper layer
38. that same host-through-language-seam live lane now also derives
    worker-batch dependency, keyword, and tag scores from real plan-aware
    batch semantics and validates those staged request-batch columns before the
    live solver-demo downcall, so the host proof is less synthetic without
    moving planner ranking semantics into `xiuxian-julia-core`
39. that same host-through-language-seam live lane now also validates the
    staged `semantic_score` request column derived from real worker-partition
    pair semantics before the live solver-demo downcall, so the outgoing Arrow
    batch is proven above one less implicit Julia-owned normalization step
    without changing the Julia contract
40. that same host-through-language-seam live lane now also validates the
    staged `query_id`, `retrieval_layer`, `query_max_layers`,
    `anchor_planes`, `anchor_values`, and `edge_constraint_kinds` request
    columns against the same plan-aware batch fixture before the live
    solver-demo downcall, so the outgoing Arrow batch is proven above one less
    implicit host-to-Julia query-context handoff without changing the Julia
    contract
41. that same host-through-language-seam live lane now also validates the
    staged `candidate_node_ids`, `candidate_edge_sources`,
    `candidate_edge_destinations`, and `candidate_edge_kinds` request columns
    against the same plan-aware batch fixture before the live solver-demo
    downcall, so the outgoing Arrow batch is proven above one less implicit
    host-to-Julia topology handoff without changing the Julia contract
42. that same host-through-language-seam live lane now also proves one
    plan-aware worker-partition generic-topology `constraint_filter` batch
    above the same raw connected-pair collection seam, and it now validates
    the staged `constraint_kind` and `required_boundary_size` request columns
    before reusing that batch against the manifest-discovered
    `WendaoSearch.jl --mode solver_demo` filter route without changing the
    Julia contract
43. the paired Julia-plugin live lane now also proves one multi-candidate
    generic-topology `constraint_filter` batch against that same manifest-
    discovered `WendaoSearch.jl --mode solver_demo` multi-route endpoint, and
    the real Julia service tests are now serialized with a shared file lock so
    the default Rust graph-structural exchange suite remains stable without
    changing the Julia contract
44. the bounded Julia-plugin clippy frontier is now also closed in the active
    live lane: capability-manifest response validation is split into row-scoped
    helpers, generic-topology scored-pair normalization no longer uses a
    precision-loss cast, and the topology-subgraph builder now satisfies the
    `missing_errors_doc` gate without relaxing `-D warnings`
45. the host generic-topology live harness now also derives fallback edge
    labels and staged `edge_constraint_kinds` from the normalized Wendao
    agentic execution relation, so the manifest-discovered solver-demo lane no
    longer hard-codes a placeholder `"related"` edge semantic in host support
46. that same host-through-language-seam filter lane now also derives the
    staged `required_boundary_size` from plan-aware anchor and candidate-
    topology semantics, and it validates filter-side anchor and topology list
    columns before the same manifest-discovered solver-demo downcall without
    changing the Julia contract
47. that same host-through-language-seam filter lane now also derives the
    staged `constraint_kind` from the same plan-aware batch shape, and the
    paired Julia-plugin live proof now exercises the non-default
    `boundary_match` filter mode against the same solver-demo multi-route
    endpoint without changing the Julia contract

## Compatibility Plan

During `M4`:

1. deprecated Julia-named host exports may remain
2. host compatibility seams may remain, but only as wrappers
3. Julia package should increasingly become the source of truth for Julia
   behavior

`M4` is complete only when Julia-specific meaning lives physically outside the
monolithic crate.

## Acceptance Criteria

This package list is ready when:

1. Julia-owned boundaries are explicitly identified
2. non-Julia-owned boundaries are explicitly excluded
3. the dependency rewrite target is clear
4. the first externalization cut is intentionally narrow and executable

## Immediate Follow-Up

After this note lands, the next program move should be:

1. treat `M4` ownership externalization as functionally satisfied
2. move to `M5` generic artifact cutover and compatibility retirement
3. keep Julia-named outward surfaces as wrappers only while generic plugin
   artifact surfaces become canonical

:RELATIONS:
:LINKS: [[index|Wendao DocOS Kernel: Map of Content]], [[06_roadmap/412_core_runtime_plugin_program|Wendao Core Runtime Plugin Program]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md|RFC: Wendao Core Runtime and Arrow Plugin Migration]]
:END:

---
