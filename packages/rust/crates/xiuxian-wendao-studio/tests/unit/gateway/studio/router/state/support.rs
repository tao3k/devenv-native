use crate::contracts::UiProjectConfig;
use crate::studio::router::state::StudioState;
use std::path::Path;
use xiuxian_wendao::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlanePhase,
    SearchPlaneService,
};

pub(super) async fn warm_start_writer_corpora(
    writer: &SearchPlaneService,
    project_root: &Path,
    projects: &[UiProjectConfig],
) {
    assert!(writer.ensure_knowledge_section_index_started(project_root, project_root, projects));
    assert!(writer.ensure_attachment_index_started(project_root, project_root, projects));
    assert!(writer.ensure_local_symbol_index_started(project_root, project_root, projects));
    assert!(writer.ensure_reference_occurrence_index_started(project_root, project_root, projects));
    wait_for_search_plane_corpus_ready(writer, SearchCorpusKind::KnowledgeSection).await;
    wait_for_search_plane_corpus_ready(writer, SearchCorpusKind::Attachment).await;
    wait_for_search_plane_corpus_ready(writer, SearchCorpusKind::LocalSymbol).await;
    wait_for_search_plane_corpus_ready(writer, SearchCorpusKind::ReferenceOccurrence).await;
}

pub(super) fn assert_warm_started_cold_start_telemetry(studio: &StudioState) {
    let cold_start = studio.search_cold_start_telemetry();
    for corpus in &cold_start.corpora {
        assert!(
            corpus.first_index_started.is_none(),
            "warm-started corpus `{}` should not record a no-op start",
            corpus.corpus
        );
        assert_eq!(
            corpus
                .first_ready_observed
                .as_ref()
                .and_then(|event| event.source.as_deref()),
            Some("search_plane_bootstrap"),
            "warm-started corpus `{}` should keep bootstrap ready telemetry",
            corpus.corpus
        );
    }
}

pub(super) async fn wait_for_local_corpus_ready(studio: &StudioState, corpus: SearchCorpusKind) {
    for _ in 0..200 {
        let status = studio.search_plane.coordinator().status_for(corpus);
        if status.phase == SearchPlanePhase::Ready && status.active_epoch.is_some() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("search corpus `{corpus}` did not reach ready state");
}

pub(super) async fn wait_for_search_plane_corpus_ready(
    search_plane: &SearchPlaneService,
    corpus: SearchCorpusKind,
) {
    for _ in 0..200 {
        let status = search_plane.coordinator().status_for(corpus);
        if status.phase == SearchPlanePhase::Ready && status.active_epoch.is_some() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("search-plane corpus `{corpus}` did not reach ready state");
}

pub(super) async fn wait_for_symbol_index_ready(studio: &StudioState) {
    for _ in 0..200 {
        if studio.current_symbol_index().is_some()
            && matches!(studio.symbol_index_status(), Ok(status) if status.phase == crate::studio::symbol_index::SymbolIndexPhase::Ready)
        {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("studio symbol index did not reach ready state");
}

pub(super) fn search_plane_with_paths(
    project_root: std::path::PathBuf,
    storage_root: std::path::PathBuf,
    keyspace: &str,
) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        project_root,
        storage_root,
        SearchManifestKeyspace::new(keyspace),
        SearchMaintenancePolicy::default(),
    )
}
