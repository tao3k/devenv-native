//! Episteme source-contract read-model seed materialization.

use std::collections::BTreeMap;
use std::path::PathBuf;

use arrow::array::{Array, StringArray};
use arrow::record_batch::RecordBatch;
use serde::Serialize;
use sha2::{Digest, Sha256};
#[cfg(feature = "julia")]
use xiuxian_wendao_julia::integration_support::WendaoGraphOntologyReadModelQualityRequestBatches;
use xiuxian_wendao_parsers::{EpistemeExtractionQueueRow, EpistemeFileRow, EpistemeSourceManifest};

use crate::episteme::source_contract::facade::{
    EpistemeError, EpistemeRegistryReferenceGraphReceipt, EpistemeValidationHashCacheReport,
    read_files_tsv, read_mapping_ledger_raw, read_queue_tsv, read_source_manifest,
    source_contract_paths, validate_episteme_source_contract,
    validate_episteme_source_contract_with_hash_cache,
};

#[path = "audio/mod.rs"]
mod audio;
#[path = "tables.rs"]
mod tables;

pub use audio::{
    EpistemeAudioEvidenceReadModelRequest, EpistemeAudioEvidenceSegmentRow,
    EpistemeAudioEvidenceSourceRow, EpistemeAudioReviewedClaimObjectKind,
    EpistemeAudioReviewedClaimReadModelRequest, EpistemeAudioReviewedClaimRow,
    materialize_episteme_audio_evidence_review_seed,
    materialize_episteme_audio_reviewed_claim_seed,
};

use tables::{
    object_ids, semantic_objects_batch, semantic_projection_state_batch, semantic_relations_batch,
};

const OBJECTS_TABLE: &str = "semantic_objects";
const RELATIONS_TABLE: &str = "semantic_relations";
const PROJECTION_STATE_TABLE: &str = "semantic_projection_state";
const FILE_OBJECT_KIND: &str = "episteme_source_contract.source_file";
const TASK_OBJECT_KIND: &str = "episteme_source_contract.extraction_task";
const TASK_SOURCE_RELATION_KIND: &str = "episteme_source_contract.extraction_task.has_source_file";
const PROJECTION_ID: &str = "episteme_source_contract.source_contract_read_model_seed.v1";
const PROJECTION_REVISION: &str = "episteme_source_contract.read_model_seed.v1";
const REGISTRY_ENTRY_OBJECT_KIND: &str = "episteme_registry.loaded_entry";
const REGISTRY_DOMAIN_OBJECT_KIND: &str = "episteme_registry.domain";
const REGISTRY_ENTRY_DOMAIN_RELATION_KIND: &str = "episteme_registry.loaded_entry.owns_domain";
const REGISTRY_ENTRY_EXTENDS_RELATION_KIND: &str = "episteme_registry.loaded_entry.extends_domain";
const REGISTRY_GRAPH_PROJECTION_ID: &str = "episteme_registry.reference_graph_read_model_seed.v1";
const REGISTRY_GRAPH_PROJECTION_REVISION: &str =
    "episteme_registry.reference_graph_read_model_seed.v1";
const CONFIDENCE_SOURCE: &str = "episteme_source_contract";
const REGISTRY_CONFIDENCE_SOURCE: &str = "episteme_registry_reference_graph";
const RECORDED_BY: &str = "episteme-source-contract";
const RECORDED_AT: &str = "2026-05-14";
const STALENESS_FRESH: &str = "fresh";
const STATUS_ACTIVE: &str = "active";

/// Request for compiling episteme source-contract facts into read-model
/// seed batches.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeReadModelRequest {
    /// Episteme repository root.
    pub episteme_root: PathBuf,
    /// Raw corpus root.
    pub corpus_root: PathBuf,
}

impl EpistemeReadModelRequest {
    /// Create a episteme source-contract read-model seed request.
    #[must_use]
    pub fn new(episteme_root: impl Into<PathBuf>, corpus_root: impl Into<PathBuf>) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            corpus_root: corpus_root.into(),
        }
    }
}

/// One materialized episteme source-contract read-model table.
#[derive(Debug, Clone)]
pub struct EpistemeReadModelTable {
    table_name: &'static str,
    batch: RecordBatch,
}

impl EpistemeReadModelTable {
    fn new(table_name: &'static str, batch: RecordBatch) -> Self {
        Self { table_name, batch }
    }

    /// Stable semantic read-model table name.
    #[must_use]
    pub fn table_name(&self) -> &'static str {
        self.table_name
    }

    /// Arrow record batch for the table.
    #[must_use]
    pub fn batch(&self) -> &RecordBatch {
        &self.batch
    }

    /// Row count for this table.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.batch.num_rows()
    }
}

/// Materialized episteme source-contract read-model seed.
#[derive(Debug, Clone)]
pub struct EpistemeReadModelMaterialization {
    /// Deterministic source-contract revision derived from manifest, file, and
    /// queue facts.
    pub source_revision: String,
    /// Semantic read-model tables in service order.
    pub tables: Vec<EpistemeReadModelTable>,
}

impl EpistemeReadModelMaterialization {
    /// Return row counts in `semantic_objects`, `semantic_relations`,
    /// `semantic_projection_state` order.
    #[must_use]
    pub fn row_counts(&self) -> [usize; 3] {
        [
            self.tables
                .iter()
                .find(|table| table.table_name() == OBJECTS_TABLE)
                .map_or(0, EpistemeReadModelTable::row_count),
            self.tables
                .iter()
                .find(|table| table.table_name() == RELATIONS_TABLE)
                .map_or(0, EpistemeReadModelTable::row_count),
            self.tables
                .iter()
                .find(|table| table.table_name() == PROJECTION_STATE_TABLE)
                .map_or(0, EpistemeReadModelTable::row_count),
        ]
    }
}

/// Compile validated episteme source-contract facts into graph-readable
/// semantic read-model seed batches.
///
/// # Errors
///
/// Returns an error when source validation fails, contract files cannot be
/// parsed, or Arrow batches cannot be created.
pub fn materialize_episteme_read_model_seed(
    request: &EpistemeReadModelRequest,
) -> Result<EpistemeReadModelMaterialization, EpistemeError> {
    let validation =
        validate_episteme_source_contract(&request.episteme_root, &request.corpus_root)?;
    if !validation.passed {
        return Err(EpistemeError::InvalidContract(validation.errors));
    }

    materialize_validated_episteme_source_contract_read_model_seed(request)
}

/// Compile episteme source-contract facts using an opt-in validation hash
/// cache.
///
/// # Errors
///
/// Returns an error when cached or uncached source validation fails, contract
/// files cannot be parsed, the cache cannot be written, or Arrow batches
/// cannot be created.
pub fn materialize_episteme_read_model_seed_with_validation_hash_cache(
    request: &EpistemeReadModelRequest,
    cache_path: impl AsRef<std::path::Path>,
) -> Result<
    (
        EpistemeReadModelMaterialization,
        EpistemeValidationHashCacheReport,
    ),
    EpistemeError,
> {
    let (validation, cache_report) = validate_episteme_source_contract_with_hash_cache(
        &request.episteme_root,
        &request.corpus_root,
        cache_path,
    )?;
    if !validation.passed {
        return Err(EpistemeError::InvalidContract(validation.errors));
    }
    Ok((
        materialize_validated_episteme_source_contract_read_model_seed(request)?,
        cache_report,
    ))
}

/// Compile a validated episteme registry reference graph into graph-readable
/// semantic read-model seed batches.
///
/// # Errors
///
/// Returns an error when registry graph rows cannot be encoded into the stable
/// Arrow read-model table schemas.
pub fn materialize_episteme_registry_reference_graph_read_model_seed(
    graph: &EpistemeRegistryReferenceGraphReceipt,
) -> Result<EpistemeReadModelMaterialization, EpistemeError> {
    let source_revision = registry_graph_revision(graph);
    let relation_rows = registry_graph_relation_rows(graph, source_revision.as_str());
    let object_rows = registry_graph_object_rows(graph, &relation_rows, source_revision.as_str())?;
    let projection_rows = registry_graph_projection_rows(&object_rows, source_revision.as_str())?;

    Ok(EpistemeReadModelMaterialization {
        source_revision,
        tables: vec![
            EpistemeReadModelTable::new(OBJECTS_TABLE, semantic_objects_batch(&object_rows)?),
            EpistemeReadModelTable::new(RELATIONS_TABLE, semantic_relations_batch(&relation_rows)?),
            EpistemeReadModelTable::new(
                PROJECTION_STATE_TABLE,
                semantic_projection_state_batch(&projection_rows)?,
            ),
        ],
    })
}

/// Compile audio transcript evidence rows into graph-readable
/// review-required semantic read-model seed batches.
///
fn materialize_validated_episteme_source_contract_read_model_seed(
    request: &EpistemeReadModelRequest,
) -> Result<EpistemeReadModelMaterialization, EpistemeError> {
    let contract_paths = source_contract_paths(request.episteme_root.as_path())?;
    let manifest = read_source_manifest(request.episteme_root.as_path())?;
    let corpus_dir = contract_paths.corpus_dir(request.episteme_root.as_path())?;
    let files = read_files_tsv(&corpus_dir.join(&manifest.files))?;
    let queue = read_queue_tsv(&corpus_dir.join(&manifest.extraction_queue))?;
    let mapping_ledger = read_mapping_ledger_raw(request.episteme_root.as_path())?;
    let source_revision = source_contract_revision(&manifest, &files, &queue, &mapping_ledger);
    let read_model_paths = SourceContractReadModelPaths {
        manifest: contract_paths.source_manifest_relative_path().to_string(),
        files: contract_paths.corpus_relative_path(&manifest.files),
        queue: contract_paths.corpus_relative_path(&manifest.extraction_queue),
        corpus_root_env: manifest.corpus_root_env.clone(),
        owner_scope: manifest.domain.clone(),
    };
    let object_rows = object_rows(&files, &queue, &read_model_paths, source_revision.as_str())?;
    let relation_rows = relation_rows(&queue, &read_model_paths, source_revision.as_str());
    let projection_rows =
        projection_rows(&object_rows, &read_model_paths, source_revision.as_str())?;

    Ok(EpistemeReadModelMaterialization {
        source_revision,
        tables: vec![
            EpistemeReadModelTable::new(OBJECTS_TABLE, semantic_objects_batch(&object_rows)?),
            EpistemeReadModelTable::new(RELATIONS_TABLE, semantic_relations_batch(&relation_rows)?),
            EpistemeReadModelTable::new(
                PROJECTION_STATE_TABLE,
                semantic_projection_state_batch(&projection_rows)?,
            ),
        ],
    })
}

/// Build a `WendaoGraph` ontology quality request from compiled episteme source-contract
/// read-model seed batches.
///
/// # Errors
///
/// Returns an error when required read-model tables are missing or relation
/// endpoints do not reference emitted objects.
#[cfg(feature = "julia")]
pub fn build_episteme_wendaograph_quality_request_batches(
    materialization: &EpistemeReadModelMaterialization,
) -> Result<WendaoGraphOntologyReadModelQualityRequestBatches, EpistemeError> {
    validate_episteme_read_model_relation_endpoints(materialization)?;
    Ok(WendaoGraphOntologyReadModelQualityRequestBatches::new(
        read_model_batch(materialization, OBJECTS_TABLE)?,
        read_model_batch(materialization, RELATIONS_TABLE)?,
        read_model_batch(materialization, PROJECTION_STATE_TABLE)?,
    ))
}

#[derive(Debug, Clone)]
struct SemanticObjectRow {
    id: String,
    kind: &'static str,
    title: String,
    status: &'static str,
    confidence_score: f64,
    confidence_source: &'static str,
    owner_count: i64,
    owners_json: String,
    provenance_source: String,
    provenance_recorded_by: &'static str,
    provenance_recorded_at: &'static str,
    verification_required_json: String,
    verification_evidence_json: String,
    relation_count: i64,
    source_path: String,
    read_model_source_revision: String,
    read_model_projection_revision: &'static str,
    read_model_projection_staleness: &'static str,
}

#[derive(Debug, Clone)]
struct SemanticRelationRow {
    source: String,
    kind: &'static str,
    target: String,
    source_path: String,
    read_model_source_revision: String,
    read_model_projection_revision: &'static str,
    read_model_projection_staleness: &'static str,
}

#[derive(Debug, Clone)]
struct SemanticProjectionStateRow {
    projection: &'static str,
    status: &'static str,
    source_revision: String,
    current_source_revision: String,
    projection_revision: &'static str,
    staleness: &'static str,
    source_object_count: i64,
    source_objects_json: String,
    source_path: String,
}

#[derive(Debug, Clone)]
struct SourceContractReadModelPaths {
    manifest: String,
    files: String,
    queue: String,
    corpus_root_env: String,
    owner_scope: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Owner<'a> {
    scope: &'a str,
    role: &'a str,
}

fn object_rows(
    files: &[EpistemeFileRow],
    queue: &[EpistemeExtractionQueueRow],
    paths: &SourceContractReadModelPaths,
    source_revision: &str,
) -> Result<Vec<SemanticObjectRow>, EpistemeError> {
    let relation_counts = relation_counts(queue);
    let mut rows = Vec::with_capacity(files.len() + queue.len());
    for file in files {
        rows.push(SemanticObjectRow {
            id: file.file_id.clone(),
            kind: FILE_OBJECT_KIND,
            title: file.relative_path.clone(),
            status: STATUS_ACTIVE,
            confidence_score: 1.0,
            confidence_source: CONFIDENCE_SOURCE,
            owner_count: 1,
            owners_json: owners_json(paths.owner_scope.as_str())?,
            provenance_source: paths.files.clone(),
            provenance_recorded_by: RECORDED_BY,
            provenance_recorded_at: RECORDED_AT,
            verification_required_json: json_array(["source_contract_validation"])?,
            verification_evidence_json: json_array([
                format!("files.tsv#{}", file.file_id),
                format!("sha256:{}", file.sha256),
            ])?,
            relation_count: i64::try_from(
                relation_counts
                    .get(file.file_id.as_str())
                    .copied()
                    .unwrap_or_default(),
            )
            .map_err(|error| EpistemeError::ReadModel(error.to_string()))?,
            source_path: format!("{}/{}", paths.corpus_root_env, file.relative_path),
            read_model_source_revision: source_revision.to_string(),
            read_model_projection_revision: PROJECTION_REVISION,
            read_model_projection_staleness: STALENESS_FRESH,
        });
    }
    for task in queue {
        rows.push(SemanticObjectRow {
            id: task.queue_id.clone(),
            kind: TASK_OBJECT_KIND,
            title: task.relative_path.clone(),
            status: STATUS_ACTIVE,
            confidence_score: 1.0,
            confidence_source: CONFIDENCE_SOURCE,
            owner_count: 1,
            owners_json: owners_json(paths.owner_scope.as_str())?,
            provenance_source: paths.queue.clone(),
            provenance_recorded_by: RECORDED_BY,
            provenance_recorded_at: RECORDED_AT,
            verification_required_json: json_array(["source_contract_validation"])?,
            verification_evidence_json: json_array([
                format!("extraction_queue.tsv#{}", task.queue_id),
                format!("file_id:{}", task.file_id),
            ])?,
            relation_count: 1,
            source_path: paths.queue.clone(),
            read_model_source_revision: source_revision.to_string(),
            read_model_projection_revision: PROJECTION_REVISION,
            read_model_projection_staleness: STALENESS_FRESH,
        });
    }
    Ok(rows)
}

fn relation_rows(
    queue: &[EpistemeExtractionQueueRow],
    paths: &SourceContractReadModelPaths,
    source_revision: &str,
) -> Vec<SemanticRelationRow> {
    queue
        .iter()
        .map(|row| SemanticRelationRow {
            source: row.queue_id.clone(),
            kind: TASK_SOURCE_RELATION_KIND,
            target: row.file_id.clone(),
            source_path: paths.queue.clone(),
            read_model_source_revision: source_revision.to_string(),
            read_model_projection_revision: PROJECTION_REVISION,
            read_model_projection_staleness: STALENESS_FRESH,
        })
        .collect()
}

fn projection_rows(
    object_rows: &[SemanticObjectRow],
    paths: &SourceContractReadModelPaths,
    source_revision: &str,
) -> Result<Vec<SemanticProjectionStateRow>, EpistemeError> {
    Ok(vec![SemanticProjectionStateRow {
        projection: PROJECTION_ID,
        status: STATUS_ACTIVE,
        source_revision: source_revision.to_string(),
        current_source_revision: source_revision.to_string(),
        projection_revision: PROJECTION_REVISION,
        staleness: STALENESS_FRESH,
        source_object_count: i64::try_from(object_rows.len())
            .map_err(|error| EpistemeError::ReadModel(error.to_string()))?,
        source_objects_json: json_array(object_rows.iter().map(|row| row.id.as_str()))?,
        source_path: paths.manifest.clone(),
    }])
}

fn relation_counts(queue: &[EpistemeExtractionQueueRow]) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for row in queue {
        *counts.entry(row.file_id.as_str()).or_insert(0) += 1;
        *counts.entry(row.queue_id.as_str()).or_insert(0) += 1;
    }
    counts
}

fn source_contract_revision(
    manifest: &EpistemeSourceManifest,
    files: &[EpistemeFileRow],
    queue: &[EpistemeExtractionQueueRow],
    mapping_ledger: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(manifest.source_contract_id.as_bytes());
    hasher.update(manifest.primary_language.as_bytes());
    for file in files {
        hasher.update(file.file_id.as_bytes());
        hasher.update(file.relative_path.as_bytes());
        hasher.update(file.sha256.as_bytes());
    }
    for row in queue {
        hasher.update(row.queue_id.as_bytes());
        hasher.update(row.file_id.as_bytes());
        hasher.update(row.extraction_route.as_bytes());
    }
    hasher.update(mapping_ledger.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn registry_graph_object_rows(
    graph: &EpistemeRegistryReferenceGraphReceipt,
    relations: &[SemanticRelationRow],
    source_revision: &str,
) -> Result<Vec<SemanticObjectRow>, EpistemeError> {
    let relation_counts = semantic_relation_counts(relations);
    let mut rows = Vec::new();
    for entry in &graph.entries {
        let entry_id = registry_entry_object_id(entry.registry_id.as_str());
        rows.push(SemanticObjectRow {
            id: entry_id.clone(),
            kind: REGISTRY_ENTRY_OBJECT_KIND,
            title: entry.registry_id.clone(),
            status: STATUS_ACTIVE,
            confidence_score: 1.0,
            confidence_source: REGISTRY_CONFIDENCE_SOURCE,
            owner_count: 1,
            owners_json: owners_json(entry.registry_id.as_str())?,
            provenance_source: registry_graph_source_path(entry.registry_id.as_str()),
            provenance_recorded_by: RECORDED_BY,
            provenance_recorded_at: RECORDED_AT,
            verification_required_json: json_array(["episteme_registry_reference_graph"])?,
            verification_evidence_json: json_array([
                format!("registry:{}", entry.registry_id),
                format!("domain_count:{}", entry.domain_ids.len()),
            ])?,
            relation_count: i64::try_from(
                relation_counts
                    .get(entry_id.as_str())
                    .copied()
                    .unwrap_or_default(),
            )
            .map_err(|error| EpistemeError::ReadModel(error.to_string()))?,
            source_path: registry_graph_source_path(entry.registry_id.as_str()),
            read_model_source_revision: source_revision.to_string(),
            read_model_projection_revision: REGISTRY_GRAPH_PROJECTION_REVISION,
            read_model_projection_staleness: STALENESS_FRESH,
        });
        for domain_id in &entry.domain_ids {
            let object_id = registry_domain_object_id(domain_id);
            rows.push(SemanticObjectRow {
                id: object_id.clone(),
                kind: REGISTRY_DOMAIN_OBJECT_KIND,
                title: domain_id.clone(),
                status: STATUS_ACTIVE,
                confidence_score: 1.0,
                confidence_source: REGISTRY_CONFIDENCE_SOURCE,
                owner_count: 1,
                owners_json: owners_json(entry.registry_id.as_str())?,
                provenance_source: registry_graph_source_path(entry.registry_id.as_str()),
                provenance_recorded_by: RECORDED_BY,
                provenance_recorded_at: RECORDED_AT,
                verification_required_json: json_array(["episteme_registry_reference_graph"])?,
                verification_evidence_json: json_array([
                    format!("registry:{}", entry.registry_id),
                    format!("domain:{domain_id}"),
                ])?,
                relation_count: i64::try_from(
                    relation_counts
                        .get(object_id.as_str())
                        .copied()
                        .unwrap_or_default(),
                )
                .map_err(|error| EpistemeError::ReadModel(error.to_string()))?,
                source_path: registry_graph_source_path(entry.registry_id.as_str()),
                read_model_source_revision: source_revision.to_string(),
                read_model_projection_revision: REGISTRY_GRAPH_PROJECTION_REVISION,
                read_model_projection_staleness: STALENESS_FRESH,
            });
        }
    }
    Ok(rows)
}

fn registry_graph_relation_rows(
    graph: &EpistemeRegistryReferenceGraphReceipt,
    source_revision: &str,
) -> Vec<SemanticRelationRow> {
    let mut rows = Vec::new();
    for entry in &graph.entries {
        let entry_id = registry_entry_object_id(entry.registry_id.as_str());
        for domain_id in &entry.domain_ids {
            rows.push(SemanticRelationRow {
                source: entry_id.clone(),
                kind: REGISTRY_ENTRY_DOMAIN_RELATION_KIND,
                target: registry_domain_object_id(domain_id),
                source_path: registry_graph_source_path(entry.registry_id.as_str()),
                read_model_source_revision: source_revision.to_string(),
                read_model_projection_revision: REGISTRY_GRAPH_PROJECTION_REVISION,
                read_model_projection_staleness: STALENESS_FRESH,
            });
        }
    }
    for link in &graph.reference_links {
        rows.push(SemanticRelationRow {
            source: registry_entry_object_id(link.source_registry.as_str()),
            kind: REGISTRY_ENTRY_EXTENDS_RELATION_KIND,
            target: registry_domain_object_id(link.target_domain.as_str()),
            source_path: registry_graph_source_path(link.source_registry.as_str()),
            read_model_source_revision: source_revision.to_string(),
            read_model_projection_revision: REGISTRY_GRAPH_PROJECTION_REVISION,
            read_model_projection_staleness: STALENESS_FRESH,
        });
    }
    rows
}

fn registry_graph_projection_rows(
    object_rows: &[SemanticObjectRow],
    source_revision: &str,
) -> Result<Vec<SemanticProjectionStateRow>, EpistemeError> {
    Ok(vec![SemanticProjectionStateRow {
        projection: REGISTRY_GRAPH_PROJECTION_ID,
        status: STATUS_ACTIVE,
        source_revision: source_revision.to_string(),
        current_source_revision: source_revision.to_string(),
        projection_revision: REGISTRY_GRAPH_PROJECTION_REVISION,
        staleness: STALENESS_FRESH,
        source_object_count: i64::try_from(object_rows.len())
            .map_err(|error| EpistemeError::ReadModel(error.to_string()))?,
        source_objects_json: json_array(object_rows.iter().map(|row| row.id.as_str()))?,
        source_path: "episteme_registry:reference_graph".to_string(),
    }])
}

fn semantic_relation_counts(relations: &[SemanticRelationRow]) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for relation in relations {
        *counts.entry(relation.source.as_str()).or_insert(0) += 1;
        *counts.entry(relation.target.as_str()).or_insert(0) += 1;
    }
    counts
}

fn registry_graph_revision(graph: &EpistemeRegistryReferenceGraphReceipt) -> String {
    let mut hasher = Sha256::new();
    hasher.update(graph.schema_version.as_bytes());
    for entry in &graph.entries {
        hasher.update(entry.registry_id.as_bytes());
        for domain_id in &entry.domain_ids {
            hasher.update(domain_id.as_bytes());
        }
        for target in &entry.extension_targets {
            hasher.update(target.as_bytes());
        }
    }
    for link in &graph.reference_links {
        hasher.update(link.source_registry.as_bytes());
        hasher.update(link.target_domain.as_bytes());
        hasher.update(link.target_registry.as_bytes());
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn registry_entry_object_id(registry_id: &str) -> String {
    format!("episteme_registry.entry:{registry_id}")
}

fn registry_domain_object_id(domain_id: &str) -> String {
    format!("episteme_registry.domain:{domain_id}")
}

fn registry_graph_source_path(registry_id: &str) -> String {
    format!("episteme_registry:{registry_id}/ontology/manifest.toml")
}

fn owners_json(owner_scope: &str) -> Result<String, EpistemeError> {
    json_array([Owner {
        scope: owner_scope,
        role: "episteme_source_contract",
    }])
}

fn json_array<T, I>(values: I) -> Result<String, EpistemeError>
where
    T: Serialize,
    I: IntoIterator<Item = T>,
{
    serde_json::to_string(&values.into_iter().collect::<Vec<_>>())
        .map_err(|error| EpistemeError::ReadModel(error.to_string()))
}

#[cfg(feature = "julia")]
fn read_model_batch(
    materialization: &EpistemeReadModelMaterialization,
    table_name: &str,
) -> Result<RecordBatch, EpistemeError> {
    materialization
        .tables
        .iter()
        .find(|table| table.table_name() == table_name)
        .map(|table| table.batch().clone())
        .ok_or_else(|| EpistemeError::ReadModel(format!("missing {table_name}")))
}

/// Validate that all relation endpoints reference emitted objects.
///
/// # Errors
///
/// Returns an error when either relation endpoint is missing from the objects
/// table.
pub fn validate_episteme_read_model_relation_endpoints(
    materialization: &EpistemeReadModelMaterialization,
) -> Result<(), EpistemeError> {
    let objects = materialization
        .tables
        .iter()
        .find(|table| table.table_name() == OBJECTS_TABLE)
        .ok_or_else(|| EpistemeError::ReadModel("missing semantic_objects".to_string()))?;
    let relations = materialization
        .tables
        .iter()
        .find(|table| table.table_name() == RELATIONS_TABLE)
        .ok_or_else(|| EpistemeError::ReadModel("missing semantic_relations".to_string()))?;
    let ids = object_ids(objects.batch());
    let source_column = relations
        .batch()
        .column_by_name("source")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| EpistemeError::ReadModel("missing relation source".to_string()))?;
    let target_column = relations
        .batch()
        .column_by_name("target")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| EpistemeError::ReadModel("missing relation target".to_string()))?;
    for row_index in 0..relations.batch().num_rows() {
        let source = source_column.value(row_index);
        let target = target_column.value(row_index);
        if !ids.contains(source) || !ids.contains(target) {
            return Err(EpistemeError::ReadModel(format!(
                "relation row {row_index} references missing endpoint `{source}` -> `{target}`"
            )));
        }
    }
    Ok(())
}
