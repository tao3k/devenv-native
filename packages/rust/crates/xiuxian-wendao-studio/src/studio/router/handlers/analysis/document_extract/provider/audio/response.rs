//! Audio transcript resource row construction.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_attachments::audio::{
    AudioShardInput, AudioShardMergeReport, AudioShardResult, AudioTranscriptOrgLedgerOptions,
    build_audio_transcript_org_ledger,
};

#[cfg(test)]
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::build_audio_transcript_resource_batch;
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::build_audio_transcript_with_org_resource_batch;

#[cfg(test)]
pub(crate) fn build_audio_transcript_batch(
    source_path: &str,
    output_dir: &str,
    merge_report: &AudioShardMergeReport,
) -> Result<RecordBatch, String> {
    if !merge_report.has_complete_success_coverage() {
        return Err(format!(
            "audio transcript merge failed precision gate: failed={}, skipped={}, missing={}, duplicate={}",
            merge_report.failed_count,
            merge_report.skipped_count,
            merge_report.missing_shard_element_ids.len(),
            merge_report.duplicate_shard_element_ids.len(),
        ));
    }
    let transcript_text = if merge_report.timeline_text.trim().is_empty() {
        merge_report.text.as_str()
    } else {
        merge_report.timeline_text.as_str()
    };
    if transcript_text.trim().is_empty() {
        return Err("audio transcript merge produced empty text".to_owned());
    }
    build_audio_transcript_resource_batch(
        source_path,
        output_dir,
        transcript_text,
        "_audio_transcript",
    )
}

pub(crate) fn build_audio_transcript_with_org_batch(
    source_path: &str,
    output_dir: &str,
    merge_report: &AudioShardMergeReport,
    inputs: &[AudioShardInput],
    final_results: &[AudioShardResult],
) -> Result<RecordBatch, String> {
    if !merge_report.has_complete_success_coverage() {
        return Err(format!(
            "audio transcript merge failed precision gate: failed={}, skipped={}, missing={}, duplicate={}",
            merge_report.failed_count,
            merge_report.skipped_count,
            merge_report.missing_shard_element_ids.len(),
            merge_report.duplicate_shard_element_ids.len(),
        ));
    }
    let transcript_text = transcript_text_from_merge_report(merge_report)?;
    let org_ledger = build_audio_transcript_org_ledger(
        inputs,
        final_results,
        &AudioTranscriptOrgLedgerOptions::default(),
    )?;
    build_audio_transcript_with_org_resource_batch(
        source_path,
        output_dir,
        transcript_text,
        org_ledger.as_str(),
    )
}

fn transcript_text_from_merge_report(merge_report: &AudioShardMergeReport) -> Result<&str, String> {
    let transcript_text = if merge_report.timeline_text.trim().is_empty() {
        merge_report.text.as_str()
    } else {
        merge_report.timeline_text.as_str()
    };
    if transcript_text.trim().is_empty() {
        return Err("audio transcript merge produced empty text".to_owned());
    }
    Ok(transcript_text)
}
