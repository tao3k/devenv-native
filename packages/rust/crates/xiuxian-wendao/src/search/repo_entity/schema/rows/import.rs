use std::collections::BTreeMap;

use xiuxian_db_store::VectorStoreError;

use crate::analyzers::ImportRecord;
use crate::gateway::studio::types::SearchHit;
use crate::search::repo_entity::schema::definitions::{ENTITY_KIND_IMPORT, RepoEntityRow};
use crate::search::repo_entity::schema::helpers::{
    import_kind_tag, infer_code_language, repo_entity_tags, repo_navigation_target,
    serialize_hit_json, serialize_symbol_attributes_json,
};
use crate::search::repo_entity::schema::rows::RepoEntityContext;

pub(crate) fn build_import_row(
    context: &RepoEntityContext<'_>,
    import: &ImportRecord,
) -> Result<RepoEntityRow, VectorStoreError> {
    let path = context
        .module_path(import.module_id.as_str())
        .unwrap_or_default()
        .to_string();
    let language = infer_code_language(path.as_str());
    let import_kind = import_kind_tag(import.kind).to_string();
    let saliency_score = context
        .saliency_map
        .get(import.module_id.as_str())
        .copied()
        .unwrap_or(0.0);
    let qualified_name = format!(
        "{}.{}.{}",
        import.target_package, import.source_module, import.import_name
    );
    let attributes = import
        .resolved_id
        .as_ref()
        .map(|resolved_id| BTreeMap::from([(String::from("resolved_id"), resolved_id.clone())]))
        .unwrap_or_default();
    let hit = SearchHit {
        stem: import.import_name.clone(),
        title: Some(qualified_name.clone()),
        path: path.clone(),
        doc_type: Some(ENTITY_KIND_IMPORT.to_string()),
        tags: repo_entity_tags(
            context.repo_id,
            ENTITY_KIND_IMPORT,
            language.clone(),
            Some(import_kind.as_str()),
            None,
        ),
        score: saliency_score,
        best_section: Some(import.source_module.clone()),
        match_reason: Some("repo_import_search".to_string()),
        hierarchical_uri: None,
        hierarchy: None,
        saliency_score: Some(saliency_score),
        audit_status: None,
        verification_state: None,
        implicit_backlinks: None,
        implicit_backlink_items: None,
        navigation_target: (!path.is_empty())
            .then(|| repo_navigation_target(context.repo_id, path.as_str(), Some(1), None)),
    };
    Ok(RepoEntityRow {
        id: format!(
            "import:{}:{}:{}:{}",
            import.module_id, import.target_package, import.source_module, import.import_name
        ),
        entity_kind: ENTITY_KIND_IMPORT.to_string(),
        name: import.import_name.clone(),
        name_folded: import.import_name.to_ascii_lowercase(),
        qualified_name: qualified_name.clone(),
        qualified_name_folded: qualified_name.to_ascii_lowercase(),
        path: path.clone(),
        path_folded: path.to_ascii_lowercase(),
        language: language.unwrap_or_default(),
        symbol_kind: import_kind,
        module_id: Some(import.module_id.clone()),
        signature: Some(import.source_module.clone()),
        signature_folded: import.source_module.to_ascii_lowercase(),
        summary: Some(import.target_package.clone()),
        summary_folded: import.target_package.to_ascii_lowercase(),
        related_symbols_folded: String::new(),
        related_modules_folded: String::new(),
        line_start: Some(1),
        line_end: None,
        audit_status: None,
        verification_state: None,
        attributes_json: serialize_symbol_attributes_json(&attributes)?,
        hierarchical_uri: None,
        hierarchy: Vec::new(),
        implicit_backlinks: Vec::new(),
        implicit_backlink_items_json: None,
        projection_page_ids: Vec::new(),
        saliency_score,
        search_text: [
            import.import_name.as_str(),
            import.target_package.as_str(),
            import.source_module.as_str(),
            qualified_name.as_str(),
            path.as_str(),
        ]
        .join(" "),
        hit_json: serialize_hit_json(&hit)?,
    })
}
