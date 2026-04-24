---
type: knowledge
title: "Retired Knowledge Search Improvement Report"
category: "developer"
tags:
  - developer
  - knowledge
saliency_base: 6.3
decay_rate: 0.04
metadata:
  title: "Retired Knowledge Search Improvement Report"
---

# Retired Knowledge Search Improvement Report

## Scope

This historical report covered the old Omni-era knowledge search stack where
`knowledge.recall`, `knowledge.search`, and `link_graph_hybrid_search` were
debugged through vector-local Lance stores and `omni sync knowledge`.

That implementation is no longer the active architecture. Search and query
semantics now belong to Wendao-owned query layers, with DuckDB-backed
structured retrieval and parser-owned document surfaces. Lance-backed vector
tables remain storage evidence only; they are not the canonical knowledge
search execution plane.

## Retired Assumptions

The previous report depended on assumptions that should not guide new work:

- vector-local knowledge collections as the primary search owner
- `omni sync knowledge` as the current indexing command for Wendao documents
- path alignment under a vector-owned Lance database as the main correctness
  fix
- hybrid search behavior owned by the vector package

## Replacement Boundary

For current search-quality work:

- use Wendao/DuckDB query and retrieval gates documented from
  `docs/testing/README.md`
- document document-ingestion behavior against the owning Wendao parser and
  query layers
- keep Lance/vector references limited to storage-format evidence unless a
  live owner explicitly consumes that storage shell

This file is kept only to preserve the historical problem statement.
