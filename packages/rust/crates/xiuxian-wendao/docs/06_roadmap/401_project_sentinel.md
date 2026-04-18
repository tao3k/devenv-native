# Project Sentinel: Semantic Consistency

:PROPERTIES:
:ID: wendao-sentinel
:PARENT: [[index|Wendao DocOS Kernel: Map of Content]]
:TAGS: roadmap, auditing, sentinel
:STATUS: ACTIVE
:END:

## Current Status (v2.25)

Project Sentinel has transitioned from a vision to a functional governance kernel.

### Completed Features

- **Landed**: Native Rust Auditor (`wendao audit`).
- **Landed**: Non-invasive Code Observation via `xiuxian-ast`.
- **Landed**: Global ID Registry & Dead-link detection.
- **Landed**: Fuzzy Pattern Suggestion (v2.9).
  - If an `:OBSERVE:` pattern fails validation, the auditor searches for renamed or moved symbols.
  - Pattern skeleton extraction for structural similarity scoring
  - `xiuxian-ast` for source code scanning
  - Confidence threshold filtering (0.65 by default, configurable)
  - Generates replacement drawer content for batch application
- **Landed**: Integration tests pass, all functionality works correctly.
  - Unit tests for `fuzzy_suggest` module
  - Tests for pattern skeleton extraction, similarity scoring, suggestion finding
  - Tests for `semantic_check` integration
  - XML output with `<fuzzy_suggestion>` elements
- **Landed**: Configurable Confidence Thresholds (v2.9).
  - `fuzzy_confidence_threshold` parameter in `wendao.semantic_check`
  - Range: 0.0 - 1.0 (default: 0.65)
  - Lower values suggest more matches, higher values are more strict
- **Landed**: Audit Bridge for Batch Fixes (v2.9).
  - `audit_bridge.rs` module for qianji integration
  - `BatchFix` struct for batch repair operations
  - `AuditBridge` trait for extensibility
  - `generate_batch_fixes()` function for external tool integration
- **Landed**: Package-Local Docs Governance (v2.19).
  - `semantic_check` now audits package-local crate docs under `packages/rust/crates/*/docs/`
  - top-level property-drawer `:ID:` values are now checked for opaque hash-shaped identity instead of human-readable title-like IDs
  - the fix pipeline now emits surgical remediations for non-opaque top-level `:ID:` values and missing `:ID:` lines inside the first property drawer
  - workspace-level docs governance now also reports `missing_package_docs_index` when an existing package-local `docs/` tree lacks `docs/index.md`
  - workspace-level docs governance now also reports `missing_package_docs_tree` as a warning when a Rust crate has no package-local `docs/` tree at all
  - the same fix pipeline can bootstrap that missing docs tree by creating `docs/index.md` and its parent directory
  - the fix pipeline now materializes that missing `docs/index.md` file through the same `wendao fix` path using explicit file-creation operations
  - workspace-level docs governance now also reports `missing_package_docs_section_landing` when a crate has `docs/index.md` but is still missing the standard landing pages for `01_core`, `03_features`, `05_research`, or `06_roadmap`
  - the same create-file fix path can materialize those standard section landing pages without rewriting the existing `docs/index.md`
  - workspace-level docs governance now also reports `missing_package_docs_index_section_link` when `docs/index.md` is missing the standard section link for an already-existing landing page
  - the fix pipeline now repairs those missing package-index section links with surgical insertions instead of rebuilding the full package index
  - workspace-level docs governance now also reports `missing_package_docs_index_relations_block` when `docs/index.md` already has body links but no bottom `:RELATIONS: :LINKS:` block yet
  - the fix pipeline now inserts that missing package-index relations block surgically, typically before the footer or trailing separator instead of rebuilding the full index page
  - workspace-level docs governance now also reports `missing_package_docs_index_footer_block` when `docs/index.md` already has a bottom `:RELATIONS: :LINKS:` block but still lacks the trailing footer metadata block
  - the fix pipeline now inserts that missing package-index footer block surgically at EOF, and generated package indexes now emit the footer metadata block by default so the bootstrap path stays coherent
  - workspace-level docs governance now also reports `incomplete_package_docs_index_footer_block` when `docs/index.md` already has a footer block but is still missing required fields such as `:STANDARDS:` or `:LAST_SYNC:`
  - the fix pipeline now normalizes that incomplete package-index footer block with a surgical block rewrite instead of rebuilding the full index page
  - workspace-level docs governance now also reports `stale_package_docs_index_footer_standards` when `docs/index.md` still carries an older `:STANDARDS:` value such as `v1.0` inside an otherwise complete footer block
  - the fix pipeline now normalizes that stale footer standards version with a surgical footer-block rewrite while preserving the current `:LAST_SYNC:` value
  - workspace-level docs governance now also reports `missing_package_docs_index_relation_link` when `docs/index.md` body links have drifted ahead of the bottom `:RELATIONS: :LINKS:` block
  - the fix pipeline now repairs those stale relation links with a surgical rewrite of the `:LINKS:` value instead of rebuilding the full index page
  - workspace-level docs governance now also reports `stale_package_docs_index_relation_link` when `docs/index.md` still carries extra relation links that no longer exist in the index body
  - the fix pipeline now removes those stale extra relation links with the same surgical rewrite path instead of rebuilding the full index page
  - canonical-doc governance now also reports `canonical_doc_hidden_path_link` when package docs, RFCs, roadmap notes, READMEs, or other canonical docs link hidden workspace paths such as `.data/*`, `.cache/*`, or `.run/*`
  - the same canonical-doc rule explicitly leaves GTD, ExecPlans, and similar task-tracking records outside that warning surface so operational continuity notes do not pollute the persistent documentation audit
  - CLI fix flow now routes through `AtomicFixBatch`, resolves doc IDs to physical paths before application, and honors `--issue-type` filtering in the touched path
  - the fix pipeline now also seeds explicit workspace doc targets into `run_audit_core`, so docs-governance issues emitted from workspace scans can still generate surgical fixes when `wendao fix` is aimed at a physical package-local docs file
  - workspace-level docs governance now also recurses through package-local docs trees and emits `doc_identity_protocol` for nested package docs instead of only enforcing opaque `:ID:` values during explicit single-file audits
  - the same workspace-issued package-doc identity findings are now seeded back into `file_contents`, so the existing surgical fix path remains available when the audit scope is a package docs directory instead of one explicit markdown file
  - package-doc scope matching now uses path-aware ancestor/descendant checks instead of raw substring matching, which prevents cross-crate bleed between similarly prefixed scopes such as `xiuxian-wendao/docs` and `xiuxian-wendao-modelica/docs`
  - package-doc directory scopes now also have end-to-end remediation coverage: a single audit/fix batch can rewrite multiple nested `doc_identity_protocol` violations across one package docs tree, instead of requiring repeated single-file fixes
  - missing package-index section-link remediation now recognizes decorated standard section headings such as `## 01_core: ...` and falls back to inserting new section blocks before `:RELATIONS:` or footer metadata instead of appending them after the footer
  - audit summary/file-report rendering now deduplicates alias doc paths for the same physical package-local docs file, so explicit workspace audits no longer show separate absolute and relative file entries for a single `docs/index.md`
  - explicit package-doc audits now also execute direct document-governance rules for the requested file even when it is outside the page-index tree set, so `doc_identity_protocol` is enforced consistently on package-local `docs/index.md`
  - that hardened fix path has already been exercised against real package-local docs: `xiuxian-llm/docs/index.md` and `xiuxian-testing/docs/index.md` now self-remediate through `wendao fix` and both package indexes audit clean after the generated section-landings, section-links, relations blocks, footer fixes, and opaque top-level `:ID:` repairs are applied
  - real package-doc directory audits now prove the widened coverage boundary: `wendao audit packages/rust/crates/xiuxian-wendao/docs` reports the expected legacy package-doc identity errors, while `wendao audit packages/rust/crates/xiuxian-wendao-modelica/docs` stays isolated and reports zero top-level package-doc identity errors

### Architecture (v2.9)

```text
┌─────────────────────────────────────────────────────────────────────┐
│                        Semantic Auditor                              │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐   ┌──────────────────┐   ┌────────────────────┐  │
│  │ Code         │   │ Fuzzy Suggester  │   │ Audit Bridge       │  │
│  │ Observations │──▶│ (fuzzy_suggest)  │──▶│ (audit_bridge)     │  │
│  │ (validate)   │   │                  │   │                    │  │
│  └──────────────┘   └──────────────────┘   └────────────────────┘  │
│         │                    │                        │             │
│         ▼                    ▼                        ▼             │
│  ┌──────────────┐   ┌──────────────────┐   ┌────────────────────┐  │
│  │ SemanticIssue│   │ FuzzySuggestion  │   │ BatchFix           │  │
│  │ (error/warn) │   │ (suggestion)     │   │ (for qianji)       │  │
│  └──────────────┘   └──────────────────┘   └────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### Usage

#### Basic Semantic Check

```python
result = wendao.semantic_check(
    doc="api-docs",  # Optional: check specific doc
    checks=["code_observations", "dead_links"],
    include_warnings=True,
)
```

#### With Fuzzy Pattern Suggestions

```python
result = wendao.semantic_check(
    doc="api-docs",
    source_paths=["src/lib.rs", "src/api/"],  # Scan for renamed symbols
    fuzzy_confidence_threshold=0.65,  # Optional: adjust threshold
)
# If patterns fail, suggestions are included in XML output:
# <fuzzy_suggestion>
#   <pattern>fn process_records($$$)</pattern>
#   <confidence>0.69</confidence>
#   <source_location>src/lib.rs:42</source_location>
#   <replacement_drawer>:OBSERVE: lang:rust "fn process_records($$$)"</replacement_drawer>
# </fuzzy_suggestion>
```

#### Batch Fix Generation

```python
# Get issues with fuzzy suggestions
issues = wendao.semantic_check(doc="api-docs", source_paths=["src/"])

# Generate batch fixes for qianji
fixes = audit.generate_batch_fixes(issues)
for fix in fixes:
    print(f"{fix.doc_path}:{fix.line_number}")
    print(f"  Replace: {fix.original_content}")
    print(f"  With: {fix.replacement}")
    print(f"  Confidence: {fix.confidence}")
```

## Future Evolution: v3.0 (Planned)

These features will enhance the auditing capabilities further:

1. **Better Source File Discovery**: Integrate with existing `dependency_indexer` or scan project root directory automatically
2. **Performance Caching**: Add caching layer for scan results to improve performance on large codebases
3. **Enhanced Snapshot Tests**: Add more snapshot tests for fuzzy suggestion edge cases
4. **Code Observation Parser**: Add `extract_skeleton()` helper method to CodeObservation for testing
5. **Multi-language Pattern Libraries**: Pre-built patterns for common frameworks and libraries
6. **Docs Scaffolding Governance**: Extend Sentinel beyond the current package-doc bootstrap slice so richer section page sets and deeper package-index footer/drawer normalization can be materialized automatically once the current footer-field and standards-version slices settle
