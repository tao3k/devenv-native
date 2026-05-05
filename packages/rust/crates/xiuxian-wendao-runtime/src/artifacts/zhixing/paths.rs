//! Zhixing artifact path builders for runtime package wiring.

use include_dir::Dir;

/// Embedded skill document path relative to the `resources/` root.
pub const ZHIXING_SKILL_DOC_PATH: &str = "zhixing/skills/agenda-management/SKILL.md";
/// Stable embedded crate id used by mounted zhixing resource readers.
pub const ZHIXING_EMBEDDED_CRATE_ID: &str = "xiuxian-zhixing";

static EMBEDDED_ZHIXING_RESOURCES: Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../xiuxian-wendao/resources");

/// Returns the embedded zhixing resource root bundled with the workspace.
#[must_use]
pub fn embedded_resource_dir() -> &'static Dir<'static> {
    &EMBEDDED_ZHIXING_RESOURCES
}

/// Normalizes an embedded resource path to slash-separated relative form.
#[must_use]
pub fn normalize_embedded_resource_path(path: &str) -> String {
    path.trim().trim_start_matches("./").replace('\\', "/")
}
