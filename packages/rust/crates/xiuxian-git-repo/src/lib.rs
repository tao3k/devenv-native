//! Reusable repository substrate for materialization, revision resolution, and
//! checkout observation.

mod backend;
mod diff;
mod error;
mod layout;
mod lock;
mod metadata;
mod revision;
mod spec;
mod sync;

pub use diff::{
    RevisionChangeKind, RevisionDiffSummary, RevisionPathChange, diff_checkout_revisions,
    read_checkout_file_bytes_at_revision,
};
pub use error::{RepoError, RepoErrorKind};
pub use layout::{managed_checkout_root_for, managed_mirror_root_for, sanitize_repo_id};
pub use lock::{
    ManagedCheckoutLock, acquire_managed_checkout_lock, acquire_managed_checkout_lock_with_policy,
    checkout_lock_max_wait_with_lookup, is_descriptor_pressure_error, managed_lock_path_for,
};
pub use metadata::{
    LocalCheckoutMetadata, ManagedRemoteProbeState, ManagedRemoteProbeStatus, RepoDriftState,
    clear_managed_remote_probe_state, discover_checkout_metadata,
    discover_managed_remote_probe_state, record_managed_remote_probe_failure,
    record_managed_remote_probe_state,
};
pub use spec::{RepoRefreshPolicy, RepoSpec, RevisionSelector};
pub use sync::{
    MaterializedRepo, RepoLifecycleState, RepoSourceKind, SyncMode, resolve_repository_source,
};

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config().with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public package API for cargo-test verification"),
        )
    }
);
