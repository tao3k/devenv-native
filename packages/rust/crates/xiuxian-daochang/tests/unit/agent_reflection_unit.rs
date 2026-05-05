//! Top-level integration harness for `agent::reflection`.

use xiuxian_daochang::test_support::{
    TestReflectiveRuntime, TestReflectiveRuntimeStage, test_build_turn_reflection,
    test_derive_policy_hint,
};

#[path = "agent/reflection/lifecycle.rs"]
mod tests;
