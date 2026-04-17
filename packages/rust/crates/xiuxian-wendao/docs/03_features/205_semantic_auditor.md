# Semantic Auditor (wendao audit)

:PROPERTIES:
:ID: feat-semantic-auditor
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: feature, auditing, sentinel, integrity
:STATUS: STABLE
:VERSION: 2.25
:END:

## Overview

The Semantic Auditor is the core component of Project Sentinel. It performs deep structural and relational analysis across the entire knowledge base to identify "semantic rot".

## Check Categories

The auditor executes a multi-pass diagnostic flow:

1. **DeadLinks**: Verifies all `[[id]]` references.
2. **DeprecatedRefs**: Flags usage of nodes marked as `:STATUS: DEPRECATED`.
3. **IdCollisions**: Ensures global uniqueness of all manual `:ID:` entries.
4. **CodeObservations**: Validates `ast-grep` patterns against `xiuxian-ast`.
5. **HashAlignment**: Checks for content drift using content fingerprints.
6. **DocGovernance**: Validates package-local crate docs under `packages/rust/crates/*/docs/`, currently enforcing opaque top-level `:ID:` values inside the first property drawer for both explicit package-doc audits and workspace/package-doc tree scans, the presence of `docs/index.md` when a package-local docs tree already exists, a minimal docs-tree bootstrap path for crates that still have no package-local `docs/` directory, the presence of standard section landing pages once `docs/index.md` already exists, the presence of matching standard section links inside the package index once those landing pages exist, the presence of a package-index `:RELATIONS: :LINKS:` block when body links already exist, the presence of a package-index `:FOOTER:` block once the relations block exists, completeness of required footer fields once that footer block exists, normalization of stale footer standards versions, and coherence between package-index body links and that bottom relations block in both directions.

## Diagnostic Protocol (XML)

Designed for machine processing, the XML output provides exact byte ranges for automated remediation.

## Automatic Remediation

The Sentinel fix pipeline now supports one docs-governance remediation slice in addition to fuzzy code-observation fixes:

- top-level package-doc `:ID:` values that are human-readable or otherwise non-opaque can be rewritten to hash-shaped identifiers
- missing top-level `:ID:` lines inside an existing top property drawer can be inserted automatically
- missing package-local `docs/index.md` files can be created automatically when the package already has a `docs/` tree with section documents
- missing package-local `docs/` roots can be bootstrapped by materializing `docs/index.md` for a Rust crate that currently has no package-local docs tree
- missing standard package-doc section landing pages can be created automatically for `01_core`, `03_features`, `05_research`, and `06_roadmap` once a package-local `docs/index.md` is already present
- missing standard package-doc section links inside `docs/index.md` can be inserted automatically once the corresponding landing pages already exist
- a missing package-index `:RELATIONS: :LINKS:` block can be inserted automatically once the index body already contains links
- a missing package-index `:FOOTER:` block can be inserted automatically once the index already carries a bottom `:RELATIONS: :LINKS:` block
- an incomplete package-index `:FOOTER:` block can be normalized automatically when required fields such as `:STANDARDS:` or `:LAST_SYNC:` are missing
- a package-index `:FOOTER:` block with a stale `:STANDARDS:` value can be normalized automatically while preserving the current `:LAST_SYNC:` value
- stale package-index `:RELATIONS: :LINKS:` values can be surgically rewritten when the index body already carries links that the relations block omitted
- stale extra package-index `:RELATIONS: :LINKS:` values can be surgically removed when the relations block still references links that the index body no longer exposes

This remediation currently rides the existing `wendao fix` path. In-place drawer repair, missing package-index section links, missing package-index relations blocks, missing package-index footer blocks, incomplete package-index footer blocks, stale package-index footer standards values, missing package-index relation links, and stale extra package-index relation links use surgical byte-range fixes, while missing package indexes, missing package docs trees, and missing section landing pages are materialized through the same batch-fix pipeline as explicit file-creation operations. Newly generated package indexes now also emit a default footer metadata block so the bootstrap path stays aligned with the governance rule.

The fix path now also resolves explicit workspace doc paths reliably when docs-governance issues originate from workspace scans instead of page-index trees. Explicit audits of package-local docs now run direct document-governance rules even when the requested file is not present in the page-index tree set, so `wendao audit packages/rust/crates/.../docs/index.md` can surface and remediate non-opaque top-level `:ID:` values on package indexes themselves. Workspace-level docs-governance now also recurses through package-local docs trees and emits the same `doc_identity_protocol` issues for nested package docs, then seeds those issue documents back into `file_contents` so the same surgical fix path stays available when the audit scope is a package docs directory instead of a single file. Package-doc scope matching is now path-aware instead of plain substring matching, which closes false-positive bleed between similarly prefixed crate paths such as `xiuxian-wendao/docs` and `xiuxian-wendao-julia/docs`. Section-link remediation now recognizes decorated headings such as `## 01_core: Architecture and Foundation` instead of only exact bare section names, and when a section heading is genuinely absent it inserts the new block before the package-index relations/footer area instead of appending it after the footer. Audit reporting now also deduplicates alias doc paths when the same physical file is present through both relative and absolute keys, so `wendao audit` reports a single file entry and a single document count for package-local docs. That execution hardening was validated against real package-local docs by driving `wendao fix` end to end on `xiuxian-llm/docs/index.md` and `xiuxian-testing/docs/index.md` until both package indexes returned audit-clean reports with opaque top-level `:ID:` values, then re-checking package-doc directory scopes: `xiuxian-wendao/docs` now surfaces the expected legacy `ERR_DOC_IDENTITY_PROTOCOL` set across its package-doc tree, while `xiuxian-wendao-julia/docs` stays isolated and reports zero top-level doc-identity errors. The same widened execution path is now pinned by a dedicated end-to-end regression: a package-doc directory scope can audit, plan surgical fixes, and atomically rewrite multiple nested `doc_identity_protocol` violations in one batch instead of relying on repeated single-file `wendao fix` invocations.

:RELATIONS:
:LINKS: [[06_roadmap/401_project_sentinel|Project Sentinel: Semantic Consistency]], [[03_features/204_code_observation|Code Observation (:OBSERVE:)]]
:END:
