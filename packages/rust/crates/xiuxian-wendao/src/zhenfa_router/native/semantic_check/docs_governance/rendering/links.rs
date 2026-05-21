//! `zhenfa_router::native::semantic_check::docs_governance::rendering::links` owns Wendao docs governance rendering links behavior.

/// Convert a relative markdown path into a wikilink target without the `.md` suffix.
#[must_use]
pub fn link_target(relative_path: &str) -> String {
    relative_path
        .strip_suffix(".md")
        .unwrap_or(relative_path)
        .replace('\\', "/")
}
