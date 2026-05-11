use std::path::PathBuf;

use crate::search::coordinator::SearchCompactionReason;
use crate::search::service::core::{
    RepoCompactionTask, RepoMaintenanceTask, RepoPrewarmTask, SearchPlaneService,
};
use crate::search::{SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace};

pub fn make_service(temp_dir: &tempfile::TempDir, keyspace: &str) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new(keyspace),
        SearchMaintenancePolicy::default(),
    )
}

pub fn make_prewarm_task(
    corpus: SearchCorpusKind,
    repo_id: &str,
    table_name: &str,
    projected_columns: &[&str],
) -> RepoMaintenanceTask {
    RepoMaintenanceTask::Prewarm(RepoPrewarmTask {
        corpus,
        repo_id: repo_id.to_string().into(),
        table_name: table_name.to_string(),
        projected_columns: projected_columns
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
    })
}

pub fn make_compaction_task(
    corpus: SearchCorpusKind,
    repo_id: &str,
    publication_id: &str,
    table_name: &str,
    row_count: u64,
    reason: SearchCompactionReason,
) -> RepoMaintenanceTask {
    RepoMaintenanceTask::Compaction(RepoCompactionTask {
        corpus,
        repo_id: repo_id.to_string().into(),
        publication_id: publication_id.to_string(),
        table_name: table_name.to_string(),
        row_count,
        reason,
    })
}
