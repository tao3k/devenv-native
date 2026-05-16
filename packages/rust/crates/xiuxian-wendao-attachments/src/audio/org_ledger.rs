//! Canonical Org transcript ledger construction for audio shard evidence.

use std::{collections::HashMap, fmt::Write as _, path::Path};

use super::{
    merge_audio_shard_results,
    types::{AudioShardInput, AudioShardResult},
};

/// Stable schema marker for generated audio transcript Org ledgers.
pub const AUDIO_TRANSCRIPT_ORG_LEDGER_SCHEMA: &str =
    "xiuxian_wendao.audio_transcript_org_ledger.v1";

/// Options for building a canonical audio transcript Org ledger.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AudioTranscriptOrgLedgerOptions {
    /// Org document title.
    pub title: String,
    /// Attachment directory relative to the ledger file.
    pub attachment_dir: String,
}

impl Default for AudioTranscriptOrgLedgerOptions {
    fn default() -> Self {
        Self {
            title: "Audio Transcript Ledger".to_owned(),
            attachment_dir: "audio_shards".to_owned(),
        }
    }
}

impl AudioTranscriptOrgLedgerOptions {
    /// Create Org ledger options with explicit title and attachment directory.
    #[must_use]
    pub fn new(title: impl Into<String>, attachment_dir: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            attachment_dir: attachment_dir.into(),
        }
    }
}

/// Build a canonical Org transcript ledger from audio shard input/result rows.
///
/// The generated Org document uses a section-level `DIR` property and standard
/// `attachment:` links, so updated `orgize` can parse and lint the shard
/// evidence without a Wendao-specific attachment syntax.
///
/// # Errors
///
/// Returns an error when audio shard merge validation fails, complete success
/// coverage is missing, the attachment directory is empty, or a successful
/// input row cannot be matched to its result.
pub fn build_audio_transcript_org_ledger(
    inputs: &[AudioShardInput],
    results: &[AudioShardResult],
    options: &AudioTranscriptOrgLedgerOptions,
) -> Result<String, String> {
    let attachment_dir = options.attachment_dir.trim();
    if attachment_dir.is_empty() {
        return Err("audio transcript Org ledger attachment dir must not be empty".to_owned());
    }

    let merge_report = merge_audio_shard_results(inputs, results)?;
    if !merge_report.has_complete_success_coverage() {
        return Err(format!(
            "audio transcript Org ledger requires complete success coverage: failed={}, skipped={}, missing={}, duplicate={}",
            merge_report.failed_count,
            merge_report.skipped_count,
            merge_report.missing_shard_element_ids.len(),
            merge_report.duplicate_shard_element_ids.len(),
        ));
    }
    if merge_report.text.trim().is_empty() {
        return Err("audio transcript Org ledger merge produced empty text".to_owned());
    }

    let indexed_results = index_results(results);
    let mut ordered_inputs = inputs.iter().collect::<Vec<_>>();
    ordered_inputs.sort_by(|left, right| {
        left.reading_order_key
            .cmp(&right.reading_order_key)
            .then_with(|| left.shard_element_id.cmp(&right.shard_element_id))
    });

    let source = ordered_inputs
        .first()
        .ok_or_else(|| "audio transcript Org ledger requires at least one input row".to_owned())?;

    let mut output = String::new();
    push_header(&mut output, options);
    push_root_section(&mut output, source, attachment_dir);
    push_shards_section(
        &mut output,
        ordered_inputs.as_slice(),
        &indexed_results,
        attachment_dir,
    )?;
    Ok(output)
}

fn push_header(output: &mut String, options: &AudioTranscriptOrgLedgerOptions) {
    let _ = writeln!(output, "#+TITLE: {}", sanitize_keyword(&options.title));
    let _ = writeln!(output, "#+OPTIONS: toc:nil");
    output.push('\n');
}

fn push_root_section(output: &mut String, source: &AudioShardInput, attachment_dir: &str) {
    output.push_str("* Transcript Ledger :ATTACH:\n");
    output.push_str(":PROPERTIES:\n");
    push_property(output, "DIR", attachment_dir);
    push_property(output, "WENDAO_SCHEMA", AUDIO_TRANSCRIPT_ORG_LEDGER_SCHEMA);
    push_property(output, "WENDAO_KIND", "audio_transcript_ledger");
    push_property(output, "WENDAO_SOURCE_PATH", &source.source_path);
    push_property(output, "WENDAO_SOURCE_SHA256", &source.source_content_hash);
    push_property(output, "WENDAO_SHARD_PROFILE", &source.shard_profile);
    push_property(output, "WENDAO_TASK_PROFILE", &source.task_profile);
    push_property(output, "WENDAO_BACKEND_PROFILE", &source.backend_profile);
    output.push_str(":END:\n\n");
}

fn push_shards_section(
    output: &mut String,
    ordered_inputs: &[&AudioShardInput],
    indexed_results: &HashMap<&str, &AudioShardResult>,
    attachment_dir: &str,
) -> Result<(), String> {
    output.push_str("** Shard Evidence\n");
    output.push_str(":PROPERTIES:\n");
    push_property(output, "DIR", attachment_dir);
    output.push_str(":END:\n\n");

    for (index, input) in ordered_inputs.iter().enumerate() {
        let result = indexed_results
            .get(input.shard_element_id.as_str())
            .copied()
            .ok_or_else(|| {
                format!(
                    "audio transcript Org ledger missing result for shard {}",
                    input.shard_element_id
                )
            })?;
        push_shard_entry(output, index, input, result)?;
    }
    Ok(())
}

fn push_shard_entry(
    output: &mut String,
    index: usize,
    input: &AudioShardInput,
    result: &AudioShardResult,
) -> Result<(), String> {
    let attachment_name = shard_attachment_name(input)?;
    let transcript = result.text.as_deref().unwrap_or_default().trim();
    if transcript.is_empty() {
        return Err(format!(
            "audio transcript Org ledger shard {} has empty transcript",
            input.shard_element_id
        ));
    }

    let start_ms = input.start_ms;
    let end_ms = input.start_ms.saturating_add(input.duration_ms);
    let source_name = source_basename(&input.source_path);
    let _ = writeln!(
        output,
        "*** {} -- {} {} shard {index:06}",
        format_timestamp_ms(start_ms),
        format_timestamp_ms(end_ms),
        source_name,
    );
    output.push_str(":PROPERTIES:\n");
    push_property(output, "WENDAO_KIND", "audio_transcript_shard");
    push_property(output, "WENDAO_SHARD_ELEMENT_ID", &input.shard_element_id);
    push_property(output, "WENDAO_RESULT_ELEMENT_ID", &result.element_id);
    push_property(output, "SOURCE", &source_name);
    push_property(output, "CHUNK_INDEX", &index.to_string());
    push_property(output, "START_SECONDS", &format_seconds(start_ms));
    push_property(output, "END_SECONDS", &format_seconds(end_ms));
    push_property(output, "WENDAO_SOURCE_SHA256", &input.source_content_hash);
    push_property(output, "WENDAO_SHARD_SHA256", &input.shard_sha256);
    push_property(output, "WENDAO_READING_ORDER_KEY", &input.reading_order_key);
    push_property(output, "WENDAO_START_MS", &input.start_ms.to_string());
    push_property(output, "WENDAO_DURATION_MS", &input.duration_ms.to_string());
    push_property(
        output,
        "WENDAO_MEDIA_START_MS",
        &input.media_start_ms.to_string(),
    );
    push_property(
        output,
        "WENDAO_MEDIA_DURATION_MS",
        &input.media_duration_ms.to_string(),
    );
    push_property(
        output,
        "WENDAO_SAMPLE_RATE_HZ",
        &input.sample_rate_hz.to_string(),
    );
    push_property(output, "WENDAO_CHANNELS", &input.channels.to_string());
    push_property(output, "WENDAO_AUDIO_FORMAT", &input.audio_format);
    if let Some(confidence) = result.confidence {
        push_property(output, "WENDAO_CONFIDENCE", &format!("{confidence:.6}"));
    }
    output.push_str(":END:\n");
    let _ = writeln!(
        output,
        "[[attachment:{}][normalized audio shard]]\n",
        sanitize_attachment_link_path(&attachment_name)
    );
    output.push_str(transcript);
    output.push_str("\n\n");
    Ok(())
}

fn index_results(results: &[AudioShardResult]) -> HashMap<&str, &AudioShardResult> {
    results
        .iter()
        .map(|result| (result.shard_element_id.as_str(), result))
        .collect()
}

fn shard_attachment_name(input: &AudioShardInput) -> Result<String, String> {
    Path::new(input.shard_path.as_str())
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            format!(
                "audio transcript Org ledger shard {} has invalid shard path `{}`",
                input.shard_element_id, input.shard_path
            )
        })
}

fn source_basename(value: &str) -> String {
    Path::new(value)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .filter(|file_name| !file_name.trim().is_empty())
        .unwrap_or(value)
        .to_owned()
}

fn format_timestamp_ms(milliseconds: u64) -> String {
    let hours = milliseconds / 3_600_000;
    let minutes = (milliseconds % 3_600_000) / 60_000;
    let seconds = (milliseconds % 60_000) / 1_000;
    let millis = milliseconds % 1_000;
    format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
}

fn format_seconds(milliseconds: u64) -> String {
    let whole_seconds = milliseconds / 1_000;
    let millis = milliseconds % 1_000;
    format!("{whole_seconds}.{millis:03}")
}

fn push_property(output: &mut String, key: &str, value: &str) {
    let _ = writeln!(output, ":{key}: {}", sanitize_property_value(value));
}

fn sanitize_keyword(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_property_value(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_attachment_link_path(value: &str) -> String {
    value.replace(['[', ']'], "_")
}
