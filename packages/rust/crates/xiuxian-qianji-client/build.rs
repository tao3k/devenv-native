//! Qianji client build-time project harness gate.

use xiuxian_rust_workspace_harness::assert_member_harness_build_gate_from_env_with_configure;

fn main() {
    assert_member_harness_build_gate_from_env_with_configure(|config| config);
}
