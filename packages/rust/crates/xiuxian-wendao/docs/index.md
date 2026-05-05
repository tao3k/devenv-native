---
type: knowledge
kind: index
title: "Wendao DocOS Kernel Map of Content"
category: "documentation"
status: "active"
author: Xiuxian Artisan Workshop
date: 2026-05-05T00:00-07:00
description: "Canonical map of content for Wendao package documentation."
tags:
  - wendao
  - documentation
  - index
metadata:
  title: "Wendao DocOS Kernel Map of Content"
  retrieval:
    saliency_base: 7.0
    decay_rate: 0.03
---

# Wendao DocOS Kernel: Map of Content

:PROPERTIES:
:ID: wendao-moc
:TYPE: INDEX
:STATUS: ACTIVE
:END:

Standardized documentation repository for the Wendao DocOS Kernel, leveraging AST-based identity and structured properties.

## 📁 01_core: Architecture & Foundation

:PROPERTIES:
:ID: core-foundation
:OBSERVE: lang:rust "pub enum ThisDoesNotExistAnywhere { $$$ }"
:CONTRACT: must_contain("Id", "Path", "Hash")
:END:

- [[01_core/101_triple_a_protocol|Triple-A Addressing Protocol]]: Identity-based addressing.
- Atomic mutation: Reserved note slot for byte-level modification safety.
- [[01_core/103_package_layering|Wendao Package Layering]]: Package ownership rules for `core`, `runtime`, `wendao`, and plugin crates.

## 📁 02_parser: Parser Architecture

- [[02_parser/index.md|02_parser/index]]: Canonical parser namespace, parser-family matrix, and parser-vs-helper rules.
- [[02_parser/wikilinks.md|02_parser/wikilinks]]: Obsidian-aligned ordinary body wikilink parsing, source-span preservation, and `link_graph_refs` consumer boundaries.
- [[02_parser/relation_semantics.md|02_parser/relation_semantics]]: Scoped relation semantics for `PROPERTIES`, global `[[...]]` topology links, and parser-owned target grammar.

## 📁 03_features: Functional Ledger

:PROPERTIES:
:ID: functional-ledger
:OBSERVE: lang:rust "pub struct LinkGraphIndex { $$$ }"
:END:

- [[03_features/201_property_drawers|Property Drawers]]: Metadata management.
- Block addressing: Reserved note slot for paragraph-level granularity.
- [[03_features/203_agentic_navigation|Agentic Navigation (wendao.agentic_nav)]]: Reasoning-driven discovery.
- [[03_features/204_code_observation|Code Observation (:OBSERVE:)]]: Non-invasive sgrep binding.
- [[03_features/205_semantic_auditor|Semantic Auditor (wendao audit)]]: Native sentinel engine.
- [[03_features/206_openai_semantic_ignition|OpenAI-Compatible Semantic Ignition]]: OpenAI-compatible query ignition bridge.
- [[03_features/207_gateway_openapi_contract_surface|Gateway OpenAPI Contract Surface]]: Stable gateway OpenAPI contract surface for `rest_docs`.
- [[03_features/208_performance_gate_v1|Wendao Performance Gate V1]]: Feature-gated Wendao performance gate, stress lane, and Criterion analysis layer.
- [[03_features/209_datafusion_sql_query_surface|DataFusion SQL Query Surface]]: Request-scoped DataFusion SQL query surface, discovery catalogs, and snapshot contract.
- [[03_features/210_search_queries_architecture|Search Queries Architecture]]: Native Flight plus one shared queries system for SQL, FlightSQL, GraphQL, REST, and CLI query entrypoints.
- [[03_features/211_graphql_query_surface|GraphQL Query Surface]]: First DataFusion-aligned GraphQL table-query adapter over the shared SQL surface.
- [[03_features/212_flightsql_query_surface|FlightSQL Query Surface]]: First FlightSQL statement-query and sql-info adapter over the shared SQL surface.
- [[03_features/213_rest_query_surface|REST Query Surface]]: First thin REST-style request/response adapter over the shared query service.
- [[03_features/214_config_import_overlay|Config Import Overlay]]: Canonical import-based config merge model for Wendao runtime settings, gateway startup config, and legacy Studio overlay compatibility.
- [[03_features/215_vector_boundary_split|Wendao Vector Boundary Split]]: Explicit separation between lightweight Arrow/DataFusion substrate ownership and Lance-backed vector-store ownership.

## 📁 05_research: Theoretical Hardening

- [[05_research/301_research_papers|Research Index: Map of Content]]: Academic foundations.
- [[05_research/305_http_grpc_tower_performance_audit|HTTP, gRPC, and Tower Performance Audit]]: Audit of Wendao's Axum, Tonic, Tower, and Arrow Flight transport surfaces.
- [[05_research/306_pdf_hybrid_ocr_implementation_report|PDF Hybrid OCR Implementation Report]]: Precision-preserving PDF routing, page OCR sharding, and Rust-side document extraction acceleration plan.
- [[05_research/307_attachment_format_baseline|Attachment Format Precision And Speed Baseline]]: Non-PDF Docling fixture precision, structure-order, cache, and class-level latency evidence.
- [[05_research/308_document_extract_pr_closing_report|Document Extraction PR Closing Report]]: PDF OCR milestone guard, auto-scheduler live evidence, and PR-closing validation record.

## 📁 06_roadmap: Future Evolution

:PROPERTIES:
:ID: roadmap-sentinel
:OBSERVE: lang:rust "pub trait AuditBridge { $$$ }"
:CONTRACT: must_contain("generate_fixes", "apply_fixes")
:END:

- [[06_roadmap/401_project_sentinel|Project Sentinel: Semantic Consistency]]: Project Sentinel (Auditing).
- [[06_roadmap/402_repo_intelligence_mvp|Repo Intelligence MVP]]: Repo Intelligence common core and plugin API MVP.
- [[06_roadmap/403_document_projection_and_retrieval_enhancement|Document Projection and Retrieval Enhancement]]: Document projection and retrieval enhancement on top of Repo Intelligence.
- [[06_roadmap/404_repo_intelligence_for_sciml_and_msl|Repo Intelligence for SciML and MSL]]: SciML and MSL repo intelligence architecture and boundary mapping.
- [[06_roadmap/405_large_rust_modularization|Large Rust File Modularization]]: Lossless modularization plan for oversized Rust files in `xiuxian-wendao`.
- [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]]: `P0 / Mapping Gate` inventory for Julia-specific host surfaces and their target `core` / `runtime` / plugin-package ownership.
- [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]]: `P1` staging note for generic plugin capability, artifact, provider, and transport contracts.
- [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]]: First `P1` implementation slice plan with module tree, compatibility shims, and file touch order.
- [[06_roadmap/412_core_runtime_plugin_program|Wendao Core Runtime Plugin Program]]: Program-level execution entrypoint for the overall core/runtime/plugin migration.
- [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]]: First package list for the physical `xiuxian-wendao-core` extraction.
- [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]]: First package list for the physical `xiuxian-wendao-runtime` extraction.
- [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]]: First package list for Julia ownership externalization into `xiuxian-wendao-julia`.
- [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]]: Program ledger for compatibility surface retirement order, unlock phases, and target end states.
- [[06_roadmap/417_wendao_package_boundary_matrix|Wendao Package Boundary Matrix]]: Contributor-facing boundary matrix for `xiuxian-wendao-core`, `xiuxian-wendao-runtime`, and `xiuxian-wendao`.
- [[06_roadmap/418_julia_plugin_first_rollout|Julia Plugin-First Rollout]]: Julia-first plugin rollout for keeping thick Julia implementation inside `xiuxian-wendao-julia`.
- `src/compatibility/`: Explicit crate-root compatibility namespace for compat-first and legacy Julia migration paths.
- `docs/rfcs/2026-03-27-wendao-arrow-plugin-flight-rfc.md`: Arrow-first plugin protocol with Flight-first transport and Arrow IPC fallback.
- `docs/rfcs/2026-03-27-wendao-core-runtime-plugin-migration-rfc.md`: Complete migration path from monolithic Wendao ownership toward `core`, `runtime`, and independently published plugin packages.

Transient blueprint and ExecPlan tracking records are intentionally omitted
from this canonical index. Use the RFC and roadmap notes above as the stable
documentation surface.

:RELATIONS:
:LINKS: [[01_core/101_triple_a_protocol|Triple-A Addressing Protocol]], [[01_core/103_package_layering|Wendao Package Layering]], [[02_parser/index.md|02_parser/index]], [[02_parser/wikilinks.md|02_parser/wikilinks]], [[02_parser/relation_semantics.md|02_parser/relation_semantics]], [[03_features/201_property_drawers|Property Drawers]], [[03_features/203_agentic_navigation|Agentic Navigation (wendao.agentic_nav)]], [[03_features/204_code_observation|Code Observation (:OBSERVE:)]], [[03_features/205_semantic_auditor|Semantic Auditor (wendao audit)]], [[03_features/206_openai_semantic_ignition|OpenAI-Compatible Semantic Ignition]], [[03_features/207_gateway_openapi_contract_surface|Gateway OpenAPI Contract Surface]], [[03_features/208_performance_gate_v1|Wendao Performance Gate V1]], [[03_features/209_datafusion_sql_query_surface|DataFusion SQL Query Surface]], [[03_features/210_search_queries_architecture|Search Queries Architecture]], [[03_features/211_graphql_query_surface|GraphQL Query Surface]], [[03_features/212_flightsql_query_surface|FlightSQL Query Surface]], [[03_features/213_rest_query_surface|REST Query Surface]], [[03_features/214_config_import_overlay|Config Import Overlay]], [[03_features/215_vector_boundary_split|Wendao Vector Boundary Split]], [[05_research/301_research_papers|Research Index: Map of Content]], [[05_research/305_http_grpc_tower_performance_audit|HTTP, gRPC, and Tower Performance Audit]], [[05_research/306_pdf_hybrid_ocr_implementation_report|PDF Hybrid OCR Implementation Report]], [[05_research/307_attachment_format_baseline|Attachment Format Precision And Speed Baseline]], [[05_research/308_document_extract_pr_closing_report|Document Extraction PR Closing Report]], [[06_roadmap/401_project_sentinel|Project Sentinel: Semantic Consistency]], [[06_roadmap/402_repo_intelligence_mvp|Repo Intelligence MVP]], [[06_roadmap/403_document_projection_and_retrieval_enhancement|Document Projection and Retrieval Enhancement]], [[06_roadmap/404_repo_intelligence_for_sciml_and_msl|Repo Intelligence for SciML and MSL]], [[06_roadmap/405_large_rust_modularization|Large Rust File Modularization]], [[06_roadmap/409_core_runtime_plugin_surface_inventory|Wendao Core Runtime Plugin Surface Inventory]], [[06_roadmap/410_p1_generic_plugin_contract_staging|P1 Generic Plugin Contract Staging]], [[06_roadmap/411_p1_first_code_slice_plan|P1 First Code Slice Plan]], [[06_roadmap/412_core_runtime_plugin_program|Wendao Core Runtime Plugin Program]], [[06_roadmap/413_m2_core_extraction_package_list|M2 Core Extraction Package List]], [[06_roadmap/414_m3_runtime_extraction_package_list|M3 Runtime Extraction Package List]], [[06_roadmap/415_m4_julia_externalization_package_list|M4 Julia Externalization Package List]], [[06_roadmap/416_compatibility_retirement_ledger|Compatibility Retirement Ledger]], [[06_roadmap/417_wendao_package_boundary_matrix|Wendao Package Boundary Matrix]], [[06_roadmap/418_julia_plugin_first_rollout|Julia Plugin-First Rollout]]
:END:

---

:FOOTER:
:STANDARDS: v2.0
:LAST_SYNC: 2026-05-05
:END:
