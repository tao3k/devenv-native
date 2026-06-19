---
id: "20260315150000"
type: knowledge
title: "Feature: Zhenfa (阵法) Contract System"
category: "features"
tags:
  - zhenfa
  - contract
  - xsd
  - agent-validation
saliency_base: 8.5
decay_rate: 0.01
metadata:
  title: "Feature: Zhenfa (阵法) Contract System"
---

# Feature: Zhenfa (阵法) Contract System

## 1. Overview

The **Zhenfa (阵法) Contract System** provides a rigid, physical governance layer for AI Agents. It prevents agent-output drift by enforcing structured XML communication protocols between agents and the system.

## 2. Key Capabilities

- **`zhenfa.contract` Validation**: Mandatory XSD (XML Schema Definition) checking before entering execution nodes.
- **Physical Enforcement**: Any agent response that fails XSD validation is blocked, and an error report is fed back to the agent for auto-correction.
- **Unified Schemas**: Centrally managed or scenario-local schemas (e.g., `qianji_plan.xsd`) define the "Physical Law" of the workspace.

---

## Linked Notes

- Parent MOC: [[20260315151000-zhenfa-matrix-moc]]
- Client Implementation: [[20260315140000-autonomous-audit-feature-v3]]
- Theoretical Foundation: [[docs/01_core/zhenfa/architecture/schema-contract]]
