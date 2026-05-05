//! DTOs for Markdown section creation planning.

/// Narrative context about one sibling heading near the insertion point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiblingInfo {
    /// Heading title of the sibling section.
    pub title: String,
    /// Short preview taken from the sibling body when available.
    pub preview: String,
}

/// Parser-owned description of where and how new heading content should be
/// inserted into one Markdown document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertionInfo {
    /// Byte offset where the new content should be inserted.
    pub insertion_byte: usize,
    /// Heading level to use for the first newly created heading.
    pub start_level: usize,
    /// Remaining heading path components that still need to be created.
    pub remaining_path: Vec<String>,
    /// Previous sibling context when one exists at the insertion level.
    pub prev_sibling: Option<SiblingInfo>,
    /// Next sibling context when one exists at the insertion level.
    pub next_sibling: Option<SiblingInfo>,
}

impl Default for InsertionInfo {
    fn default() -> Self {
        Self {
            insertion_byte: 0,
            start_level: 1,
            remaining_path: Vec::new(),
            prev_sibling: None,
            next_sibling: None,
        }
    }
}

/// Options for rendering newly created section content.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildSectionOptions {
    /// Whether to emit an `:ID:` property drawer for each created heading.
    pub generate_id: bool,
    /// Optional caller-owned prefix for generated identifiers.
    pub id_prefix: Option<String>,
}
