use std::path::Path;

use arrow::datatypes::{Field, Schema};
use serde::{Deserialize, Serialize};
use xiuxian_wendao_parsers::semantic_ssot::{SemanticRepository, load_semantic_repository};

use super::register::{
    SEMANTIC_OBJECTS_TABLE_NAME, SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    SEMANTIC_RELATIONS_TABLE_NAME, build_semantic_read_model_rows,
};
use super::rows::SemanticReadModelRows;
use super::schema::{
    semantic_objects_schema, semantic_projection_state_schema, semantic_relations_schema,
};

/// Stable advisory catalog for the semantic read-model surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticReadModelCatalog {
    /// Whether the catalog describes advisory derived rows.
    pub advisory: bool,
    /// Canonical authority that owns the source facts.
    pub authority: String,
    /// Number of available read-model tables.
    pub table_count: usize,
    /// Total row count across all read-model tables.
    pub total_row_count: usize,
    /// Table descriptions in registration order.
    pub tables: Vec<SemanticReadModelTableCatalog>,
}

/// One semantic read-model table description.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticReadModelTableCatalog {
    /// Table name exposed to read-only query consumers.
    pub name: String,
    /// Current projected row count.
    pub row_count: usize,
    /// Number of exposed columns.
    pub column_count: usize,
    /// Column descriptions in table order.
    pub columns: Vec<SemanticReadModelColumnCatalog>,
}

/// One semantic read-model column description.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticReadModelColumnCatalog {
    /// Column name exposed to read-only query consumers.
    pub name: String,
    /// Arrow data type rendered as a stable text token.
    pub data_type: String,
    /// Whether the column permits null values.
    pub nullable: bool,
}

/// Build the semantic read-model catalog from a semantic artifact root.
///
/// # Errors
///
/// Returns an error when the semantic repository under `root` is invalid or
/// when row JSON metadata cannot be encoded.
pub fn semantic_read_model_catalog_from_root(
    root: impl AsRef<Path>,
) -> Result<SemanticReadModelCatalog, String> {
    let repository = load_semantic_repository(root);
    semantic_read_model_catalog(&repository)
}

/// Build the semantic read-model catalog from one loaded repository.
///
/// # Errors
///
/// Returns an error when the repository validation report contains issues or
/// when row JSON metadata cannot be encoded.
pub fn semantic_read_model_catalog(
    repository: &SemanticRepository,
) -> Result<SemanticReadModelCatalog, String> {
    let rows = build_semantic_read_model_rows(repository)?;
    Ok(semantic_read_model_catalog_from_rows(&rows))
}

fn semantic_read_model_catalog_from_rows(rows: &SemanticReadModelRows) -> SemanticReadModelCatalog {
    let tables = vec![
        table_catalog(
            SEMANTIC_OBJECTS_TABLE_NAME,
            rows.objects.len(),
            semantic_objects_schema().as_ref(),
        ),
        table_catalog(
            SEMANTIC_RELATIONS_TABLE_NAME,
            rows.relations.len(),
            semantic_relations_schema().as_ref(),
        ),
        table_catalog(
            SEMANTIC_PROJECTION_STATE_TABLE_NAME,
            rows.projection_state.len(),
            semantic_projection_state_schema().as_ref(),
        ),
    ];
    let total_row_count = tables.iter().map(|table| table.row_count).sum();
    SemanticReadModelCatalog {
        advisory: true,
        authority: "repo_native_semantic_artifacts".to_string(),
        table_count: tables.len(),
        total_row_count,
        tables,
    }
}

fn table_catalog(name: &str, row_count: usize, schema: &Schema) -> SemanticReadModelTableCatalog {
    let columns = schema
        .fields()
        .iter()
        .map(|field| column_catalog(field.as_ref()))
        .collect::<Vec<_>>();
    SemanticReadModelTableCatalog {
        name: name.to_string(),
        row_count,
        column_count: columns.len(),
        columns,
    }
}

fn column_catalog(field: &Field) -> SemanticReadModelColumnCatalog {
    SemanticReadModelColumnCatalog {
        name: field.name().clone(),
        data_type: field.data_type().to_string(),
        nullable: field.is_nullable(),
    }
}
