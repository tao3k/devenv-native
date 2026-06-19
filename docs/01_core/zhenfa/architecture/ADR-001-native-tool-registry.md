---
type: knowledge
title: "ADR-001: Retired Zhenfa Native Tool Registry"
status: "Superseded"
date: "2026-02-26"
category: "architecture"
tags:
  - zhenfa
  - adr
  - retired
metadata:
  title: "ADR-001: Retired Zhenfa Native Tool Registry"
---

# ADR-001: Retired Zhenfa Native Tool Registry

## Status

Superseded. The in-process LLM tool-call registry design is no longer part of `xiuxian-zhenfa`.

## Current Boundary

Zhenfa owns typed context, contract errors, signal fan-out, XML/Markdown transmutation, and optional JSON-RPC gateway compatibility. It does not own model-facing tool registration, tool schema generation, or dynamic tool dispatch.

## Replacement

Domain crates expose direct typed functions and DTOs for Zhenfa-adjacent operations. Agent/model tool-call policy belongs in the agent runtime layer, not in Zhenfa.

## Consequence

Historical registry, wrapper, and dispatch plans are retained only as retired context. New work must not add replacement trait-object tool registries inside this crate.
