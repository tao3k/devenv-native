---
type: knowledge
title: "ADR-002: Retired Zhenfa Native Tool Macro"
status: "Superseded"
date: "2026-02-26"
category: "architecture"
tags:
  - zhenfa
  - adr
  - macro
  - retired
saliency_base: 2.0
decay_rate: 0.05
metadata:
  title: "ADR-002: Retired Zhenfa Native Tool Macro"
---

# ADR-002: Retired Zhenfa Native Tool Macro

## Status

Superseded. Zhenfa no longer owns an LLM tool-call registry, native dispatch macro, or generated tool wrapper surface.

## Current Decision

Zhenfa remains a contract, context, signal, and streaming boundary. Domain crates that need Zhenfa integration expose direct typed functions and DTOs instead of generated LLM tool-call adapters.

## Rationale

LLM routing and model-facing tool-call policy moved out of Zhenfa. Keeping a macro or registry in this crate would preserve a misleading public boundary and duplicate ownership that now belongs to the agent runtime layer.

## Migration Note

Historical search and render adapters are now direct functions. Tests should construct typed argument DTOs directly or through serde deserialization and call those functions without a generated wrapper.
