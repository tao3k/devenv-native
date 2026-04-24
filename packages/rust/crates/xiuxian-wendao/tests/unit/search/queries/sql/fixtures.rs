use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{Array, BooleanArray, Float32Array, Float64Array, Int32Array, Int64Array};
use arrow::datatypes::DataType;
use arrow::util::display::array_value_to_string;
use serde_json::{Map, Number, Value, json};
#[cfg(feature = "duckdb")]
use std::fs;
use xiuxian_db_store::{
    ColumnarScanOptions, LanceBooleanArray, LanceRecordBatch, LanceUInt64Array,
    lance_batch_to_engine_batch,
};
use xiuxian_wendao_runtime::transport::SqlFlightRouteResponse;

use crate::analyzers::{
    ExampleRecord, ImportKind, ImportRecord, ModuleRecord, RepoSymbolKind,
    RepositoryAnalysisOutput, SymbolRecord,
};
use crate::gateway::studio::types::{AstSearchHit, ReferenceSearchHit, StudioNavigationTarget};
use crate::repo_index::RepoCodeDocument;
use crate::search::queries::sql::provider::metadata::StudioSqlFlightMetadata;
use crate::search::{
    BeginBuildDecision, SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace,
    SearchPlaneService, reference_occurrence_batches, reference_occurrence_schema,
};
#[cfg(feature = "duckdb")]
use crate::set_link_graph_wendao_config_override;

pub(super) fn fixture_service(temp_dir: &tempfile::TempDir) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:studio_sql_flight"),
        SearchMaintenancePolicy::default(),
    )
}

#[cfg(feature = "duckdb")]
pub(super) fn write_search_duckdb_runtime_override(
    body: &str,
) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(&config_path, body)?;
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());
    Ok(temp)
}

pub(super) fn sample_hit(name: &str, path: &str, line: usize) -> ReferenceSearchHit {
    ReferenceSearchHit {
        name: name.to_string(),
        path: path.to_string(),
        language: "rust".to_string(),
        crate_name: "kernel".to_string(),
        project_name: None,
        root_label: None,
        line,
        column: 5,
        line_text: format!("let _value = {name};"),
        navigation_target: StudioNavigationTarget {
            path: path.to_string(),
            category: "doc".to_string(),
            project_name: None,
            root_label: None,
            line: Some(line),
            line_end: Some(line),
            column: Some(5),
        },
        score: 0.0,
    }
}

pub(super) fn sample_local_symbol_hit(name: &str, path: &str, line_start: usize) -> AstSearchHit {
    AstSearchHit {
        name: name.to_string(),
        signature: format!("fn {name}()"),
        path: path.to_string(),
        language: "rust".to_string(),
        crate_name: "kernel".to_string(),
        project_name: None,
        root_label: None,
        node_kind: None,
        owner_title: None,
        navigation_target: StudioNavigationTarget {
            path: path.to_string(),
            category: "symbol".to_string(),
            project_name: None,
            root_label: None,
            line: Some(line_start),
            line_end: Some(line_start),
            column: Some(1),
        },
        line_start,
        line_end: line_start,
        score: 0.0,
    }
}

pub(super) fn repo_document(
    path: &str,
    contents: &str,
    language: &str,
    modified_unix_ms: u64,
) -> RepoCodeDocument {
    RepoCodeDocument {
        path: path.to_string(),
        language: Some(language.to_string()),
        contents: Arc::<str>::from(contents),
        size_bytes: u64::try_from(contents.len()).unwrap_or(u64::MAX),
        modified_unix_ms,
    }
}

pub(super) async fn publish_reference_hits(
    service: &SearchPlaneService,
    build_id: &str,
    hits: &[ReferenceSearchHit],
) -> u64 {
    let lease = match service.coordinator().begin_build(
        SearchCorpusKind::ReferenceOccurrence,
        build_id,
        SearchCorpusKind::ReferenceOccurrence.schema_version(),
    ) {
        BeginBuildDecision::Started(lease) => lease,
        other => panic!("unexpected begin decision: {other:?}"),
    };
    let store = service
        .open_store(SearchCorpusKind::ReferenceOccurrence)
        .await
        .unwrap_or_else(|error| panic!("open store: {error}"));
    let table_name =
        SearchPlaneService::table_name(SearchCorpusKind::ReferenceOccurrence, lease.epoch);
    store
        .replace_record_batches(
            table_name.as_str(),
            reference_occurrence_schema(),
            reference_occurrence_batches(hits)
                .unwrap_or_else(|error| panic!("reference occurrence batches: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("replace record batches: {error}"));
    store
        .write_vector_store_table_to_parquet_file(
            table_name.as_str(),
            service
                .local_epoch_parquet_path(SearchCorpusKind::ReferenceOccurrence, lease.epoch)
                .as_path(),
            ColumnarScanOptions::default(),
        )
        .await
        .unwrap_or_else(|error| panic!("export parquet: {error}"));
    service
        .coordinator()
        .publish_ready(&lease, hits.len() as u64, 1);
    lease.epoch
}

pub(super) async fn publish_local_symbol_hits(
    service: &SearchPlaneService,
    build_id: &str,
    hits: &[AstSearchHit],
) {
    service
        .publish_local_symbol_hits(build_id, hits)
        .await
        .unwrap_or_else(|error| panic!("publish local symbol hits: {error}"));
}

pub(super) async fn publish_repo_content_chunks(
    service: &SearchPlaneService,
    repo_id: &str,
    documents: &[RepoCodeDocument],
    source_revision: &str,
) {
    service
        .publish_repo_content_chunks_with_revision(repo_id, documents, Some(source_revision))
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks: {error}"));
}

pub(super) async fn publish_repo_entities(
    service: &SearchPlaneService,
    repo_id: &str,
    symbol_name: &str,
    example_summary: &str,
    source_revision: &str,
) {
    let analysis = sample_repo_entity_analysis(repo_id, symbol_name, example_summary);
    let documents = sample_repo_entity_documents(symbol_name, 10);
    service
        .publish_repo_entities_with_revision(repo_id, &analysis, &documents, Some(source_revision))
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
}

pub(super) fn string_column_values(batch: &LanceRecordBatch, column_name: &str) -> Vec<String> {
    let engine_batch = lance_batch_to_engine_batch(batch)
        .unwrap_or_else(|error| panic!("convert batch for `{column_name}`: {error}"));
    let column = engine_batch
        .column_by_name(column_name)
        .unwrap_or_else(|| panic!("missing column `{column_name}`"));
    (0..column.len())
        .map(|index| {
            array_value_to_string(column.as_ref(), index)
                .unwrap_or_else(|error| panic!("string value decode for `{column_name}`: {error}"))
        })
        .collect()
}

pub(super) fn nullable_string_column_values(
    batch: &LanceRecordBatch,
    column_name: &str,
) -> Vec<Option<String>> {
    let engine_batch = lance_batch_to_engine_batch(batch)
        .unwrap_or_else(|error| panic!("convert batch for `{column_name}`: {error}"));
    let column = engine_batch
        .column_by_name(column_name)
        .unwrap_or_else(|| panic!("missing column `{column_name}`"));
    (0..column.len())
        .map(|index| {
            (!column.is_null(index)).then(|| {
                array_value_to_string(column.as_ref(), index).unwrap_or_else(|error| {
                    panic!("nullable string value decode for `{column_name}`: {error}")
                })
            })
        })
        .collect()
}

pub(super) fn bool_column_values(batch: &LanceRecordBatch, column_name: &str) -> Vec<bool> {
    let column = batch
        .column_by_name(column_name)
        .unwrap_or_else(|| panic!("missing column `{column_name}`"));
    if let Some(values) = column.as_any().downcast_ref::<LanceBooleanArray>() {
        return values.iter().map(|value| value.unwrap_or(false)).collect();
    }

    panic!("column `{column_name}` should be boolean");
}

pub(super) fn u64_column_values(batch: &LanceRecordBatch, column_name: &str) -> Vec<u64> {
    let column = batch
        .column_by_name(column_name)
        .unwrap_or_else(|| panic!("missing column `{column_name}`"));
    if let Some(values) = column.as_any().downcast_ref::<LanceUInt64Array>() {
        return values
            .iter()
            .map(std::option::Option::unwrap_or_default)
            .collect();
    }

    panic!("column `{column_name}` should be uint64");
}

pub(super) fn sql_response_snapshot(response: &SqlFlightRouteResponse) -> Value {
    let app_metadata: StudioSqlFlightMetadata = serde_json::from_slice(&response.app_metadata)
        .unwrap_or_else(|error| panic!("decode SQL app metadata for snapshot: {error}"));
    let batches = response
        .batches
        .iter()
        .map(batch_snapshot)
        .collect::<Vec<_>>();

    json!({
        "app_metadata": app_metadata,
        "batches": batches,
    })
}

fn batch_snapshot(batch: &LanceRecordBatch) -> Value {
    let engine_batch = lance_batch_to_engine_batch(batch)
        .unwrap_or_else(|error| panic!("convert SQL batch for snapshot: {error}"));
    let schema = engine_batch.schema();
    let columns = schema
        .fields()
        .iter()
        .map(|field| {
            json!({
                "name": field.name(),
                "data_type": field.data_type().to_string(),
                "nullable": field.is_nullable(),
            })
        })
        .collect::<Vec<_>>();
    let rows = (0..engine_batch.num_rows())
        .map(|row_index| {
            let mut row = Map::new();
            for field in schema.fields() {
                let column = engine_batch
                    .column_by_name(field.name())
                    .unwrap_or_else(|| panic!("missing snapshot column `{}`", field.name()));
                row.insert(
                    field.name().clone(),
                    column_json_value(column.as_ref(), row_index),
                );
            }
            Value::Object(row)
        })
        .collect::<Vec<_>>();

    json!({
        "row_count": engine_batch.num_rows(),
        "columns": columns,
        "rows": rows,
    })
}

fn column_json_value(column: &dyn Array, index: usize) -> Value {
    if column.is_null(index) {
        return Value::Null;
    }

    match column.data_type() {
        DataType::Boolean => column.as_any().downcast_ref::<BooleanArray>().map_or_else(
            || fallback_column_json_value(column, index),
            |values| Value::Bool(values.value(index)),
        ),
        DataType::UInt64 => Value::Number(Number::from(
            column
                .as_any()
                .downcast_ref::<arrow::array::UInt64Array>()
                .unwrap_or_else(|| panic!("uint64 snapshot decode"))
                .value(index),
        )),
        DataType::UInt32 => Value::Number(Number::from(
            column
                .as_any()
                .downcast_ref::<arrow::array::UInt32Array>()
                .unwrap_or_else(|| panic!("uint32 snapshot decode"))
                .value(index),
        )),
        DataType::Int64 => Value::Number(Number::from(
            column
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap_or_else(|| panic!("int64 snapshot decode"))
                .value(index),
        )),
        DataType::Int32 => Value::Number(Number::from(
            column
                .as_any()
                .downcast_ref::<Int32Array>()
                .unwrap_or_else(|| panic!("int32 snapshot decode"))
                .value(index),
        )),
        DataType::Float64 => Number::from_f64(
            column
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap_or_else(|| panic!("float64 snapshot decode"))
                .value(index),
        )
        .map_or_else(|| fallback_column_json_value(column, index), Value::Number),
        DataType::Float32 => Number::from_f64(f64::from(
            column
                .as_any()
                .downcast_ref::<Float32Array>()
                .unwrap_or_else(|| panic!("float32 snapshot decode"))
                .value(index),
        ))
        .map_or_else(|| fallback_column_json_value(column, index), Value::Number),
        _ => fallback_column_json_value(column, index),
    }
}

fn fallback_column_json_value(column: &dyn Array, index: usize) -> Value {
    Value::String(
        array_value_to_string(column, index)
            .unwrap_or_else(|error| panic!("fallback snapshot decode: {error}")),
    )
}

fn sample_repo_entity_analysis(
    repo_id: &str,
    symbol_name: &str,
    example_summary: &str,
) -> RepositoryAnalysisOutput {
    let mut attributes = BTreeMap::new();
    attributes.insert("arity".to_string(), "0".to_string());
    RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: repo_id.to_string(),
            module_id: "module:BaseModelica".to_string(),
            qualified_name: "BaseModelica".to_string(),
            path: "src/BaseModelica.jl".to_string(),
        }],
        symbols: vec![SymbolRecord {
            repo_id: repo_id.to_string(),
            symbol_id: format!("symbol:{symbol_name}"),
            module_id: Some("module:BaseModelica".to_string()),
            name: symbol_name.to_string(),
            qualified_name: format!("BaseModelica.{symbol_name}"),
            kind: RepoSymbolKind::Function,
            path: "src/BaseModelica.jl".to_string(),
            line_start: Some(7),
            line_end: Some(9),
            signature: Some(format!("{symbol_name}()")),
            audit_status: Some("verified".to_string()),
            verification_state: Some("verified".to_string()),
            attributes,
        }],
        examples: vec![ExampleRecord {
            repo_id: repo_id.to_string(),
            example_id: format!("example:{symbol_name}"),
            title: format!("{symbol_name} example"),
            path: "examples/solve.jl".to_string(),
            summary: Some(example_summary.to_string()),
        }],
        imports: vec![ImportRecord {
            repo_id: repo_id.to_string(),
            module_id: "module:BaseModelica".to_string(),
            path: "src/BaseModelica.jl".to_string(),
            import_name: symbol_name.to_string(),
            target_package: "SciMLBase".to_string(),
            source_module: "BaseModelica".to_string(),
            kind: ImportKind::Reexport,
            line_start: None,
            resolved_id: Some(format!("symbol:{symbol_name}")),
            attributes: BTreeMap::new(),
        }],
        ..RepositoryAnalysisOutput::default()
    }
}

fn sample_repo_entity_documents(
    symbol_name: &str,
    source_modified_unix_ms: u64,
) -> Vec<RepoCodeDocument> {
    vec![
        repo_document(
            "src/BaseModelica.jl",
            format!("module BaseModelica\n{symbol_name}() = nothing\nend\n").as_str(),
            "julia",
            source_modified_unix_ms,
        ),
        repo_document(
            "examples/solve.jl",
            "using BaseModelica\nsolve()\n",
            "julia",
            10,
        ),
    ]
}
