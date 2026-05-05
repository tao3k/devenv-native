use xiuxian_db_store::VectorStoreError;

use crate::analyzers::ExampleRecord;
use crate::analyzers::{
    backlinks_for, hierarchy_segments_from_path, projection_pages_for, record_hierarchical_uri,
    related_modules_for_example, related_symbols_for_example,
};
use crate::search::contracts::{SearchBacklinkItem, SearchHit};
use crate::search::repo_entity::schema::definitions::{ENTITY_KIND_EXAMPLE, RepoEntityRow};
use crate::search::repo_entity::schema::helpers::{
    infer_code_language, map_backlink_items, repo_entity_tags, repo_navigation_target,
    serialize_backlink_items_json, serialize_hit_json,
};
use crate::search::repo_entity::schema::rows::RepoEntityContext;

pub(crate) fn build_example_row(
    context: &RepoEntityContext<'_>,
    example: &ExampleRecord,
) -> Result<RepoEntityRow, VectorStoreError> {
    let derived = ExampleRowDerivedData::from_context(context, example);
    let hit = build_example_search_hit(context, example, &derived);
    Ok(RepoEntityRow {
        id: derived.example_id.clone(),
        entity_kind: ENTITY_KIND_EXAMPLE.to_string(),
        name: example.title.clone(),
        name_folded: example.title.to_ascii_lowercase(),
        qualified_name: example.title.clone(),
        qualified_name_folded: example.title.to_ascii_lowercase(),
        path: derived.path.clone(),
        path_folded: derived.path.to_ascii_lowercase(),
        language: derived.language.clone().unwrap_or_default(),
        symbol_kind: "example".to_string(),
        module_id: None,
        signature: None,
        signature_folded: String::new(),
        summary: example.summary.clone(),
        summary_folded: derived.summary_text.to_ascii_lowercase(),
        related_symbols_folded: derived.related_symbols.join("\n"),
        related_modules_folded: derived.related_modules.join("\n"),
        line_start: Some(1),
        line_end: None,
        audit_status: None,
        verification_state: None,
        attributes_json: None,
        hierarchical_uri: Some(derived.hierarchical_uri.clone()),
        hierarchy: derived.hierarchy.clone().unwrap_or_default(),
        implicit_backlinks: hit.implicit_backlinks.clone().unwrap_or_default(),
        implicit_backlink_items_json: serialize_backlink_items_json(
            hit.implicit_backlink_items.as_ref(),
        )?,
        projection_page_ids: derived.projection_page_ids.clone(),
        saliency_score: derived.saliency_score,
        search_text: build_example_search_text(
            example.title.as_str(),
            derived.summary_text.as_str(),
            derived.related_symbols_text.as_str(),
            derived.related_modules_text.as_str(),
            derived.path.as_str(),
        ),
        hit_json: serialize_hit_json(&hit)?,
    })
}

struct ExampleRowDerivedData {
    example_id: String,
    path: String,
    language: Option<String>,
    summary_text: String,
    hierarchy: Option<Vec<String>>,
    related_symbols: Vec<String>,
    related_modules: Vec<String>,
    related_symbols_text: String,
    related_modules_text: String,
    implicit_backlinks: Option<Vec<String>>,
    implicit_backlink_items: Option<Vec<SearchBacklinkItem>>,
    saliency_score: f64,
    projection_page_ids: Vec<String>,
    hierarchical_uri: String,
}

impl ExampleRowDerivedData {
    fn from_context(context: &RepoEntityContext<'_>, example: &ExampleRecord) -> Self {
        let example_id = example.example_id.clone();
        let path = example.path.clone();
        let language = infer_code_language(path.as_str());
        let summary_text = example.summary.clone().unwrap_or_default();
        let hierarchy = hierarchy_segments_from_path(path.as_str());
        let related_symbols = related_symbols_for_example(
            example_id.as_str(),
            &context.example_relations,
            &context.analysis.symbols,
        );
        let related_modules = related_modules_for_example(
            example_id.as_str(),
            &context.example_relations,
            &context.analysis.modules,
        );
        let (implicit_backlinks, implicit_backlink_items) =
            backlinks_for(example_id.as_str(), &context.backlink_lookup);

        Self {
            projection_page_ids: projection_pages_for(
                example_id.as_str(),
                &context.projection_lookup,
            )
            .unwrap_or_default(),
            saliency_score: context
                .saliency_map
                .get(example_id.as_str())
                .copied()
                .unwrap_or(0.0),
            hierarchical_uri: record_hierarchical_uri(
                context.repo_id,
                context.ecosystem,
                "examples",
                path.as_str(),
                example_id.as_str(),
            ),
            related_symbols_text: related_symbols.join(" "),
            related_modules_text: related_modules.join(" "),
            implicit_backlinks,
            implicit_backlink_items: map_backlink_items(implicit_backlink_items),
            example_id,
            path,
            language,
            summary_text,
            hierarchy,
            related_symbols,
            related_modules,
        }
    }
}

fn build_example_search_hit(
    context: &RepoEntityContext<'_>,
    example: &ExampleRecord,
    derived: &ExampleRowDerivedData,
) -> SearchHit {
    SearchHit {
        stem: example.title.clone(),
        title: Some(example.title.clone()),
        path: derived.path.clone(),
        doc_type: Some(ENTITY_KIND_EXAMPLE.to_string()),
        tags: repo_entity_tags(
            context.repo_id,
            ENTITY_KIND_EXAMPLE,
            derived.language.clone(),
            Some("example"),
            None,
        ),
        score: derived.saliency_score,
        best_section: example.summary.clone(),
        match_reason: Some("repo_example_search".to_string()),
        hierarchical_uri: Some(derived.hierarchical_uri.clone()),
        hierarchy: derived.hierarchy.clone(),
        saliency_score: Some(derived.saliency_score),
        audit_status: None,
        verification_state: None,
        implicit_backlinks: derived.implicit_backlinks.clone(),
        implicit_backlink_items: derived.implicit_backlink_items.clone(),
        navigation_target: Some(repo_navigation_target(
            context.repo_id,
            derived.path.as_str(),
            Some(1),
            None,
        )),
    }
}

fn build_example_search_text(
    title: &str,
    summary: &str,
    related_symbols_text: &str,
    related_modules_text: &str,
    path: &str,
) -> String {
    [
        title,
        summary,
        related_symbols_text,
        related_modules_text,
        path,
    ]
    .join(" ")
}
