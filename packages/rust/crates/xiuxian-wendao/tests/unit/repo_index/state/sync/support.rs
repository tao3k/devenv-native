pub(super) use std::fs;
pub(super) use std::path::{Path, PathBuf};
pub(super) use std::time::Duration;

pub(super) use crate::analyzers::{
    ModuleRecord, RegisteredRepository, RepoIntelligenceError, RepositoryAnalysisOutput,
    RepositoryPluginConfig, RepositoryRefreshPolicy, resolve_registered_repository_source,
};
pub(super) use crate::repo_index::state::collect::await_analysis_completion;
pub(super) use crate::repo_index::state::fingerprint::{fingerprint, timestamp_now};
pub(super) use crate::repo_index::state::task::RepoIndexTaskPriority;
pub(super) use crate::repo_index::state::tests::{
    init_test_repository, new_coordinator, remote_repo, repo,
};
pub(super) use crate::repo_index::types::{RepoCodeDocument, RepoIndexEntryStatus, RepoIndexPhase};
pub(super) use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService,
};
pub(super) use uuid::Uuid;
pub(super) use xiuxian_git_repo::{
    SyncMode, discover_checkout_metadata, record_managed_remote_probe_failure,
    record_managed_remote_probe_state,
};

pub(super) fn repo_documents() -> Vec<RepoCodeDocument> {
    vec![RepoCodeDocument {
        path: "src/lib.rs".to_string().into(),
        language: Some("rust".to_string()),
        contents: std::sync::Arc::<str>::from("fn alpha() {}\n"),
        size_bytes: 14,
        modified_unix_ms: 0,
    }]
}

pub(super) fn repo_analysis_output(repo_id: &str) -> RepositoryAnalysisOutput {
    RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: repo_id.to_string().into(),
            module_id: "module:alpha".to_string().into(),
            qualified_name: "Alpha".to_string(),
            path: "src/lib.rs".to_string().into(),
        }],
        ..RepositoryAnalysisOutput::default()
    }
}

pub(super) fn set_mirror_fetch_age(mirror_root: &Path, age: Duration) {
    let target_time = std::time::SystemTime::now()
        .checked_sub(age)
        .unwrap_or_else(|| panic!("failed to compute mirror age timestamp"));

    for candidate in [mirror_root.join("FETCH_HEAD"), mirror_root.join("HEAD")] {
        if candidate.exists() {
            let file = fs::OpenOptions::new()
                .write(true)
                .open(&candidate)
                .unwrap_or_else(|error| panic!("open `{}`: {error}", candidate.display()));
            let times = fs::FileTimes::new().set_modified(target_time);
            file.set_times(times)
                .unwrap_or_else(|error| panic!("set times for `{}`: {error}", candidate.display()));
        }
    }
}

pub(super) fn set_managed_remote_probe_state_age(
    mirror_root: &Path,
    probe_age: Duration,
    last_success_age: Option<Duration>,
) {
    let state_path = mirror_root.join("xiuxian-upstream-probe-state.json");
    let mut payload: serde_json::Value = serde_json::from_slice(
        &fs::read(&state_path)
            .unwrap_or_else(|error| panic!("read `{}`: {error}", state_path.display())),
    )
    .unwrap_or_else(|error| panic!("parse `{}`: {error}", state_path.display()));
    payload["checked_at"] = serde_json::Value::String(
        chrono::DateTime::<chrono::Utc>::from(
            std::time::SystemTime::now()
                .checked_sub(probe_age)
                .unwrap_or_else(|| panic!("failed to compute probe timestamp")),
        )
        .to_rfc3339(),
    );
    match last_success_age {
        Some(age) => {
            payload["last_success_checked_at"] = serde_json::Value::String(
                chrono::DateTime::<chrono::Utc>::from(
                    std::time::SystemTime::now()
                        .checked_sub(age)
                        .unwrap_or_else(|| panic!("failed to compute success timestamp")),
                )
                .to_rfc3339(),
            );
        }
        None => {
            payload
                .as_object_mut()
                .unwrap_or_else(|| panic!("probe payload should be an object"))
                .remove("last_success_checked_at");
        }
    }
    fs::write(
        &state_path,
        serde_json::to_vec(&payload)
            .unwrap_or_else(|error| panic!("encode `{}`: {error}", state_path.display())),
    )
    .unwrap_or_else(|error| panic!("write `{}`: {error}", state_path.display()));
}
