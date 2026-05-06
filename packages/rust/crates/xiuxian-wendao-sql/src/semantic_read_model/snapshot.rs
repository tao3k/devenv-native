use std::path::Path;

use arrow::datatypes::Schema;
use serde::{Deserialize, Serialize};
use xiuxian_wendao_parsers::semantic_ssot::{SemanticRepository, load_semantic_repository};

use super::catalog::{SemanticReadModelCatalog, semantic_read_model_catalog_from_rows};
use super::register::{
    SEMANTIC_OBJECTS_TABLE_NAME, SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    SEMANTIC_RELATIONS_TABLE_NAME, build_semantic_read_model_rows,
};
use super::rows::{
    SemanticObjectReadModelRow, SemanticProjectionStateReadModelRow, SemanticReadModelRows,
    SemanticRelationReadModelRow,
};
use super::schema::{
    semantic_objects_schema, semantic_projection_state_schema, semantic_relations_schema,
};

/// Deterministic advisory snapshot for the semantic read-model surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticReadModelSnapshot {
    /// Whether the snapshot describes advisory derived rows.
    pub advisory: bool,
    /// Canonical authority that owns the source facts.
    pub authority: String,
    /// Deterministic revision over table schemas and row revisions.
    pub snapshot_revision: String,
    /// Catalog used by this snapshot.
    pub catalog: SemanticReadModelCatalog,
    /// Per-table deterministic row revisions.
    pub tables: Vec<SemanticReadModelTableSnapshot>,
}

/// One table revision inside a semantic read-model snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticReadModelTableSnapshot {
    /// Table name exposed to read-only query consumers.
    pub name: String,
    /// Current projected row count.
    pub row_count: usize,
    /// Number of exposed columns.
    pub column_count: usize,
    /// Deterministic revision over this table schema and rows.
    pub row_revision: String,
}

/// Exact-revision check for one advisory semantic read-model snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticReadModelSnapshotCheck {
    /// Whether the current snapshot revision exactly matches the expected revision.
    pub matches: bool,
    /// Operator-provided expected aggregate snapshot revision.
    pub expected_snapshot_revision: String,
    /// Current aggregate snapshot revision computed from repo-native artifacts.
    pub current_snapshot_revision: String,
    /// Current snapshot used for the comparison.
    pub current_snapshot: SemanticReadModelSnapshot,
}

/// Build a semantic read-model snapshot from a semantic artifact root.
///
/// # Errors
///
/// Returns an error when the semantic repository under `root` is invalid or
/// when row JSON metadata cannot be encoded.
pub fn semantic_read_model_snapshot_from_root(
    root: impl AsRef<Path>,
) -> Result<SemanticReadModelSnapshot, String> {
    let repository = load_semantic_repository(root);
    semantic_read_model_snapshot(&repository)
}

/// Build a semantic read-model snapshot from one loaded repository.
///
/// # Errors
///
/// Returns an error when the repository validation report contains issues or
/// when row JSON metadata cannot be encoded.
pub fn semantic_read_model_snapshot(
    repository: &SemanticRepository,
) -> Result<SemanticReadModelSnapshot, String> {
    let rows = build_semantic_read_model_rows(repository)?;
    Ok(semantic_read_model_snapshot_from_rows(&rows))
}

/// Check a semantic read-model snapshot from a semantic artifact root.
///
/// # Errors
///
/// Returns an error when the expected revision is not a `blake3:` revision, the
/// semantic repository under `root` is invalid, or row JSON metadata cannot be
/// encoded.
pub fn semantic_read_model_snapshot_check_from_root(
    root: impl AsRef<Path>,
    expected_snapshot_revision: &str,
) -> Result<SemanticReadModelSnapshotCheck, String> {
    let snapshot = semantic_read_model_snapshot_from_root(root)?;
    semantic_read_model_snapshot_check(snapshot, expected_snapshot_revision)
}

/// Check one semantic read-model snapshot against an expected revision.
///
/// # Errors
///
/// Returns an error when `expected_snapshot_revision` is blank, contains
/// surrounding whitespace, or does not use the `blake3:` revision scheme.
pub fn semantic_read_model_snapshot_check(
    snapshot: SemanticReadModelSnapshot,
    expected_snapshot_revision: &str,
) -> Result<SemanticReadModelSnapshotCheck, String> {
    validate_expected_snapshot_revision(expected_snapshot_revision)?;
    let current_snapshot_revision = snapshot.snapshot_revision.clone();
    Ok(SemanticReadModelSnapshotCheck {
        matches: current_snapshot_revision == expected_snapshot_revision,
        expected_snapshot_revision: expected_snapshot_revision.to_string(),
        current_snapshot_revision,
        current_snapshot: snapshot,
    })
}

fn semantic_read_model_snapshot_from_rows(
    rows: &SemanticReadModelRows,
) -> SemanticReadModelSnapshot {
    let catalog = semantic_read_model_catalog_from_rows(rows);
    let objects_schema = semantic_objects_schema();
    let relations_schema = semantic_relations_schema();
    let projection_state_schema = semantic_projection_state_schema();
    let tables = vec![
        object_table_snapshot(rows.objects.as_slice(), objects_schema.as_ref()),
        relation_table_snapshot(rows.relations.as_slice(), relations_schema.as_ref()),
        projection_state_table_snapshot(
            rows.projection_state.as_slice(),
            projection_state_schema.as_ref(),
        ),
    ];
    let snapshot_revision = snapshot_revision(&catalog, tables.as_slice());
    SemanticReadModelSnapshot {
        advisory: true,
        authority: "repo_native_semantic_artifacts".to_string(),
        snapshot_revision,
        catalog,
        tables,
    }
}

fn object_table_snapshot(
    rows: &[SemanticObjectReadModelRow],
    schema: &Schema,
) -> SemanticReadModelTableSnapshot {
    let mut hasher = table_revision_hasher(SEMANTIC_OBJECTS_TABLE_NAME, rows.len(), schema);
    let mut sorted_rows = rows.iter().collect::<Vec<_>>();
    sorted_rows.sort_by(|left, right| left.id.cmp(&right.id));
    for row in sorted_rows {
        update_hash_field(&mut hasher, "id", row.id.as_str());
        update_hash_field(&mut hasher, "kind", row.kind.as_str());
        update_hash_field(&mut hasher, "title", row.title.as_str());
        update_hash_field(&mut hasher, "status", row.status.as_str());
        update_hash_field(
            &mut hasher,
            "confidence_score_bits",
            row.confidence_score.to_bits().to_string().as_str(),
        );
        update_hash_field(
            &mut hasher,
            "confidence_source",
            row.confidence_source.as_str(),
        );
        update_hash_i64(&mut hasher, "owner_count", row.owner_count);
        update_hash_field(&mut hasher, "owners_json", row.owners_json.as_str());
        update_hash_field(
            &mut hasher,
            "provenance_source",
            row.provenance_source.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "provenance_recorded_by",
            row.provenance_recorded_by.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "provenance_recorded_at",
            row.provenance_recorded_at.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "verification_required_json",
            row.verification_required_json.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "verification_evidence_json",
            row.verification_evidence_json.as_str(),
        );
        update_hash_i64(&mut hasher, "relation_count", row.relation_count);
        update_hash_field(&mut hasher, "source_path", row.source_path.as_str());
        update_hash_field(
            &mut hasher,
            "read_model_source_revision",
            row.read_model_source_revision.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "read_model_projection_revision",
            row.read_model_projection_revision.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "read_model_projection_staleness",
            row.read_model_projection_staleness.as_str(),
        );
    }
    SemanticReadModelTableSnapshot {
        name: SEMANTIC_OBJECTS_TABLE_NAME.to_string(),
        row_count: rows.len(),
        column_count: schema.fields().len(),
        row_revision: finalize_revision(&hasher),
    }
}

fn relation_table_snapshot(
    rows: &[SemanticRelationReadModelRow],
    schema: &Schema,
) -> SemanticReadModelTableSnapshot {
    let mut hasher = table_revision_hasher(SEMANTIC_RELATIONS_TABLE_NAME, rows.len(), schema);
    let mut sorted_rows = rows.iter().collect::<Vec<_>>();
    sorted_rows.sort_by(|left, right| {
        (&left.source, &left.kind, &left.target).cmp(&(&right.source, &right.kind, &right.target))
    });
    for row in sorted_rows {
        update_hash_field(&mut hasher, "source", row.source.as_str());
        update_hash_field(&mut hasher, "kind", row.kind.as_str());
        update_hash_field(&mut hasher, "target", row.target.as_str());
        update_hash_field(&mut hasher, "source_path", row.source_path.as_str());
        update_hash_field(
            &mut hasher,
            "read_model_source_revision",
            row.read_model_source_revision.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "read_model_projection_revision",
            row.read_model_projection_revision.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "read_model_projection_staleness",
            row.read_model_projection_staleness.as_str(),
        );
    }
    SemanticReadModelTableSnapshot {
        name: SEMANTIC_RELATIONS_TABLE_NAME.to_string(),
        row_count: rows.len(),
        column_count: schema.fields().len(),
        row_revision: finalize_revision(&hasher),
    }
}

fn projection_state_table_snapshot(
    rows: &[SemanticProjectionStateReadModelRow],
    schema: &Schema,
) -> SemanticReadModelTableSnapshot {
    let mut hasher =
        table_revision_hasher(SEMANTIC_PROJECTION_STATE_TABLE_NAME, rows.len(), schema);
    let mut sorted_rows = rows.iter().collect::<Vec<_>>();
    sorted_rows.sort_by(|left, right| {
        (&left.projection, &left.source_path).cmp(&(&right.projection, &right.source_path))
    });
    for row in sorted_rows {
        update_hash_field(&mut hasher, "projection", row.projection.as_str());
        update_hash_field(&mut hasher, "status", row.status.as_str());
        update_hash_field(&mut hasher, "source_revision", row.source_revision.as_str());
        update_hash_field(
            &mut hasher,
            "current_source_revision",
            row.current_source_revision.as_str(),
        );
        update_hash_field(
            &mut hasher,
            "projection_revision",
            row.projection_revision.as_str(),
        );
        update_hash_field(&mut hasher, "staleness", row.staleness.as_str());
        update_hash_i64(&mut hasher, "source_object_count", row.source_object_count);
        update_hash_field(
            &mut hasher,
            "source_objects_json",
            row.source_objects_json.as_str(),
        );
        update_hash_field(&mut hasher, "source_path", row.source_path.as_str());
    }
    SemanticReadModelTableSnapshot {
        name: SEMANTIC_PROJECTION_STATE_TABLE_NAME.to_string(),
        row_count: rows.len(),
        column_count: schema.fields().len(),
        row_revision: finalize_revision(&hasher),
    }
}

fn table_revision_hasher(table_name: &str, row_count: usize, schema: &Schema) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    update_hash_field(
        &mut hasher,
        "contract",
        "semantic_read_model_table_snapshot.v1",
    );
    update_hash_field(&mut hasher, "table", table_name);
    update_hash_usize(&mut hasher, "row_count", row_count);
    update_hash_usize(&mut hasher, "column_count", schema.fields().len());
    for field in schema.fields() {
        update_hash_field(&mut hasher, "column_name", field.name());
        update_hash_field(
            &mut hasher,
            "column_type",
            field.data_type().to_string().as_str(),
        );
        update_hash_bool(&mut hasher, "column_nullable", field.is_nullable());
    }
    hasher
}

fn snapshot_revision(
    catalog: &SemanticReadModelCatalog,
    tables: &[SemanticReadModelTableSnapshot],
) -> String {
    let mut hasher = blake3::Hasher::new();
    update_hash_field(&mut hasher, "contract", "semantic_read_model_snapshot.v1");
    update_hash_bool(&mut hasher, "advisory", catalog.advisory);
    update_hash_field(&mut hasher, "authority", catalog.authority.as_str());
    update_hash_usize(&mut hasher, "table_count", catalog.table_count);
    update_hash_usize(&mut hasher, "total_row_count", catalog.total_row_count);
    for table in tables {
        update_hash_field(&mut hasher, "table", table.name.as_str());
        update_hash_usize(&mut hasher, "row_count", table.row_count);
        update_hash_usize(&mut hasher, "column_count", table.column_count);
        update_hash_field(&mut hasher, "row_revision", table.row_revision.as_str());
    }
    finalize_revision(&hasher)
}

fn update_hash_field(hasher: &mut blake3::Hasher, name: &str, value: &str) {
    hasher.update(name.as_bytes());
    hasher.update(b"\0");
    hasher.update(value.as_bytes());
    hasher.update(b"\0");
}

fn update_hash_bool(hasher: &mut blake3::Hasher, name: &str, value: bool) {
    update_hash_field(hasher, name, if value { "true" } else { "false" });
}

fn update_hash_i64(hasher: &mut blake3::Hasher, name: &str, value: i64) {
    update_hash_field(hasher, name, value.to_string().as_str());
}

fn update_hash_usize(hasher: &mut blake3::Hasher, name: &str, value: usize) {
    update_hash_field(hasher, name, value.to_string().as_str());
}

fn finalize_revision(hasher: &blake3::Hasher) -> String {
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn validate_expected_snapshot_revision(expected_snapshot_revision: &str) -> Result<(), String> {
    if expected_snapshot_revision.is_empty() {
        return Err("expected semantic read-model snapshot revision must not be empty".to_string());
    }
    if expected_snapshot_revision.trim() != expected_snapshot_revision {
        return Err(
            "expected semantic read-model snapshot revision must not contain surrounding whitespace"
                .to_string(),
        );
    }
    if !expected_snapshot_revision.starts_with("blake3:") {
        return Err(
            "expected semantic read-model snapshot revision must use the `blake3:` scheme"
                .to_string(),
        );
    }
    Ok(())
}
