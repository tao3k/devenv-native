//! Integration tests for bounded agentic expansion planning.

mod plan;
mod support;

#[cfg(feature = "julia")]
#[path = "../expansion_plan_batch_tests.rs"]
mod expansion_plan_batch_tests;
#[cfg(feature = "julia")]
mod live;
#[cfg(feature = "julia")]
mod projection;
