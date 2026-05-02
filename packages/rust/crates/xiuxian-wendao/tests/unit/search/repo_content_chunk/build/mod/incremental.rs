use std::collections::BTreeMap;

use crate::search::repo_content_chunk::build::plan::{
    merge_repo_content_chunk_file_fingerprints, plan_repo_content_chunk_build,
    plan_repo_content_chunk_incremental_build,
};
use crate::search::repo_content_chunk::build::types::RepoContentChunkBuildAction;
use crate::search::{SearchCorpusKind, SearchRepoPublicationInput, SearchRepoPublicationRecord};

use super::repo_document;

#[test]
fn plan_repo_content_chunk_incremental_build_only_rewrites_changed_files() {
    let first_documents = vec![
        repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10),
        repo_document("src/util.rs", "fn beta() {}\n", 13, 10),
    ];
    let first_plan = plan_repo_content_chunk_build(
        "alpha/repo",
        &first_documents,
        Some("rev-1"),
        None,
        &BTreeMap::new(),
    );
    let previous_publication = match first_plan.action {
        RepoContentChunkBuildAction::ReplaceAll { ref table_name, .. } => {
            SearchRepoPublicationRecord::new(
                SearchCorpusKind::RepoContentChunk,
                "alpha/repo",
                SearchRepoPublicationInput {
                    table_name: table_name.clone(),
                    schema_version: SearchCorpusKind::RepoContentChunk.schema_version(),
                    source_revision: Some("rev-1".to_string()),
                    table_version_id: 1,
                    row_count: 2,
                    fragment_count: 1,
                    published_at: "2026-03-24T12:00:00Z".to_string(),
                },
            )
        }
        other => panic!("unexpected first build action: {other:?}"),
    };
    let changed_documents = vec![repo_document("src/lib.rs", "fn gamma() {}\n", 14, 20)];
    let merged_fingerprints = merge_repo_content_chunk_file_fingerprints(
        &first_plan.file_fingerprints,
        &changed_documents,
        &std::collections::BTreeSet::new(),
    );
    let second_plan = plan_repo_content_chunk_incremental_build(
        "alpha/repo",
        &changed_documents,
        &merged_fingerprints,
        Some("rev-2"),
        Some(&previous_publication),
        &first_plan.file_fingerprints,
    );

    match second_plan.action {
        RepoContentChunkBuildAction::CloneAndMutate {
            base_table_name,
            target_table_name,
            replaced_paths,
            changed_payload,
        } => {
            assert_eq!(base_table_name, previous_publication.table_name);
            assert_ne!(target_table_name, previous_publication.table_name);
            assert_eq!(
                replaced_paths.into_iter().collect::<Vec<_>>(),
                vec!["src/lib.rs".to_string()]
            );
            assert_eq!(changed_payload.len(), 1);
            assert_eq!(changed_payload[0].path, "src/lib.rs");
        }
        other => panic!("unexpected second build action: {other:?}"),
    }
}
