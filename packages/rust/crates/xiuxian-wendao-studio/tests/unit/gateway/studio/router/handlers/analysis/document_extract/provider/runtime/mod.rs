use super::{
    Arc, DocumentExtractJobRegistry, Duration, StudioDocumentExtractFlightRouteProvider,
    document_extract_batches_are_cacheable, fs, read_arrow_file,
    shared_document_extract_provider_runtime, sleep, test_document_resource_batch,
    write_arrow_file,
};

mod cache;
mod flight_support;
mod native_org;
mod native_org_flight;
mod queue;
mod snapshot;

fn collect_document_extract_string_values(
    batches: &[arrow::record_batch::RecordBatch],
    name: &str,
) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    for batch in batches {
        let column = batch
            .column_by_name(name)
            .ok_or_else(|| format!("missing {name} column"))?
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .ok_or_else(|| format!("{name} column is not Utf8"))?;
        values.extend((0..batch.num_rows()).map(|row| column.value(row).to_string()));
    }
    Ok(values)
}
