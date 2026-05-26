//! Arrow IPC request builders for the `WendaoGraph` ontology read-model bridge.

use std::io::Cursor;
use std::sync::Arc;

use arrow::array::{ArrayRef, BinaryArray, StringArray};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, build_arrow_schema, validate_record_batch_schema_with_options,
};

use crate::arrow_metadata::attach_record_batch_metadata;

use super::constants::{
    METHOD_METADATA_KEY, SCHEMA_VERSION_METADATA_KEY, SERVICE_METADATA_KEY, TABLE_METADATA_KEY,
    WENDAO_GRAPH_ONTOLOGY_EXTENSION_DOMAIN_PREFIX_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_EXTENSION_PROOF_REQUEST_TABLES,
    WENDAO_GRAPH_ONTOLOGY_EXTENSION_PROOF_RESPONSE_TABLE,
    WENDAO_GRAPH_ONTOLOGY_PARENT_LINK_TYPES_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_PARENT_LINK_TYPES_TABLE,
    WENDAO_GRAPH_ONTOLOGY_PARENT_OBJECT_TYPES_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_PARENT_OBJECT_TYPES_TABLE, WENDAO_GRAPH_ONTOLOGY_RDF_NAMESPACE_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_METHOD,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_BUNDLE_TABLE,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_TABLES,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_RESPONSE_TABLE,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SERVICE,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PROJECTION_STATE_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_RELATIONS_PAYLOAD_COLUMN,
};
use super::types::{
    WendaoGraphOntologyExtensionProofArrowRequest, WendaoGraphOntologyExtensionProofRequestBatches,
    WendaoGraphOntologyReadModelQualityArrowRequest,
    WendaoGraphOntologyReadModelQualityRequestBatches,
};

/// Build Arrow IPC request payloads for the `WendaoGraph` ontology quality service.
///
/// # Errors
///
/// Returns an error when metadata cannot be attached to a request table or when
/// any request table cannot be encoded as an Arrow IPC stream.
pub fn build_wendaograph_ontology_read_model_quality_arrow_request(
    batches: &WendaoGraphOntologyReadModelQualityRequestBatches,
) -> Result<WendaoGraphOntologyReadModelQualityArrowRequest, String> {
    let semantic_objects_payload = encode_request_table(&batches.objects, "semantic_objects")?;
    let semantic_relations_payload =
        encode_request_table(&batches.relations, "semantic_relations")?;
    let semantic_projection_state_payload =
        encode_request_table(&batches.projection_state, "semantic_projection_state")?;

    Ok(WendaoGraphOntologyReadModelQualityArrowRequest {
        schema_version: WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION,
        request_mime_type: WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME,
        response_mime_type: WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME,
        request_tables: WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_TABLES,
        response_table: WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_RESPONSE_TABLE,
        semantic_objects_payload,
        semantic_relations_payload,
        semantic_projection_state_payload,
    })
}

/// Build Arrow IPC request payloads for the `WendaoGraph` ontology extension proof mode.
///
/// # Errors
///
/// Returns an error when the extension domain prefix is blank, metadata cannot
/// be attached to a request table, or any request table cannot be encoded as an
/// Arrow IPC stream.
pub fn build_wendaograph_ontology_extension_proof_arrow_request(
    batches: &WendaoGraphOntologyExtensionProofRequestBatches,
    extension_domain_prefix: &str,
    rdf_namespace: &str,
) -> Result<WendaoGraphOntologyExtensionProofArrowRequest, String> {
    let extension_domain_prefix = extension_domain_prefix.trim();
    if extension_domain_prefix.is_empty() {
        return Err(
            "WendaoGraph ontology extension proof domain prefix must not be blank".to_string(),
        );
    }

    let quality_request =
        build_wendaograph_ontology_read_model_quality_arrow_request(&batches.read_model)?;
    let parent_object_types_payload = encode_request_table(
        &batches.parent_object_types,
        WENDAO_GRAPH_ONTOLOGY_PARENT_OBJECT_TYPES_TABLE,
    )?;
    let parent_link_types_payload = encode_request_table(
        &batches.parent_link_types,
        WENDAO_GRAPH_ONTOLOGY_PARENT_LINK_TYPES_TABLE,
    )?;

    Ok(WendaoGraphOntologyExtensionProofArrowRequest {
        schema_version: WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION,
        request_mime_type: WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME,
        response_mime_type: WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME,
        request_tables: WENDAO_GRAPH_ONTOLOGY_EXTENSION_PROOF_REQUEST_TABLES,
        response_table: WENDAO_GRAPH_ONTOLOGY_EXTENSION_PROOF_RESPONSE_TABLE,
        semantic_objects_payload: quality_request.semantic_objects_payload,
        semantic_relations_payload: quality_request.semantic_relations_payload,
        semantic_projection_state_payload: quality_request.semantic_projection_state_payload,
        parent_object_types_payload,
        parent_link_types_payload,
        extension_domain_prefix: extension_domain_prefix.to_owned(),
        rdf_namespace: rdf_namespace.trim().to_owned(),
    })
}

/// Build the single-table Arrow Flight request bundle for ontology quality scoring.
///
/// # Errors
///
/// Returns an error when the request bundle cannot be encoded as an Arrow
/// `RecordBatch`.
pub fn build_wendaograph_ontology_read_model_quality_flight_request_batch(
    request: &WendaoGraphOntologyReadModelQualityArrowRequest,
) -> Result<RecordBatch, String> {
    let contract = ontology_read_model_quality_bundle_contract();
    let schema = Arc::new(build_arrow_schema(&contract, request_bundle_metadata()));

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(BinaryArray::from(vec![
                request.semantic_objects_payload.as_slice(),
            ])) as ArrayRef,
            Arc::new(BinaryArray::from(vec![
                request.semantic_relations_payload.as_slice(),
            ])) as ArrayRef,
            Arc::new(BinaryArray::from(vec![
                request.semantic_projection_state_payload.as_slice(),
            ])) as ArrayRef,
        ],
    )
    .map_err(|error| format!("build WendaoGraph ontology Flight request batch: {error}"))?;

    validate_bundle_batch(
        &batch,
        &contract,
        "WendaoGraph ontology Flight request batch",
    )?;
    Ok(batch)
}

fn ontology_read_model_quality_bundle_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_BUNDLE_TABLE,
        true,
        vec![
            ArrowSchemaColumn::new(
                WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN,
                ArrowSchemaDataType::Binary,
            ),
            ArrowSchemaColumn::new(
                WENDAO_GRAPH_ONTOLOGY_SEMANTIC_RELATIONS_PAYLOAD_COLUMN,
                ArrowSchemaDataType::Binary,
            ),
            ArrowSchemaColumn::new(
                WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PROJECTION_STATE_PAYLOAD_COLUMN,
                ArrowSchemaDataType::Binary,
            ),
        ],
    )
}

/// Build the single-table Arrow Flight request bundle for ontology extension proof.
///
/// # Errors
///
/// Returns an error when the request bundle cannot be encoded as an Arrow
/// `RecordBatch`.
pub fn build_wendaograph_ontology_extension_proof_flight_request_batch(
    request: &WendaoGraphOntologyExtensionProofArrowRequest,
) -> Result<RecordBatch, String> {
    let contract = ontology_extension_proof_bundle_contract();
    let schema = Arc::new(build_arrow_schema(&contract, request_bundle_metadata()));

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(BinaryArray::from(vec![
                request.semantic_objects_payload.as_slice(),
            ])) as ArrayRef,
            Arc::new(BinaryArray::from(vec![
                request.semantic_relations_payload.as_slice(),
            ])) as ArrayRef,
            Arc::new(BinaryArray::from(vec![
                request.semantic_projection_state_payload.as_slice(),
            ])) as ArrayRef,
            Arc::new(BinaryArray::from(vec![
                request.parent_object_types_payload.as_slice(),
            ])) as ArrayRef,
            Arc::new(BinaryArray::from(vec![
                request.parent_link_types_payload.as_slice(),
            ])) as ArrayRef,
            Arc::new(StringArray::from(vec![
                request.extension_domain_prefix.as_str(),
            ])) as ArrayRef,
            Arc::new(StringArray::from(vec![request.rdf_namespace.as_str()])) as ArrayRef,
        ],
    )
    .map_err(|error| {
        format!("build WendaoGraph ontology extension Flight request batch: {error}")
    })?;

    validate_bundle_batch(
        &batch,
        &contract,
        "WendaoGraph ontology extension Flight request batch",
    )?;
    Ok(batch)
}

fn ontology_extension_proof_bundle_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_BUNDLE_TABLE,
        true,
        vec![
            ArrowSchemaColumn::new(
                WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN,
                ArrowSchemaDataType::Binary,
            ),
            ArrowSchemaColumn::new(
                WENDAO_GRAPH_ONTOLOGY_SEMANTIC_RELATIONS_PAYLOAD_COLUMN,
                ArrowSchemaDataType::Binary,
            ),
            ArrowSchemaColumn::new(
                WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PROJECTION_STATE_PAYLOAD_COLUMN,
                ArrowSchemaDataType::Binary,
            ),
            ArrowSchemaColumn::new(
                WENDAO_GRAPH_ONTOLOGY_PARENT_OBJECT_TYPES_PAYLOAD_COLUMN,
                ArrowSchemaDataType::Binary,
            ),
            ArrowSchemaColumn::new(
                WENDAO_GRAPH_ONTOLOGY_PARENT_LINK_TYPES_PAYLOAD_COLUMN,
                ArrowSchemaDataType::Binary,
            ),
            ArrowSchemaColumn::new(
                WENDAO_GRAPH_ONTOLOGY_EXTENSION_DOMAIN_PREFIX_COLUMN,
                ArrowSchemaDataType::Utf8,
            ),
            ArrowSchemaColumn::new(
                WENDAO_GRAPH_ONTOLOGY_RDF_NAMESPACE_COLUMN,
                ArrowSchemaDataType::Utf8,
            ),
        ],
    )
}

fn request_bundle_metadata() -> std::collections::HashMap<String, String> {
    [
        (
            SERVICE_METADATA_KEY.to_owned(),
            WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SERVICE.to_owned(),
        ),
        (
            METHOD_METADATA_KEY.to_owned(),
            WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_METHOD.to_owned(),
        ),
        (
            SCHEMA_VERSION_METADATA_KEY.to_owned(),
            WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION.to_owned(),
        ),
        (
            TABLE_METADATA_KEY.to_owned(),
            WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_BUNDLE_TABLE.to_owned(),
        ),
    ]
    .into_iter()
    .collect()
}

fn validate_bundle_batch(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
    context: &str,
) -> Result<(), String> {
    validate_record_batch_schema_with_options(
        batch,
        contract,
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .map_err(|error| format!("validate {context}: {error}"))
}

fn encode_request_table(batch: &RecordBatch, table_name: &'static str) -> Result<Vec<u8>, String> {
    let batch = attach_record_batch_metadata(
        batch,
        [
            (
                SERVICE_METADATA_KEY,
                WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SERVICE,
            ),
            (
                METHOD_METADATA_KEY,
                WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_METHOD,
            ),
            (
                SCHEMA_VERSION_METADATA_KEY,
                WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION,
            ),
            (TABLE_METADATA_KEY, table_name),
        ],
    )
    .map_err(|error| format!("attach WendaoGraph ontology request metadata: {error}"))?;

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = StreamWriter::try_new(&mut buffer, batch.schema().as_ref())
            .map_err(|error| format!("open WendaoGraph ontology Arrow IPC writer: {error}"))?;
        writer
            .write(&batch)
            .map_err(|error| format!("write WendaoGraph ontology Arrow IPC table: {error}"))?;
        writer
            .finish()
            .map_err(|error| format!("finish WendaoGraph ontology Arrow IPC stream: {error}"))?;
    }
    Ok(buffer.into_inner())
}
