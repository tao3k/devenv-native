//! Integration tests for bounded agentic expansion planning.

mod plan;
mod support;

use support::{TestResult, build_index_fixture, expansion_config};

#[cfg(feature = "julia")]
#[path = "../expansion_plan_batch_tests.rs"]
mod expansion_plan_batch_tests;
#[cfg(feature = "julia")]
mod live;
#[cfg(feature = "julia")]
mod projection;
