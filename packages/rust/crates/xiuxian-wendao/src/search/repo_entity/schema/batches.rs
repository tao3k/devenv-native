use std::sync::Arc;

use xiuxian_db_store::{
    LanceArray, LanceFloat64Array, LanceListArray, LanceListBuilder, LanceRecordBatch,
    LanceStringArray, LanceStringBuilder, LanceUInt32Array, VectorStoreError,
};

use crate::search::repo_entity::schema::definitions::RepoEntityRow;
use crate::search::repo_entity::schema::rows::repo_entity_schema;

const CHUNK_SIZE: usize = 1_000;

pub(crate) fn repo_entity_batches(
    rows: &[RepoEntityRow],
) -> Result<Vec<LanceRecordBatch>, VectorStoreError> {
    rows.chunks(CHUNK_SIZE)
        .map(batch_from_rows)
        .collect::<Result<Vec<_>, _>>()
}

fn batch_from_rows(rows: &[RepoEntityRow]) -> Result<LanceRecordBatch, VectorStoreError> {
    let hierarchy = build_utf8_list_array(&collect_list_rows(rows, |row| row.hierarchy.as_slice()));
    let implicit_backlinks = build_utf8_list_array(&collect_list_rows(rows, |row| {
        row.implicit_backlinks.as_slice()
    }));
    let projection_page_ids = build_utf8_list_array(&collect_list_rows(rows, |row| {
        row.projection_page_ids.as_slice()
    }));

    LanceRecordBatch::try_new(
        repo_entity_schema(),
        build_repo_entity_arrays(rows, hierarchy, implicit_backlinks, projection_page_ids),
    )
    .map_err(VectorStoreError::Arrow)
}

fn build_repo_entity_arrays(
    rows: &[RepoEntityRow],
    hierarchy: LanceListArray,
    implicit_backlinks: LanceListArray,
    projection_page_ids: LanceListArray,
) -> Vec<Arc<dyn LanceArray>> {
    vec![
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.id.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.entity_kind.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.name.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.name_folded.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.qualified_name.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.qualified_name_folded.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.path.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.path_folded.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.language.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.symbol_kind.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_optional_string_column(
            rows,
            |row| row.module_id.clone(),
        ))),
        Arc::new(LanceStringArray::from(collect_optional_string_column(
            rows,
            |row| row.signature.clone(),
        ))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.signature_folded.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_optional_string_column(
            rows,
            |row| row.summary.clone(),
        ))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.summary_folded.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.related_symbols_folded.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.related_modules_folded.clone()
        }))),
        Arc::new(LanceUInt32Array::from(collect_optional_u32_column(
            rows,
            |row| row.line_start,
        ))),
        Arc::new(LanceUInt32Array::from(collect_optional_u32_column(
            rows,
            |row| row.line_end,
        ))),
        Arc::new(LanceStringArray::from(collect_optional_string_column(
            rows,
            |row| row.audit_status.clone(),
        ))),
        Arc::new(LanceStringArray::from(collect_optional_string_column(
            rows,
            |row| row.verification_state.clone(),
        ))),
        Arc::new(LanceStringArray::from(collect_optional_string_column(
            rows,
            |row| row.attributes_json.clone(),
        ))),
        Arc::new(LanceStringArray::from(collect_optional_string_column(
            rows,
            |row| row.hierarchical_uri.clone(),
        ))),
        Arc::new(hierarchy),
        Arc::new(implicit_backlinks),
        Arc::new(LanceStringArray::from(collect_optional_string_column(
            rows,
            |row| row.implicit_backlink_items_json.clone(),
        ))),
        Arc::new(projection_page_ids),
        Arc::new(LanceFloat64Array::from(collect_f64_column(rows, |row| {
            row.saliency_score
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.search_text.clone()
        }))),
        Arc::new(LanceStringArray::from(collect_string_column(rows, |row| {
            row.hit_json.clone()
        }))),
    ]
}

fn collect_string_column<F>(rows: &[RepoEntityRow], accessor: F) -> Vec<String>
where
    F: FnMut(&RepoEntityRow) -> String,
{
    rows.iter().map(accessor).collect::<Vec<_>>()
}

fn collect_optional_string_column<F>(rows: &[RepoEntityRow], accessor: F) -> Vec<Option<String>>
where
    F: FnMut(&RepoEntityRow) -> Option<String>,
{
    rows.iter().map(accessor).collect::<Vec<_>>()
}

fn collect_optional_u32_column<F>(rows: &[RepoEntityRow], accessor: F) -> Vec<Option<u32>>
where
    F: FnMut(&RepoEntityRow) -> Option<u32>,
{
    rows.iter().map(accessor).collect::<Vec<_>>()
}

fn collect_f64_column<F>(rows: &[RepoEntityRow], accessor: F) -> Vec<f64>
where
    F: FnMut(&RepoEntityRow) -> f64,
{
    rows.iter().map(accessor).collect::<Vec<_>>()
}

fn collect_list_rows<'a, F>(rows: &'a [RepoEntityRow], accessor: F) -> Vec<&'a [String]>
where
    F: FnMut(&'a RepoEntityRow) -> &'a [String],
{
    rows.iter().map(accessor).collect::<Vec<_>>()
}

fn build_utf8_list_array(rows: &[&[String]]) -> LanceListArray {
    let mut builder = LanceListBuilder::new(LanceStringBuilder::new());
    for row in rows {
        for value in *row {
            builder.values().append_value(value);
        }
        builder.append(true);
    }
    builder.finish()
}
