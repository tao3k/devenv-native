# Wendao Testing Gate Post-Harness Curation

## Context

`xiuxian-wendao` already mounts the shared crate test-policy gate through both
`tests/unit_test.rs` and `tests/xiuxian-testing-gate.rs`, so the current
problem is not missing harness wiring.

The real debt is post-harness growth inside `tests/unit/...`: large suites were
externalized out of `src/`, but many of them kept expanding into multi-thousand
line files after the harness migration.

## Current Inventory

An approximate filesystem-level scan against the current shared thresholds
(`effective_code_lines >= 260` and `test-ish attr count >= 8`) shows that
Wendao still has a large unit-leaf backlog, including:

- `tests/unit/studio_repo_sync_api/mod.rs`
- `tests/unit/repo_index/state/runtime.rs`
- `tests/unit/gateway/studio/search.rs`
- `tests/unit/gateway/studio/graph.rs`
- `tests/unit/gateway/studio/search/handlers/flight/repo_search.rs`
- `tests/unit/gateway/studio/router/state/mod.rs`
- `tests/unit/zhenfa_router/native/audit/audit_bridge.rs`
- `tests/unit/link_graph_agentic/expansion.rs`
- `tests/unit/gateway/studio/search/handlers/code_search/plugin_routes.rs`
- `tests/unit/repo_index/state/sync.rs`
- `tests/unit/gateway/studio/router/config.rs`
- `tests/unit/query_core/mod.rs`
- `tests/unit/gateway/studio/router/handlers/repo/analysis/search/mod.rs`
- `tests/unit/gateway/studio/router/code_ast/native_routes.rs`
- `tests/unit/search/cache/mod.rs`
- `tests/unit/search/service/publication/search.rs`

## Rule Interpretation

The shared policy is already present; it simply has not been brought back to a
green Wendao state. Historical remediation notes already recorded the
expectation that `cargo test -p xiuxian-wendao --test unit_test
enforce_crate_test_policy_harness -- --exact` would keep failing until the
remaining oversized suites were split.

In other words:

- the policy exists
- the crate mounts the policy
- the crate still carries grandfathered post-harness leaf debt

## Current Gate Result

The lightweight shared gate now confirms the exact current failure shape:

- `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`

That run currently fails with:

- 24 bloated `tests/unit/...` leaves

After the latest link-graph runtime-config split, the same lightweight gate now
fails with:

- 21 bloated `tests/unit/...` leaves

So the large test files are not slipping past the policy; they are the only
remaining active shared-gate debt in this crate.

## Next Slice Shape

Wendao remediation should stay owner-bounded instead of attempting one
crate-wide rewrite. The first bounded slice should start from the search-owned
unit suites that are already adjacent to current search/build work.

The first bounded split is now complete:

- `tests/unit/search/local_symbol/build/mod.rs` is no longer a single
  oversized leaf
- the suite now uses a folder-first root with focused
  `support.rs`, `fingerprint.rs`, `planning.rs`, `markdown.rs`, and
  `publication.rs`
- the focused proof
  `cargo test -p xiuxian-wendao --lib search::local_symbol::build::tests:: -- --nocapture`
  now passes with 8 green tests
- rerunning the lightweight shared `unit_test` harness confirms the bloated
  unit-leaf debt dropped from 27 to 26 while the 6 inline `src/` policy
  issues stayed unchanged
- `tests/unit/search/cache/mod.rs` is also no longer a single oversized leaf;
  it now stays source-mounted as a small root with focused `support.rs`,
  `keys.rs`, `shadow.rs`, `file_fingerprints.rs`, and
  `repo_publication.rs` children
- `tests/unit/search/service/publication/search.rs` is now a folder-first
  `search/` suite with focused `entity.rs`, `content.rs`, `duckdb.rs`, and
  `support.rs` children
- focused non-duckdb proofs now pass for both new splits:
  `search::cache::tests::` is 10 green and
  `search::service::tests::publication::search::` is 6 green
- rerunning the lightweight shared `unit_test` harness confirms the bloated
  unit-leaf debt dropped again from 26 to 25
- a focused `duckdb` proof for the new `publication/search` folder layout was
  attempted twice, but both local sessions aborted before Rust test output
  appeared, so feature-on confirmation remains open
- the formerly surfaced inline `src/` policy offenders are now cleared:
  `src/repo_index/perf_support.rs` mounts
  `tests/unit/repo_index/perf_support.rs`,
  `src/parsers/mod.rs` no longer hides real parser modules behind `test` cfgs,
  and
  `src/analyzers/service/projection/docs_tool/{segment,service,contracts}.rs`
  now mount focused leaves under
  `tests/unit/analyzers/service/projection/docs_tool/`
- the docs-tool seam also tightened its runtime boundary:
  `DocsToolRuntime` and `DocsToolRuntimeHandle` are back to feature-owned
  `zhenfa-router` wiring instead of `test`-driven module exposure
- focused proofs now cover the docs-tool contract/service/segment leaves,
  default `cargo check -p xiuxian-wendao --tests`, and one feature-on native
  docs consumer run under `--features "zhenfa-router julia"`
- `tests/unit/analyzers/cache/mod.rs` is now also a source-mounted small root
  with focused `support.rs`, `key_normalization.rs`, `julia.rs`,
  `modelica.rs`, `mixed_julia_modelica.rs`, `mixed_modelica_rust.rs`,
  `mixed_unknown.rs`, `rust.rs`, and `storage.rs` leaves
- focused
  `cargo test -p xiuxian-wendao --lib analyzers::cache::tests:: -- --nocapture`
  now passes with 21 green tests
- rerunning the lightweight shared `unit_test` harness confirms the bloated
  unit-leaf debt dropped again from 25 to 24
- `tests/unit/analyzers/service/search/mod.rs` is now a small TOC root with
  focused `support.rs`, `fuzzy.rs`, `snapshot.rs`, and `artifacts.rs` leaves
- focused
  `cargo test -p xiuxian-wendao --lib analyzers::service::search::tests:: -- --nocapture`
  now passes with 8 green tests
- `tests/unit/analyzers/service/mod.rs` is now also a source-mounted small root
  with focused `support.rs`, `core.rs`, `target_file.rs`, `julia_cache.rs`,
  `rust_cache.rs`, and `mixed_cache.rs` leaves
- focused
  `cargo test -p xiuxian-wendao --lib analyzers::service::tests:: -- --nocapture`
  now passes with 14 green tests
- touched-scope `cargo check -p xiuxian-wendao --tests` remains green after the
  analyzers/service splits
- rerunning the lightweight shared `unit_test` harness confirms the bloated
  unit-leaf debt dropped again from 24 to 22
- `tests/unit/link_graph/runtime_config.rs` is now a folder-first
  `tests/unit/link_graph/runtime_config/` suite with a small `mod.rs` root and
  focused `support.rs`, `coactivation.rs`, `agentic.rs`, `retrieval.rs`,
  `julia_rerank.rs`, and `artifacts.rs` leaves
- focused
  `cargo test -p xiuxian-wendao --lib link_graph::runtime_config::tests:: -- --nocapture`
  now passes with 8 green tests
- touched-scope `cargo check -p xiuxian-wendao --tests` remains green after the
  link-graph runtime-config split
- rerunning the lightweight shared `unit_test` harness confirms the bloated
  unit-leaf debt dropped again from 22 to 21
- `tests/unit/parsers/markdown/sections.rs` is now a folder-first
  `tests/unit/parsers/markdown/sections/` suite with a small `mod.rs` root and
  focused `properties.rs`, `section_extraction.rs`, `logbook.rs`, and
  `support.rs` leaves; `src/parsers/markdown/sections/mod.rs` now mounts the
  new root instead of the former monolithic leaf
- focused
  `cargo test -p xiuxian-wendao --lib parsers::markdown::sections::tests:: -- --nocapture`
  now passes with 18 green tests
- touched-scope `cargo check -p xiuxian-wendao --tests` remains green after the
  markdown sections split
- rerunning the lightweight shared `unit_test` harness confirms the bloated
  unit-leaf debt dropped again from 21 to 20

The next bounded slice should therefore stay on the remaining oversized
`tests/unit/...` leaves. The immediate priority candidates are now the
remaining shallow leaves or top-level monoliths, such as
`tests/unit/gateway/studio/router/code_ast/native_routes.rs`, or one of the
Studio/router
surfaces.

`tests/unit/bin/wendao/execute/gateway.rs` has now been replaced with the
folder-first `tests/unit/bin/wendao/execute/gateway/` suite. The source mount
in `src/bin/wendao/execute/gateway/mod.rs` now points at the new `mod.rs`
root, the existing `command.rs` and `config.rs` leaves remain independently
source-mounted from their production owners, and the new root only carries the
root-owned `health`, `router`, `runtime`, `status`, and `support` leaves.

Rerunning the lightweight shared `unit_test` harness confirms the bloated
unit-leaf debt dropped again from 20 to 19.

`tests/unit/gateway/studio/router/code_ast/native_routes.rs` has now been
replaced with the folder-first
`tests/unit/gateway/studio/router/code_ast/native_routes/` suite. The
`code_ast` test root still mounts `native_routes` as one bounded surface, but
the oversized leaf is now split into `plugin_repos.rs`, `nested_modelica.rs`,
`search_only.rs`, and `support.rs` under a small `mod.rs` TOC root.

This split also exposed two real adjacent drifts that were fixed in the same
owner cluster:

- `tests/unit/bin/wendao/execute/gateway/config.rs` now reuses the shared
  gateway test helpers through the mounted `crate::execute::gateway::tests`
  path, and
  `tests/unit/bin/wendao/execute/gateway/support.rs` promotes the temp-config
  helpers to `pub(crate)` so the source-mounted leaf can compile again
- the search-only and import-backed Modelica code-AST snapshots were updated to
  the current `line_start` / `line_end` and owner-path payload shapes

Focused validation for this slice now includes:

- touched-scope `cargo check -p xiuxian-wendao --tests`, which is green again
- exact
  `cargo test -p xiuxian-wendao --lib gateway::studio::router::tests::code_ast::native_routes::plugin_repos::load_code_ast_analysis_response_supports_import_backed_modelica_package_repository -- --exact --nocapture`,
  which now passes after the snapshot refresh

Rerunning the lightweight shared `unit_test` harness confirms the bloated
unit-leaf debt dropped again from 19 to 18.

One focused live-path proof remains open:

- exact
  `gateway::studio::router::tests::code_ast::native_routes::nested_modelica::load_code_ast_analysis_response_supports_nested_modelica_package_repository_from_linked_service_within_timeout`
  still hangs locally without emitting a Rust failure text, so this slice is
  structurally landed and gate-improving, but the linked-service live proof is
  not yet closed

`tests/unit/gateway/studio/router/config.rs` has now also been replaced with a
folder-first router-owned suite, but this split needed one extra boundary
decision: `tests/unit/gateway/studio/router/config/mod.rs` was already occupied
as the source-mounted test root for production
`src/gateway/studio/router/config/mod.rs`, so the new router-owned large-file
split could not reuse `config/mod.rs` without colliding with the existing mount.

The bounded fix is now:

- `tests/unit/gateway/studio/router/mod.rs` mounts
  `#[path = "config/router/mod.rs"] mod config;`
- the oversized router-owned suite lives under
  `tests/unit/gateway/studio/router/config/router/`
- the existing source-mounted
  `tests/unit/gateway/studio/router/config/mod.rs` remains untouched for the
  production `config` feature-owner seam

The new router-owned `config/router/` suite is now split into:

- `bootstrap.rs`
- `capabilities.rs`
- `plugin_artifacts.rs`
- `support.rs`

Focused validation for this slice now includes:

- touched-scope `cargo check -p xiuxian-wendao --tests`, which is green again
- focused
  `cargo test -p xiuxian-wendao --lib gateway::studio::router::tests::config:: -- --nocapture`,
  which now passes with 13 green tests

Rerunning the lightweight shared `unit_test` harness confirms the bloated
unit-leaf debt dropped again from 18 to 17.

`tests/unit/gateway/studio/router/state/mod.rs` has now also been replaced
with a folder-first source-mounted suite. This split stayed simpler than the
adjacent `config/router` case because production already mounted
`tests/unit/gateway/studio/router/state/mod.rs` directly from
`src/gateway/studio/router/state/mod.rs`, so the remediation only needed to
turn the oversized root into a small TOC and push the real coverage into
focused leaves.

The new `router/state/` suite is now split into:

- `basics.rs`
- `local_startup.rs`
- `search_bundle.rs`
- `support.rs`
- `warm_start.rs`

Focused validation for this slice now includes:

- touched-scope `cargo check -p xiuxian-wendao --tests`, which is green again
- focused
  `cargo test -p xiuxian-wendao --lib gateway::studio::router::state::tests:: -- --nocapture`,
  which now passes with 16 green tests

Rerunning the lightweight shared `unit_test` harness confirms the bloated
unit-leaf debt dropped again from 17 to 16.

`tests/unit/gateway/studio/search.rs` has now also been replaced with the
folder-first `tests/unit/gateway/studio/search/` suite. The source mount in
`src/gateway/studio/search/handlers/mod.rs` now points at `search/mod.rs`,
the new root keeps the shared `StudioStateFixture` and search-publication
helpers, and the former monolithic coverage is now split across focused leaves:

- `knowledge.rs`
- `intent.rs`
- `status.rs`
- `attachments.rs`
- `autocomplete.rs`
- `ast.rs`
- `definition_api.rs`
- `references_symbols.rs`
- `content.rs`

The pre-existing `code_search_intent.rs` owner leaf remains in the same
feature folder and now lives beside the smaller root-owned leaves instead of
under one oversized `search.rs` monolith.

Focused validation for this slice now includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-gateway-search-split cargo check -p xiuxian-wendao --tests`,
  which is green
- focused
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-gateway-search-split cargo test -p xiuxian-wendao --lib gateway::studio::search::handlers::studio_search_tests:: -- --nocapture`,
  which now passes with 50 green tests in `65.53s`

Rerunning the lightweight shared `unit_test` harness confirms the bloated
unit-leaf debt dropped again from 16 to 15. The remaining repeated-namespace
path findings stay warning-only and do not change the failure surface for this
slice.

`tests/unit/gateway/studio/search/handlers/code_search/plugin_routes.rs` has
now also been replaced with the folder-first
`tests/unit/gateway/studio/search/handlers/code_search/plugin_routes/` suite.
The existing `code_search/mod.rs` owner root still mounts `plugin_routes` as
one bounded surface, but the oversized leaf is now split into:

- `live_plugins.rs`
- `repo_scoped_ast.rs`
- `search_only.rs`
- `alias_scope.rs`
- `guardrails.rs`
- `support.rs`

This slice stayed entirely inside the current Studio/search owner cluster. No
production mount changes were needed because
`tests/unit/gateway/studio/search/handlers/code_search/mod.rs` was already the
test owner seam and only had to resolve `plugin_routes` through the new folder
root.

Focused validation for this slice now includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-gateway-search-split cargo check -p xiuxian-wendao --tests`,
  which is green
- focused
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-gateway-search-split cargo test -p xiuxian-wendao --lib gateway::studio::search::handlers::tests::code_search::plugin_routes:: -- --nocapture`,
  which now passes with 14 green tests

Rerunning the lightweight shared `unit_test` harness confirms the bloated
unit-leaf debt dropped again from 15 to 14. The repeated-namespace path
warnings remain unchanged and advisory-only.

`tests/unit/gateway/studio/search/handlers/flight/repo_search.rs` has now also
been replaced with the folder-first
`tests/unit/gateway/studio/search/handlers/flight/repo_search/` suite. Because
this was a source-mounted test surface, the production mount in
`src/gateway/studio/search/handlers/flight/repo_search.rs` now points at
`repo_search/mod.rs`, and the former monolithic suite is split into:

- `provider.rs`
- `routes.rs`
- `filters.rs`
- `ranking.rs`
- `bootstrap.rs`
- `support.rs`

This slice stayed inside the same Flight owner seam. The bounded production
change was only the mount retarget from the flat `repo_search.rs` test file to
the new folder-first root.

Focused validation for this slice now includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-gateway-search-split cargo check -p xiuxian-wendao --tests`,
  which is green
- focused
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-gateway-search-split cargo test -p xiuxian-wendao --lib gateway::studio::search::handlers::flight::repo_search::tests:: -- --nocapture`,
  which now passes with 19 green tests

Rerunning the lightweight shared `unit_test` harness confirms the bloated
unit-leaf debt dropped again from 14 to 13. The repeated-namespace path
warnings remain unchanged and advisory-only.

`tests/unit/gateway/studio/graph.rs` has now also been replaced with the
folder-first `tests/unit/gateway/studio/graph/` suite. This leaf was a pure
shared-gate unit offender rather than a source-mounted production test seam, so
the remediation stayed entirely inside the `tests/unit` tree and split the
former monolith into:

- `live_neighbors.rs`
- `markdown_analysis.rs`
- `pathing.rs`
- `topology.rs`
- `configured_projects.rs`
- `support.rs`

The new `graph/mod.rs` root is now an interface-only TOC for the Studio graph
unit suite. Shared fixture/config helpers moved into `support.rs`, while the
graph behavior proofs are grouped by responsibility instead of accumulating in
one oversized leaf.

Focused validation for this slice now includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-studio-graph-split cargo check -p xiuxian-wendao --tests`,
  which is green
- shared gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`,
  which now reports 12 remaining bloated `tests/unit/...` leaves

An exact focused `cargo test -p xiuxian-wendao --lib graph_neighbors_rebuilds_after_ui_config_update -- --exact --nocapture`
attempt was started after the split, but local test-linking stalled in the
transitive `lance` build and did not produce closure evidence. The warmed
`cargo check` plus the shared gate remeasure are the authoritative proofs for
this slice.

`tests/unit/gateway/studio/router/handlers/repo/analysis/search/mod.rs` has
now also been replaced with a folder-first source-mounted suite. The production
owner seam in
`src/gateway/studio/router/handlers/repo/analysis/search/mod.rs` still points
at the same test root, but the former oversized mounted file is now split into:

- `cache_behavior.rs`
- `query_core.rs`
- `import_fast_path.rs`
- `support.rs`

The new `search/mod.rs` root is now a TOC-only mounted test seam. Shared
repo-analysis fixtures and publication/bootstrap helpers moved into
`support.rs`, while the actual proofs are grouped by stable behavior families
instead of growing inside one monolithic mounted root. The pre-existing
source-mounted `import.rs` and `service.rs` leaves were left untouched.

Focused validation for this slice now includes:

- shared gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`,
  which now reports 11 remaining bloated `tests/unit/...` leaves

An exact focused
`cargo test -p xiuxian-wendao --lib repo_import_search_payload_snapshot -- --exact --nocapture`
attempt was started after the split, but local test-linking again stalled in
the transitive `lance` build before a Rust verdict was emitted. The shared gate
remeasure is the authoritative proof for this slice.

`tests/unit/semantic_check_tests.rs` has now also been replaced with the
folder-first `tests/unit/semantic_check_tests/` suite. Because this was a
source-mounted test surface, the production owner seam in
`src/zhenfa_router/native/semantic_check/mod.rs` now points at
`tests/unit/semantic_check_tests/mod.rs`, and the former monolithic suite is
split into:

- `extraction.rs`
- `contract_validation.rs`
- `helper_functions.rs`
- `health_score.rs`
- `observation_checks.rs`
- `support.rs`

The new `semantic_check_tests/mod.rs` root is now a TOC-only mounted test seam.
Shared observation fixtures moved into `support.rs`, while the actual semantic
check proofs are grouped by extraction, contract, helper, health-score, and
observation behavior instead of accumulating in one oversized mounted root.

Focused validation for this slice now includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-semantic-check-split cargo check -p xiuxian-wendao --tests`,
  which is green
- shared gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`,
  which now reports 10 remaining bloated `tests/unit/...` leaves

A broader focused
`cargo test -p xiuxian-wendao --lib semantic_check::tests:: -- --nocapture`
attempt was started after the split, but the test-profile rebuild cost was too
high for this active remediation lane, so the closure evidence for this slice
is the warmed `cargo check --tests` plus the shared gate remeasure.

`tests/unit/query_core/mod.rs` has now also been reduced to a folder-first
source-mounted suite. The production owner seam in `src/query_core/mod.rs`
still mounts the same test root, but the former oversized mounted file is now
split into:

- `defaults.rs`
- `execution.rs`
- `relation_queries.rs`
- `graph_projection.rs`
- `support.rs`

The new `tests/unit/query_core/mod.rs` root is now a TOC-only mounted test
seam. Shared repo fixtures, graph stubs, and retrieval helpers moved into
`support.rs`, while the actual proofs are grouped by operator defaults,
execution flows, relation-query routing, and graph projection behavior instead
of accumulating in one oversized mounted root.

Focused validation for this slice now includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-query-core-split cargo check -p xiuxian-wendao --tests`,
  which is green
- shared gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`,
  which now reports 9 remaining bloated `tests/unit/...` leaves

A broader focused `cargo test -p xiuxian-wendao --lib query_core::tests::`
rerun was intentionally skipped in this slice: the warmed `cargo check --tests`
already proved the mounted suite compiles cleanly, and the shared gate remeasure
is the authoritative closure evidence for the leaf-curation lane.

`tests/unit/repo_index/state/sync.rs` has now also been reduced to a
folder-first unit suite. The existing `tests/unit/repo_index/state/mod.rs`
owner seam stayed stable: `mod sync;` now resolves to `sync/mod.rs`, while the
former oversized leaf is split into:

- `enqueue.rs`
- `warm_start.rs`
- `probe_policy.rs`
- `queue_status.rs`
- `support.rs`

The new `tests/unit/repo_index/state/sync/mod.rs` root is now a TOC-only test
seam. Shared publication fixtures, repo-resolution imports, and stale-probe
aging helpers moved into `support.rs`, while the actual proofs are grouped by
enqueue behavior, warm-start recovery, stale managed-remote probe policy, and
queue/status transitions instead of accumulating in one oversized state leaf.

Focused validation for this slice now includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-repo-sync-split cargo check -p xiuxian-wendao --tests`,
  which is green
- shared gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`,
  which now reports 8 remaining bloated `tests/unit/...` leaves

No additional broad focused `cargo test -p xiuxian-wendao --lib repo_index::state::tests::sync::`
rerun was counted in this slice; the warmed `cargo check --tests` plus the
shared gate remeasure are the closure evidence for this remediation step.

`tests/unit/zhenfa_router/native/agentic_nav.rs` has now also been reduced to a
folder-first source-mounted suite. The production owner seam in
`src/zhenfa_router/native/agentic_nav.rs` now points at
`tests/unit/zhenfa_router/native/agentic_nav/mod.rs`, and the former flat leaf
is split into:

- `escape.rs`
- `navigation_hint.rs`
- `render.rs`
- `support.rs`

The new `tests/unit/zhenfa_router/native/agentic_nav/mod.rs` root is now a
TOC-only mounted test seam. Shared hit construction moved into `support.rs`,
while the actual proofs are grouped by XML escaping, navigation-hint depth
policy, and rendered XML output behavior instead of accumulating in one
source-mounted file.

Focused validation for this slice now includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-agentic-nav-split cargo check -p xiuxian-wendao --tests`,
  which is green
- shared gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`,
  which now reports 7 remaining bloated `tests/unit/...` leaves

A broader focused
`cargo test -p xiuxian-wendao --lib zhenfa_router::native::agentic_nav::tests:: -- --nocapture`
was started after the split, but the test-profile rebuild cost was too high for
this active remediation lane, so it is not counted as closure evidence for this
slice.

`tests/unit/studio_repo_sync_api/planner.rs` has now also been reduced to a
folder-first unit suite. The surrounding
`tests/unit/studio_repo_sync_api/mod.rs` owner seam stayed stable:
`mod planner;` now resolves to `planner/mod.rs`, while the former oversized
leaf is split into:

- `search.rs`
- `queue.rs`
- `rank.rs`
- `workset.rs`
- `support.rs`

The new `tests/unit/studio_repo_sync_api/planner/mod.rs` root is now a TOC-only
test seam. Shared router construction, JSON helpers, queue-preview assertions,
priority-sort keys, and external Modelica repo setup moved into `support.rs`,
while the actual proofs are grouped by planner search, queue, rank, and
workset behavior instead of accumulating in one oversized Studio repo-sync
leaf.

Focused validation for this slice now includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-planner-split cargo check -p xiuxian-wendao --tests`,
  which is green after removing one transient unused-import warning from the
  new `planner/mod.rs` root
- shared gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`,
  which now reports 6 remaining bloated `tests/unit/...` leaves while the
  repeated-namespace path findings remain 9 advisory warnings

No additional broad `cargo test -p xiuxian-wendao --lib studio_repo_sync_api::planner::`
rerun was counted in this slice; the warmed `cargo check --tests` plus the
shared gate remeasure are the closure evidence here. The parent
`tests/unit/studio_repo_sync_api/mod.rs` root still remains a separate
oversized offender for a later slice.

`tests/unit/zhenfa_router/native/docs.rs` has now also been reduced to a
folder-first source-mounted suite. The production owner seam in
`src/zhenfa_router/native/docs/registry.rs` now points at
`tests/unit/zhenfa_router/native/docs/mod.rs`, and the former flat leaf is
split into:

- `payloads.rs`
- `context.rs`
- `registry.rs`
- `support.rs`

The new `tests/unit/zhenfa_router/native/docs/mod.rs` root is now a TOC-only
mounted seam. The fake docs runtime, shared page constants, context builders,
and runtime helpers moved into `support.rs`, while the actual proofs are
grouped by payload serialization, extension fallback/context behavior, and
registry capability exposure instead of accumulating in one source-mounted
file.

Focused validation for this slice now includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-native-docs-split cargo check -p xiuxian-wendao --tests`,
  which is green
- shared gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`,
  which now reports 4 remaining bloated `tests/unit/...` leaves while the
  repeated-namespace path findings remain 9 advisory warnings

A broader feature-on focused
`cargo test -p xiuxian-wendao --lib --features "zhenfa-router julia" zhenfa_router::native::docs::registry::tests:: -- --nocapture`
was started after the split, but the feature graph rebuild cost was too high
for this active remediation lane, so it is not counted as closure evidence for
this slice.

`tests/unit/zhenfa_router/native/audit/audit_bridge/package_docs.rs` has now
also been reduced to a folder-first unit suite. The surrounding
`tests/unit/zhenfa_router/native/audit/audit_bridge/mod.rs` owner seam stayed
stable: `mod package_docs;` now resolves to `package_docs/mod.rs`, while the
former oversized leaf is split into:

- `create_files.rs`
- `index_links.rs`
- `footer.rs`

The new `tests/unit/zhenfa_router/native/audit/audit_bridge/package_docs/mod.rs`
root is now a TOC-only test seam. Shared audit-bridge imports still come from
the parent owner root, while the actual proofs are grouped by package-doc
create-file fixes, index/link repair, and footer repair behavior instead of
accumulating in one oversized audit leaf.

Focused validation for this slice now includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-native-docs-split cargo check -p xiuxian-wendao --tests`,
  which remains green after the incremental split
- shared gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`,
  which now reports 3 remaining bloated `tests/unit/...` leaves while the
  repeated-namespace path findings remain 9 advisory warnings

`tests/unit/link_graph_agentic/expansion.rs` has now also been reduced to a
folder-first unit suite. The surrounding
`tests/unit/link_graph_agentic/mod.rs` owner seam stayed stable:
`mod expansion;` now resolves to `expansion/mod.rs`, while the former
oversized leaf is split into:

- `support.rs`
- `plan.rs`
- `projection.rs`
- `live.rs`

The new `tests/unit/link_graph_agentic/expansion/mod.rs` root is now a TOC-only
test seam, and the existing feature-gated
`expansion_plan_batch_tests.rs` still hangs off the same `expansion` owner
module. Shared index-fixture construction, plugin config helpers, and Julia
transport imports moved into `support.rs`, while the actual proofs are grouped
by plan-budget behavior, non-live Julia request projection, and live structural
transport behavior instead of accumulating in one oversized link-graph leaf.

Focused validation for this slice now includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-link-graph-expansion-split cargo check -p xiuxian-wendao --tests`,
  which is green after one small follow-up fix to keep the new `expansion/mod.rs`
  aliases parent-private and remove an unused import from `support.rs`
- shared gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`,
  which now reports 2 remaining bloated `tests/unit/...` leaves while the
  repeated-namespace path findings remain 9 advisory warnings

The final Studio repo-sync API monolith has now also been cleared. The owner
root `tests/unit/studio_repo_sync_api/mod.rs` is currently a small hub that
fans out into grouped feature folders:

- `docs_endpoints/`
- `error_cases/`
- `repo_endpoints/`
- `repo_projected_context/`
- `repo_projected_lookup/`
- existing `gap_reports.rs`
- existing `planner/`
- shared `support.rs`

This keeps the public route families separated without reopening the parent
root as another oversized leaf, and it leaves the existing `planner/` and
`gap_reports.rs` seams intact.

Focused validation for the fully curated `studio_repo_sync_api` surface now
includes:

- shared gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`,
  which now passes with zero remaining test-structure violations while the
  repeated-namespace path findings remain 9 advisory warnings
- touched-scope
  `direnv exec . cargo check -p xiuxian-wendao --tests`,
  which was rerun for closure on the same lane

At this point the Wendao shared `unit_test` testing-gate remediation lane is
structurally green. Remaining work in this crate is no longer oversized
test-leaf debt; it is the separate repeated-namespace warning backlog and any
future owner-cluster curation done for readability rather than gate recovery.

That repeated-namespace backlog is now also cleared for the search query owner
paths. The six advisory `src/search/*/query/search` branches were renamed to
`query/lookup`, and the corresponding source imports, unit-test imports, and
package README path inventory were updated to match the new owner seams:

- `attachment/query/lookup/`
- `knowledge_section/query/lookup/`
- `local_symbol/query/lookup/`
- `reference_occurrence/query/lookup/`
- `repo_content_chunk/query/lookup/`
- `repo_entity/query/lookup/`

Focused validation for that follow-up cut includes:

- shared gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-test-policy-gate cargo test -p xiuxian-wendao --test unit_test enforce_crate_test_policy_harness -- --exact --nocapture`,
  which now passes without any repeated-namespace warnings
- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-path-warnings cargo check -p xiuxian-wendao --tests`,
  which stays green after the owner-path rename

At this point the Wendao shared `unit_test` gate is clean both for structure
violations and for repeated-namespace advisory warnings.

Follow-on package-level clippy closure on the same lane is now green too. The
remaining production-side `code_ast.rs` monolith was split into small source
resolution and response-shaping helpers, the shared search fixture builder in
`tests/unit/analyzers/service/search/support.rs` was decomposed into focused
record helpers, and the `code_search/indexing.rs` all-repo test now relies on
fixture/setup helpers instead of one oversized body. One adjacent performance
support duplicate-match warning was folded away during the same sweep.

Focused validation for this closure cut includes:

- touched-scope
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-code-ast-split cargo check -p xiuxian-wendao --tests`
- source-mounted code-AST proofs
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-code-ast-split cargo test -p xiuxian-wendao --lib gateway::studio::router::tests::code_ast::native_routes::search_only::load_code_ast_analysis_response_supports_search_only_ast_grep_rust_repository -- --exact --nocapture`
- source-mounted code-AST live plugin proofs
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-code-ast-split cargo test -p xiuxian-wendao --lib gateway::studio::router::tests::code_ast::native_routes::plugin_repos:: -- --nocapture`
- search-fixture regression proofs
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-code-ast-split cargo test -p xiuxian-wendao --lib analyzers::service::search::tests:: -- --nocapture`
- code-search indexing regression proofs
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-code-ast-split cargo test -p xiuxian-wendao --lib gateway::studio::search::handlers::tests::code_search::indexing:: -- --nocapture`
- package gate
  `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-code-ast-split cargo clippy -p xiuxian-wendao --all-targets --all-features -- -D warnings`
