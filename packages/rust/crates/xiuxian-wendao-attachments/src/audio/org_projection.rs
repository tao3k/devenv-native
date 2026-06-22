//! Evidence projection for generated audio transcript Org ledgers.

use serde::{Deserialize, Serialize};

use super::{identity::sha256_hex, org_ledger::AUDIO_TRANSCRIPT_ORG_LEDGER_SCHEMA};

#[cfg(feature = "audio-shard-arrow")]
use std::{collections::HashMap, sync::Arc};

#[cfg(feature = "audio-shard-arrow")]
use arrow::{
    array::{ArrayRef, Float64Array, Int32Array, Int64Array, StringArray},
    record_batch::RecordBatch,
};

#[cfg(feature = "audio-shard-arrow")]
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    validate_record_batch_schema_with_options,
};

/// Stable schema marker for audio Org evidence source rows.
pub const AUDIO_ORG_EVIDENCE_SOURCE_SCHEMA_VERSION: &str =
    "xiuxian_wendao.audio_org_evidence_source.v1";
/// Stable schema marker for audio Org evidence segment rows.
pub const AUDIO_ORG_EVIDENCE_SEGMENT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.audio_org_evidence_segment.v1";

#[cfg(feature = "audio-shard-arrow")]
const AUDIO_ORG_EVIDENCE_SOURCE_TABLE: &str = "audio_org_evidence_source";
#[cfg(feature = "audio-shard-arrow")]
const AUDIO_ORG_EVIDENCE_SEGMENT_TABLE: &str = "audio_org_evidence_segment";

/// Typed ledger kind emitted by generated audio transcript Org ledgers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioOrgEvidenceLedgerKind {
    /// Generated audio transcript ledger.
    AudioTranscriptLedger,
}

impl AudioOrgEvidenceLedgerKind {
    /// Stable string used in Arrow and JSON compatibility surfaces.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AudioTranscriptLedger => "audio_transcript_ledger",
        }
    }
}

impl PartialEq<&str> for AudioOrgEvidenceLedgerKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Projection rows compiled from one generated audio transcript Org ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioOrgEvidenceProjection {
    /// One source row describing the ledger and source audio identity.
    pub source: AudioOrgEvidenceSource,
    /// Ordered evidence segments compiled from shard headings.
    pub segments: Vec<AudioOrgEvidenceSegment>,
}

/// Source-level evidence row for one audio transcript Org ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioOrgEvidenceSource {
    /// Contract version for source rows.
    pub contract_version: String,
    /// Stable source row id derived from source identity and ledger hash.
    pub evidence_source_id: String,
    /// Ledger schema found in the root Org property drawer.
    pub ledger_schema: String,
    /// Ledger kind found in the root Org property drawer.
    pub ledger_kind: AudioOrgEvidenceLedgerKind,
    /// Source path recorded by the generated ledger.
    pub source_path: String,
    /// SHA-256 of the original source bytes, copied from the ledger.
    pub source_sha256: String,
    /// Audio shard profile recorded by the ledger.
    pub shard_profile: String,
    /// Logical task profile recorded by the ledger.
    pub task_profile: String,
    /// Backend profile recorded by the ledger.
    pub backend_profile: String,
    /// SHA-256 of the complete Org ledger text.
    pub ledger_sha256: String,
    /// Number of segment rows projected from the ledger.
    pub segment_count: u32,
}

/// Segment-level evidence row for one transcript shard heading.
///
/// Raw DTO boundary: this row mirrors the audio Org ledger projection contract.
/// Primitive ids and offsets are preserved for Arrow compatibility and
/// validated before downstream read-model materialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioOrgEvidenceSegment {
    /// Contract version for segment rows.
    pub contract_version: String,
    /// Parent source row id.
    pub evidence_source_id: String,
    /// Stable segment id derived from source id and shard element id.
    pub evidence_segment_id: String,
    /// Shard element id recorded by the generated ledger.
    pub shard_element_id: String,
    /// Result element id recorded by the generated ledger.
    pub result_element_id: String,
    /// Source display name recorded by the generated ledger.
    pub source_name: String,
    /// Zero-based chunk index in reading order.
    pub chunk_index: u32,
    /// Logical segment start offset in milliseconds.
    pub start_ms: u64,
    /// Logical segment duration in milliseconds.
    pub duration_ms: u64,
    /// Logical segment end offset in milliseconds.
    pub end_ms: u64,
    /// Original source SHA-256 copied from the segment property drawer.
    pub source_sha256: String,
    /// Materialized shard SHA-256 copied from the segment property drawer.
    pub shard_sha256: String,
    /// Stable reading order key copied from the segment property drawer.
    pub reading_order_key: String,
    /// Media start offset after applying context.
    pub media_start_ms: u64,
    /// Media duration after applying context.
    pub media_duration_ms: u64,
    /// Materialized shard sample rate.
    pub sample_rate_hz: u32,
    /// Materialized shard channel count.
    pub channels: u8,
    /// Materialized shard audio format.
    pub audio_format: String,
    /// Optional model confidence copied from the result row.
    pub confidence: Option<f64>,
    /// SHA-256 of the transcript text.
    pub transcript_sha256: String,
    /// Transcript text from the Org body. This remains evidence, not RDF truth.
    pub transcript_text: String,
}

/// Compile a generated audio transcript Org ledger into evidence rows.
///
/// # Errors
///
/// Returns an error when the Org ledger does not match the generated audio
/// transcript ledger subset, required source or shard properties are missing,
/// timestamps are invalid, transcript text is empty, or shard rows are not in
/// deterministic reading order.
pub fn project_audio_transcript_org_evidence(
    org: &str,
) -> Result<AudioOrgEvidenceProjection, String> {
    let ledger_sha256 = sha256_hex(org.as_bytes());
    let entries = parse_org_entries(org)?;
    let source = project_source(&entries, ledger_sha256.as_str())?;
    let segments = project_segments(
        &entries,
        source.evidence_source_id.as_str(),
        source.source_sha256.as_str(),
    )?;
    Ok(AudioOrgEvidenceProjection { source, segments })
}

fn project_source(
    entries: &[OrgEntry],
    ledger_sha256: &str,
) -> Result<AudioOrgEvidenceSource, String> {
    let root = audio_ledger_root(entries)?;
    let ledger_schema = validated_ledger_schema(root)?;
    let source_path = required_property(root, "WENDAO_SOURCE_PATH")?;
    let source_sha256 = required_property(root, "WENDAO_SOURCE_SHA256")?;
    let shard_profile = required_property(root, "WENDAO_SHARD_PROFILE")?;
    let task_profile = required_property(root, "WENDAO_TASK_PROFILE")?;
    let backend_profile = required_property(root, "WENDAO_BACKEND_PROFILE")?;
    let evidence_source_id =
        evidence_source_id(source_path.as_str(), source_sha256.as_str(), ledger_sha256);
    let segment_count = u32::try_from(audio_shard_entries(entries).count())
        .map_err(|error| format!("audio Org evidence segment count exceeds u32: {error}"))?;
    Ok(AudioOrgEvidenceSource {
        contract_version: AUDIO_ORG_EVIDENCE_SOURCE_SCHEMA_VERSION.to_owned(),
        evidence_source_id,
        ledger_schema,
        ledger_kind: AudioOrgEvidenceLedgerKind::AudioTranscriptLedger,
        source_path,
        source_sha256,
        shard_profile,
        task_profile,
        backend_profile,
        ledger_sha256: ledger_sha256.to_owned(),
        segment_count,
    })
}

fn project_segments(
    entries: &[OrgEntry],
    evidence_source_id: &str,
    source_sha256: &str,
) -> Result<Vec<AudioOrgEvidenceSegment>, String> {
    let mut segments = audio_shard_entries(entries)
        .map(|entry| project_segment(entry, evidence_source_id, source_sha256))
        .collect::<Result<Vec<_>, _>>()?;
    require_projected_segments(&segments)?;
    sort_projected_segments(&mut segments);
    validate_segment_order(&segments)?;
    Ok(segments)
}

fn audio_ledger_root(entries: &[OrgEntry]) -> Result<&OrgEntry, String> {
    entries
        .iter()
        .find(|entry| {
            entry.properties().get("WENDAO_KIND").copied() == Some("audio_transcript_ledger")
        })
        .ok_or_else(|| {
            "audio Org evidence projection requires an audio_transcript_ledger root".to_owned()
        })
}

fn validated_ledger_schema(root: &OrgEntry) -> Result<String, String> {
    let ledger_schema = required_property(root, "WENDAO_SCHEMA")?;
    if ledger_schema == AUDIO_TRANSCRIPT_ORG_LEDGER_SCHEMA {
        Ok(ledger_schema)
    } else {
        Err(format!(
            "audio Org evidence projection expected ledger schema `{AUDIO_TRANSCRIPT_ORG_LEDGER_SCHEMA}`, got `{ledger_schema}`"
        ))
    }
}

fn evidence_source_id(source_path: &str, source_sha256: &str, ledger_sha256: &str) -> String {
    format!(
        "audio-org-source:{}",
        sha256_hex(format!("{source_path}:{source_sha256}:{ledger_sha256}").as_bytes())
    )
}

fn audio_shard_entries(entries: &[OrgEntry]) -> impl Iterator<Item = &OrgEntry> {
    entries.iter().filter(|entry| {
        entry.properties().get("WENDAO_KIND").copied() == Some("audio_transcript_shard")
    })
}

fn require_projected_segments(segments: &[AudioOrgEvidenceSegment]) -> Result<(), String> {
    if segments.is_empty() {
        Err("audio Org evidence projection requires at least one shard segment".to_owned())
    } else {
        Ok(())
    }
}

fn sort_projected_segments(segments: &mut [AudioOrgEvidenceSegment]) {
    segments.sort_by(|left, right| {
        left.reading_order_key
            .cmp(&right.reading_order_key)
            .then_with(|| left.shard_element_id.cmp(&right.shard_element_id))
    });
}

#[cfg(feature = "audio-shard-arrow")]
/// Build an Arrow batch for audio Org evidence source rows.
///
/// # Errors
///
/// Returns an error if Arrow cannot build the source row batch.
pub fn build_audio_org_evidence_source_batch(
    sources: &[AudioOrgEvidenceSource],
) -> Result<RecordBatch, String> {
    record_batch(
        &audio_org_evidence_source_contract(),
        vec![
            source_string_column(sources, |row| row.contract_version.clone()),
            source_string_column(sources, |row| row.evidence_source_id.clone()),
            source_string_column(sources, |row| row.ledger_schema.clone()),
            source_string_column(sources, |row| row.ledger_kind.as_str().to_owned()),
            source_string_column(sources, |row| row.source_path.clone()),
            source_string_column(sources, |row| row.source_sha256.clone()),
            source_string_column(sources, |row| row.shard_profile.clone()),
            source_string_column(sources, |row| row.task_profile.clone()),
            source_string_column(sources, |row| row.backend_profile.clone()),
            source_string_column(sources, |row| row.ledger_sha256.clone()),
            source_i64_column(sources, |row| i64::from(row.segment_count)),
        ],
        "build audio Org evidence source Arrow batch",
    )
}

#[cfg(feature = "audio-shard-arrow")]
/// Build an Arrow batch for audio Org evidence segment rows.
///
/// # Errors
///
/// Returns an error if Arrow cannot build the segment row batch.
pub fn build_audio_org_evidence_segment_batch(
    segments: &[AudioOrgEvidenceSegment],
) -> Result<RecordBatch, String> {
    record_batch(
        &audio_org_evidence_segment_contract(),
        vec![
            segment_string_column(segments, |row| row.contract_version.clone()),
            segment_string_column(segments, |row| row.evidence_source_id.clone()),
            segment_string_column(segments, |row| row.evidence_segment_id.clone()),
            segment_string_column(segments, |row| row.shard_element_id.clone()),
            segment_string_column(segments, |row| row.result_element_id.clone()),
            segment_string_column(segments, |row| row.source_name.clone()),
            segment_i64_column(segments, |row| i64::from(row.chunk_index)),
            segment_u64_column(segments, |row| row.start_ms)?,
            segment_u64_column(segments, |row| row.duration_ms)?,
            segment_u64_column(segments, |row| row.end_ms)?,
            segment_string_column(segments, |row| row.source_sha256.clone()),
            segment_string_column(segments, |row| row.shard_sha256.clone()),
            segment_string_column(segments, |row| row.reading_order_key.clone()),
            segment_u64_column(segments, |row| row.media_start_ms)?,
            segment_u64_column(segments, |row| row.media_duration_ms)?,
            segment_i64_column(segments, |row| i64::from(row.sample_rate_hz)),
            Arc::new(Int32Array::from(
                segments
                    .iter()
                    .map(|row| i32::from(row.channels))
                    .collect::<Vec<_>>(),
            )),
            segment_string_column(segments, |row| row.audio_format.clone()),
            Arc::new(Float64Array::from(
                segments
                    .iter()
                    .map(|row| row.confidence)
                    .collect::<Vec<_>>(),
            )),
            segment_string_column(segments, |row| row.transcript_sha256.clone()),
            segment_string_column(segments, |row| row.transcript_text.clone()),
        ],
        "build audio Org evidence segment Arrow batch",
    )
}

#[derive(Debug)]
struct OrgEntry {
    properties: Vec<(String, String)>,
    body: String,
}

impl OrgEntry {
    fn properties(&self) -> std::collections::HashMap<&str, &str> {
        self.properties
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect()
    }
}

fn project_segment(
    entry: &OrgEntry,
    evidence_source_id: &str,
    root_source_sha256: &str,
) -> Result<AudioOrgEvidenceSegment, String> {
    let properties = entry.properties();
    let shard_element_id = required_property_map(&properties, "WENDAO_SHARD_ELEMENT_ID")?;
    let result_element_id = required_property_map(&properties, "WENDAO_RESULT_ELEMENT_ID")?;
    let source_name = required_property_map(&properties, "SOURCE")?;
    let chunk_index = parse_u32_property(&properties, "CHUNK_INDEX")?;
    let start_ms = parse_u64_property(&properties, "WENDAO_START_MS")?;
    let duration_ms = parse_u64_property(&properties, "WENDAO_DURATION_MS")?;
    let end_ms = start_ms.checked_add(duration_ms).ok_or_else(|| {
        format!("audio Org segment `{shard_element_id}` end offset overflows u64")
    })?;
    let source_sha256 = required_property_map(&properties, "WENDAO_SOURCE_SHA256")?;
    if source_sha256 != root_source_sha256 {
        return Err(format!(
            "audio Org segment `{shard_element_id}` source hash does not match ledger root"
        ));
    }
    let transcript_text = transcript_body_without_attachment_links(entry.body.as_str());
    if transcript_text.is_empty() {
        return Err(format!(
            "audio Org segment `{shard_element_id}` has empty transcript text"
        ));
    }
    let evidence_segment_id = format!(
        "audio-org-segment:{}",
        sha256_hex(format!("{evidence_source_id}:{shard_element_id}").as_bytes())
    );
    Ok(AudioOrgEvidenceSegment {
        contract_version: AUDIO_ORG_EVIDENCE_SEGMENT_SCHEMA_VERSION.to_owned(),
        evidence_source_id: evidence_source_id.to_owned(),
        evidence_segment_id,
        shard_element_id,
        result_element_id,
        source_name,
        chunk_index,
        start_ms,
        duration_ms,
        end_ms,
        source_sha256,
        shard_sha256: required_property_map(&properties, "WENDAO_SHARD_SHA256")?,
        reading_order_key: required_property_map(&properties, "WENDAO_READING_ORDER_KEY")?,
        media_start_ms: parse_u64_property(&properties, "WENDAO_MEDIA_START_MS")?,
        media_duration_ms: parse_u64_property(&properties, "WENDAO_MEDIA_DURATION_MS")?,
        sample_rate_hz: parse_u32_property(&properties, "WENDAO_SAMPLE_RATE_HZ")?,
        channels: parse_u8_property(&properties, "WENDAO_CHANNELS")?,
        audio_format: required_property_map(&properties, "WENDAO_AUDIO_FORMAT")?,
        confidence: optional_f64_property(&properties, "WENDAO_CONFIDENCE")?,
        transcript_sha256: sha256_hex(transcript_text.as_bytes()),
        transcript_text,
    })
}

fn parse_org_entries(org: &str) -> Result<Vec<OrgEntry>, String> {
    let mut entries = Vec::new();
    let mut current: Option<OrgEntryBuilder> = None;
    let mut in_properties = false;
    for raw_line in org.lines() {
        let line = raw_line.trim_end();
        if org_heading_title(line).is_some() {
            if let Some(entry) = current.take() {
                entries.push(entry.finish());
            }
            current = Some(OrgEntryBuilder::new());
            in_properties = false;
            continue;
        }
        let Some(entry) = current.as_mut() else {
            continue;
        };
        let trimmed = line.trim();
        if trimmed == ":PROPERTIES:" {
            in_properties = true;
            continue;
        }
        if in_properties && trimmed == ":END:" {
            in_properties = false;
            continue;
        }
        if in_properties {
            let Some((key, value)) = parse_property_line(trimmed) else {
                return Err(format!("invalid Org property line `{trimmed}`"));
            };
            entry.properties.push((key, value));
            continue;
        }
        entry.body.push_str(line);
        entry.body.push('\n');
    }
    if let Some(entry) = current {
        entries.push(entry.finish());
    }
    Ok(entries)
}

fn org_heading_title(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let star_count = trimmed.chars().take_while(|value| *value == '*').count();
    if star_count == 0 {
        return None;
    }
    let remainder = trimmed.get(star_count..)?;
    if !remainder.starts_with(' ') {
        return None;
    }
    Some(remainder.trim().to_owned())
}

fn transcript_body_without_attachment_links(value: &str) -> String {
    value
        .lines()
        .filter(|line| !line.trim_start().starts_with("[[attachment:"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

fn parse_property_line(line: &str) -> Option<(String, String)> {
    let without_prefix = line.strip_prefix(':')?;
    let (key, value) = without_prefix.split_once(':')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some((key.to_owned(), value.trim().to_owned()))
}

fn required_property(entry: &OrgEntry, key: &str) -> Result<String, String> {
    entry
        .properties()
        .get(key)
        .map(|value| (*value).to_owned())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("audio Org ledger root missing `{key}` property"))
}

fn required_property_map(
    properties: &std::collections::HashMap<&str, &str>,
    key: &str,
) -> Result<String, String> {
    properties
        .get(key)
        .map(|value| (*value).to_owned())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("audio Org segment missing `{key}` property"))
}

fn parse_u64_property(
    properties: &std::collections::HashMap<&str, &str>,
    key: &str,
) -> Result<u64, String> {
    required_property_map(properties, key)?
        .parse::<u64>()
        .map_err(|error| format!("audio Org property `{key}` is not u64: {error}"))
}

fn parse_u32_property(
    properties: &std::collections::HashMap<&str, &str>,
    key: &str,
) -> Result<u32, String> {
    required_property_map(properties, key)?
        .parse::<u32>()
        .map_err(|error| format!("audio Org property `{key}` is not u32: {error}"))
}

fn parse_u8_property(
    properties: &std::collections::HashMap<&str, &str>,
    key: &str,
) -> Result<u8, String> {
    required_property_map(properties, key)?
        .parse::<u8>()
        .map_err(|error| format!("audio Org property `{key}` is not u8: {error}"))
}

fn optional_f64_property(
    properties: &std::collections::HashMap<&str, &str>,
    key: &str,
) -> Result<Option<f64>, String> {
    properties
        .get(key)
        .map(|value| {
            value
                .parse::<f64>()
                .map_err(|error| format!("audio Org property `{key}` is not f64: {error}"))
        })
        .transpose()
}

fn validate_segment_order(segments: &[AudioOrgEvidenceSegment]) -> Result<(), String> {
    for pair in segments.windows(2) {
        let [left, right] = pair else {
            continue;
        };
        if left.reading_order_key > right.reading_order_key {
            return Err("audio Org evidence segments are not sorted by reading order".to_owned());
        }
        if left.shard_element_id == right.shard_element_id {
            return Err(format!(
                "audio Org evidence duplicate shard segment `{}`",
                left.shard_element_id
            ));
        }
    }
    Ok(())
}

#[derive(Debug)]
struct OrgEntryBuilder {
    properties: Vec<(String, String)>,
    body: String,
}

impl OrgEntryBuilder {
    fn new() -> Self {
        Self {
            properties: Vec::new(),
            body: String::new(),
        }
    }

    fn finish(self) -> OrgEntry {
        OrgEntry {
            properties: self.properties,
            body: self.body.trim().to_owned(),
        }
    }
}

#[cfg(feature = "audio-shard-arrow")]
fn audio_org_evidence_source_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        AUDIO_ORG_EVIDENCE_SOURCE_TABLE,
        true,
        vec![
            utf8_contract_column("contractVersion"),
            utf8_contract_column("evidenceSourceId"),
            utf8_contract_column("ledgerSchema"),
            utf8_contract_column("ledgerKind"),
            utf8_contract_column("sourcePath"),
            utf8_contract_column("sourceSha256"),
            utf8_contract_column("shardProfile"),
            utf8_contract_column("taskProfile"),
            utf8_contract_column("backendProfile"),
            utf8_contract_column("ledgerSha256"),
            int64_contract_column("segmentCount"),
        ],
    )
}

#[cfg(feature = "audio-shard-arrow")]
fn audio_org_evidence_segment_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        AUDIO_ORG_EVIDENCE_SEGMENT_TABLE,
        true,
        vec![
            utf8_contract_column("contractVersion"),
            utf8_contract_column("evidenceSourceId"),
            utf8_contract_column("evidenceSegmentId"),
            utf8_contract_column("shardElementId"),
            utf8_contract_column("resultElementId"),
            utf8_contract_column("sourceName"),
            int64_contract_column("chunkIndex"),
            int64_contract_column("startMs"),
            int64_contract_column("durationMs"),
            int64_contract_column("endMs"),
            utf8_contract_column("sourceSha256"),
            utf8_contract_column("shardSha256"),
            utf8_contract_column("readingOrderKey"),
            int64_contract_column("mediaStartMs"),
            int64_contract_column("mediaDurationMs"),
            int64_contract_column("sampleRateHz"),
            int32_contract_column("channels"),
            utf8_contract_column("audioFormat"),
            nullable_float64_contract_column("confidence"),
            utf8_contract_column("transcriptSha256"),
            utf8_contract_column("transcriptText"),
        ],
    )
}

#[cfg(feature = "audio-shard-arrow")]
fn record_batch(
    contract: &ArrowSchemaContract,
    columns: Vec<ArrayRef>,
    context: &'static str,
) -> Result<RecordBatch, String> {
    let batch = RecordBatch::try_new(schema_ref(contract), columns)
        .map_err(|error| format!("{context}: {error}"))?;
    validate_record_batch_schema_with_options(&batch, contract, exact_schema_options())
        .map_err(|error| format!("{context}: {error}"))?;
    Ok(batch)
}

#[cfg(feature = "audio-shard-arrow")]
fn schema_ref(contract: &ArrowSchemaContract) -> Arc<arrow::datatypes::Schema> {
    Arc::new(build_arrow_schema(
        contract,
        [(
            WENDAO_TABLE_METADATA_KEY.to_owned(),
            contract.table_name().to_owned(),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>(),
    ))
}

#[cfg(feature = "audio-shard-arrow")]
const fn exact_schema_options() -> ArrowSchemaValidationOptions {
    ArrowSchemaValidationOptions::new().with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact)
}

#[cfg(feature = "audio-shard-arrow")]
const fn utf8_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}

#[cfg(feature = "audio-shard-arrow")]
const fn int32_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Int32)
}

#[cfg(feature = "audio-shard-arrow")]
const fn int64_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Int64)
}

#[cfg(feature = "audio-shard-arrow")]
const fn nullable_float64_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Float64)
}

#[cfg(feature = "audio-shard-arrow")]
fn source_string_column(
    sources: &[AudioOrgEvidenceSource],
    value: impl Fn(&AudioOrgEvidenceSource) -> String,
) -> ArrayRef {
    Arc::new(StringArray::from(
        sources.iter().map(value).collect::<Vec<_>>(),
    ))
}

#[cfg(feature = "audio-shard-arrow")]
fn source_i64_column(
    sources: &[AudioOrgEvidenceSource],
    value: impl Fn(&AudioOrgEvidenceSource) -> i64,
) -> ArrayRef {
    Arc::new(Int64Array::from(
        sources.iter().map(value).collect::<Vec<_>>(),
    ))
}

#[cfg(feature = "audio-shard-arrow")]
fn segment_string_column(
    segments: &[AudioOrgEvidenceSegment],
    value: impl Fn(&AudioOrgEvidenceSegment) -> String,
) -> ArrayRef {
    Arc::new(StringArray::from(
        segments.iter().map(value).collect::<Vec<_>>(),
    ))
}

#[cfg(feature = "audio-shard-arrow")]
fn segment_i64_column(
    segments: &[AudioOrgEvidenceSegment],
    value: impl Fn(&AudioOrgEvidenceSegment) -> i64,
) -> ArrayRef {
    Arc::new(Int64Array::from(
        segments.iter().map(value).collect::<Vec<_>>(),
    ))
}

#[cfg(feature = "audio-shard-arrow")]
fn segment_u64_column(
    segments: &[AudioOrgEvidenceSegment],
    value: impl Fn(&AudioOrgEvidenceSegment) -> u64,
) -> Result<ArrayRef, String> {
    Ok(Arc::new(Int64Array::from(
        segments
            .iter()
            .map(value)
            .map(i64::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("audio Org evidence value exceeds Int64: {error}"))?,
    )))
}
