use arrow::array::{Array, BooleanArray, Float32Array, Float64Array, Int32Array, Int64Array};
use arrow::datatypes::DataType;
use arrow::util::display::array_value_to_string;
use serde_json::{Map, Number, Value};
use xiuxian_db_store::EngineRecordBatch;
pub use xiuxian_wendao_core::sql_query::{
    SqlBatchPayload, SqlColumnPayload, SqlQueryMetadata, SqlQueryPayload,
};

#[cfg(feature = "search-runtime")]
#[derive(Debug)]
pub(crate) struct SqlQueryResult {
    metadata: SqlQueryMetadata,
    batches: Vec<EngineRecordBatch>,
}

#[cfg(feature = "search-runtime")]
impl SqlQueryResult {
    pub(crate) fn new(metadata: SqlQueryMetadata, batches: Vec<EngineRecordBatch>) -> Self {
        Self { metadata, batches }
    }

    pub(crate) fn into_parts(self) -> (SqlQueryMetadata, Vec<EngineRecordBatch>) {
        (self.metadata, self.batches)
    }

    pub(crate) fn payload(&self) -> Result<SqlQueryPayload, String> {
        sql_query_payload_from_engine_batches(self.metadata.clone(), &self.batches)
    }
}

pub(crate) fn sql_query_payload_from_engine_batches(
    metadata: SqlQueryMetadata,
    batches: &[EngineRecordBatch],
) -> Result<SqlQueryPayload, String> {
    let batches = batches
        .iter()
        .map(batch_payload)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SqlQueryPayload { metadata, batches })
}

#[cfg(feature = "search-runtime")]
pub(crate) fn engine_batches_rows_payload(
    batches: &[EngineRecordBatch],
) -> Result<Vec<Map<String, Value>>, String> {
    batches.iter().try_fold(Vec::new(), |mut rows, batch| {
        rows.extend(engine_batch_rows_payload(batch)?);
        Ok(rows)
    })
}

pub(crate) fn engine_batch_rows_payload(
    batch: &EngineRecordBatch,
) -> Result<Vec<Map<String, Value>>, String> {
    let schema = batch.schema();
    (0..batch.num_rows())
        .map(|row_index| {
            let mut row = Map::new();
            for field in schema.fields() {
                let Some(column) = batch.column_by_name(field.name()) else {
                    return Err(format!(
                        "shared SQL query payload is missing result column `{}`",
                        field.name()
                    ));
                };
                row.insert(
                    field.name().clone(),
                    column_json_value(column.as_ref(), row_index),
                );
            }
            Ok(row)
        })
        .collect::<Result<Vec<_>, _>>()
}

fn batch_payload(batch: &EngineRecordBatch) -> Result<SqlBatchPayload, String> {
    let schema = batch.schema();
    let columns = schema
        .fields()
        .iter()
        .map(|field| SqlColumnPayload {
            name: field.name().clone(),
            data_type: field.data_type().to_string(),
            nullable: field.is_nullable(),
        })
        .collect::<Vec<_>>();
    let rows = engine_batch_rows_payload(batch)?;

    Ok(SqlBatchPayload {
        row_count: batch.num_rows(),
        columns,
        rows,
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
                .unwrap_or_else(|| panic!("uint64 query payload decode"))
                .value(index),
        )),
        DataType::UInt32 => Value::Number(Number::from(
            column
                .as_any()
                .downcast_ref::<arrow::array::UInt32Array>()
                .unwrap_or_else(|| panic!("uint32 query payload decode"))
                .value(index),
        )),
        DataType::Int64 => Value::Number(Number::from(
            column
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap_or_else(|| panic!("int64 query payload decode"))
                .value(index),
        )),
        DataType::Int32 => Value::Number(Number::from(
            column
                .as_any()
                .downcast_ref::<Int32Array>()
                .unwrap_or_else(|| panic!("int32 query payload decode"))
                .value(index),
        )),
        DataType::Float64 => Number::from_f64(
            column
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap_or_else(|| panic!("float64 query payload decode"))
                .value(index),
        )
        .map_or_else(|| fallback_column_json_value(column, index), Value::Number),
        DataType::Float32 => Number::from_f64(f64::from(
            column
                .as_any()
                .downcast_ref::<Float32Array>()
                .unwrap_or_else(|| panic!("float32 query payload decode"))
                .value(index),
        ))
        .map_or_else(|| fallback_column_json_value(column, index), Value::Number),
        _ => fallback_column_json_value(column, index),
    }
}

fn fallback_column_json_value(column: &dyn Array, index: usize) -> Value {
    Value::String(
        array_value_to_string(column, index)
            .unwrap_or_else(|error| panic!("query payload value decode failed: {error}")),
    )
}
