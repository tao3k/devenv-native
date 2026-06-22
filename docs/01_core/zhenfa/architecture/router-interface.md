---
type: knowledge
title: "Zhenfa Router Interface"
category: "architecture"
tags:
  - zhenfa
  - trait
  - axum
saliency_base: 8.0
decay_rate: 0.02
metadata:
  title: "Zhenfa Router Interface"
---

# Zhenfa Router Interface

To prevent `xiuxian-zhenfa` from becoming a monolithic bottleneck that knows about all domain logic, it exposes a singular extension trait.

## The `ZhenfaRouter` Trait

Any domain crate (e.g., `xiuxian-wendao` or a downstream runtime crate) that wishes to expose HTTP endpoints must implement this trait.

```rust
use axum::Router;
use async_trait::async_trait;

#[async_trait]
pub trait ZhenfaRouter {
    /// The base prefix for this module (e.g., "/v1/wendao")
    fn prefix(&self) -> &'static str;

    /// Mounts the module's specific routes into the Axum router
    fn mount(&self, router: Router) -> Router;
}
```

## Implementation Paradigm

1. **Feature Flags**: Domain crates should place their `ZhenfaRouter` implementation behind a cargo feature flag (e.g., `[features] http = ["dep:axum"]`). This ensures that if the crate is used purely as a CLI tool or native library, it does not pay the compilation cost of the web framework.
2. **State Management**: The domain crate is responsible for passing its own thread-safe state (`Arc<Mutex<T>>` or `tokio::sync::RwLock`) into the `Router` during the `mount` phase. `xiuxian-zhenfa` does not manage domain state.
