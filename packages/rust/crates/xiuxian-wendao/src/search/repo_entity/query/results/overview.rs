use std::collections::BTreeSet;

use crate::search::SearchPlaneService;
use crate::search::repo_entity::query::hydrate::{
    engine_list_string_column, engine_list_string_values, engine_string_column,
};
use crate::search::repo_entity::query::lookup::{
    PreparedRepoEntityPublication, RepoEntitySearchError, prepare_repo_entity_publication,
};
use crate::search::repo_entity::schema::{
    COLUMN_ENTITY_KIND, COLUMN_PROJECTION_PAGE_IDS, COLUMN_QUALIFIED_NAME, ENTITY_KIND_EXAMPLE,
    ENTITY_KIND_MODULE, ENTITY_KIND_SYMBOL,
};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Summary statistics for the repository entity publication overview.
pub struct RepoEntityOverviewSummary {
    /// Display name inferred from the shallowest module entity.
    pub display_name: Option<String>,
    /// Source revision associated with the indexed repository publication.
    pub source_revision: Option<String>,
    /// Number of module entities in the publication.
    pub module_count: usize,
    /// Number of symbol entities in the publication.
    pub symbol_count: usize,
    /// Number of example entities in the publication.
    pub example_count: usize,
    /// Number of projected documentation pages referenced by entities.
    pub doc_count: usize,
}

/// Build a compact repository entity overview from the published entity table.
///
/// # Errors
///
/// Returns a repository entity search error when the underlying publication
/// cannot be opened or decoded.
pub async fn summarize_repo_entity_overview(
    service: &SearchPlaneService,
    repo_id: &str,
) -> Result<Option<RepoEntityOverviewSummary>, RepoEntitySearchError> {
    let Some(prepared) = prepare_repo_entity_publication(service, repo_id).await? else {
        return Ok(None);
    };
    let PreparedRepoEntityPublication {
        _read_permit,
        query_engine,
        engine_table_name,
        source_revision,
    } = prepared;
    let batches = query_engine
        .query_batches(
            format!(
                "SELECT {COLUMN_ENTITY_KIND}, {COLUMN_QUALIFIED_NAME}, {COLUMN_PROJECTION_PAGE_IDS} FROM {engine_table_name}"
            )
            .as_str(),
        )
        .await?;
    let mut summary = RepoEntityOverviewSummary {
        display_name: None,
        source_revision,
        module_count: 0,
        symbol_count: 0,
        example_count: 0,
        doc_count: 0,
    };
    let mut projected_pages = BTreeSet::new();

    for batch in batches {
        let entity_kind = engine_string_column(&batch, COLUMN_ENTITY_KIND)?;
        let qualified_name = engine_string_column(&batch, COLUMN_QUALIFIED_NAME)?;
        let projection_page_ids = engine_list_string_column(&batch, COLUMN_PROJECTION_PAGE_IDS)?;
        for row in 0..batch.num_rows() {
            match entity_kind.value(row) {
                ENTITY_KIND_MODULE => {
                    summary.module_count += 1;
                    update_display_name_candidate(
                        &mut summary.display_name,
                        qualified_name.value(row),
                    );
                }
                ENTITY_KIND_SYMBOL => summary.symbol_count += 1,
                ENTITY_KIND_EXAMPLE => summary.example_count += 1,
                _ => {}
            }
            projected_pages.extend(engine_list_string_values(projection_page_ids, row));
        }
    }

    summary.doc_count = projected_pages.len();
    Ok(Some(summary))
}

fn update_display_name_candidate(display_name: &mut Option<String>, candidate: &str) {
    let candidate = candidate.trim();
    if candidate.is_empty() {
        return;
    }
    match display_name {
        Some(current) if !should_replace_display_name(current.as_str(), candidate) => {}
        Some(current) => *current = candidate.to_string(),
        None => *display_name = Some(candidate.to_string()),
    }
}

fn should_replace_display_name(current: &str, candidate: &str) -> bool {
    let current_depth = current.matches('.').count();
    let candidate_depth = candidate.matches('.').count();
    if candidate_depth != current_depth {
        return candidate_depth < current_depth;
    }
    candidate.len() < current.len()
}
