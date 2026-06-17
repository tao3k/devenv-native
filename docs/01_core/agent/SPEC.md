---
type: knowledge
title: "Externalized Agent Boundary"
category: "core"
tags:
  - agent
  - external-service
  - archived
metadata:
  title: "Externalized Agent Boundary"
---

# Externalized Agent Boundary

This document is retained as a historical architecture note. The current
workspace does not own an in-process Agent kernel, long-lived provider runtime,
or LLM orchestration core. Those capabilities are external service concerns.
This repository owns durable workflow contracts through Qianji and knowledge
engine/search contracts through Wendao.

## Current Boundary

- Qianji owns workflow state, BPMN/Flowhub control, leases, checkpoints,
  activity contracts, and deterministic completion/failure recording.
- Wendao owns knowledge-engine indexing, search, graph, Arrow/SQL, and Gateway
  protocol surfaces.
- Provider execution, model routing, subagent kernels, and long-lived LLM
  orchestration are integrated through service protocols or explicit
  compatibility adapters, not as this workspace's core ownership.

## Compatibility Surfaces

Existing `xiuxian-llm` and OpenAI-compatible paths remain compatibility
adapters where explicitly enabled. They must not be treated as the default
durable workflow core or as an Agent kernel implementation.

Future work that combines an external Agent kernel with Xiuxian should live in
a separate integration repository or service deployment, with this workspace
providing stable workflow and knowledge-engine contracts.
