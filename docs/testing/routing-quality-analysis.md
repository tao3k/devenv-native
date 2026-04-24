---
type: knowledge
title: "Retired Routing Quality Analysis"
category: "testing"
tags:
  - testing
  - routing
saliency_base: 6.5
decay_rate: 0.04
metadata:
  title: "Retired Routing Quality Analysis"
---

# Retired Routing Quality Analysis

## Scope

This historical report analyzed an old router result where a URL-oriented crawl
tool outranked a repository research tool. The report belonged to the retired
vector-local router search surface.

That surface is no longer active. `xiuxian-vector` no longer owns tool
discovery, keyword rescue, FTS search, hybrid fusion, or route ranking.
Routing and search semantics belong to Wendao/DuckDB-owned query layers or to
the owning router crate.

## Retired Assumptions

Do not use the previous analysis as a current runbook. It depended on retired
assumptions:

- vector-local hybrid search as the route-test execution plane
- vector-local Lance/Tantivy store alignment as the correctness fix
- `omni sync` as the current route-index update path
- skill-ranking behavior owned by `xiuxian-vector`

## Replacement Boundary

Future routing-quality reports should be written against the owning router
contract and the current Wendao/DuckDB retrieval gates. Data fixes remain valid
only when they target the active owner of the indexed metadata.
