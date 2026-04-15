# Wendao Full Clippy Follow-On Cleanup

## Context

The shared test-curation and qianji warning-closure lanes have already pushed
the main validation driver forward:

- `direnv exec . cargo clippy -p xiuxian-qianji --all-targets --all-features -- -D warnings`

That command no longer stops on qianji-local debt first. It now exposes a
Wendao-owned follow-on blocker set spanning repo-index incremental planning,
search manifest/staging helpers, Studio gateway handler cleanup, and the
docs-native `zhenfa_tool` wrappers.

## Cleanup Rule

This roadmap intentionally separates two fronts:

1. low-risk cleanup debt that should be removed immediately
2. structural refactors that deserve their own bounded slices once the easy
   noise is gone

The first front includes:

- clone/style closure in manifest and staging helpers
- doc-markdown, default-construction, and collapsible-if cleanup in Studio
  gateway surfaces
- docs-native wrapper normalization where argument passing or option
  construction can be made clearer without changing the tool contract
- case-insensitive extension checks in repo-index incremental filters

The second front includes:

- `too_many_lines` functions in repo-index incremental planning
- `too_many_lines` planning functions in local symbol build
- multi-argument markdown metadata helpers that likely need a more structural
  payload object
- type-shape naming warnings where renaming propagates across more than one
  owner path

## Immediate Goal

The immediate slice should remove the low-risk blockers first, rerun the same
full qianji `clippy` command, and then record the remaining structural frontier
explicitly instead of widening silently.

That first reduction is now complete. The remaining frontier is only:

- `repo_index/state/coordinator/runtime/incremental/mod.rs`
  - `collect_safe_incremental_julia_files(...)`
  - `collect_safe_incremental_modelica_files(...)`
- `search/local_symbol/build/plan.rs`
  - `plan_local_symbol_build_with_scanned_files(...)`

The next slice is therefore purely structural helper extraction inside those
three functions.

That structural slice is now complete as well:

- `collect_safe_incremental_julia_files(...)` was decomposed into smaller
  Julia-specific change/fingerprint helpers
- `collect_safe_incremental_modelica_files(...)` was decomposed into
  owner-clear shape-detection, validation, and fingerprint helpers
- `plan_local_symbol_build_with_scanned_files(...)` was decomposed into
  explicit file-selection, evaluation, and plan-assembly helpers

With those refactors in place, the validation driver is green again:

- `direnv exec . cargo clippy -p xiuxian-qianji --all-targets --all-features -- -D warnings`

Focused follow-up proofs now split into two buckets:

- `local_symbol_build_` proofs are green after the plan decomposition
- `prepare_incremental_analysis_` proofs still expose one semantic Modelica
  fallback assertion and three Julia-backed parser-summary readiness timeouts,
  which are follow-on validation work rather than remaining clippy blockers
