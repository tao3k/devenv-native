---
type: knowledge
title: "RFC: Active Citations and Executable Evidence in Org-Mode"
category: "rfc"
status: "draft"
authors:
  - auditor_neuron
  - sovereign
created: 2026-05-11
tags:
  - rfc
  - active-citations
  - orgize
  - executable-evidence
  - agentic-reasoning
---

# Active Citations and Executable Evidence in Org-Mode

## 1. Vision and Motivation

In the current `xiuxian-wendao-parsers` implementation, evidence linkage relies on static references (e.g., Markdown standard links or Wikilinks like `[[target|label]]`). While these are structurally robust and heavily linted for semantic purity, they suffer from a fatal flaw in highly dynamic software environments: **Static links decay.** A link to `auth.rs` may be valid today but lose its specific semantic relevance when the file is refactored tomorrow.

To achieve true "Agentic Autonomy" without hallucinations, an Agent's reasoning must be grounded in real-time, provable facts.

This RFC introduces **Active Citations (Executable Evidence)** to our `orgize`-based Agentic Memory pipeline. We shift from pointing to _locations_ (where a fact lived) to pointing to _queries_ (how to compute the fact now).

## 2. Core Concept: Query-as-Evidence via Babel Source Blocks

An Active Citation is a hyper-specialized reference mechanism embedded within an Org-mode document. Instead of resolving to a static file path, it resolves to an executable query against the system's underlying parsers, DuckDB, or Git history.

### 2.1 Standardized Syntax: Babel Blocks and Name References

A critical architectural principle is that **we do not invent new markup syntax or abuse property constraints.** Stuffing multi-line SQL or complex `ast-grep` patterns into single-line Property Drawers or Macro arguments is an anti-pattern that breaks syntax highlighting and escaping logic.

Instead, we strictly leverage Org-mode's native **Babel (Source Block)** infrastructure and **Named References**, which the `tao3k/orgize` parser natively supports.

**The Implementation Pattern:**

1. **Define the Executable Block**: The actual query is written in a properly syntax-highlighted source block, tagged with a unique `#+name`.

```org
#+name: check-auth-dependencies
#+begin_src sql
  SELECT target_id
  FROM dependencies
  WHERE source_id = 'auth'
#+end_src
```

2. **Reference in the Property Drawer**: The operational node (e.g., an Agentic reasoning step or a validation gate) links to this query using a standardized property key that references the block's name.

```org
* DONE Verify Architecture Compliance
  :PROPERTIES:
  :EVIDENCE_REF: check-auth-dependencies
  :EXPECTED_ROWS: 1
  :END:
```

### 2.2 Inline Results (Literate Provenance)

Because this utilizes standard Babel syntax, the Rust orchestrator can execute the query and inject the result directly beneath the source block using `#+RESULTS:`, providing an auditable, human-readable trace of the exact data the Agent used to make its decision.

## 3. The Execution Pipeline (Rust Orchestration)

The magic occurs not in the syntax, but in the Rust-driven execution loop.

1.  **Parsing (`orgize`)**: The `orgize` crate parses the Org document. When it projects the Semantic AST, it tags these specific links or properties with a new `ElementData` variant (e.g., `ActiveCitation`).
2.  **Interception (`xiuxian-wendao-server`)**: Before feeding this Org-mode memory back to the LLM Sub-agent, the Rust orchestrator traverses the AST and evaluates all `ActiveCitation` nodes.
3.  **Execution (The Engines)**:
    - If the prefix is `sql:`, it executes the query against the materialized DuckDB graph.
    - If the prefix is `ast-grep:`, it delegates to the Native Parser CLI.
4.  **Materialization**: The Rust orchestrator injects the _real-time result_ of that query directly into the context payload sent to the LLM (or updates the `#+RESULTS:` block in the Org file itself).

## 4. Architectural Impact & Business Value

1.  **Zero-Decay Knowledge Graphs**: The knowledge graph becomes "living." A policy document stating "All controllers must inherit from `BaseController`" is no longer just text; it is an active test suite embedded in the documentation.
2.  **Deterministic Agent Validation**: When a Sub-agent proposes a code change, the system can automatically re-run all Active Citations associated with that module's RFC. If the query results deviate from the expected baseline, the proposal is deterministically rejected.
3.  **Auditable Provenance**: The highest standard of enterprise compliance. We can prove exactly _what the database looked like at the millisecond the Agent made its decision_.

## 5. Implementation Roadmap

- **Phase 1 (orgize)**: Enhance the `tao3k/orgize` AST schema to specifically categorize and parse `query:` URI schemes within standard link elements.
- **Phase 2 (wendao-parsers)**: Implement the extraction logic to map these Active Citations into the Semantic SSoT representation.
- **Phase 3 (Orchestrator)**: Implement the "Eval" pass in Rust that intercepts these citations and safely executes the underlying SQL or AST searches before finalizing the Agent payload.
