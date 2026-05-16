//! Audio transcript resource row construction.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_attachments::audio::AudioShardMergeReport;

use crate::studio::router::handlers::analysis::document_extract::arrow_cache::build_audio_transcript_resource_batch;

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
