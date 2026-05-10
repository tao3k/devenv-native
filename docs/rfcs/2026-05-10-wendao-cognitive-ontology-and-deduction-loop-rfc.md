---
type: knowledge
title: "RFC: Cognitive Ontology and BPMN Deduction Loop"
category: "rfc"
status: "draft"
authors:
  - auditor_neuron
  - sovereign
created: 2026-05-10
tags:
  - rfc
  - ontology
  - johnny-decimal
  - bpmn
  - agentic-reasoning
---

# Cognitive Ontology and BPMN Deduction Loop

## 1. Vision and Motivation

In the face of enterprise-scale, unstructured data, relying solely on Large Language Models (LLMs) for exhaustive reading leads to latency bottlenecks and hallucinations. This RFC establishes the cognitive architecture for `xiuxian-artisan-workshop`. By defining a Hierarchical Ontology Cascade organized via the Johnny.Decimal system, and combining it with a "Rust Parser + Episteme Priors + BPMN Validation" deductive loop, we achieve high-precision structural alignment over unknown enterprise data.

## 2. Directory Architecture: Pure Johnny.Decimal Ontology Mapping

To prevent structural entropy, the physical mapping of `wendao-episteme/ontology/` is rigidly anchored to a pure **Johnny.Decimal (JD)** methodology. The architecture relies on an authoritative `Index.md` to allocate logical areas, while physical directories utilize strict two-digit Category prefixes.

### Directory Structure Definition

```text
wendao-episteme/ontology/
├── Index.md                     # The absolute source of truth for Category allocation
│
│   # === AREA: L0 Foundation (Allocated Categories 00-09) ===
├── 00_Core_Primitives/          # The absolute philosophical base
│   ├── 00.01_entity.rdf
│   └── 00.02_relation.rdf
│
│   # === AREA: L1 Domain (Allocated Categories 10-49 for Community Ecosystem) ===
├── 10_Software_Engineering/     # Vertical: Software Engineering
│   └── 10.01_architecture.rdf
├── 20_Commercial/               # Vertical: Commerce & Logistics
│   └── 20.01_supply_chain.rdf
│
│   # === AREA: L2 Application (Allocated Categories 50-99 for Implicit/User Instances) ===
├── 50_Xiuxian_Internal/         # Internal project specific knowledge graph
│   └── 50.01_nexus_graph.rdf
└── 60_Customer_Alpha/           # Isolated tenant-specific implicit schema
    └── 60.01_legacy_system.rdf
```

## 3. The BPMN Deductive Loop (Implicit Schema Discovery)

For legacy systems lacking explicit schemas, the architecture employs a "Scan-Hypothesize-Validate" loop orchestrated by the `qianji-bpmn-engine`:

1.  **Physical Scanner (Rust)**: Extracts high-frequency physical entities and noun phrases across the data swamp.
2.  **Episteme Priors**: Maps extracted terms to L1 Domain hypotheses (e.g., guessing 'Gateway' maps to `Service`).
3.  **Deductive Validation (BPMN)**: Orchestrates targeted, micro-context LLM queries to confirm relationships without full-text reading.
4.  **Materialization**: Verified relationships are implicitly written into the L2 Application ontology, transforming unstructured text into queryable knowledge graphs.

## 4. Academic References & Extensions

The architecture described in this RFC is heavily informed by state-of-the-art research (2025-2026) in Dynamic Schema Induction and Agentic Knowledge Graphs:

- **AutoSchemaKG & ATLAS (2025)**: _Bai et al._ demonstrated that predefined schemas are no longer a hard prerequisite. Their framework extracts factual triples and induces coherent schemas directly from web-scale unstructured text, achieving ~92% semantic alignment with human-crafted schemas. This validates our BPMN Deduction Loop approach for implicit L2 discovery.
- **Agentic Context Engineering (2026)**: Shifted the paradigm from single-pass extraction to "Proposer-Critic" multi-agent loops. Our use of BPMN to orchestrate targeted micro-context queries directly mirrors this proactive, iterative validation pattern.
- **Implicit Schema-Overlap Matching (2026)**: Research on Intermediate Semantic Representations (ISR) supports our design of allowing agents to operate on flexible, inferred structures before materializing them into rigid DuckDB/SQL tables.
