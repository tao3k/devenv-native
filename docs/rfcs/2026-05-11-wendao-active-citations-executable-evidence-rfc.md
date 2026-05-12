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

## 2. Core Concept: Query-as-Evidence

An Active Citation is a hyper-specialized reference syntax embedded within an Org-mode document. Instead of resolving to a static file path, it resolves to an executable query against the system's underlying parsers, DuckDB, or Git history.

### 2.1 Standardized Syntax via Org-Mode Macros

A core architectural principle is that **we do not invent new markdown or markup syntax.** Doing so breaks existing tooling and creates proprietary lock-in.

Instead, we map our "Active Citations" strictly to the native, highly-standardized **Macro** capabilities of Org-mode, which the `tao3k/orgize` parser already supports natively.

**Approach A: The Macro Evaluation (Recommended for inline evidence)**
We utilize Org-mode macros to explicitly and cleanly declare the execution context (e.g., `sql` or `ast-grep`).

```org
The system must reject this payload because it violates the dependency constraint:
{{{sql(SELECT target_id FROM dependencies WHERE source_id='auth')}}}
```

Or for an AST query:

```org
Ensure the target function still exists: {{{ast-grep(pattern: pub fn verify_token($$$))}}}
```

**Approach B: The Standard Property Drawer (Recommended for structural logic)**

```org
* DONE Verify Architecture Compliance
  :PROPERTIES:
  :EVIDENCE_SQL: SELECT COUNT(*) FROM dependencies WHERE source_id='auth'
  :EXPECTED_COUNT: 1
  :END:
```

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
