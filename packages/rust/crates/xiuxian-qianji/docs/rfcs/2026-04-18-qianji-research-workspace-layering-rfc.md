---
type: knowledge
title: "RFC: Qianji Research Workspace Layering"
category: "rfc"
status: "draft"
authors:
  - codex
created: 2026-04-18
tags:
  - rfc
  - qianji
  - flowhub
  - research
  - papers
  - topics
  - runs
---

# RFC: Qianji Research Workspace Layering

## 1. Summary

This RFC defines the research-layered workspace model for Qianji.

The current localized run surface remains intentionally small, but it must no
longer be treated as the full research repository.

The corrected model has four explicit surfaces:

```text
research/
  flowhub/
  runs/
  papers/
  topics/
```

Those surfaces have different responsibilities:

1. `flowhub/` stores reusable graph contracts
2. `runs/` stores one bounded execution surface for one active run
3. `papers/` stores persistent single-paper knowledge objects
4. `topics/` stores persistent cross-paper synthesis

## 2. Motivation

The older framing mixed three different things:

1. the run-local writable execution surface
2. the persistent knowledge state of a paper-reading workflow
3. the user-visible answer or report

That mixing produces the wrong system behavior:

1. every node appears to serve one terminal answer file
2. durable research objects are flattened into ephemeral session artifacts
3. cross-paper comparison has no first-class home
4. the localized run directory quietly turns into an accidental research
   repository

Qianji already freezes two constraints that push against that mistake:

1. Codex is the execution layer, `qianji-flowhub` is the constraint layer, and
   `qianji check` is the evaluation layer
2. the localized workdir contract stays intentionally small

Wendao also already favors hierarchical structured objects over flat blob
storage. The research workspace should follow the same object-first shape.

## 3. Design Principles

### 3.1 Object-First, View-Second

Research state is stored as durable objects first. Human-facing answers are
materialized views over those objects.

### 3.2 Localized Runs Stay Small

The run-local surface exists to execute one bounded graph under one contract.
It does not become the authority for paper knowledge or topic synthesis.

### 3.3 Persistent Research Lives Outside the Run

Single-paper and cross-paper state must survive beyond one run and therefore
must live under `papers/` and `topics/`, not only under `runs/`.

### 3.4 Flowhub Stores Constraints, Not Research Data

`flowhub/` stores reusable graph contracts. It must not become a storage
surface for extracted paper objects or topic notebooks.

## 4. Workspace Layers

### 4.1 Flowhub

The Flowhub research contracts stay small:

```text
research/
  flowhub/
    paper/
      qianji.toml
      paper-canonicalize.mmd
      paper-deep-read.mmd
      paper-compare.mmd
```

These graphs are separate on purpose:

1. `paper-canonicalize.mmd` turns a raw paper source into a stable paper
   package
2. `paper-deep-read.mmd` derives claims, evidence, methods, results, and
   critique from one canonical paper package
3. `paper-compare.mmd` aligns multiple papers into topic-level comparison and
   synthesis objects

### 4.2 Runs

`runs/` is the bounded execution plane:

```text
runs/
  <run_id>/
    qianji.toml
    flowchart.mmd
    state/
      current_node.toml
      trace.jsonl
      allowed_next.json
      checkpoints/
    inputs/
      task.json
      paper_refs.json
      topic_ref.json
    outputs/
      node_results.jsonl
      materialized_refs.json
      response_preview.md
    diagnostics/
      check.md
      blocked.json
      failed.json
```

Normative rules:

1. `runs/` does not own the full persistent paper package
2. `response_preview.md` is only a session-local answer preview
3. `materialized_refs.json` points at the persistent `papers/` / `topics/`
   objects used or produced by the run

### 4.3 Papers

`papers/` is the persistent single-paper knowledge package:

```text
papers/
  <paper_id>/
    source/
      source.pdf
      envelope.json
      pages/
    extraction/
      text_pass.json
      layout_regions.json
      vision_patches.jsonl
    structure/
      section_tree.json
      figure_index.json
      table_index.json
      equation_index.json
      reference_index.json
      citation_graph.json
    semantics/
      claim_ledger.jsonl
      evidence_ledger.jsonl
      entity_graph.json
      method_card.json
      experiment_sheet.json
      result_sheet.json
      limitation_sheet.json
    notebook/
      reading_journal.md
      open_questions.jsonl
      contradictions.jsonl
      followups.jsonl
      critique_log.md
    syntheses/
      skim.md
      deep_read.md
      methods_digest.md
      results_digest.md
      critique_memo.md
      figure_walkthrough.md
```

This layer stores durable research assets, not only a one-shot summary.

### 4.4 Topics

`topics/` is the persistent cross-paper synthesis layer:

```text
topics/
  <topic_id>/
    corpus/
      paper_refs.json
    notebook/
      question_map.jsonl
      hypothesis_tree.mmd
      contradiction_map.jsonl
      gap_log.md
      idea_log.md
    comparison/
      method_matrix.csv
      result_matrix.csv
      dataset_matrix.csv
      benchmark_grid.csv
    syntheses/
      state_of_field.md
      literature_review_outline.md
      research_gap_memo.md
      proposal_notes.md
```

This is where multi-paper comparison and review drafting become durable.

## 5. Canonical Research Objects

The system should treat these as primary research objects:

1. `Section`
2. `Figure`
3. `Table`
4. `Reference`
5. `CitationSpan`
6. `Claim`
7. `Evidence`
8. `Method`
9. `Experiment`
10. `Result`
11. `Limitation`
12. `Question`

Derived views are secondary and may be regenerated:

1. `skim.md`
2. `deep_read.md`
3. `critique_memo.md`
4. `literature_review_outline.md`
5. `proposal_notes.md`
6. `runs/<run_id>/outputs/response_preview.md`

## 6. Flow Contracts

### 6.1 Paper Canonicalize

Purpose:

1. intake raw paper sources
2. stabilize extraction and structure
3. emit the minimal persistent paper package needed for later deep reading

Primary outputs:

1. `papers/<paper_id>/extraction/*`
2. `papers/<paper_id>/structure/*`

### 6.2 Paper Deep Read

Purpose:

1. load the canonical paper package
2. derive semantic ledgers, method/result cards, and notebook state
3. materialize paper-level syntheses

Primary outputs:

1. `papers/<paper_id>/semantics/*`
2. `papers/<paper_id>/notebook/*`
3. `papers/<paper_id>/syntheses/*`

### 6.3 Paper Compare

Purpose:

1. align a topic corpus
2. compare methods, results, and contradictions
3. materialize topic-level synthesis

Primary outputs:

1. `topics/<topic_id>/comparison/*`
2. `topics/<topic_id>/notebook/*`
3. `topics/<topic_id>/syntheses/*`

## 7. `qianji check` Responsibilities

The current implementation already checks localized run/workdir contracts. The
target layered model extends that evaluation in three levels:

### 7.1 Runs

`qianji check` over `runs/<run_id>` should validate:

1. node progression against the active flow
2. `trace.jsonl` alignment
3. required inputs and outputs for the current node
4. blocked versus failed diagnostics separation

### 7.2 Papers

`qianji check` over `papers/<paper_id>` should validate:

1. required structural objects such as `section_tree.json` and
   `citation_graph.json`
2. source grounding for `claim_ledger.jsonl`
3. source grounding for `evidence_ledger.jsonl`
4. completeness of `method_card.json`, `result_sheet.json`, and
   `limitation_sheet.json`
5. synthesis references back to paper-level IDs rather than unsupported prose

### 7.3 Topics

`qianji check` over `topics/<topic_id>` should validate:

1. corpus integrity
2. matrix coverage over the declared paper set
3. contradiction references back to paper and claim IDs
4. synthesis references back to notebook or comparison objects

This RFC only defines the contract. It does not claim that those `papers/` and
`topics/` checks are already implemented in the current workspace.

## 8. Compatibility With the Existing Localized Contract

This RFC does not replace the current localized-run principle.

Instead it narrows it correctly:

1. the localized run surface is still the execution plane
2. it remains intentionally small
3. it now points at durable paper and topic objects instead of pretending to
   own them

The existing compact localized contract therefore remains valid, but only for
`runs/<run_id>`.

## 9. Non-Goals

This slice does not:

1. add a new long-lived monolithic research workdir
2. make `response_preview.md` authoritative
3. claim that `papers/` and `topics/` validation is already implemented
4. collapse all research flows into one oversized Mermaid graph

## 10. Initial Landing Criteria

The first bounded landing of this RFC should provide:

1. repo-owned Flowhub research graphs for canonicalize, deep-read, and compare
2. minimal `research` / `paper` Flowhub contracts
3. canonical docs that freeze the `runs/`, `papers/`, `topics/` split
4. live CLI proofs that the new graphs are accepted by the current graph
   contract machinery

## 11. Open Follow-On Work

Follow-on implementation slices should:

1. add first-class `qianji check` semantics for `papers/` and `topics/`
2. materialize stable references between `runs/` outputs and persistent
   research objects
3. connect the paper/topic objects to Wendao PageIndex and hierarchical
   retrieval surfaces
