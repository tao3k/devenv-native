use super::types::ArchiveAttachmentAudit;

pub(crate) fn assign_routing_decision(audit: &mut ArchiveAttachmentAudit) {
    let (candidate, reason) = if audit.archive_format == "unsupported" {
        (
            "unsupported_non_archive",
            "The source is not a supported archive attachment.",
        )
    } else if audit.xml_member_count > 0 && audit.image_member_count > 0 {
        (
            "mets_gbs_member_manifest_candidate",
            "Rust can cache the archive member manifest and route XML/image members while Docling remains document authority.",
        )
    } else if audit.member_count > 0 {
        (
            "archive_member_manifest_candidate",
            "Rust can cache the archive member manifest while Docling remains document authority.",
        )
    } else {
        (
            "empty_archive_docling_passthrough",
            "The archive has no manifest entries; leave extraction to Docling.",
        )
    };

    candidate.clone_into(&mut audit.rust_acceleration_candidate);
    reason.clone_into(&mut audit.decision_reason);
}
