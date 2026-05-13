//! Executors wendao sql discover surface for `xiuxian-qianji`.

use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};

use super::contract::{SurfaceBundle, SurfaceColumn, SurfaceObject, SurfacePolicy};
use super::gateway::{SqlQueryPayload, query_sql_endpoint};
use super::input::{resolve_endpoint, resolve_project_root};
use super::render::surface_bundle_xml;

const TABLES_CATALOG_QUERY: &str = "SELECT sql_table_name, corpus, scope, sql_object_kind, source_count, repo_id FROM wendao_sql_tables ORDER BY sql_table_name, COALESCE(repo_id, '')";
const COLUMNS_CATALOG_QUERY: &str = "SELECT sql_table_name, column_name, data_type, is_nullable, ordinal_position, column_origin_kind FROM wendao_sql_columns ORDER BY sql_table_name, ordinal_position, column_name";

/// Deterministic discovery node for request-scoped Wendao SQL surface metadata.
/// Semantic field boundary: this public DTO preserves externally serialized field names.
pub struct WendaoSqlDiscoverMechanism {
    /// Output context key storing the rendered XML surface bundle.
    pub output_key: String,
    /// Optional context key resolving the Wendao query endpoint.
    pub endpoint_key: Option<String>,
    /// Optional static Wendao query endpoint override.
    pub endpoint: Option<String>,
    /// Optional context key resolved into the bundle metadata.
    pub project_root_key: Option<String>,
    /// Optional allowlist of SQL-visible objects to expose.
    pub allowed_objects: Vec<String>,
    /// Maximum `LIMIT` the downstream authoring loop may request.
    pub max_limit: usize,
    /// Allowed filter operators rendered into the bundle policy.
    pub allowed_ops: Vec<String>,
    /// Objects that require at least one narrowing filter before execution.
    pub require_filter_for: Vec<String>,
}

#[async_trait]
impl QianjiMechanism for WendaoSqlDiscoverMechanism {
    async fn execute(&self, context: &Value) -> Result<QianjiOutput, String> {
        let endpoint = resolve_endpoint(
            context,
            self.endpoint.as_deref(),
            self.endpoint_key.as_deref(),
        )?;
        let tables_payload = query_sql_endpoint(endpoint.as_str(), TABLES_CATALOG_QUERY).await?;
        let columns_payload = query_sql_endpoint(endpoint.as_str(), COLUMNS_CATALOG_QUERY).await?;
        let resolved_objects = self.collect_surface_objects(&tables_payload, &columns_payload);
        if resolved_objects.is_empty() {
            return Err(discovery_empty_error(&self.allowed_objects));
        }

        let bundle = SurfaceBundle {
            project_root: resolve_project_root(context, self.project_root_key.as_deref()),
            catalog_table_name: tables_payload.metadata.catalog_table_name,
            column_catalog_table_name: tables_payload.metadata.column_catalog_table_name,
            view_source_catalog_table_name: tables_payload.metadata.view_source_catalog_table_name,
            policy: SurfacePolicy {
                max_limit: self.max_limit,
                allowed_ops: self.allowed_ops.clone(),
                require_filter_for: self.require_filter_for.clone(),
            },
            objects: resolved_objects,
        };
        let xml = surface_bundle_xml(&bundle);

        Ok(QianjiOutput {
            data: json!({
                self.output_key.clone(): xml,
            }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        1.0
    }
}

impl WendaoSqlDiscoverMechanism {
    fn collect_surface_objects(
        &self,
        tables_payload: &SqlQueryPayload,
        columns_payload: &SqlQueryPayload,
    ) -> Vec<SurfaceObject> {
        let allowed = allowed_object_filter(&self.allowed_objects);
        let mut objects = seed_surface_objects(tables_payload, &allowed);
        append_surface_columns(&mut objects, columns_payload);
        finalize_surface_objects(objects)
    }
}

fn allowed_object_filter(allowed_objects: &[String]) -> BTreeSet<String> {
    allowed_objects
        .iter()
        .map(|item| item.to_ascii_lowercase())
        .collect()
}

fn seed_surface_objects(
    tables_payload: &SqlQueryPayload,
    allowed: &BTreeSet<String>,
) -> BTreeMap<String, SurfaceObject> {
    let mut objects = BTreeMap::<String, SurfaceObject>::new();

    for row in tables_payload
        .batches
        .iter()
        .flat_map(|batch| batch.rows.iter())
    {
        let Some(name) = row_string(row, "sql_table_name") else {
            continue;
        };
        if !allowed.is_empty() && !allowed.contains(&name.to_ascii_lowercase()) {
            continue;
        }

        let source_count = row_usize(row, "source_count").unwrap_or(0);
        let repo_id = row_string(row, "repo_id");
        let entry = objects
            .entry(name.to_ascii_lowercase())
            .or_insert_with(|| SurfaceObject {
                name: name.clone(),
                kind: row_string(row, "sql_object_kind").unwrap_or_else(|| "table".to_string()),
                scope: row_string(row, "scope").unwrap_or_else(|| "request".to_string()),
                corpus: row_string(row, "corpus").unwrap_or_else(|| "repo".to_string()),
                repo_id: repo_id.clone(),
                source_count,
                columns: Vec::new(),
            });
        entry.source_count = entry.source_count.max(source_count);
        if entry.repo_id.is_none() {
            entry.repo_id = repo_id;
        }
    }

    objects
}

fn append_surface_columns(
    objects: &mut BTreeMap<String, SurfaceObject>,
    columns_payload: &SqlQueryPayload,
) {
    for row in columns_payload
        .batches
        .iter()
        .flat_map(|batch| batch.rows.iter())
    {
        let Some(table_name) = row_string(row, "sql_table_name") else {
            continue;
        };
        let Some(object) = objects.get_mut(&table_name.to_ascii_lowercase()) else {
            continue;
        };
        let Some(name) = row_string(row, "column_name") else {
            continue;
        };
        object.columns.push(SurfaceColumn {
            name,
            data_type: row_string(row, "data_type").unwrap_or_else(|| "Utf8".to_string()),
            nullable: row_bool(row, "is_nullable").unwrap_or(true),
            ordinal_position: row_usize(row, "ordinal_position").unwrap_or(0),
            origin_kind: row_string(row, "column_origin_kind").unwrap_or_default(),
        });
    }
}

fn finalize_surface_objects(objects: BTreeMap<String, SurfaceObject>) -> Vec<SurfaceObject> {
    let mut resolved_objects = objects.into_values().collect::<Vec<_>>();
    for object in &mut resolved_objects {
        object.columns.sort_by(|left, right| {
            left.ordinal_position
                .cmp(&right.ordinal_position)
                .then_with(|| left.name.cmp(&right.name))
        });
    }
    resolved_objects.sort_by(|left, right| left.name.cmp(&right.name));
    resolved_objects
}

fn discovery_empty_error(allowed_objects: &[String]) -> String {
    if allowed_objects.is_empty() {
        "wendao_sql_discover resolved no SQL-visible objects".to_string()
    } else {
        format!(
            "wendao_sql_discover resolved no SQL-visible objects for allowlist: {}",
            allowed_objects.join(", ")
        )
    }
}

fn row_string(row: &Map<String, Value>, key: &str) -> Option<String> {
    row.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn row_usize(row: &Map<String, Value>, key: &str) -> Option<usize> {
    row.get(key).and_then(|value| match value {
        Value::Number(raw) => raw
            .as_u64()
            .and_then(|candidate| usize::try_from(candidate).ok())
            .or_else(|| {
                raw.as_i64().and_then(|candidate| {
                    u64::try_from(candidate)
                        .ok()
                        .and_then(|value| usize::try_from(value).ok())
                })
            }),
        _ => None,
    })
}

fn row_bool(row: &Map<String, Value>, key: &str) -> Option<bool> {
    row.get(key).and_then(Value::as_bool)
}
