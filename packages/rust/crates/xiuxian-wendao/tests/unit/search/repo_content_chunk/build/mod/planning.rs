use super::*;

#[test]
fn repo_content_chunk_partition_policy_is_repo_size_aware() {
    assert_eq!(
        repo_content_chunk_partition_count_for_document_count(1_000),
        16
    );
    assert_eq!(
        repo_content_chunk_partition_count_for_document_count(6_000),
        64
    );
    assert_eq!(
        repo_content_chunk_partition_count_for_document_count(10_000),
        64
    );
}

#[test]
fn plan_repo_content_chunk_build_only_rewrites_changed_files() {
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

    let second_documents = vec![
        repo_document("src/lib.rs", "fn gamma() {}\n", 14, 20),
        repo_document("src/util.rs", "fn beta() {}\n", 13, 10),
    ];
    let second_plan = plan_repo_content_chunk_build(
        "alpha/repo",
        &second_documents,
        Some("rev-2"),
        Some(&previous_publication),
        &first_plan.file_fingerprints,
    );

    match second_plan.action {
        RepoContentChunkBuildAction::CloneAndMutate {
            base_table_name,
            target_table_name,
            replaced_paths,
            changed_payload: changed_documents,
        } => {
            assert_eq!(base_table_name, previous_publication.table_name);
            assert_ne!(target_table_name, previous_publication.table_name);
            assert_eq!(
                replaced_paths.into_iter().collect::<Vec<_>>(),
                vec!["src/lib.rs".to_string()]
            );
            assert_eq!(changed_documents.len(), 1);
            assert_eq!(changed_documents[0].path, "src/lib.rs");
        }
        other => panic!("unexpected second build action: {other:?}"),
    }
}

#[test]
fn plan_repo_content_chunk_build_reuses_table_for_revision_only_refresh() {
    let documents = vec![repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10)];
    let table_name = versioned_repo_content_table_name(
        "alpha/repo",
        &repo_content_chunk_file_fingerprints(&documents),
        Some("rev-1"),
    );
    let publication = SearchRepoPublicationRecord::new(
        SearchCorpusKind::RepoContentChunk,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: table_name.clone(),
            schema_version: SearchCorpusKind::RepoContentChunk.schema_version(),
            source_revision: Some("rev-1".to_string()),
            table_version_id: 1,
            row_count: 1,
            fragment_count: 1,
            published_at: "2026-03-24T12:00:00Z".to_string(),
        },
    );
    let plan = plan_repo_content_chunk_build(
        "alpha/repo",
        &documents,
        Some("rev-2"),
        Some(&publication),
        &repo_content_chunk_file_fingerprints(&documents),
    );

    match plan.action {
        RepoContentChunkBuildAction::RefreshPublication { table_name } => {
            assert_eq!(table_name, publication.table_name);
        }
        other => panic!("unexpected build action: {other:?}"),
    }
}

#[test]
fn plan_repo_content_chunk_build_ignores_metadata_only_edits_when_contents_are_unchanged() {
    let first_documents = vec![repo_document("src/lib.rs", "fn alpha() {}\n", 14, 10)];
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
                    row_count: 1,
                    fragment_count: 1,
                    published_at: "2026-03-24T12:00:00Z".to_string(),
                },
            )
        }
        other => panic!("unexpected first build action: {other:?}"),
    };

    let second_documents = vec![repo_document("src/lib.rs", "fn alpha() {}\n", 14, 20)];
    let second_plan = plan_repo_content_chunk_build(
        "alpha/repo",
        &second_documents,
        Some("rev-2"),
        Some(&previous_publication),
        &first_plan.file_fingerprints,
    );

    let first_table_name = versioned_repo_content_table_name(
        "alpha/repo",
        &first_plan.file_fingerprints,
        Some("rev-2"),
    );
    let second_table_name = versioned_repo_content_table_name(
        "alpha/repo",
        &second_plan.file_fingerprints,
        Some("rev-2"),
    );
    assert_eq!(first_table_name, second_table_name);
    assert_eq!(
        first_plan
            .file_fingerprints
            .get("src/lib.rs")
            .and_then(|fingerprint| fingerprint.blake3.as_deref()),
        second_plan
            .file_fingerprints
            .get("src/lib.rs")
            .and_then(|fingerprint| fingerprint.blake3.as_deref())
    );
    assert_eq!(
        second_plan
            .file_fingerprints
            .get("src/lib.rs")
            .map(|fingerprint| fingerprint.modified_unix_ms),
        Some(20)
    );

    match second_plan.action {
        RepoContentChunkBuildAction::RefreshPublication { table_name } => {
            assert_eq!(table_name, previous_publication.table_name);
        }
        other => panic!("unexpected second build action: {other:?}"),
    }
}
