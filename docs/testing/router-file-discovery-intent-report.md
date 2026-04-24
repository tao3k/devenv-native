---
type: knowledge
title: "Retired Router File Discovery Intent Report"
category: "testing"
tags:
  - testing
  - router
saliency_base: 6.5
decay_rate: 0.04
metadata:
  title: "Retired Router File Discovery Intent Report"
---

# Retired Router File Discovery Intent Report

## Scope

This historical report covered the old vector-local router search surface.
That implementation has been retired: `xiuxian-vector` no longer owns skill
scanning, tool search, keyword rescue, FTS search, hybrid fusion, or route
ranking.

Current routing and search semantics belong to Wendao/DuckDB-owned query
layers or to the owning router crate. `xiuxian-vector` is only a Lance/Arrow
storage-format boundary.

## Retired Surface

The following surfaces described by the previous report are no longer active:

- vector-local skill scanner indexing
- vector-local keyword or FTS rescue
- vector-local hybrid fusion
- vector-local route ranking tests

## Replacement Boundary

Future route diagnostics should be documented against the owning router
contract, not against `xiuxian-vector`.
