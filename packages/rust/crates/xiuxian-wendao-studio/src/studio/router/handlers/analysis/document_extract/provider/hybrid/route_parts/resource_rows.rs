use super::{
    Arc, Array, ArrayRef, EngineRecordBatch, HybridDocumentResourceBatch, Int32Array,
    PdfOcrShardInput, PdfOcrShardMetric, PdfOcrShardResult, PdfPageRenderShardReport, StringArray,
    build_ocr_result_resource_batch, concat_batches, order_ocr_results_by_inputs,
    validate_hybrid_page_coverage, validate_hybrid_shard_coverage,
    validate_ocr_results_match_inputs, validate_successful_ocr_results_for_inputs,
};

pub(super) fn concat_document_resource_batches(
    batches: &[EngineRecordBatch],
) -> Result<EngineRecordBatch, String> {
    let Some(first) = batches.first() else {
        return Err("Docling page-range fallback returned no resource batches".to_string());
    };
    let batch = concat_batches(&first.schema(), batches)
        .map_err(|error| format!("concatenate Docling page-range resource batches: {error}"))?;
    normalize_docling_page_range_wrapper_rows(batch)
}

pub(super) fn normalize_docling_page_range_wrapper_rows(
    batch: EngineRecordBatch,
) -> Result<EngineRecordBatch, String> {
    let resource_type = resource_string_column(&batch, "resourceType")?;
    let mut document_rows = Vec::new();
    let mut kept_rows = Vec::with_capacity(batch.num_rows());
    let mut kept_docling_json = false;

    for row in 0..batch.num_rows() {
        match nullable_string_value(resource_type, row).as_deref() {
            Some("document") => {
                if document_rows.is_empty() {
                    kept_rows.push(row);
                }
                document_rows.push(row);
            }
            Some("docling_json") => {
                if !kept_docling_json {
                    kept_rows.push(row);
                    kept_docling_json = true;
                }
            }
            _ => kept_rows.push(row),
        }
    }

    if kept_rows.len() == batch.num_rows() && document_rows.len() <= 1 {
        return Ok(batch);
    }

    rebuild_document_resource_batch_with_rows(
        &batch,
        kept_rows.as_slice(),
        document_rows.as_slice(),
    )
}

pub(super) fn rebuild_document_resource_batch_with_rows(
    batch: &EngineRecordBatch,
    kept_rows: &[usize],
    document_rows: &[usize],
) -> Result<EngineRecordBatch, String> {
    let source_path = resource_string_column(batch, "sourcePath")?;
    let resource_type = resource_string_column(batch, "resourceType")?;
    let resource_path = resource_string_column(batch, "resourcePath")?;
    let page_index = resource_i32_column(batch, "pageIndex")?;
    let caption = resource_string_column(batch, "caption")?;
    let content = resource_string_column(batch, "content")?;
    let mime_type = resource_string_column(batch, "mimeType")?;
    let status = resource_string_column(batch, "status")?;
    let element_id = resource_string_column(batch, "elementId")?;

    let merged_document_content = (document_rows.len() > 1).then(|| {
        document_rows
            .iter()
            .filter_map(|row| nullable_string_value(content, *row))
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n\n")
    });

    let mut output_source_path = Vec::with_capacity(kept_rows.len());
    let mut output_resource_type = Vec::with_capacity(kept_rows.len());
    let mut output_resource_path = Vec::with_capacity(kept_rows.len());
    let mut output_page_index = Vec::with_capacity(kept_rows.len());
    let mut output_caption = Vec::with_capacity(kept_rows.len());
    let mut output_content = Vec::with_capacity(kept_rows.len());
    let mut output_mime_type = Vec::with_capacity(kept_rows.len());
    let mut output_status = Vec::with_capacity(kept_rows.len());
    let mut output_element_id = Vec::with_capacity(kept_rows.len());

    for row in kept_rows {
        let is_merged_document = document_rows.first().is_some_and(|first| first == row);
        output_source_path.push(nullable_string_value(source_path, *row));
        output_resource_type.push(nullable_string_value(resource_type, *row));
        output_resource_path.push(nullable_string_value(resource_path, *row));
        output_page_index.push(nullable_i32_value(page_index, *row));
        output_caption.push(nullable_string_value(caption, *row));
        output_content.push(if is_merged_document {
            merged_document_content
                .clone()
                .or_else(|| nullable_string_value(content, *row))
        } else {
            nullable_string_value(content, *row)
        });
        output_mime_type.push(nullable_string_value(mime_type, *row));
        output_status.push(nullable_string_value(status, *row));
        output_element_id.push(nullable_string_value(element_id, *row));
    }

    EngineRecordBatch::try_new(
        batch.schema(),
        vec![
            Arc::new(StringArray::from(output_source_path)) as ArrayRef,
            Arc::new(StringArray::from(output_resource_type)) as ArrayRef,
            Arc::new(StringArray::from(output_resource_path)) as ArrayRef,
            Arc::new(Int32Array::from(output_page_index)) as ArrayRef,
            Arc::new(StringArray::from(output_caption)) as ArrayRef,
            Arc::new(StringArray::from(output_content)) as ArrayRef,
            Arc::new(StringArray::from(output_mime_type)) as ArrayRef,
            Arc::new(StringArray::from(output_status)) as ArrayRef,
            Arc::new(StringArray::from(output_element_id)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("normalize Docling page-range wrapper rows: {error}"))
}

pub(super) fn resource_string_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("Docling resource batch missing `{name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("Docling resource batch `{name}` column is not utf8"))
}

pub(super) fn resource_i32_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<&'a Int32Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("Docling resource batch missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Int32Array>()
        .ok_or_else(|| format!("Docling resource batch `{name}` column is not int32"))
}

pub(super) fn nullable_string_value(column: &StringArray, row: usize) -> Option<String> {
    (!column.is_null(row)).then(|| column.value(row).to_string())
}

pub(super) fn nullable_i32_value(column: &Int32Array, row: usize) -> Option<i32> {
    (!column.is_null(row)).then(|| column.value(row))
}

pub(super) fn materialize_hybrid_page_ocr_resource_batch_from_results(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
    results: Vec<PdfOcrShardResult>,
    scheduler_elapsed_ms: f64,
) -> Result<HybridDocumentResourceBatch, String> {
    let results = order_ocr_results_by_inputs(inputs.as_slice(), results)?;
    validate_successful_ocr_results_for_inputs(
        results.as_slice(),
        render_report.page_count,
        u32::try_from(inputs.len()).unwrap_or(u32::MAX),
        inputs.as_slice(),
    )?;
    validate_ocr_results_match_inputs(inputs.as_slice(), results.as_slice())?;
    let has_region_shards = inputs.iter().any(|input| input.shard_type == "region");
    let resource_batch = build_ocr_result_resource_batch(results.as_slice())?;

    if render_report.shard_count == render_report.page_count && !has_region_shards {
        validate_hybrid_page_coverage(render_report.page_count, &[], results.as_slice())?;
        let metrics = results
            .iter()
            .zip(inputs.iter())
            .map(|(result, input)| {
                PdfOcrShardMetric::from_ocr_result(
                    input,
                    result,
                    render_report.page_count,
                    Some(scheduler_elapsed_ms),
                )
            })
            .collect::<Vec<_>>();
        return Ok(HybridDocumentResourceBatch::new(
            resource_batch,
            inputs,
            results,
            metrics,
            render_report.page_count,
            Vec::new(),
        ));
    }

    validate_hybrid_shard_coverage(
        render_report.page_count,
        &[],
        inputs.as_slice(),
        results.as_slice(),
    )?;
    let metrics = results
        .iter()
        .zip(inputs.iter())
        .map(|(result, input)| {
            PdfOcrShardMetric::from_ocr_result(
                input,
                result,
                render_report.page_count,
                Some(scheduler_elapsed_ms),
            )
        })
        .collect::<Vec<_>>();
    Ok(HybridDocumentResourceBatch::new(
        resource_batch,
        inputs,
        results,
        metrics,
        render_report.page_count,
        Vec::new(),
    ))
}
