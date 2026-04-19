---
id: "20260315141000"
type: knowledge
title: "MOC: Qianji Orchestrator (The Brain)"
category: "architecture"
tags:
  - qianji
  - moc
  - index
saliency_base: 9.5
decay_rate: 0.01
metadata:
  title: "MOC: Qianji Orchestrator (The Brain)"
---

# MOC: Qianji Orchestrator (The Brain)

This is the **Map of Content** for the Qianji Orchestration engine. It serves as the primary gateway for all technical, theoretical, and operational knowledge related to the project.

## 1. Primary Entry Points (Permanent Specs)

- [[20260315140000-autonomous-audit-feature-v3]]: The native graph-driven Triple Loop.
- [[20260315142000-streaming-llm-executor-spec]]: High-precision streaming analyzer with live supervision.
- [[2026-04-07-qianji-runtime-config-layering]]: Runtime-config discovery, TOML-first precedence, and checkpoint Valkey ownership for the `qianji` CLI and embedding surfaces.
- [[qianji-spec]]: The fundamental LinkGraph and Consensus architecture (Global Ref).
- [[2026-04-07-qianji-flowhub-graph-contract-model-rfc]]: Flowhub flows, materialized plan work surfaces, scenario-as-guard-graph semantics, done-gate acceptance, CLI-first `show` / `check` control, validation-first diagnostics, and Wendao SQL for bounded exact-fragment retrieval.
- [[2026-04-18-qianji-research-workspace-layering-rfc]]: Research-layered workspace model with small localized runs plus persistent `papers/` and `topics/` knowledge packages.
- [[2026-04-07-wendao-sql-minimal-retrieval-surface-rfc]]: Minimal SQL retrieval surface for bounded plan work, including the `markdown` table, `skeleton` / `body` split, and Codex read-order guidance.
- [[2026-04-08-compact-validation-flowchart-alignment-rfc]]: Compact `[plan]` plus `[check]`, minimum `flowchart_alignment`, and the default `qianji show --dir` / `qianji check --dir` behavior for bounded plan work.
- [[2026-04-08-markdown-skeleton-minimal-rules-rfc]]: Minimum preservation, omission, and read-order rules for `markdown.skeleton` under `blueprint/` and `plan/`.
- [[2026-04-08-markdown-row-segmentation-minimal-rules-rfc]]: Stable row-model and minimum split rules for the `markdown` retrieval surface, including heading-based units and anti-fragmentation constraints.
- [[2026-04-08-flowchart-backbone-minimal-rules-rfc]]: Minimum visible `flowchart.mmd` backbone rules for principal surfaces, accepted compression, and obvious-conflict rejection under `qianji check`.
- [[2026-04-08-heading-path-minimal-conventions-rfc]]: Minimum external `heading_path` conventions for the `markdown` retrieval surface, including separator, ancestry order, root rows, and the relation to `path` plus `title`.
- [[2026-04-08-qianji-rfc-implementation-coverage-audit]]: Current code-backed implementation matrix for the active Flowhub and bounded-work RFC cluster, including implemented-core versus partial lanes and the remaining draft-exit gaps.
- `qianji` now includes a unified `show --dir` / `check --dir` implementation that covers both compact bounded work-surface contracts and real `qianji-flowhub` roots/modules, backed by dedicated `workdir` and `flowhub` library surfaces in `xiuxian-qianji`, with focused binary coverage for rendered output and invalid-check blocking behavior.
- `xiuxian-qianji` now keeps Flowhub module/scenario manifest parsing, hierarchical resolution, scenario preview/check, and the early library-only materialize core under one `flowhub` namespace, so the old `planning_source` term is no longer part of the runtime surface.
- The real `qianji-flowhub/` tree now mirrors the node graph through qianji.toml-only node anchors; checked-in `template/` and `validation/` surfaces are no longer part of the live Flowhub root and only remain in test-only materialize fixtures.
- The Flowhub root is now also anchored by a top-level `qianji.toml` `[contract]` block, so registered top-level nodes (`coding`, `rust`, `blueprint`, `plan`) and their required child manifests are declared once in the root contract rather than inferred from directory scans alone.
- Flowhub `[contract]` is now also the directory-allowlist surface: undeclared child directories under a leaf or composite node are treated as structural drift by `qianji check`.
- The main Flowhub planning RFC now formalizes the split that code already follows: `[contract]` is the primary structure contract, while `[[validation]]` is an optional secondary rule surface for additional checks.
- The main Flowhub planning RFC now also freezes the current `[contract]` grammar surface: relative-only `register`, relative `required`, duplicate rejection, and `*/...` expansion at the Flowhub root.
- The main Flowhub planning RFC now also defines the minimum markdown contract-diagnostic surface for `qianji check`: title, location, problem, why-it-blocks, and fix.
- `xiuxian-qianji` now keeps that markdown diagnostics skeleton shared across Flowhub root/module checks, Flowhub scenario checks, and bounded work-surface checks through one internal markdown renderer.
- `xiuxian-qianji` now also keeps the `qianji show` markdown surface shared across Flowhub root/module previews, Flowhub scenario previews, and bounded work-surface previews: one H1 title, metadata lines, and H2 sections for first-order surfaces.
- `xiuxian-qianji` now also parses immediate Mermaid scenario-case graphs owned by live Flowhub nodes through `merman-core`. Parser callers resolve `merimind_graph_name` from Flowhub-owned metadata in this order: `%% qianji.scenario.name`, then module-owned `[[graph]].name`, then the filename-stem fallback before handing the graph to `parse_mermaid_flowchart(...)`. `qianji check` classifies Mermaid labels matching live Flowhub module names as graph-module nodes while requiring coverage of every registered Flowhub module plus one connected module backbone. Undeclared graph-node labels now fail validation explicitly, and `qianji show --dir .../plan` exposes those scenario cases through explicit `Graph name: <merimind_graph_name>` and `Path: ./plan/<file>.mmd` fields.
- the shared control-plane markdown renderer path is now also deduplicated through a public `xiuxian-qianhuan` embedded-template catalog helper: `show`, `check`, Flowhub-root/module fragments, and Flowhub-scenario preview fragments all reuse one exported `qianhuan` template-catalog surface instead of each maintaining a separate local renderer bootstrap path inside `xiuxian-qianji`
- those `qianji` control-plane templates now also live as checked-in `.md.j2` files under `packages/rust/crates/xiuxian-qianji/resources/templates/control_plane/`, so the consumer crate keeps payload wiring while the actual template text leaves the Rust constant bodies
- `xiuxian-qianji` now also exposes `qianji show --graph <scenario.mmd>` for Flowhub-owned Mermaid scenario-case graphs. That surface now stays executor-facing but still bounded: it renders graph metadata together with `Execution`, `Nodes`, a compact `Check Surface`, an optional `Persistent Target Surface`, an optional `Done Gate`, and `Mermaid`. Parser callers still resolve `merimind_graph_name` before invoking `parse_mermaid_flowchart(...)`, so the parser only consumes the caller-owned graph identity together with node/edge structure and module-export alignment. The live graph contract is now dual-source: legacy module-owned `[[graph]]` / `[[graph.node]]` TOML still works, while enriched Mermaid graphs can instead own scenario metadata, node semantics, localized work-surface metadata, and completion gates through `%% qianji.*` annotations that compile into the same shared scenario IR. When a graph has no declared `workdir` contract, `show --graph` now keeps that gap visible instead of synthesizing a starter manifest in Rust. `qianji check --dir <workdir>` remains the localized-contract evaluation surface instead of re-evaluating raw Flowhub graphs directly.
- `qianji check --dir <workdir>` now also has a step-aware path for enriched Mermaid workdirs: when `flowchart.mmd` carries `%% qianji.*` annotations, the checker derives a static baseline from the scenario contract, adds only the current node checkpoint and writes from `state/current_node.toml`, and validates `state/allowed_next.json` against the current graph adjacency. Plain bounded workdirs still keep the older full-surface path.
- the main Flowhub RFC now also freezes the three-layer execution model for this lane: Codex is the execution layer, `qianji-flowhub` is the constraint layer, and `qianji check` is the evaluation layer over the localized workdir contract
- the new research-layered RFC now also freezes the higher-level research workspace split for this lane: localized runs stay intentionally small, persistent single-paper state lives under `papers/`, and persistent cross-paper synthesis lives under `topics/`
- the current implementation now treats `show --graph` as a bridge from Flowhub graph contract to localized executor materialization: graph metadata, execution summary, node semantics, declared check surface, optional persistent target preview, optional done-gate preview, and raw Mermaid
- `xiuxian-qianji` now also exposes `qianji show --anchor <module-qianji.toml> --scenario <ref> [--dir <run-dir>]` as the module-anchor bridge for Mermaid-enriched Flowhub scenarios. That surface resolves one declared scenario from the anchor `[[graph]]` list, reuses the same graph-contract brief as `show --graph`, and can overlay current-node plus allowed-next runtime state from a run dir even before that run dir fully passes `qianji check --dir`.
- `qianji check --dir <run-dir>` now also keeps missing or uninitialized localized run roots inside the bounded-work contract language: instead of failing with the generic unsupported-directory topology error, it renders a structured validation failure that tells the executor to materialize `qianji.toml`, `flowchart.mmd`, and the declared runtime surfaces first.
- `xiuxian-qianji` now also exposes `qianji materialize --anchor <module-qianji.toml> --scenario <ref> --dir <run-dir> [--current-node <node>]` as the run-bootstrap bridge for Mermaid-enriched Flowhub scenarios. That surface resolves one scenario from the anchor manifest, materializes the minimal localized run root declared by the compiled scenario contract, seeds `state/current_node.toml`, `state/trace.jsonl`, and `state/allowed_next.json` from either the graph start node or one selected current node, scaffolds only the selected current-step checkpoint/write surface, renders that scaffold in the materialize output, and immediately validates the generated run through the same step-aware `qianji check --dir` path.
- `xiuxian-qianji` now also exposes `qianji advance --dir <run-dir> --to <node>` as the guarded localized step-control surface for Mermaid-enriched Flowhub runs. That surface validates the current runtime state against Mermaid adjacency and `state/allowed_next.json`, rewrites `state/current_node.toml`, rewrites `state/allowed_next.json`, appends one bounded `step_advance` record to `state/trace.jsonl`, scaffolds only missing checkpoint/write placeholders for the new current node, and then re-validates the advanced run through the same step-aware `qianji check --dir` path without widening into patch synthesis or canonical merge behavior.
- the same RFC now also freezes the contract-owned node-semantics model for `show --graph`: `[[graph.node]]` entries define `kind`, `role`, and `agent_action`, while Rust keeps only generic undeclared-node diagnostics
- the same RFC now also freezes the v0 `Next` edge semantics for `show --graph`: backbone edges, fail edges, and repair-loop edges all remain flattened into one stable adjacency slot because the raw Mermaid surface is still rendered above them
- the same RFC now also freezes the wording boundary inside the `Nodes` section: `Role` stays descriptive, while `Agent action` stays imperative and bounded to the current work surface
- the same RFC now also freezes `unknown` node failure semantics for the graph lane: `unknown` stays visible in `show --graph`, becomes blocking drift in `qianji check`, and stays outside localized contract materialization guidance
- the same RFC now also freezes module and export alignment for `show --graph`: module nodes are anchored by Flowhub root `contract.register`, while export alignment stays bounded to module `entry` and `ready`
- the same RFC now also freezes graph path and naming for `show --graph`: `Name` is the caller-resolved `merimind_graph_name` from `[[graph]].name` or the filename-stem fallback, and `Path` points to the owning Mermaid file with repo-root-relative display when the graph lives under the active checkout
- the same RFC now also freezes the Mermaid consumption boundary for `show --graph`: the raw Mermaid block stays verbatim, while graph-contract semantics consume only first-order node labels plus directed adjacency rather than Mermaid presentation directives
- the current parser path now also code-backs that Mermaid boundary directly by delegating flowchart syntax acceptance to `merman-core`; repeated labels, subgraphs, directives, and expanded node-shape syntax stay Mermaid-owned, while Qianji still projects only direction plus first-order node and edge semantics and preserves the rendered `## Mermaid` block verbatim
- `xiuxian-wendao-sql` now owns the first code-backed bounded-work retrieval helper, while `xiuxian-wendao` keeps a compatibility re-export at `search::queries::sql::bounded_work_markdown`: the bounded owner scans only `blueprint/` and `plan/`, builds the RFC-minimum `markdown` rows (`path`, `surface`, `heading_path`, `title`, `level`, `skeleton`, `body`), registers them into a local DataFusion table, and also offers an opt-in bootstrap helper that creates a fresh query engine plus registers the local `markdown` table without yet widening into the global SQL surface collector.
- `xiuxian-wendao-sql` now also exposes `query_bounded_work_markdown_payload(...)`, and `xiuxian-qianji` now mirrors that surface through a thin `workdir` wrapper. Library callers can execute exact SQL retrieval over a bounded workdir without changing the current `qianji show --dir` / `qianji check --dir` CLI-only control plane.
- the same `workdir` library lane now also derives one default follow-up skeleton query from failing bounded work-surface diagnostics, so repair-oriented callers can fetch only the implicated `blueprint/` / `plan/` markdown context after `check_workdir(...)` reports blocking drift
- the user-facing bounded workdir `qianji check --dir ...` output now also appends that guidance as a `## Follow-up Query` section on failure, while Flowhub root/module and scenario check surfaces keep their existing markdown contract
- the Flowhub materialize lane now also consumes that follow-up workdir query surface when generated work surfaces fail validation, so materialize-time errors include both the blocking markdown diagnostics and one bounded SQL repair query
- `xiuxian-qianji` now also mounts the shared crate-test-policy source harness in `src/lib.rs`, and the previously inline source test modules in `src/bin/qianji.rs`, `src/contract_feedback/rest_docs.rs`, `src/executors/`, and `src/sovereign/` now live under `tests/unit/`. The current crate-test-policy harness for this crate is back to a passing state.
- The touched `show/check/materialize` test coverage now resolves the repository root through the shared `PRJ_ROOT`-aware helper in `xiuxian-config-core`, removing crate-local workspace-root guessing from this lane.

## 2. Theoretical Foundations (Research)

- [[20260315140500-autonomous-engineering-foundations]]: Literature notes on Agent-R, SpecLoop, and Process Supervision.

## 3. Atomic Concepts (Zettelkasten)

- `10_concepts/`: (Pending alchemical capture of Node/Edge semantics).

## 4. Execution Artifacts

- `40_artifacts/`: (Automated audit traces and Wendao ingestion logs).

---

## Linked Notes

- Parent: [[docs/01_core/qianji/PROTOTYPE_DESIGN]]
- Related: [[20260315151000-zhenfa-matrix-moc]]
- Depends on: [[xiuxian-llm]]
