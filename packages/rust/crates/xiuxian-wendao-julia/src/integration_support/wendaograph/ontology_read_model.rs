//! Arrow IPC bridge contract for `WendaoGraph` ontology read-model quality checks.

use std::io::Cursor;
use std::sync::Arc;

use arrow::array::{ArrayRef, BinaryArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use arrow_flight::FlightDescriptor;
use serde_json::{Map, Value};
use xiuxian_wendao_core::PluginProviderSelector;
use xiuxian_wendao_core::capabilities::{ContractVersion, PluginCapabilityBinding};
use xiuxian_wendao_core::ids::{CapabilityId, PluginId};
use xiuxian_wendao_core::transport::{PluginTransportEndpoint, PluginTransportKind};
use xiuxian_wendao_runtime::transport::{
    NegotiatedTransportSelection, negotiate_flight_transport_client_from_bindings,
};

use crate::arrow_metadata::attach_record_batch_metadata;

/// `WendaoGraph` service name for ontology read-model quality scoring.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SERVICE: &str =
    "wendao.graph.v1.OntologyReadModelQuality";
/// `WendaoGraph` service method for ontology read-model quality scoring.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_METHOD: &str = "RunOntologyReadModelQuality";
/// `WendaoGraph` ontology read-model quality service schema version.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION: &str =
    "xiuxian_wendao.graph.ontology_read_model_quality.service.v1";
/// MIME type used by the `WendaoGraph` ontology read-model quality Arrow IPC service.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME: &str =
    "application/vnd.apache.arrow.stream";
/// Flight descriptor path for the `WendaoGraph` ontology read-model quality service.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_FLIGHT_DESCRIPTOR_PATH: [&str; 3] =
    ["wendao", "graph", "ontology_read_model_quality"];
/// Canonical route form used by runtime Flight transport negotiation.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE: &str =
    "/wendao/graph/ontology_read_model_quality";
/// Stable provider id for the `WendaoGraph` ontology read-model quality service.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_PROVIDER_ID: &str = "wendaograph";
/// Stable capability id for the `WendaoGraph` ontology read-model quality service.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_CAPABILITY_ID: &str =
    "ontology-read-model-quality";
/// Single request table name used to bundle the three read-model Arrow payloads over Flight.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_BUNDLE_TABLE: &str =
    "ontology_read_model_quality_request";
/// Request table names expected by the `WendaoGraph` ontology read-model quality service.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_TABLES: [&str; 3] = [
    "semantic_objects",
    "semantic_relations",
    "semantic_projection_state",
];
/// Response table name returned by the `WendaoGraph` ontology read-model quality service.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_RESPONSE_TABLE: &str = "ontology_quality_rows";
/// Bundle column containing the `semantic_objects` Arrow IPC payload.
pub const WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN: &str = "semantic_objects_payload";
/// Bundle column containing the `semantic_relations` Arrow IPC payload.
pub const WENDAO_GRAPH_ONTOLOGY_SEMANTIC_RELATIONS_PAYLOAD_COLUMN: &str =
    "semantic_relations_payload";
/// Bundle column containing the `semantic_projection_state` Arrow IPC payload.
pub const WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PROJECTION_STATE_PAYLOAD_COLUMN: &str =
    "semantic_projection_state_payload";

const SERVICE_METADATA_KEY: &str = "wendao.service";
const METHOD_METADATA_KEY: &str = "wendao.method";
const SCHEMA_VERSION_METADATA_KEY: &str = "wendao.schema_version";
const TABLE_METADATA_KEY: &str = "wendao.table";
const DATASET_ONTOLOGY_ENVELOPE_RECORD_KIND_COLUMN: &str = "recordKind";
const DATASET_ONTOLOGY_ENVELOPE_TABLE_NAME_COLUMN: &str = "tableName";
const DATASET_ONTOLOGY_ENVELOPE_PAYLOAD_JSON_COLUMN: &str = "payloadJson";
const DATASET_ONTOLOGY_SEMANTIC_READ_MODEL_KIND: &str = "semantic_read_model";
const SEMANTIC_OBJECTS_TABLE: &str = "semantic_objects";
const SEMANTIC_RELATIONS_TABLE: &str = "semantic_relations";
const SEMANTIC_PROJECTION_STATE_TABLE: &str = "semantic_projection_state";

/// Semantic read-model Arrow tables accepted by the `WendaoGraph` quality service.
#[derive(Debug, Clone)]
pub struct WendaoGraphOntologyReadModelQualityRequestBatches {
    /// Accepted `semantic_objects` read-model table.
    pub objects: RecordBatch,
    /// Accepted `semantic_relations` read-model table.
    pub relations: RecordBatch,
    /// Accepted `semantic_projection_state` read-model table.
    pub projection_state: RecordBatch,
}

impl WendaoGraphOntologyReadModelQualityRequestBatches {
    /// Create a request batch bundle from already materialized read-model tables.
    #[must_use]
    pub fn new(
        semantic_objects: RecordBatch,
        semantic_relations: RecordBatch,
        semantic_projection_state: RecordBatch,
    ) -> Self {
        Self {
            objects: semantic_objects,
            relations: semantic_relations,
            projection_state: semantic_projection_state,
        }
    }

    /// Return the row counts for the request tables in service order.
    #[must_use]
    pub fn row_counts(&self) -> [usize; 3] {
        [
            self.objects.num_rows(),
            self.relations.num_rows(),
            self.projection_state.num_rows(),
        ]
    }
}

/// Arrow IPC request payloads for the `WendaoGraph` ontology quality service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoGraphOntologyReadModelQualityArrowRequest {
    /// Service schema version expected by `WendaoGraph`.
    pub schema_version: &'static str,
    /// Request MIME type for every payload.
    pub request_mime_type: &'static str,
    /// Response MIME type expected from `WendaoGraph`.
    pub response_mime_type: &'static str,
    /// Request table names in payload order.
    pub request_tables: [&'static str; 3],
    /// Response table name expected from `WendaoGraph`.
    pub response_table: &'static str,
    /// Arrow IPC stream for `semantic_objects`.
    pub semantic_objects_payload: Vec<u8>,
    /// Arrow IPC stream for `semantic_relations`.
    pub semantic_relations_payload: Vec<u8>,
    /// Arrow IPC stream for `semantic_projection_state`.
    pub semantic_projection_state_payload: Vec<u8>,
}

/// Runtime binding options for the `WendaoGraph` ontology quality Flight route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoGraphOntologyReadModelQualityFlightBindingOptions {
    /// Flight service base URL, for example `http://127.0.0.1:41082`.
    pub base_url: String,
    /// Optional health route for service readiness probes.
    pub health_route: Option<String>,
    /// Optional request timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Optional maximum in-flight requests for one transport client.
    pub max_in_flight_requests: Option<u64>,
}

/// Response from one negotiated ontology quality Flight exchange.
#[derive(Debug, Clone)]
pub struct WendaoGraphOntologyReadModelQualityRoundtrip {
    /// Runtime transport selected for the exchange.
    pub selection: NegotiatedTransportSelection,
    /// Raw Arrow response batches returned by `WendaoGraph`.
    pub response_batches: Vec<RecordBatch>,
}

/// Error returned when an ontology quality Flight exchange fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoGraphOntologyReadModelQualityRoundtripError {
    /// Runtime selection, when the exchange reached a negotiated transport.
    pub selection: Option<NegotiatedTransportSelection>,
    /// Human-readable failure detail.
    pub error: String,
}

impl WendaoGraphOntologyReadModelQualityArrowRequest {
    /// Return the encoded payload byte sizes in service request order.
    #[must_use]
    pub fn payload_byte_sizes(&self) -> [usize; 3] {
        [
            self.semantic_objects_payload.len(),
            self.semantic_relations_payload.len(),
            self.semantic_projection_state_payload.len(),
        ]
    }
}

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

/// Extract `WendaoGraph` quality request tables from the Gateway dataset-ontology envelope.
///
/// The Gateway envelope is the stable transport surface for dataset ontology
/// materialization. This converter intentionally accepts only compiled
/// `semantic_read_model` rows and ignores report rows, so downstream
/// `WendaoGraph` never needs to parse raw CSV fixtures, RDF, or project config.
///
/// # Errors
///
/// Returns an error when the envelope columns are missing, a semantic read-model
/// row targets an unsupported table, a payload is malformed, a required field is
/// absent, or any required read-model table is missing.
pub fn build_wendaograph_ontology_read_model_quality_request_batches_from_dataset_ontology_envelope(
    batches: &[RecordBatch],
) -> Result<WendaoGraphOntologyReadModelQualityRequestBatches, String> {
    let mut semantic_objects = Vec::new();
    let mut semantic_relations = Vec::new();
    let mut semantic_projection_state = Vec::new();

    for batch in batches {
        let record_kind_column = string_column(
            batch,
            DATASET_ONTOLOGY_ENVELOPE_RECORD_KIND_COLUMN,
            "dataset ontology envelope",
        )?;
        let table_name_column = string_column(
            batch,
            DATASET_ONTOLOGY_ENVELOPE_TABLE_NAME_COLUMN,
            "dataset ontology envelope",
        )?;
        let payload_json_column = string_column(
            batch,
            DATASET_ONTOLOGY_ENVELOPE_PAYLOAD_JSON_COLUMN,
            "dataset ontology envelope",
        )?;

        for row_index in 0..batch.num_rows() {
            let record_kind = record_kind_column.value(row_index);
            if record_kind != DATASET_ONTOLOGY_SEMANTIC_READ_MODEL_KIND {
                continue;
            }

            let table_name = table_name_column.value(row_index);
            let payload = payload_json_object(payload_json_column.value(row_index), table_name)?;
            match table_name {
                SEMANTIC_OBJECTS_TABLE => semantic_objects.push(payload),
                SEMANTIC_RELATIONS_TABLE => semantic_relations.push(payload),
                SEMANTIC_PROJECTION_STATE_TABLE => semantic_projection_state.push(payload),
                unsupported => {
                    return Err(format!(
                        "dataset ontology envelope contains unsupported semantic read-model table `{unsupported}`",
                    ));
                }
            }
        }
    }

    if semantic_objects.is_empty() {
        return Err("dataset ontology envelope omitted `semantic_objects` rows".to_string());
    }
    if semantic_relations.is_empty() {
        return Err("dataset ontology envelope omitted `semantic_relations` rows".to_string());
    }
    if semantic_projection_state.is_empty() {
        return Err(
            "dataset ontology envelope omitted `semantic_projection_state` rows".to_string(),
        );
    }

    Ok(WendaoGraphOntologyReadModelQualityRequestBatches::new(
        semantic_objects_batch(&semantic_objects)?,
        semantic_relations_batch(&semantic_relations)?,
        semantic_projection_state_batch(&semantic_projection_state)?,
    ))
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
    let schema = Arc::new(Schema::new_with_metadata(
        vec![
            Field::new(
                WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN,
                DataType::Binary,
                false,
            ),
            Field::new(
                WENDAO_GRAPH_ONTOLOGY_SEMANTIC_RELATIONS_PAYLOAD_COLUMN,
                DataType::Binary,
                false,
            ),
            Field::new(
                WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PROJECTION_STATE_PAYLOAD_COLUMN,
                DataType::Binary,
                false,
            ),
        ],
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
        .collect(),
    ));

    RecordBatch::try_new(
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
    .map_err(|error| format!("build WendaoGraph ontology Flight request batch: {error}"))
}

/// Build the Flight descriptor for the `WendaoGraph` ontology quality service.
#[must_use]
pub fn build_wendaograph_ontology_read_model_quality_flight_descriptor() -> FlightDescriptor {
    FlightDescriptor::new_path(
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_FLIGHT_DESCRIPTOR_PATH
            .into_iter()
            .map(str::to_owned)
            .collect(),
    )
}

/// Build the canonical provider selector for the `WendaoGraph` ontology quality service.
#[must_use]
pub fn wendaograph_ontology_read_model_quality_provider_selector() -> PluginProviderSelector {
    PluginProviderSelector {
        capability_id: CapabilityId(
            WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_CAPABILITY_ID.to_owned(),
        ),
        provider: PluginId(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_PROVIDER_ID.to_owned()),
    }
}

/// Build one runtime-negotiable Arrow Flight binding for ontology quality scoring.
///
/// # Errors
///
/// Returns an error when the Flight base URL is blank.
pub fn build_wendaograph_ontology_read_model_quality_flight_binding(
    options: WendaoGraphOntologyReadModelQualityFlightBindingOptions,
) -> Result<PluginCapabilityBinding, String> {
    let base_url = options.base_url.trim();
    if base_url.is_empty() {
        return Err("WendaoGraph ontology quality Flight base URL must not be blank".to_string());
    }

    Ok(PluginCapabilityBinding {
        selector: wendaograph_ontology_read_model_quality_provider_selector(),
        endpoint: PluginTransportEndpoint {
            base_url: Some(base_url.to_owned()),
            route: Some(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE.to_owned()),
            health_route: options.health_route,
            timeout_secs: options.timeout_secs,
            max_in_flight_requests: options.max_in_flight_requests,
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion(
            WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION.to_owned(),
        ),
    })
}

/// Run one ontology quality exchange through the shared runtime Flight transport.
///
/// # Errors
///
/// Returns [`WendaoGraphOntologyReadModelQualityRoundtripError`] when request
/// packaging, transport negotiation, or the Flight exchange fails.
pub async fn roundtrip_wendaograph_ontology_read_model_quality_with_binding(
    binding: &PluginCapabilityBinding,
    batches: &WendaoGraphOntologyReadModelQualityRequestBatches,
) -> Result<
    Option<WendaoGraphOntologyReadModelQualityRoundtrip>,
    WendaoGraphOntologyReadModelQualityRoundtripError,
> {
    let request =
        build_wendaograph_ontology_read_model_quality_arrow_request(batches).map_err(|error| {
            WendaoGraphOntologyReadModelQualityRoundtripError {
                selection: None,
                error,
            }
        })?;
    let request_batch = build_wendaograph_ontology_read_model_quality_flight_request_batch(
        &request,
    )
    .map_err(|error| WendaoGraphOntologyReadModelQualityRoundtripError {
        selection: None,
        error,
    })?;
    let Some(transport) = negotiate_flight_transport_client_from_bindings(std::slice::from_ref(
        binding,
    ))
    .map_err(|error| WendaoGraphOntologyReadModelQualityRoundtripError {
        selection: None,
        error,
    })?
    else {
        return Ok(None);
    };

    let selection = transport.selection().clone();
    let response_batches = transport
        .process_batch(&request_batch)
        .await
        .map_err(|error| WendaoGraphOntologyReadModelQualityRoundtripError {
            selection: Some(selection.clone()),
            error,
        })?;

    Ok(Some(WendaoGraphOntologyReadModelQualityRoundtrip {
        selection,
        response_batches,
    }))
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

fn string_column<'a>(
    batch: &'a RecordBatch,
    column_name: &str,
    subject: &str,
) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(column_name)
        .ok_or_else(|| format!("{subject} is missing `{column_name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("{subject} column `{column_name}` must be Utf8"))
}

fn payload_json_object(payload_json: &str, table_name: &str) -> Result<Map<String, Value>, String> {
    let value = serde_json::from_str::<Value>(payload_json).map_err(|error| {
        format!(
            "dataset ontology `{table_name}` semantic read-model payload is invalid JSON: {error}"
        )
    })?;
    match value {
        Value::Object(object) => Ok(object),
        _ => Err(format!(
            "dataset ontology `{table_name}` semantic read-model payload must be a JSON object"
        )),
    }
}

fn semantic_objects_batch(rows: &[Map<String, Value>]) -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("kind", DataType::Utf8, false),
        Field::new("title", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("confidence_score", DataType::Float64, false),
        Field::new("confidence_source", DataType::Utf8, false),
        Field::new("owner_count", DataType::Int64, false),
        Field::new("owners_json", DataType::Utf8, false),
        Field::new("provenance_source", DataType::Utf8, false),
        Field::new("provenance_recorded_by", DataType::Utf8, false),
        Field::new("provenance_recorded_at", DataType::Utf8, false),
        Field::new("verification_required_json", DataType::Utf8, false),
        Field::new("verification_evidence_json", DataType::Utf8, false),
        Field::new("relation_count", DataType::Int64, false),
        Field::new("source_path", DataType::Utf8, false),
        Field::new("read_model_source_revision", DataType::Utf8, false),
        Field::new("read_model_projection_revision", DataType::Utf8, false),
        Field::new("read_model_projection_staleness", DataType::Utf8, false),
    ]));

    RecordBatch::try_new(
        schema,
        vec![
            string_array(rows, "id", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "kind", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "title", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "status", SEMANTIC_OBJECTS_TABLE)?,
            float64_array(rows, "confidence_score", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "confidence_source", SEMANTIC_OBJECTS_TABLE)?,
            int64_array(rows, "owner_count", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "owners_json", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "provenance_source", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "provenance_recorded_by", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "provenance_recorded_at", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "verification_required_json", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "verification_evidence_json", SEMANTIC_OBJECTS_TABLE)?,
            int64_array(rows, "relation_count", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "source_path", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "read_model_source_revision", SEMANTIC_OBJECTS_TABLE)?,
            string_array(
                rows,
                "read_model_projection_revision",
                SEMANTIC_OBJECTS_TABLE,
            )?,
            string_array(
                rows,
                "read_model_projection_staleness",
                SEMANTIC_OBJECTS_TABLE,
            )?,
        ],
    )
    .map_err(|error| format!("build `{SEMANTIC_OBJECTS_TABLE}` request batch: {error}"))
}

fn semantic_relations_batch(rows: &[Map<String, Value>]) -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("source", DataType::Utf8, false),
        Field::new("kind", DataType::Utf8, false),
        Field::new("target", DataType::Utf8, false),
        Field::new("source_path", DataType::Utf8, false),
        Field::new("read_model_source_revision", DataType::Utf8, false),
        Field::new("read_model_projection_revision", DataType::Utf8, false),
        Field::new("read_model_projection_staleness", DataType::Utf8, false),
    ]));

    RecordBatch::try_new(
        schema,
        vec![
            string_array(rows, "source", SEMANTIC_RELATIONS_TABLE)?,
            string_array(rows, "kind", SEMANTIC_RELATIONS_TABLE)?,
            string_array(rows, "target", SEMANTIC_RELATIONS_TABLE)?,
            string_array(rows, "source_path", SEMANTIC_RELATIONS_TABLE)?,
            string_array(rows, "read_model_source_revision", SEMANTIC_RELATIONS_TABLE)?,
            string_array(
                rows,
                "read_model_projection_revision",
                SEMANTIC_RELATIONS_TABLE,
            )?,
            string_array(
                rows,
                "read_model_projection_staleness",
                SEMANTIC_RELATIONS_TABLE,
            )?,
        ],
    )
    .map_err(|error| format!("build `{SEMANTIC_RELATIONS_TABLE}` request batch: {error}"))
}

fn semantic_projection_state_batch(rows: &[Map<String, Value>]) -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("projection", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("source_revision", DataType::Utf8, false),
        Field::new("current_source_revision", DataType::Utf8, false),
        Field::new("projection_revision", DataType::Utf8, false),
        Field::new("staleness", DataType::Utf8, false),
        Field::new("source_object_count", DataType::Int64, false),
        Field::new("source_objects_json", DataType::Utf8, false),
        Field::new("source_path", DataType::Utf8, false),
    ]));

    RecordBatch::try_new(
        schema,
        vec![
            string_array(rows, "projection", SEMANTIC_PROJECTION_STATE_TABLE)?,
            string_array(rows, "status", SEMANTIC_PROJECTION_STATE_TABLE)?,
            string_array(rows, "source_revision", SEMANTIC_PROJECTION_STATE_TABLE)?,
            string_array(
                rows,
                "current_source_revision",
                SEMANTIC_PROJECTION_STATE_TABLE,
            )?,
            string_array(rows, "projection_revision", SEMANTIC_PROJECTION_STATE_TABLE)?,
            string_array(rows, "staleness", SEMANTIC_PROJECTION_STATE_TABLE)?,
            int64_array(rows, "source_object_count", SEMANTIC_PROJECTION_STATE_TABLE)?,
            string_array(rows, "source_objects_json", SEMANTIC_PROJECTION_STATE_TABLE)?,
            string_array(rows, "source_path", SEMANTIC_PROJECTION_STATE_TABLE)?,
        ],
    )
    .map_err(|error| format!("build `{SEMANTIC_PROJECTION_STATE_TABLE}` request batch: {error}"))
}

fn string_array(
    rows: &[Map<String, Value>],
    field_name: &str,
    table_name: &str,
) -> Result<ArrayRef, String> {
    let values = rows
        .iter()
        .map(|row| json_string_field(row, field_name, table_name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arc::new(StringArray::from(values)) as ArrayRef)
}

fn int64_array(
    rows: &[Map<String, Value>],
    field_name: &str,
    table_name: &str,
) -> Result<ArrayRef, String> {
    let values = rows
        .iter()
        .map(|row| json_i64_field(row, field_name, table_name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arc::new(Int64Array::from(values)) as ArrayRef)
}

fn float64_array(
    rows: &[Map<String, Value>],
    field_name: &str,
    table_name: &str,
) -> Result<ArrayRef, String> {
    let values = rows
        .iter()
        .map(|row| json_f64_field(row, field_name, table_name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arc::new(Float64Array::from(values)) as ArrayRef)
}

fn json_string_field(
    row: &Map<String, Value>,
    field_name: &str,
    table_name: &str,
) -> Result<String, String> {
    row.get(field_name)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            format!(
                "dataset ontology `{table_name}` semantic read-model row is missing string field `{field_name}`"
            )
        })
}

fn json_i64_field(
    row: &Map<String, Value>,
    field_name: &str,
    table_name: &str,
) -> Result<i64, String> {
    row.get(field_name)
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            format!(
                "dataset ontology `{table_name}` semantic read-model row is missing int64 field `{field_name}`"
            )
        })
}

fn json_f64_field(
    row: &Map<String, Value>,
    field_name: &str,
    table_name: &str,
) -> Result<f64, String> {
    row.get(field_name)
        .and_then(Value::as_f64)
        .ok_or_else(|| {
            format!(
                "dataset ontology `{table_name}` semantic read-model row is missing float64 field `{field_name}`"
            )
        })
}
