---
type: knowledge
kind: index
title: "xiuxian-wendao-attachments Map of Content"
category: "package-docs"
status: "active"
author: Xiuxian Artisan Workshop
date: 2026-05-05T00:00-07:00
description: "Map of content for the xiuxian-wendao-attachments package documentation."
tags:
  - attachments
  - wendao
  - documentation
  - polyglot
metadata:
  title: "xiuxian-wendao-attachments Map of Content"
  retrieval:
    saliency_base: 7.0
    decay_rate: 0.03
---

# xiuxian-wendao-attachments: Map of Content

:PROPERTIES:
:ID: 8f2727d0ed2e1b46b2ea779e2489317935a66631
:TYPE: INDEX
:STATUS: ACTIVE
:END:

Standardized documentation index for the `xiuxian-wendao-attachments`
package.

This package owns attachment parsing, archive and image audit helpers, PDF
source-range rendering support, OCR shard contracts, resource-cache policy,
and ordering validation for document extraction surfaces. Python/Docling
remains the OCR and conversion execution authority.

Polyglot boundary:

1. `src/polyglot.rs` translates attachment-owned OCR shard route, schema,
   pressure, and scheduling facts into `xiuxian-polyglot-orchestrator`
   contracts, including the source-range auto worker sizing helper.
2. The bridge is feature-gated by `pdf-source-range` because it depends on the
   attachment OCR shard schema surface.
3. Studio may consume the resulting inert schedule plan, but attachments still
   own cache reuse, shard ordering, and Docling fallback policy.

Verification profile:

1. `cargo test -p xiuxian-wendao-attachments --features pdf-source-range --lib polyglot`
   covers the feature-gated polyglot bridge.
2. `cargo test -p xiuxian-wendao-attachments --features pdf-source-range --lib enforce_rust_project_harness_gate`
   covers the shared harness profile gate.

---

:FOOTER:
:STANDARDS: v2.0
:LAST_SYNC: 2026-05-05
:END:
