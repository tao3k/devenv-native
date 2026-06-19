---
type: knowledge
title: "ADR-010: The Ignorant Host - Historical Agent Container Boundary"
status: "Superseded"
date: "2026-02-28"
category: "architecture"
tags:
  - agent
  - qianji
  - wendao
  - ssot
  - decoupling
metadata:
  title: "ADR-010: The Ignorant Host - Historical Agent Container Boundary"
---

# ADR-010: The Ignorant Host - Historical Agent Container Boundary

## Supersession Note

This ADR is retained as historical design context. Current Xiuxian workspace
ownership is narrower: Qianji owns durable workflow contracts, Wendao owns
knowledge-engine and Gateway contracts, and Agent kernels or long-lived provider
runtimes are external service concerns. New implementation work must not use
this ADR to justify adding an in-process Agent core to this repository.

## 1. Context and Problem Statement

Our current `Agent` implementation is "over-educated" and "over-involved." It manually manages persona registries, pre-loads templates, and hardcodes the association between logic and assets. This creates a monolithic bottleneck in the `bootstrap/` sequence and violates the principle of **Separation of Concerns**.

The `Agent` layer should not need to know the internal details of a specific domain like "Agenda Management." All those details (personas, templates, rules) already exist in the `xiuxian-zhixing` skill directory.

## 2. Decision: Total Decoupling via Late Binding

We will adopt the **"Ignorant Host"** architecture. The `Agent` will be reduced to a stateless execution container that knows nothing about the business logic it runs.

### 2.1 The Semantic Handshake Chain

1.  **Storage**: Assets are stored in `xiuxian-zhixing/resources/zhixing/skills/agenda-management/references/`.
2.  **Addressing**: `xiuxian-wendao` maps these to stable `wendao://` URIs.
3.  **Mapping**: `xiuxian-qianji` workflow TOMLs use the **`$` placeholder** to reference these URIs (e.g., `persona = "$wendao://..."`).
4.  **Late Binding**: The `Qianji` engine resolves these placeholders via the `Zhenfa` bus at the exact moment of execution.

### 2.3 The Death of Domain-Specific Rust Controllers

We strictly prohibit the creation of "special-case" Rust files within the Agent core (e.g., `agenda_validation.rs`).

**The Rule of Ultimate Thinness:**

- **No Manual Context Assembly**: The Agent MUST NOT manually gather data (like search results) to "feed" a workflow.
- **No Manual Engine Orchestration**: The Agent MUST NOT manually instantiate schedulers or registries for specific missions.
- **Total Manifest Reliance**: All reasoning steps, adversarial loops, and data requirements are defined in the **Synaptic Flow (TOML)**. The Agent acts as a pure, context-agnostic runner.

**Terminology Update:**

- **Validation** (too restrictive) -> **Synaptic Flow** (reflects the reactive, neural nature of the workload).

### 3.2 Qianji Protocol Feature

The `xiuxian-qianji` executor will be upgraded with a **Protocol Resolution Feature**.

- It detects parameters prefixed with `$`.
- It calls `ZhenfaContext::resolve_uri()` to pull the content from the graph bus.
- It injects the result into the node's local context before execution.

## 4. Consequences

### Positive

- **Extreme Purity**: Agent code only changes when the system architecture changes, not when business rules change.
- **Dynamic Evolution**: Swapping a persona or a workflow is a 0-recompile operation.
- **True Declarativity**: The entire "Steward-Teacher" adversarial loop is defined in a single TOML file, independent of the host.

### Negative

- **Traceability Requirements**: Debugging requires tracing through the `wendao://` bus (mitigated by `ZhenfaAuditSink`).

## 5. Implementation Plan

1.  **Refactor `Qianji`**: Implement the `$` protocol resolver.
2.  **Strip `xiuxian-daochang`**: Delete manual context-injection bootstrap loaders.
3.  **Relink `Zhixing`**: Ensure `agenda_flow.toml` uses the new `$` mapping syntax.
