---
type: knowledge
title: "Zhixing Theory Archive"
category: "explanation"
tags:
  - zhixing
  - theory
  - action-selector
  - prompting
  - domain-boundary
saliency_base: 6.8
decay_rate: 0.04
metadata:
  title: "Zhixing Theory Archive"
---

# Zhixing Theory Archive

Zhixing remains a design vocabulary for turning stored knowledge into action,
but it is no longer a standalone crate boundary.

Current implementation guidance:

1. Keep the knowledge graph domain-agnostic in Wendao core.
2. Store prompt, persona, and workflow resources as embedded Wendao resources.
3. Let Qianji consume those resources as workflow context.
4. Put runtime lookup and artifact helpers in `xiuxian-wendao-runtime`.

This preserves the action-selector and context-injection theory without
maintaining duplicate runtime ownership.
