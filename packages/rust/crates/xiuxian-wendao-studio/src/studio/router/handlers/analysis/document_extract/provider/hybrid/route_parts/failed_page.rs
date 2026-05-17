use super::{
    HOSTED_VLM_DIRECT_OCR_ENGINE, HybridPdfFailedPageRecoveryMode, PDF_OCR_BACKEND_TEXT_PROFILE,
    PDF_OCR_HOSTED_VLM_DIRECT_PROFILE, PdfOcrShardInput, PdfOcrShardResult,
    PdfOcrShardResultStatus, PdfOcrWorkerScheduler, PdfPageRenderShardReport,
    failed_page_recovery_mode, is_hosted_vlm_direct_profile, materialize_ocr2_recovery_page_images,
    order_ocr_results_by_inputs,
};

pub(super) fn failed_page_recovery_input(input: &PdfOcrShardInput) -> PdfOcrShardInput {
    let mut recovery_input = input.clone();
    recovery_input.ocr_profile = PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string();
    recovery_input.ocr_engine = HOSTED_VLM_DIRECT_OCR_ENGINE.to_string();
    recovery_input
}

pub(super) fn failed_page_recovery_candidates(
    inputs: &[PdfOcrShardInput],
    results: &[PdfOcrShardResult],
) -> Vec<(usize, PdfOcrShardInput)> {
    inputs
        .iter()
        .zip(results.iter())
        .enumerate()
        .filter(|(_, (input, result))| is_failed_page_recovery_candidate(input, result))
        .map(|(index, (input, _result))| (index, failed_page_recovery_input(input)))
        .collect()
}

pub(super) fn is_failed_page_recovery_candidate(
    input: &PdfOcrShardInput,
    result: &PdfOcrShardResult,
) -> bool {
    input.shard_type == "page"
        && !is_hosted_vlm_direct_profile(input.ocr_profile.as_str())
        && input.ocr_profile != PDF_OCR_BACKEND_TEXT_PROFILE
        && (result.status != PdfOcrShardResultStatus::Succeeded
            || result
                .text
                .as_deref()
                .is_none_or(|text| text.trim().is_empty()))
}

pub(super) async fn recover_failed_page_ocr_results(
    render_report: &PdfPageRenderShardReport,
    endpoint_urls: &[String],
    pdf_ocr_scheduler: &PdfOcrWorkerScheduler,
    inputs: &mut [PdfOcrShardInput],
    results: &mut [PdfOcrShardResult],
) -> Result<(), String> {
    if failed_page_recovery_mode() != HybridPdfFailedPageRecoveryMode::HostedVlmPage {
        return Ok(());
    }
    let candidates = failed_page_recovery_candidates(inputs, results);
    if candidates.is_empty() {
        return Ok(());
    }
    let positions = candidates
        .iter()
        .map(|(position, _input)| *position)
        .collect::<Vec<_>>();
    let recovery_inputs = candidates
        .into_iter()
        .map(|(_position, input)| input)
        .collect::<Vec<_>>();
    let recovery_inputs =
        materialize_ocr2_recovery_page_images(render_report, recovery_inputs).await?;
    let response = pdf_ocr_scheduler
        .request_shards_with_endpoints(endpoint_urls, recovery_inputs.as_slice())
        .await?;
    let recovery_results =
        order_ocr_results_by_inputs(recovery_inputs.as_slice(), response.results)?;
    for ((position, recovery_input), recovery_result) in positions
        .into_iter()
        .zip(recovery_inputs)
        .zip(recovery_results)
    {
        inputs[position] = recovery_input;
        results[position] = recovery_result;
    }
    Ok(())
}
