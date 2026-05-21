use arrow::array::{Array, Float64Array, ListBuilder, StringArray, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use xiuxian_types::VectorSearchResult;

/// Allowed column names for IPC projection (vector search result batch).
const IPC_VECTOR_COLUMNS: &[&str] = &[
    "id",
    "content",
    "tool_name",
    "file_path",
    "routing_keywords",
    "intents",
    "_distance",
    "metadata",
];

struct VectorIpcData {
    ids: Vec<String>,
    contents: Vec<String>,
    tool_names: Vec<String>,
    file_paths: Vec<String>,
    distances: Vec<f64>,
    metadata_json: Vec<String>,
    routing_keywords_array: std::sync::Arc<dyn Array>,
    intents_array: std::sync::Arc<dyn Array>,
}

fn collect_vector_ipc_data(results: &[VectorSearchResult]) -> VectorIpcData {
    let ids = results.iter().map(|r| r.id.clone()).collect();
    let contents = results.iter().map(|r| r.content.clone()).collect();
    let tool_names = results.iter().map(|r| r.tool_name.clone()).collect();
    let file_paths = results.iter().map(|r| r.file_path.clone()).collect();
    let distances = results.iter().map(|r| r.distance).collect();
    let metadata_json = results
        .iter()
        .map(|r| serde_json::to_string(&r.metadata).unwrap_or_else(|_| "null".to_string()))
        .collect();

    let routing_keywords_array =
        vector_result_list_array(results, |result| result.routing_keywords.split_whitespace());
    let intents_array =
        vector_result_list_array(results, |result| result.intents.split(" | ").map(str::trim));

    VectorIpcData {
        ids,
        contents,
        tool_names,
        file_paths,
        distances,
        metadata_json,
        routing_keywords_array,
        intents_array,
    }
}

fn vector_result_list_array<'a, F, I>(
    results: &'a [VectorSearchResult],
    tokens: F,
) -> std::sync::Arc<dyn Array>
where
    F: Fn(&'a VectorSearchResult) -> I,
    I: Iterator<Item = &'a str>,
{
    let mut builder = ListBuilder::new(StringBuilder::new());
    for result in results {
        append_vector_result_tokens(&mut builder, tokens(result));
    }
    std::sync::Arc::new(builder.finish())
}

fn append_vector_result_tokens<'a, I>(builder: &mut ListBuilder<StringBuilder>, tokens: I)
where
    I: Iterator<Item = &'a str>,
{
    for token in tokens.filter(|token| !token.is_empty()) {
        builder.values().append_value(token);
    }
    builder.append(true);
}

fn resolve_vector_ipc_projection(projection: Option<&[String]>) -> Result<Vec<&str>, String> {
    match projection {
        Some(columns) if !columns.is_empty() => {
            for name in columns {
                if !IPC_VECTOR_COLUMNS.contains(&name.as_str()) {
                    return Err(format!("invalid ipc_projection column: {name}"));
                }
            }
            Ok(columns.iter().map(String::as_str).collect())
        }
        _ => Ok(IPC_VECTOR_COLUMNS.to_vec()),
    }
}

fn append_vector_ipc_column(
    col: &str,
    data: &VectorIpcData,
    schema_fields: &mut Vec<Field>,
    arrays: &mut Vec<std::sync::Arc<dyn Array>>,
) {
    match col {
        "id" => {
            schema_fields.push(Field::new("id", DataType::Utf8, true));
            arrays.push(std::sync::Arc::new(StringArray::from(data.ids.clone())));
        }
        "content" => {
            schema_fields.push(Field::new("content", DataType::Utf8, true));
            arrays.push(std::sync::Arc::new(StringArray::from(
                data.contents.clone(),
            )));
        }
        "tool_name" => {
            schema_fields.push(Field::new("tool_name", DataType::Utf8, true));
            arrays.push(std::sync::Arc::new(StringArray::from(
                data.tool_names.clone(),
            )));
        }
        "file_path" => {
            schema_fields.push(Field::new("file_path", DataType::Utf8, true));
            arrays.push(std::sync::Arc::new(StringArray::from(
                data.file_paths.clone(),
            )));
        }
        "routing_keywords" => {
            schema_fields.push(Field::new(
                "routing_keywords",
                DataType::List(std::sync::Arc::new(Field::new(
                    "item",
                    DataType::Utf8,
                    true,
                ))),
                true,
            ));
            arrays.push(data.routing_keywords_array.clone());
        }
        "intents" => {
            schema_fields.push(Field::new(
                "intents",
                DataType::List(std::sync::Arc::new(Field::new(
                    "item",
                    DataType::Utf8,
                    true,
                ))),
                true,
            ));
            arrays.push(data.intents_array.clone());
        }
        "_distance" => {
            schema_fields.push(Field::new("_distance", DataType::Float64, true));
            arrays.push(std::sync::Arc::new(Float64Array::from(
                data.distances.clone(),
            )));
        }
        "metadata" => {
            schema_fields.push(Field::new("metadata", DataType::Utf8, true));
            arrays.push(std::sync::Arc::new(StringArray::from(
                data.metadata_json.clone(),
            )));
        }
        _ => {}
    }
}

fn record_batch_to_ipc_bytes(batch: &arrow::record_batch::RecordBatch) -> Result<Vec<u8>, String> {
    crate::arrow_codec::encode_record_batch_ipc(batch).map_err(|error| error.to_string())
}

/// Encode search results as Arrow IPC stream bytes (single `RecordBatch`).
/// If `projection` is Some and non-empty, only those columns are included (smaller payload).
/// Schema (full): id, content, `tool_name`, `file_path`, `routing_keywords` (List<Utf8>),
/// intents (List<Utf8>), _distance, metadata (Utf8).
pub(crate) fn search_results_to_ipc(
    results: &[VectorSearchResult],
    projection: Option<&[String]>,
) -> Result<Vec<u8>, String> {
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;

    let cols = resolve_vector_ipc_projection(projection)?;
    let data = collect_vector_ipc_data(results);

    let mut schema_fields = Vec::with_capacity(cols.len());
    let mut arrays: Vec<Arc<dyn Array>> = Vec::with_capacity(cols.len());
    for col in &cols {
        append_vector_ipc_column(col, &data, &mut schema_fields, &mut arrays);
    }

    let schema = Schema::new(schema_fields);
    let batch = RecordBatch::try_new(Arc::new(schema), arrays).map_err(|e| e.to_string())?;
    record_batch_to_ipc_bytes(&batch)
}
