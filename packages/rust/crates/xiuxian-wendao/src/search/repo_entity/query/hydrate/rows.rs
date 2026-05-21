use std::collections::BTreeMap;

use crate::analyzers::{
    ExampleRecord, ExampleSearchHit, ExampleSearchResult, ImportRecord, ImportSearchHit,
    ImportSearchResult, ModuleRecord, ModuleSearchHit, ModuleSearchResult, SymbolRecord,
    SymbolSearchHit, SymbolSearchResult,
};
use crate::search::repo_entity::query::hydrate::{
    non_empty_vec, parse_attributes_map, parse_backlink_items, parse_import_kind, parse_symbol_kind,
};
use crate::search::repo_entity::query::lookup::{
    HydratedRepoEntityRow, RepoEntityCandidate, RepoEntitySearchError,
};

pub(crate) fn build_module_search_result(
    repo_id: &str,
    candidates: Vec<RepoEntityCandidate>,
    rows: &BTreeMap<String, HydratedRepoEntityRow>,
) -> Result<ModuleSearchResult, RepoEntitySearchError> {
    let mut modules = Vec::with_capacity(candidates.len());
    let mut module_hits = Vec::with_capacity(candidates.len());
    for (index, candidate) in candidates.into_iter().enumerate() {
        let row = rows.get(candidate.id.as_str()).ok_or_else(|| {
            RepoEntitySearchError::Decode(format!(
                "repo entity hydration missing structured row for id `{}`",
                candidate.id
            ))
        })?;
        let module = ModuleRecord {
            repo_id: repo_id.to_string().into(),
            module_id: row.id.clone().into(),
            qualified_name: row.qualified_name.clone(),
            path: row.path.clone().into(),
        };
        modules.push(module.clone());
        module_hits.push(ModuleSearchHit {
            module,
            score: Some(candidate.score),
            rank: Some(index + 1),
            saliency_score: Some(row.saliency_score),
            hierarchical_uri: row.hierarchical_uri.clone(),
            hierarchy: non_empty_vec(row.hierarchy.clone()),
            implicit_backlinks: non_empty_vec(row.implicit_backlinks.clone()),
            implicit_backlink_items: parse_backlink_items(
                row.implicit_backlink_items_json.as_deref(),
            )?,
            projection_page_ids: non_empty_vec(row.projection_page_ids.clone()),
        });
    }

    Ok(ModuleSearchResult {
        repo_id: repo_id.to_string(),
        modules,
        module_hits,
    })
}

pub(crate) fn build_symbol_search_result(
    repo_id: &str,
    candidates: Vec<RepoEntityCandidate>,
    rows: &BTreeMap<String, HydratedRepoEntityRow>,
) -> Result<SymbolSearchResult, RepoEntitySearchError> {
    let mut symbols = Vec::with_capacity(candidates.len());
    let mut symbol_hits = Vec::with_capacity(candidates.len());
    for (index, candidate) in candidates.into_iter().enumerate() {
        let row = rows.get(candidate.id.as_str()).ok_or_else(|| {
            RepoEntitySearchError::Decode(format!(
                "repo entity hydration missing structured row for id `{}`",
                candidate.id
            ))
        })?;
        let audit_status = row.audit_status.clone();
        let verification_state = row.verification_state.clone().or_else(|| {
            audit_status.as_deref().map(|status| match status {
                "verified" | "approved" => "verified".to_string(),
                _ => "unverified".to_string(),
            })
        });
        let symbol = SymbolRecord {
            repo_id: repo_id.to_string().into(),
            symbol_id: row.id.clone().into(),
            module_id: row.module_id.clone().map(Into::into),
            name: row.name.clone(),
            qualified_name: row.qualified_name.clone(),
            kind: parse_symbol_kind(row.symbol_kind.as_str()),
            path: row.path.clone().into(),
            line_start: row.line_start.map(|value| value as usize),
            line_end: row.line_end.map(|value| value as usize),
            signature: row.signature.clone(),
            audit_status: audit_status.clone().map(Into::into),
            verification_state: verification_state.clone().map(Into::into),
            attributes: parse_attributes_map(row.attributes_json.as_deref())?,
        };
        symbols.push(symbol.clone());
        symbol_hits.push(SymbolSearchHit {
            symbol,
            score: Some(candidate.score),
            rank: Some(index + 1),
            saliency_score: Some(row.saliency_score),
            hierarchical_uri: row.hierarchical_uri.clone(),
            hierarchy: non_empty_vec(row.hierarchy.clone()),
            implicit_backlinks: non_empty_vec(row.implicit_backlinks.clone()),
            implicit_backlink_items: parse_backlink_items(
                row.implicit_backlink_items_json.as_deref(),
            )?,
            projection_page_ids: non_empty_vec(row.projection_page_ids.clone()),
            audit_status,
            verification_state,
        });
    }

    Ok(SymbolSearchResult {
        repo_id: repo_id.to_string(),
        symbols,
        symbol_hits,
    })
}

pub(crate) fn build_example_search_result(
    repo_id: &str,
    candidates: Vec<RepoEntityCandidate>,
    rows: &BTreeMap<String, HydratedRepoEntityRow>,
) -> Result<ExampleSearchResult, RepoEntitySearchError> {
    let mut examples = Vec::with_capacity(candidates.len());
    let mut example_hits = Vec::with_capacity(candidates.len());
    for (index, candidate) in candidates.into_iter().enumerate() {
        let row = rows.get(candidate.id.as_str()).ok_or_else(|| {
            RepoEntitySearchError::Decode(format!(
                "repo entity hydration missing structured row for id `{}`",
                candidate.id
            ))
        })?;
        let example = ExampleRecord {
            repo_id: repo_id.to_string().into(),
            example_id: row.id.clone().into(),
            title: row.name.clone(),
            path: row.path.clone().into(),
            summary: row.summary.clone(),
        };
        examples.push(example.clone());
        example_hits.push(ExampleSearchHit {
            example,
            score: Some(candidate.score),
            rank: Some(index + 1),
            saliency_score: Some(row.saliency_score),
            hierarchical_uri: row.hierarchical_uri.clone(),
            hierarchy: non_empty_vec(row.hierarchy.clone()),
            implicit_backlinks: non_empty_vec(row.implicit_backlinks.clone()),
            implicit_backlink_items: parse_backlink_items(
                row.implicit_backlink_items_json.as_deref(),
            )?,
            projection_page_ids: non_empty_vec(row.projection_page_ids.clone()),
        });
    }

    Ok(ExampleSearchResult {
        repo_id: repo_id.to_string(),
        examples,
        example_hits,
    })
}

pub(crate) fn build_import_search_result(
    repo_id: &str,
    candidates: Vec<RepoEntityCandidate>,
    rows: &BTreeMap<String, HydratedRepoEntityRow>,
) -> Result<ImportSearchResult, RepoEntitySearchError> {
    let mut imports = Vec::with_capacity(candidates.len());
    let mut import_hits = Vec::with_capacity(candidates.len());
    for (index, candidate) in candidates.into_iter().enumerate() {
        let row = rows.get(candidate.id.as_str()).ok_or_else(|| {
            RepoEntitySearchError::Decode(format!(
                "repo entity hydration missing structured row for id `{}`",
                candidate.id
            ))
        })?;
        let mut attributes = parse_attributes_map(row.attributes_json.as_deref())?;
        let resolved_id = attributes.remove("resolved_id");
        let import = ImportRecord {
            repo_id: repo_id.to_string().into(),
            module_id: row.module_id.clone().unwrap_or_default().into(),
            path: row.path.clone().into(),
            import_name: row.name.clone(),
            target_package: row.summary.clone().unwrap_or_default(),
            source_module: row.signature.clone().unwrap_or_default(),
            kind: parse_import_kind(row.symbol_kind.as_str()),
            line_start: row.line_start.map(|value| value as usize),
            resolved_id: resolved_id.map(Into::into),
            attributes,
        };
        imports.push(import.clone());
        import_hits.push(ImportSearchHit {
            import,
            score: Some(candidate.score),
            rank: Some(index + 1),
        });
    }

    Ok(ImportSearchResult {
        repo_id: repo_id.to_string(),
        imports,
        import_hits,
    })
}
