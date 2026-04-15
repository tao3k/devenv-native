# Rule-Pack Specification V1

:PROPERTIES:
:ID: xiuxian-testing-rulepack-spec-v1
:PARENT: [[../index]]
:TAGS: specification, rulepack, findings
:STATUS: ACTIVE
:END:

## Purpose

Define a stable specification for how `xiuxian-testing` rule packs declare rules, collect evidence, emit findings, and map severities into CI behavior.

## Rule-Pack Contract

Each rule pack should expose:

- a stable pack identifier
- a version
- a declared evidence scope
- a deterministic evaluation entrypoint
- an optional advisory heuristic entrypoint

Suggested shape:

```rust
pub trait RulePack {
    fn descriptor(&self) -> RulePackDescriptor;
    fn collect(&self, ctx: &CollectionContext) -> anyhow::Result<CollectedArtifacts>;
    fn evaluate(&self, artifacts: &CollectedArtifacts) -> anyhow::Result<Vec<ContractFinding>>;
}

pub struct RulePackDescriptor {
    pub id: &'static str,
    pub version: &'static str,
    pub domains: &'static [&'static str],
    pub default_mode: FindingMode,
}
```

Rule packs may optionally request an advisory audit pass after deterministic findings are produced. That execution should flow through the integration described in [[202_multi_role_audit_integration]].

## Finding Model

Every finding should be complete enough for:

- terminal output
- CI annotations
- markdown reports
- Wendao ingestion

Suggested shape:

```rust
pub struct ContractFinding {
    pub rule_id: String,
    pub pack_id: String,
    pub severity: FindingSeverity,
    pub mode: FindingMode,
    pub confidence: FindingConfidence,
    pub advisory_role_ids: Vec<String>,
    pub trace_ids: Vec<String>,
    pub title: String,
    pub summary: String,
    pub why_it_matters: String,
    pub remediation: String,
    pub evidence: Vec<FindingEvidence>,
    pub examples: FindingExamples,
    pub labels: BTreeMap<String, String>,
}
```

Recommended enums:

```rust
pub enum FindingSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

pub enum FindingMode {
    Deterministic,
    Advisory,
    Research,
}

pub enum FindingConfidence {
    High,
    Medium,
    Low,
}
```

Suggested companion request for packs that opt into advisory execution:

```rust
pub struct AdvisoryAuditPolicy {
    pub enabled: bool,
    pub requested_roles: Vec<String>,
    pub min_severity: FindingSeverity,
}
```

## Evidence Model

Evidence should stay explicit and path-addressable:

```rust
pub struct FindingEvidence {
    pub kind: EvidenceKind,
    pub path: Option<PathBuf>,
    pub locator: Option<String>,
    pub message: String,
}
```

Suggested evidence kinds:

- `SourceSpan`
- `OpenApiNode`
- `DocSection`
- `RuntimeTrace`
- `ScenarioSnapshot`
- `DerivedInvariant`

## V1 Rule Packs

### `rest_docs`

Scope:

- OpenAPI, route declarations, handlers, gateway docs, examples

Initial rule set:

- `REST-R001`: every externally reachable endpoint has a stable purpose description
- `REST-R002`: request schema and handler parameter surface are consistent
- `REST-R003`: documented success and error status codes match implementation behavior contracts
- `REST-R004`: error payloads use one documented envelope shape per service surface
- `REST-R005`: pagination and filtering semantics are documented when collection endpoints expose them
- `REST-R006`: mutation endpoints declare idempotency behavior or explicit non-idempotency
- `REST-R007`: at least one realistic example exists for non-trivial request bodies

### `modularity`

Scope:

- Rust module graph, visibility, crate boundaries, internal adapters

Initial rule set:

- `MOD-R001`: repository-local `mod.rs` facade policy remains interface-only
  Current bounded proof: parse top-level Rust items natively and accept only
  private module declarations without bodies plus explicit re-exports. Visible
  `pub mod` or `pub(crate) mod` declarations, inline module bodies, private
  `use` imports, glob re-exports, other implementation items, and syntax
  failures are rejected. This is a house style for this repository, not a
  claim about all Rust projects.
- `MOD-R002`: internal collaboration defaults to `pub(crate)` unless a public surface is required
- `MOD-R003`: forbidden cross-layer imports are rejected
- `MOD-R004`: public `Result` APIs include explicit `# Errors` documentation
- `MOD-R005`: gateway and adapter layers do not own domain logic that belongs in kernel modules
- `MOD-R006`: large multi-responsibility Rust source files should split into
  dedicated submodules or feature folders before one file becomes the ownership
  sink for types, runtime flow, and helper glue. The rule is intentionally
  sibling-friendly: more focused files inside a clear feature folder are a
  positive outcome because coding agents can infer ownership from the
  directory layout without loading one monolithic file.
- `MOD-R007`: folder-root Rust modules should stay as a small table of
  contents or facade once they already fan out into several child modules.
  The rule warns only when a root file such as `feature.rs` both declares a
  clear sibling-module fan-out and still accumulates enough implementation
  code to stop serving as a navigational seam for coding agents.
- `MOD-R008`: root facades should keep a curated re-export surface. The rule
  warns when a root file such as `feature.rs` or `mod.rs` forwards too many
  child-module symbols at the top level, because coding agents then lose a
  clear primary seam and see a flat export list instead of a small entry
  surface.
- `MOD-R009`: prefer `crate::` over multi-hop relative imports such as
  `super::super::...` in Rust `use` items. This is a repository-local clarity
  policy for coding-agent navigation rather than a universal Rust rule.
- `MOD-R010`: root facades should avoid public alias re-exports such as
  `pub use self::parser::Parser as FeatureParser;` when the canonical owner
  name is already clear. This keeps the entry seam aligned with the leaf owner
  path that coding agents will inspect next.
- `MOD-R011`: root seams that fan out into several child modules should offer
  at least one first-hop hint for coding agents: either a short root-level
  `//!` doc or a small visible re-export. This is a repository-local navigation
  policy, not a universal Rust style rule.
- `MOD-R012`: root seams should not point visible entry exports at child modules
  named like helper buckets such as `internal`, `detail`, or `helpers`. This
  keeps the first hop aligned with the canonical owner module rather than a
  support bucket.
- `MOD-R013`: folder-root seams should keep child module declarations private.
  The rule warns on visible declarations such as `pub mod service;` or
  `pub(crate) mod parser;` in root facades like `feature.rs`, because the root
  seam should expose canonical entry re-exports or a short root hint instead
  of turning child module paths into the visible contract. This is a
  repository-local coding-agent navigation policy rather than a universal Rust
  rule.
- `MOD-R014`: when a root seam uses only a `//!` doc as the first-hop hint and
  exposes no visible entry re-export, that doc should still name at least one
  declared child module. This keeps a doc-only seam actionable for coding
  agents instead of leaving the first leaf hop ambiguous. This is a
  repository-local coding-agent navigation policy rather than a universal Rust
  rule.
- `MOD-R015`: when a root seam exposes visible entry re-exports from several
  child modules, it should still identify one primary owner. The rule warns
  when the visible entry surface is spread across peer child modules without a
  dominant source module and without a root `//!` doc naming the primary
  owner. This keeps the root seam from reading like a small but still flat
  peer list to coding agents. This is a repository-local coding-agent
  navigation policy rather than a universal Rust rule.
- `MOD-R016`: when the root `//!` doc names one or more declared child modules
  as the starting owner hint, the visible entry seam should expose at least one
  entry from one of those same modules. The rule warns when the doc and visible
  entry surface point at disjoint owner modules, because that sends conflicting
  first-hop signals to coding agents. This is a repository-local coding-agent
  navigation policy rather than a universal Rust rule.
- `MOD-R017`: when the root `//!` doc and visible entry seam already overlap on
  an owner module, the dominant visible owner should still converge on that
  same module. The rule warns when the doc-named owner is present but another
  child module silently becomes the dominant visible owner, because that leaves
  coding agents with mixed primary-owner signals. This is a repository-local
  coding-agent navigation policy rather than a universal Rust rule.
- `MOD-R018`: when a root seam is still internal because its parent declares it
  as `mod feature;` or `pub(crate) mod feature;`, plain `pub use` child-module
  entries should usually stay restricted as `pub(crate)` or `pub(super)`. The
  rule warns when an internal root seam still uses plain `pub use`, because
  that makes the local entry surface look wider than the parent module
  declaration that actually owns it. This is a repository-local visibility
  hygiene policy for coding-agent-readable root seams rather than a universal
  Rust rule.
- `MOD-R019`: when a root seam is internal because its parent is still private
  or restricted, the visible entry surface should usually converge on one
  canonical child-owner module rather than re-exporting several peer owners.
  The rule warns when an internal root seam still exposes visible entries from
  multiple child modules, because Codex can already use the folder layout and a
  short root doc to find secondary leaves. This is a repository-local
  coding-agent seam-curation policy rather than a universal Rust rule.
- `MOD-R020`: when an internal root seam already exposes one canonical visible
  owner, the root `//!` doc should not restate the entire child-module set as a
  prose inventory. The rule warns when that doc names every declared child
  module even though the canonical owner is already visible, because Codex can
  read the sibling file tree directly. This is a repository-local coding-agent
  doc-curation policy rather than a universal Rust rule.
- `MOD-R021`: when an internal root seam already exposes one canonical visible
  owner and the root `//!` doc still mentions secondary seams, that canonical
  owner should be named first. The rule warns when a secondary child module is
  mentioned before the canonical visible owner, because coding agents often
  treat the first named module as the first hop. This is a repository-local
  coding-agent doc-priority policy rather than a universal Rust rule.
- `MOD-R022`: when an internal root seam already exposes one canonical visible
  owner and the root `//!` doc mentions secondary seams, that doc should stay
  within a very small secondary-seam budget. The rule warns when more than one
  secondary child module is named, because coding agents can read the sibling
  file tree directly and do not need a mini directory summary in the root doc.
  This is a repository-local coding-agent doc-budget policy rather than a
  universal Rust rule.
- `MOD-R023`: when an internal root seam already exposes one canonical visible
  owner and the root `//!` doc still keeps one secondary seam, that secondary
  seam should still be a canonical feature seam rather than `internal`,
  `detail`, or `helper` buckets. The rule warns when the remaining secondary
  seam names one of those support buckets, because coding agents can be pulled
  into the wrong leaf owner path. This is a repository-local coding-agent
  secondary-owner policy rather than a universal Rust rule.
- `MOD-R024`: when an internal root seam already exposes one canonical visible
  owner and the root `//!` doc uses inline-code owner wording, that wording
  should still name the real child module path rather than an alias or
  marketing label. The rule warns when the root hint says something like
  `FeatureService` instead of `service`, because coding agents should not have
  to translate prose aliases back into the sibling tree. This is a
  repository-local coding-agent owner-name policy rather than a universal Rust
  rule.

### `knowledge_feedback`

Scope:

- exported findings, remediation examples, Wendao ingestion payloads

Initial rule set:

- `KNOW-R001`: every error-level finding contains actionable remediation
- `KNOW-R002`: every exported finding includes source provenance
- `KNOW-R003`: advisory findings include confidence metadata
- `KNOW-R004`: pack outputs are serializable into Wendao-facing knowledge envelopes

### `multi_role_audit`

Scope:

- role execution metadata, trace attribution, advisory audit provenance

Initial rule set:

- `AUDIT-R001`: advisory findings retain the originating `role_id`
- `AUDIT-R002`: advisory findings that influence remediation include a linked `trace_id`
- `AUDIT-R003`: cognitive-drift or early-halt conditions from audit runs are surfaced in the final report
- `AUDIT-R004`: role-based critique is never promoted to deterministic pass/fail without an explicit deterministic rule backing it

## Severity Policy

V1 should treat severity as policy-configurable:

- `Info`: record only
- `Warning`: advisory by default, optionally gate in strict mode
- `Error`: fail in strict mode
- `Critical`: fail in all non-research modes

This allows frontier rule packs to start in advisory mode and harden over time.

## Deterministic vs Advisory Split

A single rule pack may include both classes of rules, but they must remain distinguishable:

- deterministic rules decide on explicit structural evidence
- advisory rules interpret underspecified or natural-language-heavy evidence

Example:

- deterministic: endpoint exists in code but is missing from the OpenAPI document
- advisory: endpoint description is too vague for safe external use
- advisory multi-role: the `rest_contract_auditor` and `runtime_trace_reviewer` both flag a suspicious contract, but the final severity remains policy-driven until deterministic evidence confirms the drift

The split must be preserved in the report schema so teams can gate only on the signals they trust.

## Reporting Contract

V1 reports should support:

- summary counts by severity and pack
- grouped findings
- stable machine-readable identifiers
- reproducible file references
- export to markdown and JSON

Recommended top-level JSON shape:

```json
{
  "suite_id": "xiuxian-testing-contracts",
  "generated_at": "2026-03-17T00:00:00Z",
  "findings": [],
  "stats": {
    "critical": 0,
    "error": 0,
    "warning": 0,
    "info": 0
  }
}
```

## Minimum V1 Pilot

Pilot the architecture in two places:

1. `xiuxian-wendao` gateway surface for `rest_docs`
2. `xiuxian-testing` and one Rust kernel crate for `modularity`
3. `qianji + qianhuan + zhenfa + wendao` as the advisory execution lane for selected `rest_docs` findings

This keeps the first implementation focused while still proving the full contract path from code and docs to findings and knowledge export.
