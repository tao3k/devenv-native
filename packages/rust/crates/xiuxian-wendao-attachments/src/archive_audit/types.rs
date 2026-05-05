//! Public archive audit summary and member row contracts.

use std::collections::BTreeMap;

/// Non-extracting audit summary for archive-backed document attachments.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveAttachmentAudit {
    /// Source archive path that was audited.
    pub source_path: String,
    /// Source archive file size in bytes.
    pub file_size_bytes: u64,
    /// Detected archive format.
    pub archive_format: String,
    /// Detected archive media type.
    pub mime_type: String,
    /// Total entries reported by the archive manifest.
    pub member_count: u64,
    /// Regular file entries reported by the archive manifest.
    pub regular_file_count: u64,
    /// Directory entries reported by the archive manifest.
    pub directory_count: u64,
    /// Sum of entry sizes reported by the archive manifest.
    pub total_member_size_bytes: u64,
    /// XML member count.
    pub xml_member_count: u64,
    /// Image-like member count.
    pub image_member_count: u64,
    /// Lowercase suffix counts for regular file members.
    pub extension_counts: BTreeMap<String, u64>,
    /// Likely METS XML member path when present.
    pub likely_mets_member_path: Option<String>,
    /// Largest regular member path.
    pub largest_member_path: Option<String>,
    /// Largest regular member size in bytes.
    pub largest_member_size_bytes: Option<u64>,
    /// Per-member audit rows.
    pub members: Vec<ArchiveMemberAudit>,
    /// Audit-only Rust acceleration candidate.
    pub rust_acceleration_candidate: String,
    /// Human-readable routing decision reason.
    pub decision_reason: String,
}

/// Non-extracting audit row for one archive member.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveMemberAudit {
    /// Member path inside the archive.
    pub path: String,
    /// Member size in bytes from the archive manifest.
    pub size_bytes: u64,
    /// Archive entry kind such as `file`, `directory`, or `other`.
    pub entry_kind: String,
    /// Lowercase file suffix without a dot.
    pub suffix: String,
    /// Audit role such as `mets_xml`, `metadata_xml`, `image`, or `document`.
    pub role: String,
}
