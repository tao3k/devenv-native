---
type: knowledge
title: "Xiuxian-Zhenfa: Contract and Streaming Boundary"
category: "architecture"
tags:
  - zhenfa
  - gateway
  - contract
  - streaming
  - core-spec
saliency_base: 8.8
decay_rate: 0.01
metadata:
  title: "Xiuxian-Zhenfa: Contract and Streaming Boundary"
---

# Xiuxian-Zhenfa: Contract and Streaming Boundary

> **Authority:** CyberXiuXian Artisan workshop
> **Status:** Architecture Evolution (Direct Function Boundary, 2026)

`xiuxian-zhenfa` provides the contract, context, signal, and transmutation boundary used by adjacent crates.

## Current Responsibilities

1. JSON-RPC compatible gateway DTOs and error mapping.
2. Typed context extension storage for direct in-process callers.
3. Signal fan-out for Sentinel and streaming pipeline coordination.
4. XML/Markdown washing and streaming parser support.
5. Optional Axum gateway compatibility for external callers.

## Explicit Non-Responsibilities

Zhenfa does not own LLM tool-call registration, model-facing schema prompts, generated tool wrappers, or dynamic native dispatch. Those responsibilities belong to the agent runtime layer.

## Integration Pattern

Domain crates expose direct typed functions and argument DTOs. Callers may wrap those functions in their own model-facing adapter, but the adapter is outside Zhenfa.
