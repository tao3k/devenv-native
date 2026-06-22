use std::sync::Arc;
use std::time::Duration;

use crate::studio::router::error::StudioApiError;
use crate::studio::router::state::types::StudioState;
use crate::studio::types::{
    AttachmentSearchHit, AutocompleteSuggestion, ReferenceSearchHit, SearchHit, SourceSymbolHit,
};
use xiuxian_wendao::link_graph::LinkGraphAttachmentKind;
use xiuxian_wendao::search::{SearchCorpusKind, SearchPlanePhase};

const LOCAL_CORPUS_READY_WAIT_ENV: &str = "XIUXIAN_WENDAO_LOCAL_CORPUS_READY_WAIT_MS";
const DEFAULT_LOCAL_CORPUS_READY_WAIT_MS: u64 = 15_000;
const LOCAL_CORPUS_READY_POLL_INTERVAL: Duration = Duration::from_millis(25);
const LOCAL_CORPUS_SCAN_COALESCE_WINDOW: Duration = Duration::from_millis(100);
const NOTE_SEARCH_BUNDLE_SOURCE: &str = "note_search_bundle";
const CODE_SEARCH_BUNDLE_SOURCE: &str = "code_search_bundle";

#[derive(Clone, Copy)]
enum LocalCorpusScanBundle {
    Note,
    Code,
}

#[derive(Debug, Clone)]
pub(crate) struct LocalCorpusBootstrapStatus {
    pub(crate) active_epoch_ready: bool,
    pub(crate) indexing_state: &'static str,
    pub(crate) index_error: Option<String>,
}

impl StudioState {
    fn local_corpus_bundle_ready(&self, corpora: &[SearchCorpusKind]) -> bool {
        corpora.iter().copied().all(|corpus| {
            self.search_plane
                .coordinator()
                .status_for(corpus)
                .active_epoch
                .is_some()
        })
    }

    fn local_corpus_bundle_indexing(&self, corpora: &[SearchCorpusKind]) -> bool {
        corpora.iter().copied().any(|corpus| {
            let status = self.search_plane.coordinator().status_for(corpus);
            matches!(status.phase, SearchPlanePhase::Indexing)
        })
    }

    fn local_corpus_scan_recent(&self, bundle: LocalCorpusScanBundle) -> bool {
        let state = self
            .local_corpus_scan_coalescing
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let last_scanned_at = match bundle {
            LocalCorpusScanBundle::Note => state.note_bundle_last_scanned_at,
            LocalCorpusScanBundle::Code => state.code_bundle_last_scanned_at,
        };
        last_scanned_at
            .is_some_and(|instant| instant.elapsed() <= LOCAL_CORPUS_SCAN_COALESCE_WINDOW)
    }

    fn should_coalesce_local_corpus_scan(
        &self,
        bundle: LocalCorpusScanBundle,
        corpora: &[SearchCorpusKind],
    ) -> bool {
        self.local_corpus_bundle_ready(corpora) && self.local_corpus_scan_recent(bundle)
    }

    fn record_local_corpus_scan(&self, bundle: LocalCorpusScanBundle) {
        let mut state = self
            .local_corpus_scan_coalescing
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match bundle {
            LocalCorpusScanBundle::Note => {
                state.note_bundle_last_scanned_at = Some(std::time::Instant::now());
            }
            LocalCorpusScanBundle::Code => {
                state.code_bundle_last_scanned_at = Some(std::time::Instant::now());
            }
        }
    }

    fn ensure_note_search_indexes_started(
        &self,
        configured_projects: &[crate::studio::types::UiProjectConfig],
        source: &'static str,
    ) {
        let corpora = [
            SearchCorpusKind::KnowledgeSection,
            SearchCorpusKind::Attachment,
        ];
        if self.local_corpus_bundle_indexing(&corpora)
            || self.should_coalesce_local_corpus_scan(LocalCorpusScanBundle::Note, &corpora)
        {
            return;
        }

        let scan_inventory = self
            .search_plane
            .scan_supported_projects_with_repeat_work_details(
                NOTE_SEARCH_BUNDLE_SOURCE,
                self.project_root.as_path(),
                self.config_root.as_path(),
                configured_projects,
            );
        let note_files = scan_inventory.note_files();
        if self
            .search_plane
            .ensure_knowledge_section_index_started_with_scanned_files(
                self.project_root.as_path(),
                self.config_root.as_path(),
                configured_projects,
                note_files.as_slice(),
            )
        {
            self.record_local_corpus_index_started(SearchCorpusKind::KnowledgeSection, source);
        }
        if self
            .search_plane
            .ensure_attachment_index_started_with_scanned_files(
                self.project_root.as_path(),
                self.config_root.as_path(),
                configured_projects,
                note_files.as_slice(),
            )
        {
            self.record_local_corpus_index_started(SearchCorpusKind::Attachment, source);
        }
        self.record_local_corpus_scan(LocalCorpusScanBundle::Note);
    }

    fn ensure_code_search_indexes_started(
        &self,
        configured_projects: &[crate::studio::types::UiProjectConfig],
        source: &'static str,
    ) {
        let corpora = [
            SearchCorpusKind::LocalSymbol,
            SearchCorpusKind::ReferenceOccurrence,
        ];
        let bundle_indexing = self.local_corpus_bundle_indexing(&corpora);
        if !bundle_indexing
            && !self.should_coalesce_local_corpus_scan(LocalCorpusScanBundle::Code, &corpora)
        {
            let scan_inventory = self
                .search_plane
                .scan_supported_projects_with_repeat_work_details(
                    CODE_SEARCH_BUNDLE_SOURCE,
                    self.project_root.as_path(),
                    self.config_root.as_path(),
                    configured_projects,
                );
            let source_files = scan_inventory.source_files();
            if self
                .search_plane
                .ensure_local_symbol_index_started_with_scanned_files(
                    self.project_root.as_path(),
                    self.config_root.as_path(),
                    configured_projects,
                    scan_inventory.symbol_files(),
                )
            {
                self.record_local_corpus_index_started(SearchCorpusKind::LocalSymbol, source);
            }
            if self
                .search_plane
                .ensure_reference_occurrence_index_started_with_scanned_files(
                    self.project_root.as_path(),
                    self.config_root.as_path(),
                    configured_projects,
                    source_files.as_slice(),
                )
            {
                self.record_local_corpus_index_started(
                    SearchCorpusKind::ReferenceOccurrence,
                    source,
                );
            }
            self.record_local_corpus_scan(LocalCorpusScanBundle::Code);
        }
        self.symbol_index_coordinator
            .sync_projects(configured_projects.to_vec(), Arc::clone(&self.symbol_index));
    }

    async fn wait_for_initial_local_corpus_ready(
        &self,
        corpus: SearchCorpusKind,
    ) -> Result<(), StudioApiError> {
        let timeout = local_corpus_ready_wait_duration();
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let status = self.search_plane.coordinator().status_for(corpus);
            if status.active_epoch.is_some() {
                return Ok(());
            }
            if matches!(status.phase, SearchPlanePhase::Failed) {
                return Err(StudioApiError::internal(
                    "SEARCH_INDEX_BUILD_FAILED",
                    format!("search corpus `{corpus}` failed to publish an index epoch"),
                    status.last_error.clone(),
                ));
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(StudioApiError::index_not_ready(corpus.as_str()));
            }
            tokio::time::sleep(LOCAL_CORPUS_READY_POLL_INTERVAL).await;
        }
    }

    pub(crate) fn ensure_local_symbol_index_started(&self) -> Result<(), StudioApiError> {
        let configured_projects = self.configured_projects();
        if configured_projects.is_empty() {
            return Err(StudioApiError::bad_request(
                "UI_CONFIG_REQUIRED",
                "Studio symbol search requires configured sources.projects",
            ));
        }
        self.ensure_code_search_indexes_started(configured_projects.as_slice(), "symbol_search");
        Ok(())
    }

    pub(crate) async fn ensure_local_symbol_index_ready(&self) -> Result<(), StudioApiError> {
        self.ensure_local_symbol_index_started()?;
        self.wait_for_initial_local_corpus_ready(SearchCorpusKind::LocalSymbol)
            .await
    }

    pub(crate) fn ensure_knowledge_section_index_started(&self) -> Result<(), StudioApiError> {
        let configured_projects = self.configured_projects();
        if configured_projects.is_empty() {
            return Err(StudioApiError::bad_request(
                "UI_CONFIG_REQUIRED",
                "Studio knowledge search requires configured sources.projects",
            ));
        }
        self.ensure_note_search_indexes_started(configured_projects.as_slice(), "knowledge_search");
        Ok(())
    }

    #[must_use]
    pub(crate) fn local_corpus_bootstrap_status(
        &self,
        corpus: SearchCorpusKind,
        source: &'static str,
    ) -> LocalCorpusBootstrapStatus {
        let status = self.search_plane.coordinator().status_for(corpus);
        if status.active_epoch.is_some() {
            if let Some(build_finished_at) = status.build_finished_at.as_deref() {
                self.record_local_corpus_ready_observed_with_recorded_at(
                    corpus,
                    build_finished_at,
                    source,
                );
            } else {
                self.record_local_corpus_ready_observed(corpus, source);
            }
        }
        LocalCorpusBootstrapStatus {
            active_epoch_ready: status.active_epoch.is_some(),
            indexing_state: search_plane_phase_label(status.phase),
            index_error: status.last_error,
        }
    }

    pub(crate) async fn search_knowledge_sections(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchHit>, StudioApiError> {
        match self
            .search_plane
            .search_knowledge_sections(query, limit)
            .await
        {
            Ok(hits) => Ok(hits.into_iter().map(Into::into).collect()),
            Err(xiuxian_wendao::search::KnowledgeSectionSearchError::NotReady) => {
                Err(StudioApiError::index_not_ready("knowledge_section"))
            }
            Err(error) => Err(StudioApiError::internal(
                "KNOWLEDGE_SECTION_SEARCH_FAILED",
                "Failed to query knowledge section search plane",
                Some(error.to_string()),
            )),
        }
    }

    pub(crate) async fn search_local_symbol_hits(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SourceSymbolHit>, StudioApiError> {
        match self.search_plane.search_local_symbols(query, limit).await {
            Ok(hits) => Ok(hits.into_iter().map(Into::into).collect()),
            Err(xiuxian_wendao::search::LocalSymbolSearchError::NotReady) => {
                Err(StudioApiError::index_not_ready("local_symbol"))
            }
            Err(error) => Err(StudioApiError::internal(
                "LOCAL_SYMBOL_SEARCH_FAILED",
                "Failed to query local symbol search plane",
                Some(error.to_string()),
            )),
        }
    }

    pub(crate) async fn autocomplete_local_symbols(
        &self,
        prefix: &str,
        limit: usize,
    ) -> Result<Vec<AutocompleteSuggestion>, StudioApiError> {
        match self
            .search_plane
            .autocomplete_local_symbols(prefix, limit)
            .await
        {
            Ok(suggestions) => Ok(suggestions.into_iter().map(Into::into).collect()),
            Err(xiuxian_wendao::search::LocalSymbolSearchError::NotReady) => {
                Err(StudioApiError::index_not_ready("local_symbol"))
            }
            Err(error) => Err(StudioApiError::internal(
                "LOCAL_SYMBOL_AUTOCOMPLETE_FAILED",
                "Failed to query local symbol autocomplete search plane",
                Some(error.to_string()),
            )),
        }
    }

    pub(crate) fn ensure_attachment_index_started(&self) -> Result<(), StudioApiError> {
        let configured_projects = self.configured_projects();
        if configured_projects.is_empty() {
            return Err(StudioApiError::bad_request(
                "UI_CONFIG_REQUIRED",
                "Studio attachment search requires configured sources.projects",
            ));
        }
        self.ensure_note_search_indexes_started(
            configured_projects.as_slice(),
            "attachment_search",
        );
        Ok(())
    }

    pub(crate) async fn search_attachment_hits(
        &self,
        query: &str,
        limit: usize,
        extensions: &[String],
        kinds: &[LinkGraphAttachmentKind],
        case_sensitive: bool,
    ) -> Result<Vec<AttachmentSearchHit>, StudioApiError> {
        match self
            .search_plane
            .search_attachment_hits(query, limit, extensions, kinds, case_sensitive)
            .await
        {
            Ok(hits) => Ok(hits.into_iter().map(Into::into).collect()),
            Err(xiuxian_wendao::search::AttachmentSearchError::NotReady) => {
                Err(StudioApiError::index_not_ready("attachment"))
            }
            Err(error) => Err(StudioApiError::internal(
                "ATTACHMENT_SEARCH_FAILED",
                "Failed to query attachment search plane",
                Some(error.to_string()),
            )),
        }
    }

    pub(crate) fn ensure_reference_occurrence_index_started(&self) -> Result<(), StudioApiError> {
        let configured_projects = self.configured_projects();
        if configured_projects.is_empty() {
            return Err(StudioApiError::bad_request(
                "UI_CONFIG_REQUIRED",
                "Studio reference search requires configured sources.projects",
            ));
        }
        self.ensure_code_search_indexes_started(configured_projects.as_slice(), "reference_search");
        Ok(())
    }

    pub(crate) async fn search_reference_occurrences(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ReferenceSearchHit>, StudioApiError> {
        match self
            .search_plane
            .search_reference_occurrences(query, limit)
            .await
        {
            Ok(hits) => Ok(hits.into_iter().map(Into::into).collect()),
            Err(xiuxian_wendao::search::ReferenceOccurrenceSearchError::NotReady) => {
                Err(StudioApiError::index_not_ready("reference_occurrence"))
            }
            Err(error) => Err(StudioApiError::internal(
                "REFERENCE_OCCURRENCE_SEARCH_FAILED",
                "Failed to query reference occurrence search plane",
                Some(error.to_string()),
            )),
        }
    }
}

fn local_corpus_ready_wait_duration() -> Duration {
    let parsed = std::env::var(LOCAL_CORPUS_READY_WAIT_ENV)
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|value| *value > 0);
    Duration::from_millis(parsed.unwrap_or(DEFAULT_LOCAL_CORPUS_READY_WAIT_MS))
}

#[must_use]
pub(crate) fn search_plane_phase_label(phase: SearchPlanePhase) -> &'static str {
    match phase {
        SearchPlanePhase::Idle => "idle",
        SearchPlanePhase::Indexing => "indexing",
        SearchPlanePhase::Ready => "ready",
        SearchPlanePhase::Degraded => "degraded",
        SearchPlanePhase::Failed => "failed",
    }
}
