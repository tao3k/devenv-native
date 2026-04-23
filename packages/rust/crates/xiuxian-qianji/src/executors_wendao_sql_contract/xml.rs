use serde::Deserialize;

use super::model::{
    SqlAuthorSpec, SqlFilter, SqlOrderTerm, SurfaceBundle, SurfaceColumn, SurfaceObject,
    SurfacePolicy,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "surface_bundle")]
pub(super) struct SurfaceBundleXml {
    project_root: String,
    catalog_table_name: String,
    column_catalog_table_name: String,
    view_source_catalog_table_name: String,
    #[serde(default)]
    policy: SurfacePolicyXml,
    #[serde(default)]
    objects: SurfaceObjectsXml,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct SurfacePolicyXml {
    max_limit: Option<usize>,
    #[serde(rename = "allowed_op", default)]
    allowed_ops: Vec<String>,
    #[serde(rename = "require_filter_for", default)]
    require_filter_for: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct SurfaceObjectsXml {
    #[serde(rename = "object", default)]
    objects: Vec<SurfaceObjectXml>,
}

#[derive(Debug, Clone, Deserialize)]
struct SurfaceObjectXml {
    name: String,
    kind: String,
    scope: String,
    corpus: String,
    #[serde(default)]
    repo_id: Option<String>,
    source_count: Option<usize>,
    #[serde(default)]
    columns: SurfaceColumnsXml,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct SurfaceColumnsXml {
    #[serde(rename = "column", default)]
    columns: Vec<SurfaceColumnXml>,
}

#[derive(Debug, Clone, Deserialize)]
struct SurfaceColumnXml {
    name: String,
    data_type: String,
    nullable: Option<bool>,
    ordinal_position: Option<usize>,
    origin_kind: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "sql_author_spec")]
pub(super) struct SqlAuthorSpecXml {
    target_object: String,
    #[serde(default)]
    projection: ProjectionXml,
    #[serde(default)]
    filters: FiltersXml,
    #[serde(default)]
    order_by: OrderByXml,
    limit: usize,
    #[serde(default)]
    sql_draft: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ProjectionXml {
    #[serde(rename = "column", default)]
    columns: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct FiltersXml {
    #[serde(rename = "filter", default)]
    filters: Vec<SqlFilterXml>,
}

#[derive(Debug, Clone, Deserialize)]
struct SqlFilterXml {
    column: String,
    op: String,
    value: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct OrderByXml {
    #[serde(rename = "item", default)]
    items: Vec<SqlOrderTermXml>,
}

#[derive(Debug, Clone, Deserialize)]
struct SqlOrderTermXml {
    column: String,
    #[serde(default)]
    direction: Option<String>,
}

impl From<SurfaceBundleXml> for SurfaceBundle {
    fn from(value: SurfaceBundleXml) -> Self {
        Self {
            project_root: value.project_root.trim().to_string(),
            catalog_table_name: value.catalog_table_name.trim().to_string(),
            column_catalog_table_name: value.column_catalog_table_name.trim().to_string(),
            view_source_catalog_table_name: value.view_source_catalog_table_name.trim().to_string(),
            policy: SurfacePolicy {
                max_limit: value.policy.max_limit.unwrap_or(8),
                allowed_ops: value
                    .policy
                    .allowed_ops
                    .into_iter()
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect(),
                require_filter_for: value
                    .policy
                    .require_filter_for
                    .into_iter()
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect(),
            },
            objects: value.objects.objects.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<SurfaceObjectXml> for SurfaceObject {
    fn from(value: SurfaceObjectXml) -> Self {
        Self {
            name: value.name.trim().to_string(),
            kind: value.kind.trim().to_string(),
            scope: value.scope.trim().to_string(),
            corpus: value.corpus.trim().to_string(),
            repo_id: value
                .repo_id
                .map(|raw| raw.trim().to_string())
                .filter(|raw| !raw.is_empty()),
            source_count: value.source_count.unwrap_or(0),
            columns: value.columns.columns.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<SurfaceColumnXml> for SurfaceColumn {
    fn from(value: SurfaceColumnXml) -> Self {
        Self {
            name: value.name.trim().to_string(),
            data_type: value.data_type.trim().to_string(),
            nullable: value.nullable.unwrap_or(true),
            ordinal_position: value.ordinal_position.unwrap_or(0),
            origin_kind: value.origin_kind.unwrap_or_default().trim().to_string(),
        }
    }
}

impl From<SqlAuthorSpecXml> for SqlAuthorSpec {
    fn from(value: SqlAuthorSpecXml) -> Self {
        Self {
            target_object: value.target_object.trim().to_string(),
            projection: value
                .projection
                .columns
                .into_iter()
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect(),
            filters: value.filters.filters.into_iter().map(Into::into).collect(),
            order_by: value.order_by.items.into_iter().map(Into::into).collect(),
            limit: value.limit,
            sql_draft: value
                .sql_draft
                .map(|raw| raw.trim().to_string())
                .filter(|raw| !raw.is_empty()),
        }
    }
}

impl From<SqlFilterXml> for SqlFilter {
    fn from(value: SqlFilterXml) -> Self {
        Self {
            column: value.column.trim().to_string(),
            op: value.op.trim().to_string(),
            value: value.value.trim().to_string(),
        }
    }
}

impl From<SqlOrderTermXml> for SqlOrderTerm {
    fn from(value: SqlOrderTermXml) -> Self {
        Self {
            column: value.column.trim().to_string(),
            direction: value
                .direction
                .unwrap_or_else(|| "asc".to_string())
                .trim()
                .to_string(),
        }
    }
}
