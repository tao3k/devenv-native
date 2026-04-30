use std::path::Path;

use xiuxian_testing::{
    assert_crate_modularity_gate, assert_crate_test_policy_with_workspace_config,
};

#[test]
fn enforce_crate_test_policy_gate() {
    assert_crate_test_policy_with_workspace_config(Path::new(env!("CARGO_MANIFEST_DIR")));
}

#[test]
fn enforce_modularity_contract_gate() {
    assert_crate_modularity_gate(Path::new(env!("CARGO_MANIFEST_DIR")));
}
