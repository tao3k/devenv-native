//! Qianji BPMN engine build-time project harness gate.

fn main() {
    xiuxian_rust_workspace_harness::assert_member_harness_build_gate_from_env_with_configure(
        |config| {
            config.with_verification_profile_hint(
                xiuxian_rust_workspace_harness::RustVerificationProfileHint::new(
                    "src/lib.rs",
                    [xiuxian_rust_workspace_harness::RustOwnerResponsibility::PublicApi],
                )
                .with_rationale(
                    "crate root owns the public package API for build-time verification",
                ),
            )
        },
    );
}
