//! Qianji control build-time project harness gate.

use xiuxian_rust_workspace_harness::prelude::{
    RustOwnerResponsibility, RustVerificationProfileHint,
};

fn main() {
    xiuxian_rust_workspace_harness::assert_member_harness_build_gate_from_env_with_configure(
        |config| {
            config.with_verification_profile_hint(
                RustVerificationProfileHint::new(
                    "src/lib.rs",
                    [RustOwnerResponsibility::PublicApi],
                )
                .with_rationale("crate root owns the public control-plane API"),
            )
        },
    );
}
