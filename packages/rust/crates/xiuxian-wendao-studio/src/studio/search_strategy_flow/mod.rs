//! Studio-owned SearchStrategyFlow integration proof surfaces.

/// Native Flight materialization receipt helpers for SearchStrategyFlow.
#[cfg(all(
    feature = "zhenfa-router",
    feature = "julia",
    any(test, feature = "test-support")
))]
pub mod materialization;
