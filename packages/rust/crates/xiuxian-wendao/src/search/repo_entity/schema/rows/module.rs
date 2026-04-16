use xiuxian_vector_store::VectorStoreError;

use crate::analyzers::ModuleRecord;
use crate::analyzers::{
    backlinks_for, hierarchy_segments_from_path, projection_pages_for, record_hierarchical_uri,
};
use crate::gateway::studio::types::SearchHit;
use crate::search::repo_entity::schema::definitions::{ENTITY_KIND_MODULE, RepoEntityRow};
use crate::search::repo_entity::schema::helpers::{
    infer_code_language, map_backlink_items, repo_entity_tags, repo_navigation_target,
    serialize_backlink_items_json, serialize_hit_json,
};
use crate::search::repo_entity::schema::rows::RepoEntityContext;

pub(crate) fn build_module_row(
    context: &RepoEntityContext<'_>,
    module: &ModuleRecord,
) -> Result<RepoEntityRow, VectorStoreError> {
    let module_id = module.module_id.clone();
    let path = module.path.clone();
    let language = infer_code_language(path.as_str());
    let hierarchy = hierarchy_segments_from_path(path.as_str());
    let (implicit_backlinks, implicit_backlink_items) =
        backlinks_for(module_id.as_str(), &context.backlink_lookup);
    let saliency_score = context
        .saliency_map
        .get(module_id.as_str())
        .copied()
        .unwrap_or(0.0);
    let projection_page_ids =
        projection_pages_for(module_id.as_str(), &context.projection_lookup).unwrap_or_default();
    let hierarchical_uri = record_hierarchical_uri(
        context.repo_id,
        context.ecosystem,
        "api",
        path.as_str(),
        module_id.as_str(),
    );
    let hit = SearchHit {
        stem: module.qualified_name.clone(),
        title: Some(module.qualified_name.clone()),
        path: path.clone(),
        doc_type: Some(ENTITY_KIND_MODULE.to_string()),
        tags: repo_entity_tags(
            context.repo_id,
            ENTITY_KIND_MODULE,
            language.clone(),
            Some("module"),
            None,
        ),
        score: saliency_score,
        best_section: Some(module.module_id.clone()),
        match_reason: Some("repo_module_search".to_string()),
        hierarchical_uri: Some(hierarchical_uri.clone()),
        hierarchy: hierarchy.clone(),
        saliency_score: Some(saliency_score),
        audit_status: None,
        verification_state: None,
        implicit_backlinks,
        implicit_backlink_items: map_backlink_items(implicit_backlink_items),
        navigation_target: Some(repo_navigation_target(
            context.repo_id,
            path.as_str(),
            Some(1),
            None,
        )),
    };
    Ok(RepoEntityRow {
        id: module_id,
        entity_kind: ENTITY_KIND_MODULE.to_string(),
        name: module.qualified_name.clone(),
        name_folded: module.qualified_name.to_ascii_lowercase(),
        qualified_name: module.qualified_name.clone(),
        qualified_name_folded: module.qualified_name.to_ascii_lowercase(),
        path: path.clone(),
        path_folded: path.to_ascii_lowercase(),
        language: language.unwrap_or_default(),
        symbol_kind: "module".to_string(),
        module_id: Some(module.module_id.clone()),
        signature: None,
        signature_folded: String::new(),
        summary: None,
        summary_folded: String::new(),
        related_symbols_folded: String::new(),
        related_modules_folded: String::new(),
        line_start: Some(1),
        line_end: None,
        audit_status: None,
        verification_state: None,
        attributes_json: None,
        hierarchical_uri: Some(hierarchical_uri),
        hierarchy: hierarchy.clone().unwrap_or_default(),
        implicit_backlinks: hit.implicit_backlinks.clone().unwrap_or_default(),
        implicit_backlink_items_json: serialize_backlink_items_json(
            hit.implicit_backlink_items.as_ref(),
        )?,
        projection_page_ids,
        saliency_score,
        search_text: [module.qualified_name.as_str(), path.as_str()].join(" "),
        hit_json: serialize_hit_json(&hit)?,
    })
}
