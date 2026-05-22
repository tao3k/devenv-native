//! Source-contract route policy checks owned by the Rust backend boundary.

use xiuxian_wendao_parsers::{EpistemeFileRow, EpistemeSourceManifest};

const DOCLING_DOCUMENT_ROUTE: &str = "document_text_evidence";
const LEGACY_OFFICE_ROUTE: &str = "legacy_office_document_evidence";
const LEGACY_OFFICE_EXTENSIONS: &[&str] = &["doc", "ppt", "xls"];

pub(super) fn validate_document_route_policy(
    manifest: &EpistemeSourceManifest,
    files: &[EpistemeFileRow],
    errors: &mut Vec<String>,
) {
    for (route, extensions) in &manifest.routes {
        for extension in extensions {
            validate_manifest_route_extension(route, extension, errors);
        }
    }

    for (index, row) in files.iter().enumerate() {
        let row_number = index + 2;
        if is_legacy_office_extension(row.extension.as_str())
            && row.extraction_route != LEGACY_OFFICE_ROUTE
        {
            errors.push(format!(
                "row {row_number} legacy Office extension {} must use {LEGACY_OFFICE_ROUTE}: {}",
                row.extension, row.relative_path
            ));
        }
    }
}

fn validate_manifest_route_extension(route: &str, extension: &str, errors: &mut Vec<String>) {
    if is_legacy_office_extension(extension) && route != LEGACY_OFFICE_ROUTE {
        errors.push(format!(
            "source manifest legacy Office extension {extension} must use {LEGACY_OFFICE_ROUTE}, not {route}"
        ));
    }
    if route == LEGACY_OFFICE_ROUTE && !is_legacy_office_extension(extension) {
        errors.push(format!(
            "source manifest {LEGACY_OFFICE_ROUTE} cannot include non-legacy extension {extension}"
        ));
    }
    if route == DOCLING_DOCUMENT_ROUTE && is_legacy_office_extension(extension) {
        errors.push(format!(
            "source manifest {DOCLING_DOCUMENT_ROUTE} cannot include legacy Office extension {extension}"
        ));
    }
}

fn is_legacy_office_extension(extension: &str) -> bool {
    LEGACY_OFFICE_EXTENSIONS.contains(&extension)
}
