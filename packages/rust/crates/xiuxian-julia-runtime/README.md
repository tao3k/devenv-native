---
type: knowledge
kind: readme
title: "xiuxian-julia-runtime"
category: "package-docs"
status: "active"
author: CyberXiuXian Artisan Workshop
date: 2026-05-24T00:00:00-07:00
description: "Package README for Julia runtime contracts and feature-scoped Wendao adapters."
tags:
  - julia
  - runtime
  - wendao
  - polyglot
metadata:
  title: "xiuxian-julia-runtime"
---

# xiuxian-julia-runtime

`xiuxian-julia-runtime` is the Rust-side Julia runtime adapter boundary.
Wendao integration is feature-scoped behind `wendao` and consumes inert Julia
profile/catalog facts from
[`xiuxian-polyglot-orchestrator`](../xiuxian-polyglot-orchestrator/README.md).

## Features

1. `wendao`: exposes Wendao-facing Julia profile ids, host entrypoints, route
   facts, schema ids, memory-family profile identities, and `WendaoGraph.jl`
   workload descriptors through the polyglot fact catalog.

## Boundary

This crate owns Julia runtime adapters only. It does not start Julia, manage
queues, perform warmup, execute profile work, or make scheduling decisions.
Cross-language admission, bridge projection, profile scheduling facts, and
scheduler-facing evidence belong in `xiuxian-polyglot-orchestrator`; Wendao
runtime configuration remains in `xiuxian-wendao-runtime` and stable Wendao
domain contracts remain in `xiuxian-wendao-core`.

`xiuxian-julia-core` consumes this runtime boundary together with the polyglot
contracts for Wendao and Modelica bridge clients. The dependency direction is
`xiuxian-julia-core` plus `xiuxian-julia-runtime` to
`xiuxian-polyglot-orchestrator`; the orchestrator does not depend back on Julia
core/runtime crates.
