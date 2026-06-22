---
type: knowledge
title: "Zhixing-Heyi Runtime Resource Surface"
category: "plans"
tags:
  - zhixing
  - heyi
  - qianji
  - wendao-runtime
saliency_base: 7.8
decay_rate: 0.03
metadata:
  title: "Zhixing-Heyi Runtime Resource Surface"
---

# Zhixing-Heyi Runtime Resource Surface

Zhixing-Heyi is no longer implemented as a standalone Rust crate. The durable
surface is now split across the current runtime owners:

1. `xiuxian-wendao` owns the embedded `resources/zhixing/...` bundle.
2. `xiuxian-wendao-runtime` owns resource path helpers and mounted resource
   text lookup.
3. `xiuxian-qianji` consumes the resources through Bootcamp VFS mounts for
   agenda, forge, and debate workflow tests.

The old agenda, reminder, strict-teacher, and thin-bridge implementation notes
are retained here only as design history. New runtime behavior should be added
to the owning crates above instead of recreating a separate Zhixing package.

## Current Boundaries

| Surface                  | Owner                                                                | Purpose                                                    |
| ------------------------ | -------------------------------------------------------------------- | ---------------------------------------------------------- |
| Embedded skill resources | `packages/rust/crates/xiuxian-wendao/resources/zhixing/`             | Canonical checked-in Markdown and TOML resources           |
| Runtime resource lookup  | `packages/rust/crates/xiuxian-wendao-runtime/src/artifacts/zhixing/` | Resource path normalization and `wendao://` lookup helpers |
| Workflow execution       | `packages/rust/crates/xiuxian-qianji/`                               | Bootcamp and workflow runtime consumption                  |

## Follow-Up Rule

Do not reintroduce a standalone Zhixing crate. If agenda or reminder behavior is
needed again, place storage and runtime code under the current Wendao/Qianji
owners and keep the resource bundle as data.
