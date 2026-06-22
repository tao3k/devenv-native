---
type: knowledge
title: "Zhixing Scenario Archive"
category: "plans"
tags:
  - zhixing
  - scenarios
  - qianji
  - wendao
saliency_base: 7.5
decay_rate: 0.03
metadata:
  title: "Zhixing Scenario Archive"
---

# Zhixing Scenario Archive

This archive records the agenda validation and strict-teacher scenarios that are
now expressed as embedded Wendao resources and Qianji workflow tests.

The standalone Zhixing package has been retired. Scenario artifacts should be
resolved through `wendao://skills/...` URIs backed by
`packages/rust/crates/xiuxian-wendao/resources/zhixing/`, not through a separate
crate.

## Runtime Shape

1. Qianji loads scenario resources through Bootcamp VFS mounts.
2. Wendao runtime resolves semantic `wendao://` paths from the embedded resource
   bundle.
3. Scenario correctness is proven by Qianji integration tests, not by a retired
   package-level test suite.
