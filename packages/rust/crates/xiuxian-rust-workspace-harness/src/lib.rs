//! Workspace-owned Rust harness policy for xiuxian member crates.

use rust_lang_project_harness::{
    RustHarnessConfig, RustHarnessReport, RustOwnerResponsibility,
    RustProjectHarnessDependencyBaseline, RustProjectHarnessDownstreamPolicy,
    RustProjectHarnessWorkspacePolicy, RustVerificationPhase, RustVerificationProfileHint,
    RustVerificationRequirement, RustVerificationSkillBinding, RustVerificationSkillDescriptor,
    RustVerificationTaskContract, RustVerificationTaskKind,
};

/// Types intentionally exposed for member `build.rs` policy hints.
pub mod prelude {
    pub use rust_lang_project_harness::{
        RustOwnerResponsibility, RustVerificationPhase, RustVerificationProfileHint,
        RustVerificationRequirement, RustVerificationTaskContract, RustVerificationTaskKind,
    };
}

/// Human-readable label used in workspace evidence graph receipts.
pub const XIUXIAN_WORKSPACE_LABEL: &str = "xiuxian-artisan-workshop";

/// The rust harness git revision pinned by the xiuxian workspace policy.
pub const XIUXIAN_RUST_HARNESS_REV: &str = "6dea4b1afc46bec0b4b480b9ab99b9d471d045ee";

/// The rust harness crate version expected at [`XIUXIAN_RUST_HARNESS_REV`].
pub const XIUXIAN_RUST_HARNESS_VERSION: &str = "0.1.2";

/// Shared build-gate explanation for advisory cargo-check findings.
pub const XIUXIAN_BUILD_GATE_ADVICE_ALLOW_EXPLANATION: &str = concat!(
    "scope=xiuxian workspace build.rs gates; ",
    "owner=xiuxian-rust-workspace-harness; ",
    "finding_category=advisory project-policy migrations; ",
    "why_safe_now=warnings/errors remain blocking and advisory findings stay visible in harness output; ",
    "cleanup_trigger=promote assert_member_harness_build_gate_from_env to the strict downstream policy gate after transitive rust-harness rev drift is removed"
);

/// Build the shared harness config used by xiuxian member build gates.
#[must_use]
pub fn xiuxian_workspace_harness_config() -> RustHarnessConfig {
    rust_lang_project_harness::default_rust_harness_config()
        .with_cargo_check_advice_allow_explanation(XIUXIAN_BUILD_GATE_ADVICE_ALLOW_EXPLANATION)
        .with_verification_profile_hint(xiuxian_member_library_performance_hint())
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::LatencySensitive,
            [RustVerificationTaskKind::Performance],
        )
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::AvailabilityCritical,
            [
                RustVerificationTaskKind::Stability,
                RustVerificationTaskKind::Regression,
            ],
        )
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::SecurityBoundary,
            [RustVerificationTaskKind::Security],
        )
        .with_verification_skill_binding(
            RustVerificationTaskKind::Performance,
            RustVerificationSkillBinding::new("rust-verification-performance")
                .with_adapter("cargo-check"),
        )
        .with_verification_skill_descriptor(xiuxian_member_performance_skill_descriptor())
        .with_verification_skill_binding(
            RustVerificationTaskKind::Stability,
            RustVerificationSkillBinding::new("rust-verification-stability")
                .with_adapter("cargo-check"),
        )
        .with_verification_skill_descriptor(xiuxian_member_stability_skill_descriptor())
        .with_verification_skill_binding(
            RustVerificationTaskKind::Security,
            RustVerificationSkillBinding::new("rust-verification-security")
                .with_adapter("cargo-check"),
        )
        .with_verification_skill_descriptor(xiuxian_member_security_skill_descriptor())
        .with_verification_skill_binding(
            RustVerificationTaskKind::Regression,
            RustVerificationSkillBinding::new("rust-verification-regression")
                .with_adapter("cargo-check"),
        )
        .with_verification_skill_descriptor(xiuxian_member_regression_skill_descriptor())
}

fn xiuxian_member_library_performance_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/lib.rs",
        [
            RustOwnerResponsibility::LatencySensitive,
            RustOwnerResponsibility::PublicApi,
        ],
    )
    .with_task_kinds([
        RustVerificationTaskKind::Performance,
        RustVerificationTaskKind::Stability,
        RustVerificationTaskKind::Security,
        RustVerificationTaskKind::Regression,
    ])
    .with_task_contract(
        RustVerificationTaskKind::Performance,
        RustVerificationTaskContract::new(
            RustVerificationPhase::AfterUnitTestsPass,
            "performance skill must report member crate cargo-check or package-specific benchmark evidence before release",
            [
                RustVerificationRequirement::new(
                    "verification_command",
                    "cargo check -p <crate> --locked or stronger package-specific gate",
                ),
                RustVerificationRequirement::new(
                    "regression_threshold",
                    "accepted package-specific runtime or build-time regression threshold",
                ),
                RustVerificationRequirement::new(
                    "artifact",
                    "local or CI log proving the verification gate ran",
                ),
            ],
        ),
    )
    .with_task_contract(
        RustVerificationTaskKind::Stability,
        RustVerificationTaskContract::new(
            RustVerificationPhase::AfterUnitTestsPass,
            "stability skill must report member crate cargo-check or stronger package-specific stability evidence before release",
            [
                RustVerificationRequirement::new(
                    "verification_command",
                    "cargo check -p <crate> --locked or stronger package-specific gate",
                ),
                RustVerificationRequirement::new(
                    "stability_surface",
                    "public API, build script, and dependency surface covered by the gate",
                ),
                RustVerificationRequirement::new(
                    "artifact",
                    "local or CI log proving the stability gate ran",
                ),
            ],
        ),
    )
    .with_task_contract(
        RustVerificationTaskKind::Security,
        RustVerificationTaskContract::new(
            RustVerificationPhase::BeforeRelease,
            "security skill must report member crate unsafe, build-script, and dependency-boundary evidence before release",
            [
                RustVerificationRequirement::new(
                    "verification_command",
                    "cargo check -p <crate> --locked or stronger package-specific gate",
                ),
                RustVerificationRequirement::new(
                    "security_surface",
                    "unsafe, build-script, and dependency-boundary surface covered by the gate",
                ),
                RustVerificationRequirement::new(
                    "artifact",
                    "local or CI log proving the security gate ran",
                ),
            ],
        ),
    )
    .with_task_contract(
        RustVerificationTaskKind::Regression,
        RustVerificationTaskContract::new(
            RustVerificationPhase::ScheduledRegression,
            "regression skill must report member crate cargo-check or stronger package-specific regression evidence before release",
            [
                RustVerificationRequirement::new(
                    "verification_command",
                    "cargo check -p <crate> --locked or stronger package-specific gate",
                ),
                RustVerificationRequirement::new(
                    "contract_surface",
                    "public API, feature, and build-script contract covered by the gate",
                ),
                RustVerificationRequirement::new(
                    "artifact",
                    "local or CI log proving the regression gate ran",
                ),
            ],
        ),
    )
    .with_rationale("every xiuxian member crate exposes src/lib.rs as its release verification anchor")
}

fn xiuxian_member_performance_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-performance")
        .with_adapter("cargo-check")
        .with_tool("cargo")
        .with_command("cargo check -p <crate> --locked")
        .with_standard("member crate release checks remain intentional and observable")
        .with_required_inputs(["verification_command", "regression_threshold", "artifact"])
        .with_pass_criteria(["check=pass", "artifact=present"])
        .with_receipt_fields([
            "verification_command",
            "regression_threshold",
            "latency_or_throughput",
            "artifact",
        ])
}

fn xiuxian_member_stability_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-stability")
        .with_adapter("cargo-check")
        .with_tool("cargo")
        .with_command("cargo check -p <crate> --locked")
        .with_standard("member crate release stability stays intentional and observable")
        .with_required_inputs(["verification_command", "stability_surface", "artifact"])
        .with_pass_criteria(["check=pass", "artifact=present"])
        .with_receipt_fields([
            "verification_command",
            "stability_surface",
            "latency_or_throughput",
            "artifact",
        ])
}

fn xiuxian_member_security_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-security")
        .with_adapter("cargo-check")
        .with_tool("cargo")
        .with_command("cargo check -p <crate> --locked")
        .with_standard("member crate release security boundaries stay intentional and observable")
        .with_required_inputs(["verification_command", "security_surface", "artifact"])
        .with_pass_criteria(["check=pass", "artifact=present"])
        .with_receipt_fields([
            "verification_command",
            "security_surface",
            "finding_summary",
            "artifact",
        ])
}

fn xiuxian_member_regression_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-regression")
        .with_adapter("cargo-check")
        .with_tool("cargo")
        .with_command("cargo check -p <crate> --locked")
        .with_standard("member crate release contracts stay intentional and observable")
        .with_required_inputs(["verification_command", "contract_surface", "artifact"])
        .with_pass_criteria(["check=pass", "artifact=present"])
        .with_receipt_fields([
            "verification_command",
            "contract_surface",
            "contract_parity",
            "artifact",
        ])
}

/// Build the shared dependency baseline for workspace evidence receipts.
#[must_use]
pub fn xiuxian_workspace_dependency_baseline() -> RustProjectHarnessDependencyBaseline {
    RustProjectHarnessDependencyBaseline::new().require_git_package(
        "rust-lang-project-harness",
        XIUXIAN_RUST_HARNESS_VERSION,
        format!("rev={XIUXIAN_RUST_HARNESS_REV}"),
    )
}

/// Build the workspace-owned policy used to derive member crate policies.
#[must_use]
pub fn xiuxian_workspace_policy() -> RustProjectHarnessWorkspacePolicy {
    xiuxian_workspace_policy_with_config(xiuxian_workspace_harness_config())
}

/// Build the workspace-owned policy with a caller-provided harness config.
#[must_use]
pub fn xiuxian_workspace_policy_with_config(
    config: RustHarnessConfig,
) -> RustProjectHarnessWorkspacePolicy {
    RustProjectHarnessWorkspacePolicy::new(XIUXIAN_WORKSPACE_LABEL, config)
        .with_dependency_baseline(xiuxian_workspace_dependency_baseline())
}

/// Derive a member crate policy from the shared xiuxian workspace policy.
#[must_use]
pub fn xiuxian_member_policy(crate_label: impl Into<String>) -> RustProjectHarnessDownstreamPolicy {
    xiuxian_workspace_policy().member_crate(crate_label)
}

/// Derive a member crate policy from a caller-provided shared config.
#[must_use]
pub fn xiuxian_member_policy_with_config(
    crate_label: impl Into<String>,
    config: RustHarnessConfig,
) -> RustProjectHarnessDownstreamPolicy {
    xiuxian_workspace_policy_with_config(config).member_crate(crate_label)
}

fn xiuxian_member_build_gate_policy_from_env_with_configure<F>(
    configure: F,
) -> RustProjectHarnessDownstreamPolicy
where
    F: FnOnce(RustHarnessConfig) -> RustHarnessConfig,
{
    let config = xiuxian_workspace_harness_config();
    let config = configure(config);
    let crate_label =
        std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "unknown-xiuxian-member".to_owned());
    xiuxian_member_build_gate_policy_with_config(crate_label, config)
}

fn xiuxian_member_build_gate_policy_with_config(
    crate_label: impl Into<String>,
    config: RustHarnessConfig,
) -> RustProjectHarnessDownstreamPolicy {
    xiuxian_workspace_policy_with_config(config).member_crate(crate_label)
}

/// Assert the legacy-compatible member harness build gate from `CARGO_MANIFEST_DIR`.
///
/// # Panics
///
/// Panics when the rust project harness cargo-check gate fails for the member
/// crate being built.
#[track_caller]
#[expect(
    clippy::must_use_candidate,
    reason = "build scripts may rely on the panic side effect without reading the report"
)]
pub fn assert_member_harness_build_gate_from_env() -> RustHarnessReport {
    assert_member_harness_build_gate_from_env_with_configure(|config| config)
}

/// Assert the member harness build gate with crate-local config extensions.
///
/// Use this when a member crate needs verification profile hints while still
/// inheriting the workspace-owned baseline policy.
///
/// # Panics
///
/// Panics when the rust project harness cargo-check gate fails for the member
/// crate being built.
#[track_caller]
pub fn assert_member_harness_build_gate_from_env_with_configure<F>(
    configure: F,
) -> RustHarnessReport
where
    F: FnOnce(RustHarnessConfig) -> RustHarnessConfig,
{
    let policy = xiuxian_member_build_gate_policy_from_env_with_configure(configure);
    rust_lang_project_harness::assert_rust_project_harness_downstream_policy_from_env(&policy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_policy_carries_harness_rev_baseline() {
        let policy = xiuxian_workspace_policy();
        let member_policy = policy.member_crate("xiuxian-config-core");
        let packages = match member_policy.dependency_baseline() {
            Some(dependency_baseline) => dependency_baseline.packages(),
            None => panic!("expected rust harness dependency baseline"),
        };

        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name(), "rust-lang-project-harness");
        assert_eq!(packages[0].version(), XIUXIAN_RUST_HARNESS_VERSION);
        assert_eq!(
            packages[0].source_contains(),
            format!("rev={XIUXIAN_RUST_HARNESS_REV}")
        );
    }

    #[test]
    fn build_gate_policy_carries_harness_rev_baseline() {
        let member_policy = xiuxian_member_build_gate_policy_with_config(
            "xiuxian-event",
            xiuxian_workspace_harness_config(),
        );
        let packages = match member_policy.dependency_baseline() {
            Some(dependency_baseline) => dependency_baseline.packages(),
            None => panic!("expected rust harness dependency baseline"),
        };

        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name(), "rust-lang-project-harness");
        assert_eq!(packages[0].version(), XIUXIAN_RUST_HARNESS_VERSION);
        assert_eq!(
            packages[0].source_contains(),
            format!("rev={XIUXIAN_RUST_HARNESS_REV}")
        );
    }
}
