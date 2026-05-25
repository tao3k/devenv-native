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

`xiuxian-julia-runtime` is the Rust-side Julia runtime contract boundary.
Wendao integration is feature-scoped behind `wendao` so Julia runtime identity,
profile, and workload facts do not need a Wendao-specific crate as their long
term owner.

## Features

1. `wendao`: exposes Wendao-facing Julia profile ids, host entrypoints, route
   facts, schema ids, memory-family profile identities, and `WendaoGraph.jl`
   workload descriptors.

## Boundary

This crate owns inert Julia runtime facts only. It does not start Julia, build
Arrow Flight transports, manage queues, perform warmup, execute profile work,
or make scheduling decisions. Cross-language admission, bridge projection,
profile scheduling facts, and scheduler-facing evidence belong in
`xiuxian-polyglot-orchestrator`; Wendao runtime configuration remains in
`xiuxian-wendao-runtime` and stable Wendao domain contracts remain in
`xiuxian-wendao-core`.

`xiuxian-wendao-julia` is a removal-track migration consumer for the `wendao`
feature. New cross-language bridge code should depend on this crate through
`xiuxian-polyglot-orchestrator` rather than adding new direct dependencies on
the Wendao-specific Julia crate.
