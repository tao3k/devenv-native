use std::path::Path;

use tar::EntryType;

use super::types::ArchiveMemberAudit;

const IMAGE_SUFFIXES: &[&str] = &[
    "bmp", "gif", "j2k", "jp2", "jpeg", "jpg", "png", "tif", "tiff", "webp",
];

pub(crate) fn build_member_audit(
    path: String,
    size_bytes: u64,
    entry_type: EntryType,
) -> ArchiveMemberAudit {
    let entry_kind = entry_kind(entry_type);
    let suffix = member_suffix(&path);
    let role = member_role(&path, &suffix);
    ArchiveMemberAudit {
        path,
        size_bytes,
        entry_kind,
        suffix,
        role,
    }
}

pub(crate) fn is_regular_file(member: &ArchiveMemberAudit) -> bool {
    member.entry_kind == "file"
}

pub(crate) fn is_directory(member: &ArchiveMemberAudit) -> bool {
    member.entry_kind == "directory"
}

pub(crate) fn is_xml(member: &ArchiveMemberAudit) -> bool {
    member.suffix == "xml"
}

pub(crate) fn is_image(member: &ArchiveMemberAudit) -> bool {
    member.role == "image"
}

fn entry_kind(entry_type: EntryType) -> String {
    if entry_type.is_file() {
        "file".to_owned()
    } else if entry_type.is_dir() {
        "directory".to_owned()
    } else if entry_type.is_symlink() {
        "symlink".to_owned()
    } else {
        "other".to_owned()
    }
}

fn member_suffix(path: &str) -> String {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default()
}

fn member_role(path: &str, suffix: &str) -> String {
    let lowercase_path = path.to_ascii_lowercase();
    if suffix == "xml" && lowercase_path.contains("mets") {
        return "mets_xml".to_owned();
    }
    if suffix == "xml" {
        return "metadata_xml".to_owned();
    }
    if IMAGE_SUFFIXES.contains(&suffix) {
        return "image".to_owned();
    }
    if suffix.is_empty() {
        return "unknown".to_owned();
    }
    "document".to_owned()
}
