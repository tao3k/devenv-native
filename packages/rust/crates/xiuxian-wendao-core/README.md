# xiuxian-wendao-core

Stable shared contracts for the Wendao package split.

## Responsibility

`xiuxian-wendao-core` owns contract surfaces that plugins, runtime helpers, and
host crates should be able to share without inheriting Wendao business logic or
host lifecycle behavior.

Current ownership:

- stable ids and selectors
- capability binding and version records
- artifact selectors, launch specs, and payload records
- stable knowledge payload records such as `KnowledgeEntry`
- pure contract-feedback projection helpers such as
  `WendaoContractFeedbackAdapter`
- stable entity and relation records such as `Entity` and `RelationType`
- stable link-graph refresh-mode contracts such as `LinkGraphRefreshMode`
- stable link-graph query contracts such as `LinkGraphSearchOptions`
- stable SQL result DTOs such as `SqlQueryPayload`
- stable semantic-document and cognitive-trace payload records
- stable semantic-scope metadata DTOs for runtime consumers that must not
  depend on Wendao parser or AST crates
- transport endpoint and transport kind descriptors
- stable repo-intelligence contract shapes
- stable `wendao://` resource URI parsing and normalization

## Non-Goals

`xiuxian-wendao-core` must not become a second host crate.

Do not place the following here:

- config file discovery or env resolution
- network clients or transport negotiation
- gateway or tool assembly
- language-specific runtime defaults
- knowledge-graph, retrieval, or storage behavior

## Selection Rule

If a type can be imported by a plugin crate and still make sense without
filesystem access, process lifecycle, or network behavior, it is a strong
candidate for `xiuxian-wendao-core`.

For the full three-package boundary matrix, see
[`../xiuxian-wendao/docs/06_roadmap/417_wendao_package_boundary_matrix.md`](../xiuxian-wendao/docs/06_roadmap/417_wendao_package_boundary_matrix.md).
