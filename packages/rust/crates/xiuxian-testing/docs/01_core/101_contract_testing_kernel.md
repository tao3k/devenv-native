# Contract Testing Kernel V1

:PROPERTIES:
:ID: xiuxian-testing-contract-kernel-v1
:PARENT: [[../index]]
:TAGS: architecture, contracts, testing
:STATUS: ACTIVE
:END:

## Goal

Evolve `xiuxian-testing` from a shared helper crate into a contract-testing kernel that can:

1. audit engineering structure and documentation contracts,
2. convert REST and documentation expectations into machine-checkable findings,
3. produce reusable evidence for human review and Wendao ingestion, and
4. separate deterministic checks from LLM-assisted heuristics.

## Design Principles

- Deterministic checks first. LLMs may suggest or classify, but they do not own pass/fail on their own.
- Contracts are explicit artifacts. Tests should consume normalized contract data, not free-form prose alone.
- Findings must be reusable. Every violation should carry machine-readable evidence, remediation guidance, and provenance.
- Rule packs stay modular. REST, modularity, and knowledge-export concerns should evolve independently.
- Advisory and strict modes must coexist. Teams need a path from warnings to hard gates.
- Runtime and role evidence are first-class. If advisory audits run through `Qianji` and `Zhenfa`, their traces should remain attachable to final findings and exportable to `Wendao`.

## Existing Base Inside `xiuxian-testing`

The current crate already has three usable foundations:

- `scenario`: reusable scenario and snapshot execution
- `policy`: crate-level test structure policy
- `external_test` and `validation`: convention validation for externalized tests and filesystem structure

V1 should build on these surfaces instead of replacing them.

## Test Layout Governance

The crate-level test gate now enforces two complementary rules:

1. `tests/` root is a harness surface, not a dumping ground.
2. `src/** #[cfg(test)]` modules must mount test code from `tests/unit/...`
   through `#[path = "..."]` instead of leaving test implementation in `src/`.

### Canonical `tests/` Root Shape

Keep only explicit entrypoints and bounded support surfaces directly under
`tests/`:

- `unit_test.rs`
- `integration_test.rs`
- `performance_test.rs`
- `scenarios_test.rs`
- structured directories such as `unit/`, `integration/`, `performance/`,
  `fixtures/`, `support/`, and `snapshots/`

Any other root file or directory is an exception and must be justified through
`tests/xiuxian-testings-rules.toml`.

### Target-Local Harness Rule

The preferred shape is for each real Cargo test entrypoint to mount the shared
gate directly:

```rust
xiuxian_testing::crate_test_policy_harness!();
```

That keeps normal commands such as `cargo test --test unit_test` or
`cargo test --test integration_test` on the same full-policy lane as a full
crate test run.

`xiuxian-testing-gate.rs` is a legacy transition surface only. Use it only
when a crate still lacks stable root harness entrypoints and record the reason
as bounded migration debt instead of treating it as the steady-state shape.

### Source-Backed `--lib` Harness Rule

If a crate keeps unit tests externalized behind `src/** -> tests/unit/**`
`#[path]` mounts, `cargo test --lib` should still execute the shared gate.

Preferred shape:

```rust
// src/lib.rs
xiuxian_testing::crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");

// tests/unit/lib_policy.rs
xiuxian_testing::crate_test_policy_harness!();
```

That keeps the gate body out of `src/` while making normal `cargo test --lib`
flows enforce the same crate policy as explicit `--test <target>` entrypoints.

`xiuxian-testing` self-hosts this pattern now: `src/lib.rs` mounts the source
gate, `tests/unit/lib_policy.rs` owns the gate body, and the contract suites
live behind `tests/integration_test.rs` instead of scattered root targets.
`xiuxian-git-repo` now proves the consumer variant of the same shape:
`src/lib.rs` mounts `tests/unit/lib_policy.rs`, `tests/unit_test.rs` owns the
root harness target, and backend-specific unit coverage can still live under
`tests/unit/backend/...` without keeping implementation in `src/`.
`xiuxian-wendao-julia` now follows the same split shape while keeping its
existing helper seams under `tests/unit/`; the former large inline
`#[cfg(test)]` blocks in `src/integration_support/`, `src/memory/`, and
`src/plugin/` now all mount canonical `tests/unit/...` files, so both
`cargo test --lib` and `cargo test --test unit_test` pass the shared gate
without crate-local exceptions.
`xiuxian-wendao` now mounts the same canonical entrypoints:
`src/lib.rs -> tests/unit/lib_policy.rs` for `cargo test --lib`, plus
`tests/unit_test.rs` as the explicit root harness target. The first bounded
consumer slice externalized the smallest inline suites under
`src/analyzers/service/bootstrap.rs`,
`src/gateway/studio/types/collection.rs`,
`src/link_graph/stats_cache/runtime.rs`,
`src/skill_vfs/zhixing/indexer/stats.rs`, and
`src/gateway/studio/router/handlers/repo/shared/repository.rs`, but both
shared harness entrypoints still fail on the crate's larger pre-existing
inline-test debt and non-standard `tests/support` mounts. That remaining debt
must keep shrinking in bounded slices instead of being hidden behind the old
standalone gate target.
The next bounded follow-up already removed the first support-path drift and two
more small inline suites by externalizing `src/valkey_common.rs`,
`src/link_graph/agentic/store/common.rs`, and the
`src/analyzers/service/projection/` test module onto canonical
`tests/unit/...` mounts, so those files are no longer present in the active
Wendao gate failure surface.
The next bounded analyzer slice removed the remaining small cache/config
mounts under `src/analyzers/cache/valkey/mod.rs`,
`src/analyzers/cache/mod.rs`, and `src/analyzers/config/mod.rs`, so the active
Wendao gate debt is now pushed into the larger analyzer service/query branches
instead of the analyzer cache/config boundary.
The next bounded service/search slice then removed
`src/analyzers/service/search/mod.rs`,
`src/analyzers/service/search/ranking/mod.rs`, and
`src/analyzers/service/search/contracts.rs` from the active Wendao failure
surface, proving the gate can keep shrinking one analyzer subcluster at a
time without widening into the surrounding service tree.
The next bounded service-root/planner slice then removed
`src/analyzers/service/mod.rs`,
`src/analyzers/service/helpers/mod.rs`,
`src/analyzers/service/projection/planner/mod.rs`, and
`src/analyzers/query/docs/planner/mod.rs` from the active Wendao failure
surface, showing the same bounded externalization pattern works for mixed
single-file, subtree, and planner-oriented analyzer seams.
Validator scans must ignore rustdoc examples and comment-only `#[path = "..."]`
snippets so the canonical documentation pattern does not create false
`MissingTestFile` policy failures.

### Exception Policy File

The rules file intentionally lives under `tests/` because it governs the
physical `tests/` layout of one crate.

Use it only when a file or directory truly must remain directly under
`tests/`. Every allowlist entry must include both:

- `name`
- `explanation`

Example:

```toml
[tests]
allowed_root_files = [
  { name = "legacy_harness.rs", explanation = "Cargo entrypoint kept at tests root until the suite is migrated into unit_test.rs." },
]
allowed_directories = [
  { name = "bench", explanation = "Temporary benchmark mount kept at tests root until performance_test.rs owns the suite." },
]
```

Missing explanations are treated as policy errors.

### Externalized Unit Test Pattern

For source-backed unit tests, keep production files clean and mount the test
implementation from `tests/unit/...`.

```rust
#[cfg(test)]
#[path = "../../tests/unit/foo/bar.rs"]
mod tests;
```

The full crate gate rejects both:

- inline `#[cfg(test)] mod tests { ... }`
- `#[cfg(test)] mod tests;` declarations that still keep the module in `src/`
  without an external `#[path]` mount

`test_api` remains the narrow exception for exposing private helpers to the
externalized unit test file.

### Post-Harness Leaf Curation

Harness migration is not the end-state by itself. Once a suite already lives in
`tests/unit/...` or `tests/integration/...`, the leaf files there must still
stay focused enough to act as clear ownership seams.

The current kernel now treats oversized post-harness Rust leaves under
`tests/unit/...` and `tests/integration/...` as a structure-policy violation
when they simultaneously:

- accumulate roughly monolithic size (bounded by effective code lines), and
- carry a dense cluster of tests that now reads more like a miniature suite
  directory than a single leaf file

The integration branch intentionally uses a looser threshold than unit leaves
so fixture builders and repeated contract setup do not create noisy debt.

Preferred remediation:

- keep the same canonical `tests/unit/...` or `tests/integration/...` tree
- split the large leaf into a feature folder such as
  `tests/unit/policy/{mod.rs,workspace_config.rs,harness.rs}` or
  `tests/integration/contracts_modularity/{mod.rs,root_facade.rs,root_hint.rs}`
- keep `mod.rs` as the local test-suite seam and move scenario/config/harness
  clusters into focused sibling files

`xiuxian-testing` now self-hosts this pattern in
`tests/unit/policy/`, `tests/unit/scenario/`, and
`tests/integration/contracts_modularity/`: the old large leaf files were split
into folder-first suites after the harness migration, so the crate proves that
canonical `tests/unit/...` and `tests/integration/...` layouts can coexist with
post-harness leaf curation without reintroducing root-level scatter or inline
`#[cfg(test)]` blocks.

## Proposed V1 Layers

### 1. Structure Policy Layer

This is the current deterministic base:

- crate test layout validation
- external test mount policy
- snapshot policy and redaction support

This layer remains purely deterministic and should keep running in CI as the low-risk baseline.

### 2. Contract Kernel Layer

Add a shared model for contract execution:

```rust
pub struct ContractSuite {
    pub id: String,
    pub version: String,
    pub rule_packs: Vec<Box<dyn RulePack>>,
}

pub trait RulePack {
    fn id(&self) -> &'static str;
    fn collect(&self, ctx: &CollectionContext) -> anyhow::Result<CollectedArtifacts>;
    fn evaluate(&self, input: &CollectedArtifacts) -> anyhow::Result<Vec<ContractFinding>>;
}

pub struct ContractFinding {
    pub rule_id: String,
    pub severity: FindingSeverity,
    pub mode: FindingMode,
    pub title: String,
    pub summary: String,
    pub why_it_matters: String,
    pub remediation: String,
    pub evidence: Vec<FindingEvidence>,
    pub source_paths: Vec<PathBuf>,
}
```

This layer is the stable spine for all future rule packs.

### 3. Artifact Collection Layer

Normalize inputs before evaluation:

- source code structure
- Rust module graph
- OpenAPI documents
- inline and external engineering documentation
- runtime traces or logs when available
- Wendao-exportable knowledge envelopes

The collector layer is where code, docs, and runtime evidence become comparable artifacts.

### 4. Rule-Pack Execution Layer

Each rule pack evaluates a bounded concern:

- `rest_docs`
- `modularity`
- `knowledge_feedback`
- later: `runtime_invariants`, `review_guidance`, `scenario_quality`

Every rule pack returns findings through the same kernel schema.

### 4.5 Advisory Audit Execution Layer

This layer reuses the existing runtime stack rather than rebuilding it inside `xiuxian-testing`:

- `Qianhuan` manifests the requested auditor roles
- `Qianji formal_audit` executes the advisory critique loop
- `ZhenfaPipeline` normalizes streaming output and cognitive metrics
- `Wendao` stores resulting `CognitiveTrace` artifacts

The contract kernel should treat this layer as an optional role-attributed supplement to deterministic findings, not as a replacement.

### 5. Reporting and Export Layer

V1 should emit two stable outputs:

- human-readable markdown or terminal summaries
- machine-readable JSON for CI, dashboards, and Wendao indexing

Suggested report surface:

```rust
pub struct ContractReport {
    pub suite_id: String,
    pub generated_at: String,
    pub findings: Vec<ContractFinding>,
    pub stats: ContractStats,
}
```

### 6. Wendao Feedback Layer

This is where the testing system becomes a knowledge system.

Every exported finding should include:

- `rule_id`
- `domain`
- `severity`
- `decision` (`pass`, `warn`, `fail`)
- `evidence_excerpt`
- `why_it_matters`
- `remediation`
- `good_example`
- `bad_example`
- `source_path`

This keeps test output usable by both humans and retrieval systems.

## Execution Modes

V1 should support three modes:

- `strict`: fail the run on configured severities
- `advisory`: report findings without failing
- `research`: collect rich evidence, retain low-confidence findings, and support paper-driven exploration

This lets the same architecture support production gating and frontier research.

Within `advisory` mode, multi-role audit is the main consumer of the runtime stack described in [[../03_features/202_multi_role_audit_integration]].

## Where REST Best Practices Fit

REST engineering quality should not be treated as one monolithic lint. V1 should model it as a contract family:

- route purpose and naming
- request and response schema completeness
- status-code coverage
- error-envelope consistency
- pagination and filtering semantics
- idempotency and mutation semantics
- examples and documentation depth
- code, docs, and OpenAPI consistency

That family becomes one rule pack, not the whole system.

## Where Modularity Fits

Modularity should be audited from source structure and visibility, not only style lints.

Examples:

- `mod.rs` interface-only discipline
- `pub(crate)` default for internal boundaries
- forbidden cross-layer imports
- adapter versus kernel boundary checks
- doc coverage on public `Result` APIs

This is how the system grows from code-style checking into architecture governance.

## Non-Goals for V1

- full automatic OpenAPI inference
- full runtime invariant mining
- autonomous rule remediation
- LLM-only pass/fail decisions

These belong to later phases after the contract schema and rule-pack interfaces stabilize.

## V1 Acceptance Signal

V1 is successful when:

1. at least one crate can run deterministic contract checks for REST docs and modularity,
2. findings share one schema across rule packs,
3. advisory findings can be attributed to role-based audit runs from `Qianji` and `Qianhuan`,
4. findings and traces are exportable to Wendao as knowledge records, and
5. advisory and strict execution modes are both supported at the design level.

## Consumer Remediation Notes

The canonical remediation shape for Rust crates is now:

- a source-side harness mounted from `src/lib.rs` or `src/main.rs` so
  `cargo test --lib` enforces the crate policy by default
- a root harness target under `tests/unit_test.rs` so explicit integration test
  selections still traverse the same policy layer
- source-resident test suites mounted directly from `tests/unit/...` with
  `#[cfg(test)] #[path = "..."] mod tests;`

When a canonical `tests/unit/...` file already exists, remediation should fold
any old `src/**/tests.rs` shim imports into that canonical file instead of
adding another forwarding layer under `src/`.

In practice, consumer crates can be remediated one bounded cluster at a time.
Once the source-side harness is active, each completed cluster should
disappear from the shared gate output immediately, which makes the next slice
selection mechanical instead of judgment-heavy.

For `packages/rust/crates/xiuxian-wendao`, the active remediation proof has
now also removed the standalone analyzer suites in
`src/analyzers/saliency.rs` and `src/analyzers/skeptic.rs` from the shared
gate output. The next bounded slice should follow the same shape on the
remaining `src/analyzers/projection/*` branch rather than reopening already
cleared standalone analyzer files.

That next bounded slice has now been applied too: the shared gate no longer
reports `src/analyzers/projection/search/mod.rs` or
`src/analyzers/projection/builder/mod.rs`. The active failure surface has
advanced beyond analyzer projection and is now fronted by parser and
memory-julia policy debt.

That parser + memory-julia debt has now been cleared as well. The active
`xiuxian-wendao` gate no longer reports
`src/parsers/markdown/links/normalize.rs` or `src/memory/julia/mod.rs`, which
means the next bounded remediation slice should move to `src/bin/wendao/execute/*`
and then the remaining non-analyzer, non-memory modules.

That `src/bin/wendao/execute/*` slice has now been cleared too. The active
`xiuxian-wendao` gate no longer reports
`src/bin/wendao/execute/repo.rs`,
`src/bin/wendao/execute/gateway/config.rs`, or
`src/bin/wendao/execute/gateway/command.rs`, so the next bounded remediation
slice should move into the remaining non-bin owners fronting the gate output:
`src/ingress/*`, `src/entity/*`, `src/enhancer/*`, and then the larger
`repo_index` / `search` clusters.

The older `bounded_work_markdown` clippy blockers are now gone as well. The
consumer lane can rely on `direnv exec . cargo clippy -p xiuxian-wendao --lib --tests -- -D warnings`
as a live closure gate again, instead of treating that command as permanently
masked by unrelated debt.

The next shared-gate top cluster has been removed too: `ingress/spider`,
`entity`, and the current `enhancer/*` module owners no longer appear in the
live `xiuxian-wendao` failure surface. The next bounded slice should therefore
start at `src/dependency_indexer/symbols/mod.rs` and then move into the
`repo_index` state tree, which is now the first large contiguous debt cluster.

That dependency-indexer entry point is now gone from the live failure surface
as well. The `src/dependency_indexer/symbols/mod.rs` owner now mounts directly
from `tests/unit/dependency_indexer/symbols/mod.rs`, and the old flat
`dependency_indexer_symbols.rs` helper has been folded into that canonical
suite instead of surviving as a parallel legacy test surface. The next bounded
consumer slice should therefore start directly at `src/repo_index/state/*`.

That milestone-style `repo_index/state` slice has now landed too. The
source-resident `src/repo_index/state/tests/` subtree has been fully moved into
`tests/unit/repo_index/state/`, the inline `coordinator/*` and `task/*` suites
now mount from the same canonical tree, and the shared `xiuxian-wendao` gate
no longer reports any `repo_index/state` owner. The next milestone should
therefore start at the now-fronting `graph`, `storage`, and `search/*`
clusters instead of reopening the cleared repo-index state tree.

That next milestone has now landed as one bounded graph-storage-cache slice.
`src/graph/valkey_persistence.rs`, `src/storage/crud.rs`, and the full
`src/search/cache/` front now mount canonical `tests/unit/...` files; the old
source-resident `src/search/cache/tests.rs` hub has been moved wholesale into
`tests/unit/search/cache/mod.rs`, and the inline `config`, `runtime`, and
`writes` suites now live beside it under the same canonical tree. The live
Wendao gate therefore no longer reports any `graph`, `storage`, or
`search/cache` owner, so the next milestone starts directly at the broader
`search/*` front, beginning with `search/reference_occurrence/*` and the other
remaining search service/query mounts.

That next bounded search-corpus milestone has landed too. The paired
`build/query` owners for `reference_occurrence`, `attachment`,
`local_symbol`, `repo_content_chunk`, `repo_entity`, and
`knowledge_section` now all mount canonical `tests/unit/search/...` trees
instead of keeping source-resident `tests.rs` or `tests/` submodules in
`src/`. The live Wendao gate no longer reports any owner from those six
search corpus families, so the next milestone starts directly at the
remaining search-platform core: `repo_search`, `coordinator`, `manifest`,
`queries`, `tantivy`, `status`, `ranking`, and the search service core
subtree.

That search-platform-core milestone has now landed too. The remaining
`search/*` front that sat on top of the live Wendao gate now mounts canonical
`tests/unit/search/...` trees across `repo_search`, `coordinator`,
`manifest`, `queries`, `tantivy`, `status`, `ranking`, `corpus`,
`staged_mutation`, and the `search/service/core/*` subtree; the old
source-resident `tests.rs` files and `tests/` subtrees for `queries` and
`search/service` have been moved wholesale into the canonical tree, and the
few inline suites (`repo_search/batch.rs`, `project_fingerprint.rs`,
`ranking.rs`, `staged_mutation.rs`, `corpus.rs`, `status/maintenance.rs`)
were externalized beside them. The live Wendao gate therefore no longer
reports any `search/*` owner at all, so the next milestone begins directly at
the new front outside search: `pybindings/*`, `query_core`, `unified_symbol`,
`skill_vfs/*`, and the broader `gateway/studio/*` tree.

That next non-search milestone has landed too. The live Wendao gate no longer
reports `src/pybindings/unified_symbol_py/mod.rs`,
`src/pybindings/dep_indexer_py/mod.rs`, `src/query_core/mod.rs`,
`src/unified_symbol/mod.rs`, `src/skill_vfs/zhixing/resources/mod.rs`,
`src/skill_vfs/internal_manifest/mod.rs`,
`src/skill_vfs/resolver/runtime.rs`, or
`src/zhenfa_router/native/semantic_check/docs_governance/mod.rs`. Their old
source-resident `tests.rs` files and `tests/` trees now mount from canonical
`tests/unit/...` locations, and the `pybindings` branch is now explicitly
validated under `--features pybindings` instead of being treated as invisible
default-lib debt. The next milestone is therefore the now-fronting
`gateway/studio/*` and `gateway/openapi/*` tree.

That first `gateway/studio` milestone has landed too. The live Wendao gate no
longer reports `src/gateway/studio/vfs/mod.rs`,
`src/gateway/studio/vfs/{flight,flight_content,flight_scan,navigation}.rs`,
`src/gateway/studio/pathing.rs`,
`src/gateway/studio/types/config.rs`,
`src/gateway/studio/types/search_index/mod.rs`,
`src/gateway/studio/analysis/markdown/compile/mod.rs`,
`src/gateway/studio/symbol_index/state/mod.rs`,
`src/gateway/studio/search/project_scope.rs`, or
`src/gateway/studio/search/definition/mod.rs`. The old source-resident
`tests.rs` and `tests/` trees for that front now mount from canonical
`tests/unit/gateway/studio/...` locations, and the stale dead surface
`tests/unit/gateway/studio/vfs.rs` has been retired instead of kept alive as a
parallel compatibility path. The next milestone therefore starts at the now
fronting `gateway/studio/search/handlers/*`, `gateway/studio/router/*`,
`gateway/studio/startup_health/*`, and `gateway/openapi/*` tree.

That gateway control-plane milestone has landed too. The live Wendao gate no
longer reports `src/gateway/studio/mod.rs`,
`src/gateway/studio/startup_health/{mod,probe}.rs`,
`src/gateway/studio/router/mod.rs`,
`src/gateway/studio/router/config/mod.rs`,
`src/gateway/studio/router/state/mod.rs`,
`src/gateway/studio/router/{sanitization,retrieval_arrow}.rs`,
`src/gateway/openapi/document.rs`, or
`src/gateway/openapi/paths/shared/mod.rs`. Their old source-resident
`tests.rs` files, the `src/gateway/studio/router/tests/*` tree, and the
source-owned `src/gateway/studio/test_support.rs` helper now mount from
canonical `tests/unit/...` locations instead. The next milestone therefore
starts directly at the remaining live front: `gateway/studio/search/handlers/*`,
`gateway/studio/perf_support/mod.rs`, and the deeper
`gateway/studio/router/handlers/*` tree.

That search-handlers plus perf-support milestone has landed too. The live
Wendao gate no longer reports any owner from
`src/gateway/studio/search/handlers/*` or
`src/gateway/studio/perf_support/mod.rs`. The old source-resident
`handlers/tests/*`, `handlers/flight/tests/*`, `handlers/test_prelude.rs`,
`handlers/ast/{http,tests}.rs`, `handlers/attachments/tests.rs`,
`handlers/code_search/search/*`, and `perf_support/tests.rs` now mount from
canonical `tests/unit/gateway/studio/...` locations, and the remaining inline
`flight/repo_search.rs` plus `knowledge/intent/flight.rs` suites were
externalized beside them. The next milestone therefore starts directly at the
remaining `gateway/studio/router/handlers/*` tree.

That final `gateway/studio/router/handlers/*` milestone has landed too. The
old source-resident `graph/tests/*` tree and
`repo/analysis/search/tests.rs` now mount from canonical
`tests/unit/gateway/studio/router/handlers/...` locations, and the remaining
inline suites for `capabilities/deployment`, `graph/{flight,topology_flight}`,
`repo/parse`, `repo/analysis/{flight,overview_flight,index_status_flight,projected_page_index_tree_flight,refine_doc_flight,sync_flight}`,
and `repo/analysis/search/{import,service}` were externalized beside them.
Both live Wendao shared harnesses now pass, so `xiuxian-wendao` no longer
carries crate-local `xiuxian-testing-gate` debt for either `--lib` or
`--test unit_test`.

The next bounded consumer follow-up has landed too. `xiuxian-types` now
follows the same canonical split layout:
`src/lib.rs -> tests/unit/lib_policy.rs` for `cargo test --lib`, plus
`tests/unit_test.rs` as the root harness target. Its old root test files
`tests/skill_definition.rs` and `tests/test_scenarios.rs` were moved under
`tests/unit/`, so the crate no longer depends on root-level scattered test
files to satisfy the shared gate.

`xiuxian-zhenfa` now passes the same shared gate in all three relevant entry
points: `cargo test --lib`, `cargo test --test unit_test`, and
`cargo test --test integration_test`. The remaining test-only modules in
`src/transmuter/streaming/mod.rs` were collapsed into one canonical
`#[path = "../../../tests/unit/transmuter/streaming/mod.rs"] mod tests;`
mount, and the former source-resident helper implementations now live under
`tests/unit/transmuter/streaming/{arc_types_support,formatter_support}.rs`
instead of staying in `src/`.

The next bounded consumer milestone has landed too. `xiuxian-ast` now uses
the same canonical split harness shape:
`src/lib.rs -> tests/unit/lib_policy.rs` for `cargo test --lib`, plus
`tests/unit_test.rs`, `tests/integration_test.rs`, `tests/scenarios_test.rs`,
and `tests/performance_test.rs` as explicit root harness targets. Its old
scattered root test files were absorbed into canonical `tests/unit/`,
`tests/integration/`, and `tests/performance/` trees, the obsolete root
`tests/mod.rs` entrypoint is gone. The former Julia and Modelica source-backed
suites have since been retired entirely because the active Wendao lane now
owns those languages through native Julia routes instead of Rust-local AST
parsers. All five shared harness entrypoints now pass, and the crate
also clears `cargo clippy -p xiuxian-ast --lib --tests --all-features -- -D warnings`
without reintroducing source-resident test logic.

The next bounded consumer milestone then landed in `xiuxian-skills`. That
crate now follows the same canonical split harness shape:
`src/lib.rs -> tests/unit/lib_policy.rs` for the source-side gate, plus
`tests/unit_test.rs`, `tests/integration_test.rs`, and
`tests/performance_test.rs` as explicit root harness targets. The old root
test scatter was absorbed into canonical `tests/unit/`, `tests/integration/`,
and `tests/performance/` trees, the remaining source-resident test owners in
`src/knowledge/scanner/`, `src/skills/metadata/index/`,
`src/skills/resource/`, `src/skills/skill_command/{annotations,category}/`,
and `src/skills/scanner/references/` now mount canonical `tests/unit/...`
files, and the integration support helpers are mounted once at the
`integration_test.rs` root and consumed through `crate::...` paths instead of
duplicated local `mod` declarations. All shared harness entrypoints now pass,
and the crate also clears `cargo clippy -p xiuxian-skills --lib --tests -- -D warnings`
without leaving source-resident test logic behind.

The next bounded consumer milestone then landed in `xiuxian-qianji`. That
crate already had the source-side `--lib` harness in `src/lib.rs` and the
older `tests/scenarios_test.rs` root harness, but a large dormant slice still
lived under nested `tests/integration/*.rs` and `tests/unit/*.rs` files that
were not mounted by any canonical root target. The remediation added
`tests/integration_test.rs` and `tests/unit_test.rs`, attached the shared
`crate_test_policy_harness!()` macro to the dormant nested suites, and kept
the existing explicit `[[test]]` Cargo targets intact. Activating those
dormant suites exposed real drift in relative resource paths, nested support
imports, evolved `ContextAnnotator` and `NodeDefinition` contracts,
`PersonaRegistry` mutability, older mock `LlmClient` stubs, and dormant
clippy debt in both `flowhub` and the newly live suites; those issues were
fixed in the same milestone. The decisive outcome is that `xiuxian-qianji`
now passes the shared gate in `--lib`, `--test scenarios_test`,
`--test integration_test`, and `--test unit_test`, and it also clears
`cargo clippy -p xiuxian-qianji --lib --tests --features llm -- -D warnings`
without leaving dormant nested tests disconnected from the default crate
surface.

The next bounded consumer milestone then landed as a utility-pack cleanup
covering `xiuxian-config-core`, `xiuxian-event`, `xiuxian-executor`, and
`xiuxian-logging`. Each crate now follows the same canonical split harness
shape: `src/lib.rs -> tests/unit/lib_policy.rs` for the source-side gate plus
`tests/unit_test.rs` as the root harness target, with the former root-scatter
tests moved under `tests/unit/`. `xiuxian-config-core` also externalized the
remaining inline `resolve/precedence.rs` suite onto
`tests/unit/resolve/precedence.rs`, so no source-owned test body remains in
that crate. Activating `xiuxian-executor` exposed two stale historical test
files that referenced other crates entirely; those misplaced suites were
replaced with crate-local `ast_analyzer`, `command_analysis`, and
`nu_bridge` coverage that matches the package's current public API. The
decisive outcome is that all four crates now trigger and pass the shared gate
as part of their default `cargo test` surfaces, and the touched pack also
clears `cargo clippy -p xiuxian-config-core -p xiuxian-event -p xiuxian-executor -p xiuxian-logging --lib --tests -- -D warnings`.

The next bounded consumer milestone then landed as a small-consumer pack
covering `xiuxian-lance`, `xiuxian-memory`, `xiuxian-sandbox`,
`xiuxian-security`, and `xiuxian-window`. Each crate now follows the same
canonical split harness shape: `src/lib.rs -> tests/unit/lib_policy.rs` for
the source-side gate plus `tests/unit_test.rs` as the root harness target,
with the former root-scattered tests moved or collapsed under
`tests/unit/`. This wave removed the old root aggregators and helper wrappers
from those crates, including `xiuxian-lance/tests/mod.rs`,
`xiuxian-memory/tests/{core.rs,lib_unit.rs}`,
`xiuxian-security/tests/{mod.rs,sandbox_unit.rs,test_security.rs}`, and the
single-file roots in `xiuxian-sandbox` and `xiuxian-window`. The decisive
outcome is that all five crates now trigger and pass the shared gate as part
of their default `cargo test` surfaces, and the touched pack also clears
`cargo clippy -p xiuxian-lance -p xiuxian-memory -p xiuxian-sandbox -p xiuxian-security -p xiuxian-window --lib --tests -- -D warnings`.

The next bounded consumer milestone then landed as a small-consumer pack
covering `xiuxian-macros`, `xiuxian-tokenizer`, and `xiuxian-tags`. Each
crate now follows the same canonical split harness shape:
`src/lib.rs -> tests/unit/lib_policy.rs` for the source-side gate plus
`tests/unit_test.rs` as the root harness target, with the former
root-scattered tests absorbed into canonical `tests/unit/...` trees.
`xiuxian-tokenizer` no longer depends on `tests/mod.rs`, and the migrated
benchmark suite now points at `xiuxian_tokenizer::...` instead of the older
`omni_tokenizer::...` path drift. `xiuxian-macros` kept the existing
config-overlay coverage but refreshed the stale plaintext API-key expectation
to match the current merge contract, and its `bench_case!` smoke test was
stabilized so the harness does not depend on zero-versus-nonzero nanosecond
timing. `xiuxian-tags` promoted `patterns` to a real public surface and
cleared the newly live strict-clippy debt in `src/extractor.rs`, including
missing `# Errors` docs, `write!`-based formatting, `let ... else` cleanup,
and removing the old `unwrap()` path in directory search. The decisive
outcome is that all three crates now trigger and pass the shared gate as part
of both `cargo test --lib` and `cargo test --test unit_test`, and the touched
pack also clears
`cargo clippy -p xiuxian-macros -p xiuxian-tokenizer -p xiuxian-tags --lib --tests -- -D warnings`.

The next bounded consumer milestone then landed as a utility-pack cleanup
covering `xiuxian-io` and `xiuxian-edit`. At the time it also touched the
since-retired `xiuxian-tui` crate. The remaining utility crates now follow
the same canonical split harness shape:
`src/lib.rs -> tests/unit/lib_policy.rs` for the source-side gate plus
`tests/unit_test.rs` as the root harness target, with the former root test
scatter absorbed into canonical `tests/unit/...` trees. `xiuxian-io`
externalized the remaining `discover` inline suite and restored the real
library ownership that its tests already assumed by wiring `discover` and the
feature-gated `assembler` surface through `src/lib.rs`. `xiuxian-edit`
externalized the remaining `src/batch.rs` inline suite and cleared the newly
live strict-clippy debt across `batch`, `diff`, `editor`, and `types`
without suppressions. The decisive outcome is that both remaining utility
crates now trigger and pass the shared gate as part of their default
`cargo test` surfaces, and the touched pack also clears
`cargo clippy -p xiuxian-io -p xiuxian-edit --lib --tests -- -D warnings`.

The next bounded consumer milestone then landed in `xiuxian-memory-engine`.
That crate now follows the same canonical split harness shape:
`src/lib.rs -> tests/unit/lib_policy.rs` for the source-side gate plus
`tests/unit_test.rs` as the root harness target, with the former flat
root-scattered `tests/test_*.rs` suites absorbed into canonical
`tests/unit/...` files and mounted explicitly from the root harness. The old
shared helper under `tests/common/mod.rs` was moved to
`tests/unit/common/mod.rs`, and the migrated suites now consume it through
`crate::common` instead of local per-file `mod common;` declarations. The
decisive outcome is that `xiuxian-memory-engine` now triggers and passes the
shared gate as part of default `cargo test`, `cargo test --lib`, and
`cargo test --test unit_test`, while keeping the full migrated suite live,
and it also clears
`cargo clippy -p xiuxian-memory-engine --lib --tests -- -D warnings`.

The next bounded consumer milestone then landed as a Wendao-adjacent plugin
pack spanning `xiuxian-wendao-builtin`, `xiuxian-wendao-core`, and
`xiuxian-wendao-julia`. All three crates now mount the canonical
source-side harness in `src/lib.rs`, own explicit `tests/unit_test.rs` root
targets, and keep their remaining test bodies under canonical
`tests/unit/...` or `tests/integration/...` trees. The Modelica plugin
coverage now lives on the Julia-owned line under
`xiuxian-wendao-julia/tests/unit/plugin/` plus
`tests/unit/plugin/snapshots/`, proving the canonical layout works for both
the Julia plugin surface and the migrated Modelica parser-summary lane
without a separate Rust crate. While validating that pack, the newly exposed
strict-clippy debt was also closed in the dependency chain:
`xiuxian-wendao-core/tests/unit/artifacts/payload.rs` no longer uses
`expect()`, `xiuxian-wendao-runtime` now passes `doc_markdown`, and
`xiuxian-wendao` cleared the bounded `large_enum_variant` and warning blockers
that sat under the pack's `--all-features` clippy lane. The decisive outcome
is that the plugin pack now passes the shared gate as part of default
`cargo test` and clears
`cargo clippy -p xiuxian-wendao-core -p xiuxian-wendao-builtin -p xiuxian-wendao-julia --lib --tests --all-features -- -D warnings`.

The next bounded consumer milestone then landed as a runtime-and-orchestration
pack spanning `xiuxian-qianhuan`, `xiuxian-zhixing`, and `xiuxian-llm`. All
three crates now mount the canonical source-side harness in `src/lib.rs`, own
explicit `tests/unit_test.rs` root targets where needed, and keep their moved
coverage under canonical `tests/unit/...` trees. `xiuxian-zhixing`
externalized the remaining source-backed suites from `src/agenda/entry.rs`
and `src/heyi/reminders.rs`; because the stronger lane made the full migrated
suite run again, the same slice also closed real agenda-path drift by
restoring `TASK` query semantics, the heart-demon blocker check, and the
Wendao-backed metadata/task reindex invariants for agenda files.
`xiuxian-llm` externalized the remaining runtime/web suites from
`src/runtime/{bus,slot}.rs` and `src/web/mod.rs`, and the newly live strict-
clippy debt in the `OpenAI` `/responses` regression tests was closed in place
by switching the moved tests to `Result` and explicit error branches instead
of `expect()`. The dependency chain also stayed green under the stricter lane:
the touched Wendao duckdb/query test modules now gate feature-specific imports
correctly, and the touched Zhixing agenda-entry suite no longer depends on
strict float equality. The decisive outcome is that the pack now passes the
shared gate as part of default `cargo test`, and
`cargo clippy -p xiuxian-qianhuan -p xiuxian-zhixing -p xiuxian-llm -p xiuxian-wendao --lib --tests -- -D warnings`
is green.

The next bounded consumer milestone then landed in `xiuxian-wendao-runtime`.
The crate now mounts the canonical source-side harness in `src/lib.rs`, owns
explicit `tests/unit_test.rs` and `tests/unit/lib_policy.rs` entrypoints, and
keeps the former source-owned suites from `src/tests/**`,
`src/transport/query_contract/tests/**`,
`src/transport/plugin_arrow_exchange/tests.rs`,
`src/config/test_support.rs`, and the remaining inline `#[cfg(test)]` blocks
under canonical `tests/unit/...` trees. The stronger default-profile lane also
exposed drift in the existing `src/bin/wendao_flight_server.rs` surface; that
bin now gates its transport-only implementation inside the source file instead
of widening Cargo target metadata, so `cargo test --test unit_test` stays
green while `transport` and `--all-features` still exercise the real Flight
server. The decisive outcome is that `xiuxian-wendao-runtime` now passes the
shared gate as part of default `cargo test`, while
`cargo test -p xiuxian-wendao-runtime --all-features` and
`cargo clippy -p xiuxian-wendao-runtime --lib --tests --all-features -- -D warnings`
are green.

The next bounded consumer milestone then landed in `xiuxian-daochang`.
That crate now follows the same canonical split harness shape:
`src/lib.rs -> tests/unit/lib_policy.rs` for the source-side gate plus
`tests/unit_test.rs` as the explicit root harness target, with the former
root-scattered `tests/*.rs` surface absorbed under canonical `tests/unit/**`.
The remaining source-owned inline suites from
`src/jobs/manager/types.rs`,
`src/agent/tool_dispatch/helpers.rs`,
`src/agent/bootstrap/native_tools.rs`,
`src/agent/system_prompt_injection_state.rs`,
`src/runtime_agent_factory/tools.rs`,
`src/channels/telegram/channel/markdown/escape.rs`,
`src/channels/telegram/runtime/dispatch/preview.rs`, and
`src/tool_runtime/bridge.rs` were externalized onto canonical `#[path]`
mounts. Because the migrated root suite had several old integration-crate
assumptions baked into it, the lane also closed the real harness drift inside
the moved test tree: shared helper folders such as `agent/` and
`telegram_media_support/` now mount explicitly, the `session_redis` harness
now provides the minimum root aliases required by the source-backed session
context tests, and the moved system-prompt-injection roundtrip test now
matches the live `Option` serialization contract. Validation also surfaced two
bounded upstream compile drifts in `xiuxian-wendao` around the new
`ParquetQueryEngine` API, and the same wave closed them in place so the
consumer gate can execute without being blocked by dependency compile drift.
The decisive outcome is that `xiuxian-daochang` now passes the shared gate as
part of both `cargo test --lib` and `cargo test --test unit_test`, while the
remaining strict-clippy failures are broader pre-existing crate debt outside
the harness migration slice.

The first post-gate convergence slice then stayed inside `xiuxian-daochang`
instead of opening another consumer. That bounded follow-up closed the initial
strict-clippy cluster under `src/agent/feedback.rs` and
`src/test_support/{bootstrap,memory_recall_metrics,reflection}.rs`: the lane
restored the missing `# Errors` contract on `clear_session`, collapsed no-op
async helper methods back to synchronous helpers, removed the remaining
precision-loss casts from feedback and test-support metrics, marked the pure
bootstrap helpers as `#[must_use]`, and tightened the tiny reflection getter
to pass `self` by value. The decisive outcome is that those files no longer
appear on the strict-clippy failure surface; the remaining debt has moved on
to later owners such as `agent/system_prompt_injection_state`,
`agent/memory_recall_metrics`, `agent/reflection/lifecycle`, and the broader
telegram/discord runtime cluster.

The next bounded follow-up stayed on that same agent lane and closed the next
strict-clippy cluster under `src/agent/memory_recall_metrics.rs`,
`src/agent/reflection/lifecycle.rs`, and the source-backed
`tests/unit/agent/system_prompt_injection_state/mod.rs`. That slice removed
the remaining precision-loss conversion in recall metrics, changed the tiny
reflection lifecycle getter to pass `self` by value, and simplified the moved
XML normalization test literal so the source-backed suite no longer trips the
raw-string lint. The decisive outcome is that those owners also disappeared
from the strict-clippy failure surface, which has now moved forward into the
telegram/discord runtime and channel cluster.

The next bounded follow-up stayed inside `xiuxian-daochang` again and closed
the runtime/helper cluster under `tests/unit/agent/bootstrap/native_tools.rs`,
`src/agent/tool_dispatch/helpers.rs`,
`src/channels/discord/runtime/managed/replies/memory/runtime_status/json.rs`,
`src/channels/discord/runtime/managed/replies/memory/snapshot/{available,not_found}.rs`,
and `src/channels/discord/send.rs`. That slice aligned the native-tools
fixtures with the live internal-authority contract by using `SKILL.md`
frontmatter `tags`, switched the runtime-status JSON path to borrow the
snapshot instead of consuming it, and tightened the remaining small clone and
`#[must_use]` lint surfaces. The decisive outcome is that those owners no
longer appear on the post-gate strict-clippy failure surface.

The next bounded follow-up then closed the telegram channel utility cluster
under `src/channels/telegram/channel/{acl/settings,acl_overrides,chunking,error,group_policy,listen,outbound_text,recipient_admin}.rs`,
`src/channels/telegram/channel/markdown/markdown_v2.rs`, and
`src/channels/telegram/channel/send_api/{media,request}.rs`. That slice split
the largest ACL/polling helpers into smaller stages, derived the default
group-policy mode, tightened the text extraction and send-adapter helpers, and
removed the remaining clone-assignment and `map(...).unwrap_or_else(...)`
surfaces Clippy was still flagging. The decisive outcome is that the entire
telegram utility cluster disappeared from the strict-clippy head surface, so
the remaining post-gate debt now starts deeper in telegram session/admin
persistence and runtime loop-control code rather than in the basic channel
helper layer.
