use xiuxian_vector_store::VectorStoreError;

use crate::analyzers::SymbolRecord;
use crate::analyzers::{
    backlinks_for, hierarchy_segments_from_path, projection_pages_for, record_hierarchical_uri,
};
use crate::gateway::studio::types::SearchHit;
use crate::search::repo_entity::schema::definitions::{ENTITY_KIND_SYMBOL, RepoEntityRow};
use crate::search::repo_entity::schema::helpers::{
    infer_code_language, map_backlink_items, repo_entity_tags, repo_navigation_target,
    serialize_backlink_items_json, serialize_hit_json, serialize_symbol_attributes_json,
    symbol_kind_tag,
};
use crate::search::repo_entity::schema::rows::RepoEntityContext;

pub(crate) fn build_symbol_row(
    context: &RepoEntityContext<'_>,
    symbol: &SymbolRecord,
) -> Result<RepoEntityRow, VectorStoreError> {
    let symbol_id = symbol.symbol_id.clone();
    let path = symbol.path.clone();
    let language = infer_code_language(path.as_str());
    let signature = symbol.signature.clone().unwrap_or_default();
    let symbol_kind = symbol_kind_tag(symbol.kind).to_string();
    let hierarchy = hierarchy_segments_from_path(path.as_str());
    let (implicit_backlinks, implicit_backlink_items) =
        backlinks_for(symbol_id.as_str(), &context.backlink_lookup);
    let saliency_score = context
        .saliency_map
        .get(symbol_id.as_str())
        .copied()
        .unwrap_or(0.0);
    let projection_page_ids =
        projection_pages_for(symbol_id.as_str(), &context.projection_lookup).unwrap_or_default();
    let hierarchical_uri = record_hierarchical_uri(
        context.repo_id,
        context.ecosystem,
        "api",
        path.as_str(),
        symbol_id.as_str(),
    );
    let hit = SearchHit {
        stem: symbol.name.clone(),
        title: Some(symbol.qualified_name.clone()),
        path: path.clone(),
        doc_type: Some(ENTITY_KIND_SYMBOL.to_string()),
        tags: repo_entity_tags(
            context.repo_id,
            ENTITY_KIND_SYMBOL,
            language.clone(),
            Some(symbol_kind.as_str()),
            symbol.audit_status.as_deref(),
        ),
        score: saliency_score,
        best_section: symbol
            .signature
            .clone()
            .or_else(|| Some(symbol.qualified_name.clone())),
        match_reason: Some("repo_symbol_search".to_string()),
        hierarchical_uri: Some(hierarchical_uri.clone()),
        hierarchy: hierarchy.clone(),
        saliency_score: Some(saliency_score),
        audit_status: symbol.audit_status.clone(),
        verification_state: symbol.verification_state.clone(),
        implicit_backlinks,
        implicit_backlink_items: map_backlink_items(implicit_backlink_items),
        navigation_target: Some(repo_navigation_target(
            context.repo_id,
            path.as_str(),
            symbol.line_start,
            symbol.line_end,
        )),
    };
    Ok(RepoEntityRow {
        id: symbol_id,
        entity_kind: ENTITY_KIND_SYMBOL.to_string(),
        name: symbol.name.clone(),
        name_folded: symbol.name.to_ascii_lowercase(),
        qualified_name: symbol.qualified_name.clone(),
        qualified_name_folded: symbol.qualified_name.to_ascii_lowercase(),
        path: path.clone(),
        path_folded: path.to_ascii_lowercase(),
        language: language.unwrap_or_default(),
        symbol_kind,
        module_id: symbol.module_id.clone(),
        signature: symbol.signature.clone(),
        signature_folded: signature.to_ascii_lowercase(),
        summary: None,
        summary_folded: String::new(),
        related_symbols_folded: String::new(),
        related_modules_folded: String::new(),
        line_start: symbol
            .line_start
            .and_then(|value| u32::try_from(value).ok()),
        line_end: symbol.line_end.and_then(|value| u32::try_from(value).ok()),
        audit_status: symbol.audit_status.clone(),
        verification_state: symbol.verification_state.clone(),
        attributes_json: serialize_symbol_attributes_json(&symbol.attributes)?,
        hierarchical_uri: Some(hierarchical_uri),
        hierarchy: hierarchy.clone().unwrap_or_default(),
        implicit_backlinks: hit.implicit_backlinks.clone().unwrap_or_default(),
        implicit_backlink_items_json: serialize_backlink_items_json(
            hit.implicit_backlink_items.as_ref(),
        )?,
        projection_page_ids,
        saliency_score,
        search_text: [
            symbol.name.as_str(),
            symbol.qualified_name.as_str(),
            signature.as_str(),
            path.as_str(),
        ]
        .join(" "),
        hit_json: serialize_hit_json(&hit)?,
    })
}
