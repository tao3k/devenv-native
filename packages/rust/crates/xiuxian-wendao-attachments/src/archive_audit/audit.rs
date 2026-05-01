use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;

use super::format::{ArchiveFormat, detect_archive_format, unsupported_format};
use super::member::{build_member_audit, is_directory, is_image, is_regular_file, is_xml};
use super::routing::assign_routing_decision;
use super::types::{ArchiveAttachmentAudit, ArchiveMemberAudit};

const MAX_AUDIT_MEMBERS: usize = 50_000;

/// Returns true when the path suffix maps to a supported archive audit reader.
pub fn is_supported_archive_path(path: impl AsRef<Path>) -> bool {
    detect_archive_format(path.as_ref()).is_some()
}

/// Audit an archive-backed attachment without extracting member contents.
///
/// # Errors
///
/// Returns an error when file metadata cannot be read, a supported archive
/// cannot be opened, or the archive manifest cannot be decoded.
pub fn audit_archive_attachment(path: impl AsRef<Path>) -> Result<ArchiveAttachmentAudit, String> {
    let path = path.as_ref();
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("failed to stat archive source {}: {error}", path.display()))?;
    let format = detect_archive_format(path).unwrap_or_else(unsupported_format);
    if format.name == "unsupported" {
        let mut audit = empty_audit(path, metadata.len(), format);
        assign_routing_decision(&mut audit);
        return Ok(audit);
    }

    let file = File::open(path)
        .map_err(|error| format!("failed to open archive source {}: {error}", path.display()))?;
    let mut audit = match format.name {
        "tar" => audit_tar_reader(path, metadata.len(), format, BufReader::new(file))?,
        "tar.gz" => {
            let decoder = GzDecoder::new(BufReader::new(file));
            audit_tar_reader(path, metadata.len(), format, decoder)?
        }
        name => {
            return Err(format!(
                "archive format {name} is unsupported after detection for {}",
                path.display()
            ));
        }
    };
    assign_routing_decision(&mut audit);
    Ok(audit)
}

fn audit_tar_reader<R: Read>(
    path: &Path,
    file_size_bytes: u64,
    format: ArchiveFormat,
    reader: R,
) -> Result<ArchiveAttachmentAudit, String> {
    let mut audit = empty_audit(path, file_size_bytes, format);
    let mut archive = Archive::new(reader);
    let entries = archive.entries().map_err(|error| {
        format!(
            "failed to read archive entries for {}: {error}",
            path.display()
        )
    })?;

    for entry in entries {
        if audit.members.len() >= MAX_AUDIT_MEMBERS {
            return Err(format!(
                "archive {} exceeds audit member limit {MAX_AUDIT_MEMBERS}",
                path.display()
            ));
        }
        let entry = entry.map_err(|error| {
            format!(
                "failed to read archive entry for {}: {error}",
                path.display()
            )
        })?;
        let entry_type = entry.header().entry_type();
        let size_bytes = entry.size();
        let member_path = entry
            .path()
            .map_err(|error| {
                format!(
                    "failed to read archive member path for {}: {error}",
                    path.display()
                )
            })?
            .to_string_lossy()
            .replace('\\', "/");
        let member = build_member_audit(member_path, size_bytes, entry_type);
        apply_member_stats(&mut audit, &member);
        audit.members.push(member);
    }

    Ok(audit)
}

fn empty_audit(path: &Path, file_size_bytes: u64, format: ArchiveFormat) -> ArchiveAttachmentAudit {
    ArchiveAttachmentAudit {
        source_path: path.display().to_string(),
        file_size_bytes,
        archive_format: format.name.to_owned(),
        mime_type: format.mime_type.to_owned(),
        member_count: 0,
        regular_file_count: 0,
        directory_count: 0,
        total_member_size_bytes: 0,
        xml_member_count: 0,
        image_member_count: 0,
        extension_counts: BTreeMap::new(),
        likely_mets_member_path: None,
        largest_member_path: None,
        largest_member_size_bytes: None,
        members: Vec::new(),
        rust_acceleration_candidate: String::new(),
        decision_reason: String::new(),
    }
}

fn apply_member_stats(audit: &mut ArchiveAttachmentAudit, member: &ArchiveMemberAudit) {
    audit.member_count += 1;
    audit.total_member_size_bytes += member.size_bytes;

    if is_directory(member) {
        audit.directory_count += 1;
    }
    if is_regular_file(member) {
        audit.regular_file_count += 1;
        update_largest_member(audit, member);
        if !member.suffix.is_empty() {
            *audit
                .extension_counts
                .entry(member.suffix.clone())
                .or_insert(0) += 1;
        }
    }
    if is_xml(member) {
        audit.xml_member_count += 1;
        update_likely_mets_member(audit, member);
    }
    if is_image(member) {
        audit.image_member_count += 1;
    }
}

fn update_largest_member(audit: &mut ArchiveAttachmentAudit, member: &ArchiveMemberAudit) {
    let should_update = audit
        .largest_member_size_bytes
        .is_none_or(|size_bytes| member.size_bytes > size_bytes);
    if should_update {
        audit.largest_member_size_bytes = Some(member.size_bytes);
        audit.largest_member_path = Some(member.path.clone());
    }
}

fn update_likely_mets_member(audit: &mut ArchiveAttachmentAudit, member: &ArchiveMemberAudit) {
    if member.role == "mets_xml" || audit.likely_mets_member_path.is_none() {
        audit.likely_mets_member_path = Some(member.path.clone());
    }
}
