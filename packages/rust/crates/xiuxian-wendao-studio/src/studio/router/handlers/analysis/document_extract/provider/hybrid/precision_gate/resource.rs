use arrow::array::{Array, StringArray};
use arrow::record_batch::RecordBatch as EngineRecordBatch;

pub(super) fn validate_resource_rows(
    resource_batch: &EngineRecordBatch,
    expected_ocr_rows: usize,
) -> Result<(), String> {
    if resource_batch.num_rows() == 0 {
        return Err("hybrid precision gate rejected an empty resource batch".to_string());
    }
    if resource_batch.num_rows() < expected_ocr_rows {
        return Err(format!(
            "hybrid resource batch has {} rows for {expected_ocr_rows} OCR results",
            resource_batch.num_rows()
        ));
    }
    let resource_type = string_column(resource_batch, "resourceType")?;
    let status = string_column(resource_batch, "status")?;
    for row in 0..resource_batch.num_rows() {
        let resource_type = string_value(resource_type, row).to_ascii_lowercase();
        let status = string_value(status, row).to_ascii_lowercase();
        if resource_type.contains("error")
            || resource_type.contains("skipped")
            || matches!(status.as_str(), "error" | "failed" | "skipped")
        {
            return Err(format!(
                "hybrid precision gate rejected resource row {row} with type `{resource_type}` and status `{status}`"
            ));
        }
    }
    Ok(())
}

fn string_column<'a>(batch: &'a EngineRecordBatch, name: &str) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| format!("hybrid resource `{name}` column is not utf8"))
}

fn string_value(column: &StringArray, row: usize) -> String {
    if column.is_null(row) {
        String::new()
    } else {
        column.value(row).to_string()
    }
}
