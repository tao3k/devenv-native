# Contract Testing Program V1 Roadmap

:PROPERTIES:
:ID: xiuxian-testing-contract-program-v1
:PARENT: [[../index]]
:TAGS: roadmap, rollout, contracts
:STATUS: ACTIVE
:END:

## Mission

Move `xiuxian-testing` from shared helper crate to a reusable engineering-governance kernel without collapsing into a vague lint bundle or an LLM-only review tool.

## Phase 0: Docs Kernel and Research Tracker

Delivered in this pass:

- docs kernel established
- research tracker seeded with post-2024 papers
- V1 architecture and rule-pack specification documented
- lightweight docs contract test added

## Phase 1: Contract Kernel Implementation

Target:

- add `contracts` module
- define `ContractFinding`, `ContractReport`, `RulePack`, and report serialization
- support `strict`, `advisory`, and `research` modes

Acceptance:

- one internal consumer can build a `ContractReport`
- findings serialize to JSON and render as markdown summaries

## Phase 2: `rest_docs` Rule Pack

Target:

- collect route metadata, OpenAPI, and endpoint docs
- emit deterministic findings for missing or inconsistent contract surface

Suggested first consumer:

- `xiuxian-wendao` gateway

Acceptance:

- deterministic rules can flag missing docs, missing examples, and schema or status drift
- outputs remain stable in CI

## Phase 3: `modularity` Rule Pack

Target:

- collect Rust module graph and visibility metadata
- enforce repository-local `mod.rs` facade policy, visibility discipline, and public API error docs

Delivered seed in this pass:

- `ModularityRulePack` implemented in `xiuxian-testing` with deterministic checks for:
  - `MOD-R001`: non-interface logic inside `mod.rs` under the repository's
    facade-only house style
  - `MOD-R002`: broad `pub` visibility in internal module files
  - `MOD-R003`: public `Result` APIs missing `# Errors` docs
  - `MOD-R006`: large multi-responsibility Rust source files that should be
    split into dedicated module seams; the rule does not treat a higher number
    of focused sibling files as a problem
  - `MOD-R007`: folder-root modules that fan out into several sibling modules
    but still accumulate enough implementation logic to stop acting as a
    navigational TOC for coding agents
  - `MOD-R008`: root facades that re-export too many child-module symbols and
    become flat export lists instead of small curated entry surfaces
  - `MOD-R009`: multi-hop relative imports such as `super::super::...` that
    should use a crate-qualified path for a clearer ownership seam
  - `MOD-R010`: public alias re-exports in root facades that obscure the
    canonical child-module symbol name
  - `MOD-R011`: root seams with several child modules that provide no first-hop
    doc hint or visible entry export for coding agents
  - `MOD-R012`: root seams that visibly export from helper/detail child modules
    instead of canonical owner modules
  - `MOD-R013`: folder-root seams that expose child modules directly through
    visible `pub mod` or `pub(crate) mod` declarations instead of a curated
    root facade
  - `MOD-R014`: doc-only root seams whose `//!` hint never names any declared
    child module, leaving the first leaf hop ambiguous for coding agents
  - `MOD-R015`: root seams whose visible entry exports span several peer child
    modules without a dominant source module or named primary owner
  - `MOD-R016`: root seams whose `//!` doc names one owner module while the
    visible entry surface only exports from different child modules
  - `MOD-R017`: root seams whose doc-named owner appears in the visible seam
    but another child module silently becomes the dominant visible owner
  - `MOD-R018`: internal root seams whose parent module is private or
    restricted but whose child-owner entry seam still uses plain `pub use`
  - `MOD-R019`: internal root seams whose visible entry surface still spans
    several child-owner modules instead of converging on one canonical owner
  - `MOD-R020`: internal root seams that already expose one canonical visible
    owner but whose root doc still inventories every declared child module
  - `MOD-R021`: internal root seams that already expose one canonical visible
    owner but whose root doc mentions a secondary child module before that
    owner
  - `MOD-R022`: internal root seams that already expose one canonical visible
    owner but whose root doc still names more than one secondary child module
  - `MOD-R023`: internal root seams that already expose one canonical visible
    owner but whose remaining secondary child module is a helper/detail bucket
  - `MOD-R024`: internal root seams that already expose one canonical visible
    owner but whose root doc still hints that owner with alias wording instead
    of the real child-module path
- current bounded implementation note: `MOD-R001` now proves the `mod.rs`
  contract from native Rust syntax parsing inside `xiuxian-testing`, so
  visible module declarations, block-bodied inline modules, private `use`
  imports, glob re-exports, and syntax failures surface as deterministic
  findings without adding a new dependency edge to `xiuxian-ast`; `MOD-R006`
  complements that by warning on monolithic ownership sinks while remaining
  compatible with folder-first, sibling-friendly layouts that work well with
  coding agents; `MOD-R007` adds one root-module navigation check so feature
  folders can keep a small TOC seam instead of turning the root back into a
  sink; `MOD-R008` then keeps the same seam selective by warning on noisy
  top-level re-export surfaces; `MOD-R009` complements both by preferring
  crate-qualified imports over repeated `super` hops when describing the same
  in-crate ownership path; `MOD-R010` keeps the same root seam aligned with
  the canonical leaf symbol names instead of teaching coding agents a second
  public alias at the entry point; `MOD-R011` then asks the same root seam to
  provide at least one explicit first-hop signal when the folder has already
  split into several sibling modules; `MOD-R012` then keeps that first hop
  pointed at a canonical owner module instead of a helper/detail bucket; and
  `MOD-R013` keeps the folder-root seam itself curated by rejecting visible
  child-module declarations that would turn leaf module paths into the visible
  boundary; `MOD-R014` then keeps doc-only root seams actionable by asking the
  root `//!` hint to name at least one declared child module when no visible
  entry export exists; `MOD-R015` then keeps visible multi-owner entry seams
  focused by asking the root to identify one primary owner module instead of
  leaving coding agents with a small but still flat peer list; `MOD-R016` then
  keeps the doc-guided primary owner aligned with the visible entry seam so the
  root doc and root exports do not point coding agents at different owners;
  `MOD-R017` then keeps that partial alignment from drifting by asking the
  dominant visible owner to converge on the same owner module named in the
  root doc; and `MOD-R018` then keeps that same internal root seam from
  syntactically over-amplifying the owner surface with plain `pub use` when
  the parent module itself is still private or restricted; and `MOD-R019` then
  keeps the remaining restricted visible seam curated by preferring one
  canonical child-owner module instead of a small internal peer list; and
  `MOD-R020` then keeps the matching root doc from regressing into a prose
  mirror of the folder tree once the canonical visible owner is already clear;
  and `MOD-R021` then keeps the remaining root hint order aligned with that
  same owner so coding agents see the primary seam first in both code and doc;
  and `MOD-R022` then keeps the same hint under a tight secondary-seam budget
  so the root doc does not expand back into a mini directory summary; and
  `MOD-R023` then keeps the last remaining secondary seam canonical instead of
  steering coding agents into `internal/detail/helper` support buckets; and
  `MOD-R024` then keeps the canonical owner itself named with the real
  child-module path so coding agents do not have to translate alias wording
  back into the sibling tree

Suggested first consumers:

- `xiuxian-testing`
- one kernel crate such as `xiuxian-wendao`

Acceptance:

- findings can point to path-level architecture issues, not only local syntax warnings

## Phase 4: Wendao Knowledge Export

Target:

- export findings as ingestion-ready knowledge envelopes
- retain provenance, remediation, and examples

Acceptance:

- Wendao can index findings in a stable schema
- retrieval surfaces can answer "what engineering contracts are currently drifting?"

## Phase 5: Runtime and Review Feedback

Target:

- integrate runtime traces or invariants
- experiment with review-guided test targeting

This phase depends on V1 proving the base schema and deterministic rule flow.

## Current Downstream Consumer Evidence

`xiuxian-llm` is now a concrete downstream example of the kind of executable contract surface this
kernel should eventually formalize.

Current downstream DeepSeek OCR Metal short-form ladder under the shared `12 GB` guard:

- `0`
- `Telegram`
- `Telegram OCR`
- `sidecar health check`
- `Managed sidecar health check`

These gates are profile-backed, evidence-addressable, and already encode a narrow semantic
contract through `expected_substring`. The fourth gate combines a passing manual probe with an
observed successful profile-backed rerun, and the fifth has a direct file-backed profile pass.
The memory-line branch remains exploratory in the latest reruns and is not currently retained
under the same `12 GB` guard. These gates are not yet produced by `xiuxian-testing`,
but they are a live consumer signal for why the future contract kernel needs stable finding
schemas, profile-aware evidence capture, and docs-backed traceability.

Current non-retained follow-ups are also informative for the future contract kernel: a direct
`Hello from Telegram OCR.` line probe stayed within the same `12 GB` guard but fell into a long
low-RSS tail instead of converging quickly. A later title-line rerun for
`Omni OCR smoke test` passed `capfox` and stayed within the same `12 GB` guard, but also fell into
a long low-RSS tail (`~1.67 GB` for more than `100s`) without converging to a retained output.
Another short title-line variant (`.run/tmp/downstream_deepseek_metal_omni_line_probe_12g_v3.log`)
now reproduces a direct guard breach. For the memory-line branch, `max_new_tokens=3` still
reproduces the same guard breach, and later `max_new_tokens=2` reruns in the same snapshot also
regress to guard breaches.
Together these runs show that downstream evidence must distinguish semantic long-tail rejection
from ambient-capacity denial.

The runner surface is also now GPU-backend aware rather than Metal-only. `xiuxian-llm` can expose
the same ignored real-GPU harness as `test_real_metal_inference` or `test_real_cuda_inference`,
and the Python runner can request `--cuda` while reusing the same contract fields. That broadens
the future contract-kernel shape, but the retained downstream evidence in this workspace snapshot
is still Metal-backed because no local CUDA proof has been captured yet.

## Immediate Next Steps

Delivered in this pass:

1. `qianji` scenario-audit (`formal_audit`) is now a named contract-feedback gate.
2. `xiuxian-testing` contract packs are enforced through CI-visible gates.
3. `xiuxian-wendao` now exposes a stable bundled gateway `OpenAPI` artifact for clean `rest_docs` validation.
4. One persisted `qianji -> wendao` downstream proof now exists through
   `wendao_persisted_rest_docs_contract_feedback`, and the strengthened
   `xiuxian_wendao_contract_feedback_consumer.sh` gate covers both adapter
   mapping and sink persistence.

Remaining next steps:

1. Stabilize Wendao export ingestion around `ContractKnowledgeBatch` and define one retrieval query for active drift tracking.
2. Expand contract-pack CI adoption to one more kernel crate after `xiuxian-wendao`.

## Snapshot Governance Notes

- `xiuxian-testing::scenario::ScenarioFramework` now fails closed on duplicate scenario ids before
  snapshot assertion. This prevents accidental Insta snapshot collisions when two fixtures reuse the
  same `id`.
- Current verification evidence:
  - `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/xiuxian-testing-nextest cargo nextest run -p xiuxian-testing --lib --tests --no-fail-fast`
  - `direnv exec . bash scripts/rust/xiuxian_qianji_scenario_audit_contracts.sh`
  - `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/xiuxian-testing-nextest cargo nextest run -p xiuxian-testing --test contracts_kernel --test contracts_rest_docs --test contracts_modularity --test contracts_runner --test contracts_knowledge_export --test docs_kernel_contract --no-fail-fast`
  - `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/xiuxian-vector-nextest cargo nextest run -p xiuxian-vector --test xiuxian-testing-gate --no-fail-fast`
  - `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-live-openapi-gate bash scripts/rust/xiuxian_wendao_live_openapi_contract_feedback.sh`
  - `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-contract-feedback-consumer bash scripts/rust/xiuxian_wendao_contract_feedback_consumer.sh`
  - `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-contract-feedback-consumer cargo test -p xiuxian-qianji --test wendao_persisted_rest_docs_contract_feedback`
  - `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/xiuxian-testing-nextest cargo clippy -p xiuxian-testing --all-targets -- -D warnings`
  - `direnv exec . bash scripts/ci_scripts_smoke.sh`
  - `direnv exec . env CARGO_TARGET_DIR=.cache/cargo-target/wendao-scenarios-nextest cargo nextest run -p xiuxian-wendao --test scenarios_test --no-fail-fast`

## Exit Criteria for the V1 Design Stage

The design stage is complete when:

- the kernel schema is agreed and coded,
- the first two rule packs exist in advisory mode,
- at least one crate receives contract reports in CI,
- Wendao export shape is fixed enough for later ingestion work.
