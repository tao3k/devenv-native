use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use crate::analyzers::{
    ExampleRecord, ModuleRecord, RepoSymbolKind, RepositoryAnalysisOutput, SymbolRecord,
};
use crate::repo_index::RepoCodeDocument;
use crate::search::cache::SearchPlaneFileFingerprintScope;
use crate::search::repo_entity::build::{
    RepoEntityBuildAction, plan_repo_entity_build, publish_repo_entities,
    repo_entity_file_fingerprints,
};
use crate::search::repo_entity::schema::rows_from_analysis;
use crate::search::repo_staging::versioned_repo_table_name;
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService,
    SearchRepoCorpusRecord, SearchRepoPublicationInput, SearchRepoPublicationRecord,
};

fn repo_document(
    path: &str,
    contents: &str,
    size_bytes: u64,
    modified_unix_ms: u64,
) -> RepoCodeDocument {
    RepoCodeDocument {
        path: path.to_string().into(),
        language: Some("julia".to_string()),
        contents: Arc::<str>::from(contents),
        size_bytes,
        modified_unix_ms,
    }
}

fn temp_dir_or_panic() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"))
}

fn repo_entity_service(temp_dir: &tempfile::TempDir) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:repo-entity-build"),
        SearchMaintenancePolicy::default(),
    )
}

fn repo_entity_record_or_panic(
    record: Option<SearchRepoCorpusRecord>,
    context: &str,
) -> SearchRepoCorpusRecord {
    let Some(record) = record else {
        panic!("{context}");
    };
    record
}

fn repo_entity_publication_or_panic<'a>(
    record: &'a SearchRepoCorpusRecord,
    context: &str,
) -> &'a SearchRepoPublicationRecord {
    let Some(publication) = record.publication.as_ref() else {
        panic!("{context}");
    };
    publication
}

fn assert_repo_entity_prewarmed(record: &SearchRepoCorpusRecord) {
    assert!(
        record
            .maintenance
            .as_ref()
            .and_then(|maintenance| maintenance.last_prewarmed_at.as_ref())
            .is_some()
    );
}

fn assert_no_repo_entity_lance_table(
    service: &SearchPlaneService,
    table_name: &str,
    context: &str,
) {
    assert!(
        !service
            .corpus_root(SearchCorpusKind::RepoEntity)
            .join(format!("{table_name}.lance"))
            .exists(),
        "{context}"
    );
}

async fn assert_repo_entity_hit_stems(
    service: &SearchPlaneService,
    search_term: &str,
    language_filters: &HashSet<String>,
    kind_filters: &HashSet<String>,
    expected_stems: &[&str],
) {
    let hits = service
        .search_repo_entities("alpha/repo", search_term, language_filters, kind_filters, 5)
        .await
        .unwrap_or_else(|error| panic!("query {search_term}: {error}"));
    let actual_stems = hits.iter().map(|hit| hit.stem.as_str()).collect::<Vec<_>>();
    assert_eq!(actual_stems, expected_stems);
}

async fn assert_repo_entity_hit_paths(
    service: &SearchPlaneService,
    search_term: &str,
    language_filters: &HashSet<String>,
    kind_filters: &HashSet<String>,
    expected_paths: &[&str],
) {
    let hits = service
        .search_repo_entities("alpha/repo", search_term, language_filters, kind_filters, 5)
        .await
        .unwrap_or_else(|error| panic!("query {search_term}: {error}"));
    let actual_paths = hits.iter().map(|hit| hit.path.as_str()).collect::<Vec<_>>();
    assert_eq!(actual_paths, expected_paths);
}

fn sample_analysis(
    repo_id: &str,
    symbol_name: &str,
    example_summary: &str,
) -> RepositoryAnalysisOutput {
    RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: repo_id.to_string().into(),
            module_id: "module:BaseModelica".to_string().into(),
            qualified_name: "BaseModelica".to_string(),
            path: "src/BaseModelica.jl".to_string().into(),
        }],
        symbols: vec![SymbolRecord {
            repo_id: repo_id.to_string().into(),
            symbol_id: format!("symbol:{symbol_name}").into(),
            module_id: Some("module:BaseModelica".to_string().into()),
            name: symbol_name.to_string(),
            qualified_name: format!("BaseModelica.{symbol_name}"),
            kind: RepoSymbolKind::Function,
            path: "src/BaseModelica.jl".to_string().into(),
            line_start: Some(7),
            line_end: Some(9),
            signature: Some(format!("{symbol_name}()")),
            audit_status: Some("verified".to_string().into()),
            verification_state: Some("verified".to_string().into()),
            attributes: BTreeMap::new(),
        }],
        examples: vec![ExampleRecord {
            repo_id: repo_id.to_string().into(),
            example_id: "example:reexport".to_string().into(),
            title: "Reexport example".to_string(),
            path: "examples/reexport.jl".to_string().into(),
            summary: Some(example_summary.to_string()),
        }],
        ..RepositoryAnalysisOutput::default()
    }
}

fn sample_documents(symbol_name: &str, source_modified_unix_ms: u64) -> Vec<RepoCodeDocument> {
    vec![
        repo_document(
            "src/BaseModelica.jl",
            format!("module BaseModelica\n{symbol_name}() = nothing\nend\n").as_str(),
            48,
            source_modified_unix_ms,
        ),
        repo_document(
            "examples/reexport.jl",
            "using BaseModelica\nreexport()\n",
            30,
            10,
        ),
    ]
}

#[test]
fn plan_repo_entity_build_only_rewrites_changed_files() {
    let first_analysis = sample_analysis("alpha/repo", "reexport", "Shows reexport");
    let first_documents = sample_documents("reexport", 10);
    let first_rows = rows_from_analysis("alpha/repo", &first_analysis)
        .unwrap_or_else(|error| panic!("first rows: {error}"));
    let first_plan = plan_repo_entity_build(
        "alpha/repo",
        &first_rows,
        &first_documents,
        Some("rev-1"),
        None,
        &BTreeMap::new(),
    );
    let previous_publication = match first_plan.action {
        RepoEntityBuildAction::ReplaceAll { ref table_name, .. } => {
            SearchRepoPublicationRecord::new(
                SearchCorpusKind::RepoEntity,
                "alpha/repo",
                SearchRepoPublicationInput {
                    table_name: table_name.clone(),
                    schema_version: SearchCorpusKind::RepoEntity.schema_version(),
                    source_revision: Some("rev-1".to_string()),
                    table_version_id: 1,
                    row_count: 3,
                    fragment_count: 1,
                    published_at: "2026-03-24T12:00:00Z".to_string(),
                },
            )
        }
        other => panic!("unexpected first build action: {other:?}"),
    };

    let second_analysis = sample_analysis("alpha/repo", "solve", "Shows reexport");
    let second_documents = sample_documents("solve", 20);
    let second_rows = rows_from_analysis("alpha/repo", &second_analysis)
        .unwrap_or_else(|error| panic!("second rows: {error}"));
    let second_plan = plan_repo_entity_build(
        "alpha/repo",
        &second_rows,
        &second_documents,
        Some("rev-2"),
        Some(&previous_publication),
        &first_plan.file_fingerprints,
    );

    match second_plan.action {
        RepoEntityBuildAction::CloneAndMutate {
            base_table_name,
            target_table_name,
            replaced_paths,
            changed_payload: changed_rows,
        } => {
            assert_eq!(base_table_name, previous_publication.table_name);
            assert_ne!(target_table_name, previous_publication.table_name);
            assert_eq!(
                replaced_paths.into_iter().collect::<Vec<_>>(),
                vec!["src/BaseModelica.jl".to_string()]
            );
            assert_eq!(changed_rows.len(), 2);
            assert!(
                changed_rows
                    .iter()
                    .all(|row| row.path() == "src/BaseModelica.jl")
            );
        }
        other => panic!("unexpected second build action: {other:?}"),
    }
}

#[test]
fn plan_repo_entity_build_reuses_table_for_revision_only_refresh() {
    let analysis = sample_analysis("alpha/repo", "reexport", "Shows reexport");
    let documents = sample_documents("reexport", 10);
    let rows =
        rows_from_analysis("alpha/repo", &analysis).unwrap_or_else(|error| panic!("rows: {error}"));
    let file_fingerprints = repo_entity_file_fingerprints(&rows, &documents);
    let table_name =
        versioned_repo_entity_table_name("alpha/repo", &file_fingerprints, Some("rev-1"));
    let publication = SearchRepoPublicationRecord::new(
        SearchCorpusKind::RepoEntity,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: table_name.clone(),
            schema_version: SearchCorpusKind::RepoEntity.schema_version(),
            source_revision: Some("rev-1".to_string()),
            table_version_id: 1,
            row_count: 3,
            fragment_count: 1,
            published_at: "2026-03-24T12:00:00Z".to_string(),
        },
    );
    let plan = plan_repo_entity_build(
        "alpha/repo",
        &rows,
        &documents,
        Some("rev-2"),
        Some(&publication),
        &file_fingerprints,
    );

    match plan.action {
        RepoEntityBuildAction::RefreshPublication { table_name } => {
            assert_eq!(table_name, publication.table_name);
        }
        other => panic!("unexpected build action: {other:?}"),
    }
}

#[tokio::test]
async fn repo_entity_incremental_refresh_reuses_unchanged_rows() {
    let temp_dir = temp_dir_or_panic();
    let service = repo_entity_service(&temp_dir);
    let first_analysis = sample_analysis("alpha/repo", "reexport", "Shows reexport");
    let first_documents = sample_documents("reexport", 10);
    publish_repo_entities(
        &service,
        "alpha/repo",
        &first_analysis,
        &first_documents,
        Some("rev-1"),
    )
    .await
    .unwrap_or_else(|error| panic!("first publish: {error}"));

    let first_record = repo_entity_record_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoEntity, "alpha/repo")
            .await,
        "first repo entity record",
    );
    let first_table_name = repo_entity_publication_or_panic(&first_record, "first publication")
        .table_name
        .clone();
    assert_no_repo_entity_lance_table(
        &service,
        first_table_name.as_str(),
        "repo entity publication should no longer create a Lance table",
    );
    assert_repo_entity_prewarmed(&first_record);

    let second_analysis = sample_analysis("alpha/repo", "solve", "Shows reexport");
    let second_documents = sample_documents("solve", 20);
    publish_repo_entities(
        &service,
        "alpha/repo",
        &second_analysis,
        &second_documents,
        Some("rev-2"),
    )
    .await
    .unwrap_or_else(|error| panic!("second publish: {error}"));

    let second_record = repo_entity_record_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoEntity, "alpha/repo")
            .await,
        "second repo entity record",
    );
    let second_publication = repo_entity_publication_or_panic(&second_record, "second publication");
    assert_ne!(second_publication.table_name, first_table_name);
    assert_no_repo_entity_lance_table(
        &service,
        second_publication.table_name.as_str(),
        "repo entity incremental publication should stay parquet-only",
    );
    assert_eq!(second_publication.source_revision.as_deref(), Some("rev-2"));
    assert_repo_entity_prewarmed(&second_record);

    let kind_filters = HashSet::from_iter([String::from("function")]);
    let language_filters = HashSet::default();
    let empty_kind_filters = HashSet::default();
    assert_repo_entity_hit_stems(
        &service,
        "solve",
        &language_filters,
        &kind_filters,
        &["solve"],
    )
    .await;
    assert_repo_entity_hit_stems(&service, "reexport", &language_filters, &kind_filters, &[]).await;
    assert_repo_entity_hit_paths(
        &service,
        "example",
        &language_filters,
        &empty_kind_filters,
        &["examples/reexport.jl"],
    )
    .await;

    let fingerprints = service
        .file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
            SearchCorpusKind::RepoEntity,
            "alpha/repo",
        ))
        .await;
    assert_eq!(fingerprints.len(), 2);
    assert_eq!(
        fingerprints
            .get("src/BaseModelica.jl")
            .map(|fingerprint| fingerprint.modified_unix_ms),
        Some(20)
    );
}

#[test]
fn plan_repo_entity_build_ignores_metadata_only_edits_when_rows_are_unchanged() {
    let analysis = sample_analysis("alpha/repo", "reexport", "Shows reexport");
    let first_documents = sample_documents("reexport", 10);
    let rows =
        rows_from_analysis("alpha/repo", &analysis).unwrap_or_else(|error| panic!("rows: {error}"));
    let first_plan = plan_repo_entity_build(
        "alpha/repo",
        &rows,
        &first_documents,
        Some("rev-1"),
        None,
        &BTreeMap::new(),
    );
    let previous_publication = match first_plan.action {
        RepoEntityBuildAction::ReplaceAll { ref table_name, .. } => {
            SearchRepoPublicationRecord::new(
                SearchCorpusKind::RepoEntity,
                "alpha/repo",
                SearchRepoPublicationInput {
                    table_name: table_name.clone(),
                    schema_version: SearchCorpusKind::RepoEntity.schema_version(),
                    source_revision: Some("rev-1".to_string()),
                    table_version_id: 1,
                    row_count: rows.len() as u64,
                    fragment_count: 1,
                    published_at: "2026-03-24T12:00:00Z".to_string(),
                },
            )
        }
        other => panic!("unexpected first build action: {other:?}"),
    };

    let second_documents = sample_documents("reexport", 20);
    let second_plan = plan_repo_entity_build(
        "alpha/repo",
        &rows,
        &second_documents,
        Some("rev-2"),
        Some(&previous_publication),
        &first_plan.file_fingerprints,
    );

    let first_table_name = versioned_repo_entity_table_name(
        "alpha/repo",
        &first_plan.file_fingerprints,
        Some("rev-2"),
    );
    let second_table_name = versioned_repo_entity_table_name(
        "alpha/repo",
        &second_plan.file_fingerprints,
        Some("rev-2"),
    );
    assert_eq!(first_table_name, second_table_name);
    assert_eq!(
        first_plan
            .file_fingerprints
            .get("src/BaseModelica.jl")
            .and_then(|fingerprint| fingerprint.blake3.as_deref()),
        second_plan
            .file_fingerprints
            .get("src/BaseModelica.jl")
            .and_then(|fingerprint| fingerprint.blake3.as_deref())
    );
    assert_eq!(
        second_plan
            .file_fingerprints
            .get("src/BaseModelica.jl")
            .map(|fingerprint| fingerprint.modified_unix_ms),
        Some(20)
    );

    match second_plan.action {
        RepoEntityBuildAction::RefreshPublication { table_name } => {
            assert_eq!(table_name, previous_publication.table_name);
        }
        other => panic!("unexpected second build action: {other:?}"),
    }
}

fn versioned_repo_entity_table_name(
    repo_id: &str,
    file_fingerprints: &BTreeMap<String, crate::search::SearchFileFingerprint>,
    source_revision: Option<&str>,
) -> String {
    versioned_repo_table_name(
        SearchPlaneService::repo_entity_table_name(repo_id).as_str(),
        repo_id,
        file_fingerprints,
        source_revision,
        SearchCorpusKind::RepoEntity,
        1,
    )
}
