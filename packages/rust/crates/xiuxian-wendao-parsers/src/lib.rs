//! Independent parser surfaces and parser-owned contracts for Wendao-adjacent
//! consumers.

xiuxian_testing::crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");

mod markdown_structure;

/// Parser-owned reusable target plus scoped-address contract.
pub mod addressed_target;
/// Shared Markdown block parsing and parser-owned block contracts.
pub mod blocks;
/// Parser-owned Markdown `:OBSERVE:` parsing and scoped-address contract.
pub mod code_observation;
/// Parser-owned Markdown document metadata extraction.
pub mod document;
/// Markdown frontmatter parsing and parser-owned frontmatter contracts.
pub mod frontmatter;
/// Parser-owned Markdown syntax lint helpers for lightweight consumers.
pub mod lint;
/// Parser-owned source-preserved addressed-target contract.
pub mod literal_addressed_target;
/// Parser-owned Markdown note aggregation.
pub mod note;
/// Parser-owned source-preserved reference payload shared across formats.
pub mod reference_core;
/// Shared Markdown reference parsing and parser-owned link contracts.
pub mod references;
/// Parser-owned Markdown section-create planning and rendering helpers.
pub mod section_create;
/// Shared Markdown section parsing and parser-owned section contracts.
pub mod sections;
/// Shared source-position helpers used by parser-owned Markdown scans.
pub mod sourcepos;
/// Parser-owned raw Markdown target-occurrence extraction.
pub mod targets;
/// Parser-owned Markdown table-of-contents aggregation.
pub mod toc;
/// Shared Markdown wikilink parsing built on top of reference parsing.
pub mod wikilinks;

pub use addressed_target::AddressedTarget;
pub use blocks::{
    BlockCore, BlockKindIdentity, MarkdownBlock, MarkdownBlockKind, compute_block_hash,
    extract_blocks, line_col_to_byte_range,
};
pub use code_observation::{CodeObservation, extract_observations, path_matches_scope};
pub use document::{
    DocumentCore, DocumentEnvelope, DocumentFormat, MarkdownDocument, parse_markdown_document,
};
pub use frontmatter::{
    NoteFrontmatter, RawFrontmatter, discover_skill_documents, frontmatter_kind,
    is_skill_descriptor_path, parse_frontmatter, parse_skill_frontmatter_lenient,
    skill_frontmatter_has_metadata_mapping, skill_frontmatter_name, split_frontmatter,
    split_frontmatter_raw, uses_skill_frontmatter,
};
pub use lint::{
    MarkdownLintKind, MarkdownSyntaxLintCode, MarkdownSyntaxLintIssue, MarkdownSyntaxLintReport,
    lint_markdown_syntax, lint_markdown_syntax_with_path,
};
pub use literal_addressed_target::LiteralAddressedTarget;
pub use note::{
    MarkdownNote, MarkdownNoteCore, MarkdownNoteParseArtifacts, NoteAggregate, NoteCore,
    fingerprint_markdown_note, fingerprint_markdown_symbol_surface, parse_markdown_note,
    parse_markdown_note_artifacts,
};
pub use reference_core::ReferenceCore;
pub use references::{
    MarkdownReference, MarkdownReferenceKind, extract_references, parse_reference_literal,
};
pub use section_create::{
    BuildSectionOptions, InsertionInfo, SiblingInfo, build_new_sections_content_with_options,
    compute_content_hash, find_insertion_point, generate_section_id, parse_heading_line,
};
pub use sections::{
    LogbookEntry, MarkdownSection, SectionCore, SectionMetadata, SectionScope, extract_sections,
};
pub use targets::{
    MarkdownTargetOccurrence, MarkdownTargetOccurrenceKind, TargetOccurrenceCore, extract_targets,
};
pub use toc::{
    MarkdownOutlineDocument, MarkdownOutlineHeading, MarkdownTocDocument, TocDocument,
    parse_markdown_outline, parse_markdown_toc,
};
pub use wikilinks::{MarkdownWikiLink, extract_wikilinks, parse_wikilink_literal};
