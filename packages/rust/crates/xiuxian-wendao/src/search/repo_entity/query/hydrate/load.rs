use std::collections::BTreeMap;

use crate::duckdb::ParquetQueryEngine;
use crate::gateway::studio::types::SearchHit;
use crate::search::repo_entity::query::hydrate::{
    engine_float64_column, engine_list_string_column, engine_list_string_values,
    engine_string_column, engine_uint32_column, hit_json_projection_columns, id_filter_expression,
    optional_engine_string_value, optional_engine_u32_value,
};
use crate::search::repo_entity::query::lookup::{
    HydratedRepoEntityRow, RepoEntityCandidate, RepoEntitySearchError,
};

pub(crate) async fn hydrate_repo_entity_hits(
    query_engine: &ParquetQueryEngine,
    table_name: &str,
    candidates: Vec<RepoEntityCandidate>,
) -> Result<Vec<SearchHit>, RepoEntitySearchError> {
    let ids = candidates
        .iter()
        .map(|candidate| candidate.id.clone())
        .collect::<Vec<_>>();
    let payloads = load_hit_payloads_by_id(query_engine, table_name, ids.as_slice()).await?;
    candidates
        .into_iter()
        .map(|candidate| {
            let hit_json = payloads.get(candidate.id.as_str()).ok_or_else(|| {
                RepoEntitySearchError::Decode(format!(
                    "repo entity hydration missing payload for id `{}`",
                    candidate.id
                ))
            })?;
            let mut hit: SearchHit = serde_json::from_str(hit_json.as_str())
                .map_err(|error| RepoEntitySearchError::Decode(error.to_string()))?;
            hit.score = candidate.score;
            Ok(hit)
        })
        .collect()
}

pub(crate) async fn load_hydrated_rows_by_id(
    query_engine: &ParquetQueryEngine,
    table_name: &str,
    ids: &[String],
    projected_columns: &[String],
) -> Result<BTreeMap<String, HydratedRepoEntityRow>, RepoEntitySearchError> {
    if ids.is_empty() {
        return Ok(BTreeMap::new());
    }

    let sql = format!(
        "SELECT {} FROM {table_name} WHERE {}",
        projected_columns.join(", "),
        id_filter_expression(ids),
    );
    let mut rows = BTreeMap::new();
    let batches = query_engine.query_batches(sql.as_str()).await?;

    for batch in batches {
        let id = engine_string_column(&batch, "id")?;
        let name = engine_string_column(&batch, "name")?;
        let qualified_name = engine_string_column(&batch, "qualified_name")?;
        let path = engine_string_column(&batch, "path")?;
        let symbol_kind = engine_string_column(&batch, "symbol_kind")?;
        let module_id = engine_string_column(&batch, "module_id")?;
        let signature = engine_string_column(&batch, "signature")?;
        let summary = engine_string_column(&batch, "summary")?;
        let line_start = engine_uint32_column(&batch, "line_start")?;
        let line_end = engine_uint32_column(&batch, "line_end")?;
        let audit_status = engine_string_column(&batch, "audit_status")?;
        let verification_state = engine_string_column(&batch, "verification_state")?;
        let attributes_json = engine_string_column(&batch, "attributes_json")?;
        let hierarchical_uri = engine_string_column(&batch, "hierarchical_uri")?;
        let hierarchy = engine_list_string_column(&batch, "hierarchy")?;
        let implicit_backlinks = engine_list_string_column(&batch, "implicit_backlinks")?;
        let implicit_backlink_items_json =
            engine_string_column(&batch, "implicit_backlink_items_json")?;
        let projection_page_ids = engine_list_string_column(&batch, "projection_page_ids")?;
        let saliency_score = engine_float64_column(&batch, "saliency_score")?;

        for row in 0..batch.num_rows() {
            let id_value = id.value(row).to_string();
            rows.insert(
                id_value.clone(),
                HydratedRepoEntityRow {
                    id: id_value,
                    name: name.value(row).to_string(),
                    qualified_name: qualified_name.value(row).to_string(),
                    path: path.value(row).to_string(),
                    symbol_kind: symbol_kind.value(row).to_string(),
                    module_id: optional_engine_string_value(module_id, row),
                    signature: optional_engine_string_value(signature, row),
                    summary: optional_engine_string_value(summary, row),
                    line_start: optional_engine_u32_value(line_start, row),
                    line_end: optional_engine_u32_value(line_end, row),
                    audit_status: optional_engine_string_value(audit_status, row),
                    verification_state: optional_engine_string_value(verification_state, row),
                    attributes_json: optional_engine_string_value(attributes_json, row),
                    hierarchical_uri: optional_engine_string_value(hierarchical_uri, row),
                    hierarchy: engine_list_string_values(hierarchy, row),
                    implicit_backlinks: engine_list_string_values(implicit_backlinks, row),
                    implicit_backlink_items_json: optional_engine_string_value(
                        implicit_backlink_items_json,
                        row,
                    ),
                    projection_page_ids: engine_list_string_values(projection_page_ids, row),
                    saliency_score: saliency_score.value(row),
                },
            );
        }
    }

    Ok(rows)
}

pub(crate) async fn load_hit_payloads_by_id(
    query_engine: &ParquetQueryEngine,
    table_name: &str,
    ids: &[String],
) -> Result<BTreeMap<String, String>, RepoEntitySearchError> {
    if ids.is_empty() {
        return Ok(BTreeMap::new());
    }

    let sql = format!(
        "SELECT {} FROM {table_name} WHERE {}",
        hit_json_projection_columns().join(", "),
        id_filter_expression(ids),
    );
    let mut payloads = BTreeMap::new();
    let batches = query_engine.query_batches(sql.as_str()).await?;

    for batch in batches {
        let id = engine_string_column(&batch, "id")?;
        let hit_json = engine_string_column(&batch, "hit_json")?;
        for row in 0..batch.num_rows() {
            payloads.insert(id.value(row).to_string(), hit_json.value(row).to_string());
        }
    }

    Ok(payloads)
}
