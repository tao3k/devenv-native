use std::collections::HashMap;
use std::sync::Arc;

use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, LanceArrayRef, LanceBooleanArray, LanceInt32Array,
    LanceRecordBatch, LanceSchema, LanceStringArray, LanceUInt64Array, WENDAO_TABLE_METADATA_KEY,
    build_arrow_schema, validate_record_batch_schema_with_options,
};

use crate::analyzers::{
    ProjectionPageKind, RepoProjectedPageIndexTreeResult, RepoProjectedRetrievalContextResult,
};
use crate::query_core::WendaoGraphProjection;

const PAGE_INDEX_TREE_TABLE: &str = "flight_repo_projected_page_index_tree";
const RETRIEVAL_CONTEXT_TABLE: &str = "flight_repo_projected_retrieval_context";
const GRAPH_NEIGHBORS_TABLE: &str = "flight_graph_neighbors_response";

pub(crate) fn repo_projected_page_index_tree_batch(
    response: &RepoProjectedPageIndexTreeResult,
) -> Result<LanceRecordBatch, String> {
    let tree = response
        .tree
        .as_ref()
        .ok_or_else(|| "repo projected page-index tree payload is missing `tree`".to_string())?;
    let roots_json = serde_json::to_string(tree.roots.as_slice())
        .map_err(|error| format!("failed to encode projected page-index roots: {error}"))?;
    let root_count = u64::try_from(tree.root_count)
        .map_err(|error| format!("failed to represent projected page-index root count: {error}"))?;

    record_batch(
        &page_index_tree_contract(),
        vec![
            Arc::new(LanceStringArray::from(vec![tree.repo_id.as_str()])),
            Arc::new(LanceStringArray::from(vec![tree.page_id.as_str()])),
            Arc::new(LanceStringArray::from(vec![projection_page_kind_token(
                tree.kind,
            )])),
            Arc::new(LanceStringArray::from(vec![tree.path.as_str()])),
            Arc::new(LanceStringArray::from(vec![tree.doc_id.as_str()])),
            Arc::new(LanceStringArray::from(vec![tree.title.as_str()])),
            Arc::new(LanceUInt64Array::from(vec![root_count])),
            Arc::new(LanceStringArray::from(vec![roots_json.as_str()])),
        ],
        "failed to build projected page-index tree Flight batch",
    )
}

pub(crate) fn repo_projected_page_index_tree_metadata(
    response: &RepoProjectedPageIndexTreeResult,
) -> Result<Vec<u8>, String> {
    let tree = response
        .tree
        .as_ref()
        .ok_or_else(|| "repo projected page-index tree payload is missing `tree`".to_string())?;
    serde_json::to_vec(&serde_json::json!({
        "repoId": tree.repo_id,
        "pageId": tree.page_id,
        "kind": projection_page_kind_token(tree.kind),
        "path": tree.path,
        "docId": tree.doc_id,
        "title": tree.title,
        "rootCount": tree.root_count,
    }))
    .map_err(|error| error.to_string())
}

pub(crate) fn repo_projected_retrieval_context_batch(
    response: &RepoProjectedRetrievalContextResult,
    requested_node_id: Option<&str>,
) -> Result<LanceRecordBatch, String> {
    let center_json = serde_json::to_string(&response.center)
        .map_err(|error| format!("failed to encode retrieval-context center: {error}"))?;
    let related_pages_json = serde_json::to_string(response.related_pages.as_slice())
        .map_err(|error| format!("failed to encode retrieval-context related pages: {error}"))?;
    let node_context_json = response
        .node_context
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| format!("failed to encode retrieval-context node context: {error}"))?;
    let related_count = u64::try_from(response.related_pages.len())
        .map_err(|error| format!("failed to represent related page count: {error}"))?;
    let node_id = response_node_id(response, requested_node_id);

    record_batch(
        &retrieval_context_contract(),
        vec![
            Arc::new(LanceStringArray::from(vec![response.repo_id.as_str()])),
            Arc::new(LanceStringArray::from(vec![
                response.center.page.page_id.as_str(),
            ])),
            Arc::new(LanceStringArray::from(vec![node_id])),
            Arc::new(LanceStringArray::from(vec![center_json.as_str()])),
            Arc::new(LanceUInt64Array::from(vec![related_count])),
            Arc::new(LanceStringArray::from(vec![related_pages_json.as_str()])),
            Arc::new(LanceStringArray::from(vec![node_context_json.as_deref()])),
        ],
        "failed to build projected retrieval-context Flight batch",
    )
}

pub(crate) fn repo_projected_retrieval_context_metadata(
    response: &RepoProjectedRetrievalContextResult,
    requested_node_id: Option<&str>,
) -> Result<Vec<u8>, String> {
    let node_id = response_node_id(response, requested_node_id);
    serde_json::to_vec(&serde_json::json!({
        "repoId": response.repo_id,
        "pageId": response.center.page.page_id,
        "nodeId": node_id,
        "relatedCount": response.related_pages.len(),
        "hasNodeContext": response.node_context.is_some(),
    }))
    .map_err(|error| error.to_string())
}

pub(crate) fn graph_neighbors_projection_batch(
    repo_id: &str,
    projection: &WendaoGraphProjection,
) -> Result<LanceRecordBatch, String> {
    let mut rows = projection
        .nodes
        .iter()
        .map(|node| FlightGraphRow {
            row_type: "node",
            node_id: Some(display_path(repo_id, node.path.as_str())),
            node_label: Some(preferred_label(node.title.as_str(), node.path.as_str())),
            node_path: Some(display_path(repo_id, node.path.as_str())),
            node_type: Some("doc".to_string()),
            node_is_center: Some(node.is_center),
            node_distance: Some(node.distance),
            navigation_path: Some(display_path(repo_id, node.path.as_str())),
            navigation_category: Some("repo".to_string()),
            navigation_project_name: Some(repo_id.to_string()),
            navigation_root_label: Some(repo_id.to_string()),
            navigation_line: None,
            navigation_line_end: None,
            navigation_column: None,
            link_source: None,
            link_target: None,
            link_direction: None,
            link_distance: None,
        })
        .collect::<Vec<_>>();
    rows.extend(projection.links.iter().map(|link| FlightGraphRow {
        row_type: "link",
        node_id: None,
        node_label: None,
        node_path: None,
        node_type: None,
        node_is_center: None,
        node_distance: None,
        navigation_path: None,
        navigation_category: None,
        navigation_project_name: None,
        navigation_root_label: None,
        navigation_line: None,
        navigation_line_end: None,
        navigation_column: None,
        link_source: Some(display_path(repo_id, link.source_path.as_str())),
        link_target: Some(display_path(repo_id, link.target_path.as_str())),
        link_direction: Some(link.direction.clone()),
        link_distance: Some(link.distance),
    }));
    let columns = graph_neighbors_response_columns(rows.as_slice())?;
    record_batch(
        &graph_neighbors_contract(),
        columns,
        "failed to build graph-neighbors Flight batch",
    )
}

pub(crate) fn projection_page_kind_token(kind: ProjectionPageKind) -> &'static str {
    match kind {
        ProjectionPageKind::Reference => "reference",
        ProjectionPageKind::HowTo => "howto",
        ProjectionPageKind::Tutorial => "tutorial",
        ProjectionPageKind::Explanation => "explanation",
    }
}

pub(crate) fn response_node_id<'a>(
    response: &'a RepoProjectedRetrievalContextResult,
    requested_node_id: Option<&'a str>,
) -> Option<&'a str> {
    response
        .center
        .node
        .as_ref()
        .map(|node| node.node_id.as_str())
        .or_else(|| response.node_context.as_ref().and(requested_node_id))
}

pub(crate) fn display_path(repo_id: &str, path: &str) -> String {
    let normalized_path = path.trim().trim_matches('/');
    let normalized_repo = repo_id.trim().trim_matches('/');
    if normalized_path.is_empty()
        || normalized_path.starts_with(format!("{normalized_repo}/").as_str())
    {
        normalized_path.to_string()
    } else {
        format!("{normalized_repo}/{normalized_path}")
    }
}

pub(crate) fn preferred_label(title: &str, fallback_path: &str) -> String {
    if !title.trim().is_empty() {
        return title.to_string();
    }
    std::path::Path::new(fallback_path)
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|stem| !stem.trim().is_empty())
        .unwrap_or(fallback_path)
        .to_string()
}

#[cfg(test)]
pub(crate) fn graph_neighbors_response_schema() -> Arc<LanceSchema> {
    schema_ref(&graph_neighbors_contract())
}

pub(crate) fn graph_neighbors_response_columns(
    rows: &[FlightGraphRow],
) -> Result<Vec<LanceArrayRef>, String> {
    Ok(vec![
        Arc::new(LanceStringArray::from(
            rows.iter().map(|row| row.row_type).collect::<Vec<_>>(),
        )),
        graph_neighbors_string_column(rows, |row| row.node_id.as_deref()),
        graph_neighbors_string_column(rows, |row| row.node_label.as_deref()),
        graph_neighbors_string_column(rows, |row| row.node_path.as_deref()),
        graph_neighbors_string_column(rows, |row| row.node_type.as_deref()),
        Arc::new(LanceBooleanArray::from(
            rows.iter()
                .map(|row| row.node_is_center)
                .collect::<Vec<_>>(),
        )),
        graph_neighbors_int32_column(rows, |row| row.node_distance)?,
        graph_neighbors_string_column(rows, |row| row.navigation_path.as_deref()),
        graph_neighbors_string_column(rows, |row| row.navigation_category.as_deref()),
        graph_neighbors_string_column(rows, |row| row.navigation_project_name.as_deref()),
        graph_neighbors_string_column(rows, |row| row.navigation_root_label.as_deref()),
        graph_neighbors_int32_column(rows, |row| row.navigation_line)?,
        graph_neighbors_int32_column(rows, |row| row.navigation_line_end)?,
        graph_neighbors_int32_column(rows, |row| row.navigation_column)?,
        graph_neighbors_string_column(rows, |row| row.link_source.as_deref()),
        graph_neighbors_string_column(rows, |row| row.link_target.as_deref()),
        graph_neighbors_string_column(rows, |row| row.link_direction.as_deref()),
        graph_neighbors_int32_column(rows, |row| row.link_distance)?,
    ])
}

pub(crate) fn graph_neighbors_string_column<F>(
    rows: &[FlightGraphRow],
    accessor: F,
) -> LanceArrayRef
where
    F: Fn(&FlightGraphRow) -> Option<&str>,
{
    Arc::new(LanceStringArray::from(
        rows.iter().map(accessor).collect::<Vec<_>>(),
    ))
}

pub(crate) fn graph_neighbors_int32_column<F>(
    rows: &[FlightGraphRow],
    accessor: F,
) -> Result<LanceArrayRef, String>
where
    F: Fn(&FlightGraphRow) -> Option<usize>,
{
    Ok(Arc::new(LanceInt32Array::from(
        rows.iter()
            .map(|row| accessor(row).map(usize_to_i32).transpose())
            .collect::<Result<Vec<_>, _>>()?,
    )))
}

pub(crate) fn usize_to_i32(value: usize) -> Result<i32, String> {
    i32::try_from(value).map_err(|error| format!("failed to represent graph row int: {error}"))
}

#[derive(Debug, Clone)]
pub(crate) struct FlightGraphRow {
    row_type: &'static str,
    node_id: Option<String>,
    node_label: Option<String>,
    node_path: Option<String>,
    node_type: Option<String>,
    node_is_center: Option<bool>,
    node_distance: Option<usize>,
    navigation_path: Option<String>,
    navigation_category: Option<String>,
    navigation_project_name: Option<String>,
    navigation_root_label: Option<String>,
    navigation_line: Option<usize>,
    navigation_line_end: Option<usize>,
    navigation_column: Option<usize>,
    link_source: Option<String>,
    link_target: Option<String>,
    link_direction: Option<String>,
    link_distance: Option<usize>,
}

fn record_batch(
    contract: &ArrowSchemaContract,
    columns: Vec<LanceArrayRef>,
    context: &'static str,
) -> Result<LanceRecordBatch, String> {
    let batch = LanceRecordBatch::try_new(schema_ref(contract), columns)
        .map_err(|error| format!("{context}: {error}"))?;
    validate_record_batch_schema_with_options(
        &batch,
        contract,
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .map_err(|error| format!("{context} schema validation: {error}"))?;
    Ok(batch)
}

fn page_index_tree_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        PAGE_INDEX_TREE_TABLE,
        true,
        vec![
            utf8_column("repoId"),
            utf8_column("pageId"),
            utf8_column("kind"),
            utf8_column("path"),
            utf8_column("docId"),
            utf8_column("title"),
            uint64_column("rootCount"),
            utf8_column("rootsJson"),
        ],
    )
}

fn retrieval_context_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        RETRIEVAL_CONTEXT_TABLE,
        true,
        vec![
            utf8_column("repoId"),
            utf8_column("pageId"),
            nullable_utf8_column("nodeId"),
            utf8_column("centerJson"),
            uint64_column("relatedCount"),
            utf8_column("relatedPagesJson"),
            nullable_utf8_column("nodeContextJson"),
        ],
    )
}

fn graph_neighbors_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        GRAPH_NEIGHBORS_TABLE,
        true,
        vec![
            utf8_column("rowType"),
            nullable_utf8_column("nodeId"),
            nullable_utf8_column("nodeLabel"),
            nullable_utf8_column("nodePath"),
            nullable_utf8_column("nodeType"),
            nullable_bool_column("nodeIsCenter"),
            nullable_int32_column("nodeDistance"),
            nullable_utf8_column("navigationPath"),
            nullable_utf8_column("navigationCategory"),
            nullable_utf8_column("navigationProjectName"),
            nullable_utf8_column("navigationRootLabel"),
            nullable_int32_column("navigationLine"),
            nullable_int32_column("navigationLineEnd"),
            nullable_int32_column("navigationColumn"),
            nullable_utf8_column("linkSource"),
            nullable_utf8_column("linkTarget"),
            nullable_utf8_column("linkDirection"),
            nullable_int32_column("linkDistance"),
        ],
    )
}

fn schema_ref(contract: &ArrowSchemaContract) -> Arc<LanceSchema> {
    Arc::new(build_arrow_schema(
        contract,
        [(
            WENDAO_TABLE_METADATA_KEY.to_string(),
            contract.table_name().to_string(),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>(),
    ))
}

const fn utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}

const fn uint64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::UInt64)
}

const fn nullable_utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Utf8)
}

const fn nullable_int32_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Int32)
}

const fn nullable_bool_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Boolean)
}
