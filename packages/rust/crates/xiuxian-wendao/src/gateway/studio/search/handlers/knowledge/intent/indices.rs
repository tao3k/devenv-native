use crate::gateway::studio::StudioState;
use crate::gateway::studio::search::handlers::knowledge::intent::types::IntentIndexState;
use crate::search::SearchCorpusKind;

pub(crate) fn ensure_intent_indices(studio: &StudioState) -> IntentIndexState {
    let configured_projects = studio.configured_projects();
    if configured_projects.is_empty() {
        return IntentIndexState {
            knowledge_config_missing: true,
            symbol_config_missing: true,
        };
    }

    let scan_inventory = studio
        .search_plane
        .scan_supported_projects_with_repeat_work_details(
            "knowledge_intent",
            studio.project_root.as_path(),
            studio.config_root.as_path(),
            configured_projects.as_slice(),
        );
    let note_files = scan_inventory.note_files();
    if studio
        .search_plane
        .ensure_knowledge_section_index_started_with_scanned_files(
            studio.project_root.as_path(),
            studio.config_root.as_path(),
            configured_projects.as_slice(),
            note_files.as_slice(),
        )
    {
        studio.record_local_corpus_index_started(
            SearchCorpusKind::KnowledgeSection,
            "knowledge_search",
        );
    }
    if studio
        .search_plane
        .ensure_local_symbol_index_started_with_scanned_files(
            studio.project_root.as_path(),
            studio.config_root.as_path(),
            configured_projects.as_slice(),
            scan_inventory.symbol_files(),
        )
    {
        studio.record_local_corpus_index_started(SearchCorpusKind::LocalSymbol, "symbol_search");
    }

    IntentIndexState {
        knowledge_config_missing: false,
        symbol_config_missing: false,
    }
}
