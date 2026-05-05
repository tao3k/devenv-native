//! Markdown section creation rendering.

use std::fmt::Write;

use super::types::BuildSectionOptions;

/// Render one or more missing Markdown heading levels plus the caller-provided
/// section content.
#[must_use]
pub fn build_new_sections_content_with_options(
    remaining_path: &[String],
    start_level: usize,
    content: &str,
    options: &BuildSectionOptions,
) -> String {
    let mut result = String::new();
    for (current_level, (index, heading)) in (start_level..).zip(remaining_path.iter().enumerate())
    {
        let level = current_level.clamp(1, 6);
        let heading_marker = "#".repeat(level);

        if index > 0 {
            result.push('\n');
        }
        let _ = write!(result, "{heading_marker} {heading}");

        if options.generate_id {
            let id = generate_section_id(options.id_prefix.as_deref());
            let _ = write!(result, "\n:ID: {id}");
        }
        result.push_str("\n\n");
    }

    result.push_str(content);
    result.push('\n');

    result
}

/// Generate one section identifier, optionally prefixed for caller-owned
/// namespaces.
#[must_use]
pub fn generate_section_id(prefix: Option<&str>) -> String {
    let uuid = uuid::Uuid::new_v4();
    let uuid_str = uuid.simple().to_string();

    match prefix {
        Some(prefix) => format!("{prefix}-{}", &uuid_str[..8]),
        None => uuid_str[..12].to_string(),
    }
}

/// Compute one short content hash for optimistic section-create follow-up
/// checks.
#[must_use]
pub fn compute_content_hash(content: &str) -> String {
    use blake3::Hasher;

    let mut hasher = Hasher::new();
    hasher.update(content.as_bytes());
    let hash = hasher.finalize();
    hash.to_hex()[..16].to_string()
}
