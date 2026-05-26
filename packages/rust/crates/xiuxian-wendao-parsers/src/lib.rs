//! Independent parser surfaces and parser-owned contracts for Wendao-adjacent
//! consumers.

mod markdown_structure;

/// Parser-owned reusable target plus scoped-address contract.
pub mod addressed_target;
/// Shared Markdown block parsing and parser-owned block contracts.
pub mod blocks;
/// Parser-owned Markdown `:OBSERVE:` parsing and scoped-address contract.
pub mod code_observation;
/// Parser-owned Markdown document metadata extraction.
pub mod document;
/// Parser-owned episteme source-contract DTOs and parsers.
pub mod episteme_contract;
/// Markdown frontmatter parsing and parser-owned frontmatter contracts.
pub mod frontmatter;
/// Parser-owned Markdown syntax lint helpers for lightweight consumers.
pub mod lint;
/// Parser-owned source-preserved addressed-target contract.
pub mod literal_addressed_target;
/// Parser-owned Markdown note aggregation.
pub mod note;
/// Parser-owned Org-mode document and note aggregation.
pub mod org;
/// Parser-owned Orgize tooling adapters for client surfaces.
pub mod orgize_tool;
/// Parser-owned source-preserved reference payload shared across formats.
pub mod reference_core;
/// Shared Markdown reference parsing and parser-owned link contracts.
pub mod references;
/// Parser-owned Markdown section-create planning and rendering helpers.
pub mod section_create;
/// Shared Markdown section parsing and parser-owned section contracts.
pub mod sections;
/// Parser-owned repo-native semantic SSOT artifact contracts.
pub mod semantic_ssot;
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
    BlockCore, BlockCoreRequest, BlockExplicitId, BlockKindIdentity, MarkdownBlock,
    MarkdownBlockKind, compute_block_hash, extract_blocks,
};
pub use code_observation::{CodeObservation, extract_observations, path_matches_scope};
pub use document::{
    DocumentCore, DocumentEnvelope, DocumentFormat, DocumentType, MarkdownDocument, OrgDocument,
    OrgDocumentMetadata, parse_markdown_document,
};
pub use episteme_contract::{
    EpistemeExtractionQueueRow, EpistemeFileRow, EpistemeMappingLedgerValidation,
    EpistemeSourceContractParseError, EpistemeSourceManifest, parse_episteme_extraction_queue_tsv,
    parse_episteme_files_tsv, parse_episteme_source_manifest_toml,
    validate_episteme_mapping_ledger_org,
};
pub use frontmatter::{
    NoteCategory, NoteFrontmatter, RawFrontmatter, SkillFrontmatterParseError,
    discover_skill_documents, frontmatter_kind, is_skill_descriptor_path, parse_frontmatter,
    parse_skill_frontmatter, split_frontmatter, split_frontmatter_raw, uses_skill_frontmatter,
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
pub use org::{
    ORG_ONTOLOGY_AUTHORING_SCHEMA_ID, ORG_PROP_BLANK_VALUE, ORG_PROP_INVALID_CONFIDENCE,
    ORG_PROP_INVALID_ENUM, ORG_PROP_INVALID_SHA256, ORG_PROP_INVALID_UUID,
    ORG_PROP_MISSING_REQUIRED, ORG_PROP_UNKNOWN_PROPERTY, ORG_REASONING_PROPERTY_SCHEMA_ID,
    OrgAttachmentLink, OrgAttachmentLinkProtocol, OrgNote, OrgNoteCore,
    OrgOntologyAuthoringDocument, OrgOntologyAuthoringError, OrgOntologyAuthoringKind,
    OrgOntologyAuthoringSection, OrgOntologyAuthoringTable, OrgOntologyEmbeddedArtifact,
    OrgOntologyLifecycleState, OrgOntologySourceSpan, OrgOntologyTableKind,
    OrgReasoningPropertyDiagnostic, OrgReasoningPropertyRecord, OrgSection, OrgTocDocument,
    compile_org_ontology_authoring_document, compile_org_reasoning_property_records,
    extract_org_attachment_links, extract_org_sections, parse_org_document, parse_org_note,
    parse_org_toc, validate_org_reasoning_properties, validate_org_reasoning_property_records,
};
pub use orgize_tool::{
    OrgizeAgentPlanningRequest, OrgizeAgentTaskProperty, OrgizeAgentTaskReadModelReport,
    OrgizeAgentTaskReadModelRequest, OrgizeAgentTaskRepeater, OrgizeAgentTaskRow,
    OrgizeEvalPatchRequest, OrgizeEvalPlanRequest, OrgizeFormatReport, OrgizeFormatRequest,
    OrgizeLintFileReport, OrgizeLintOutputFormat, OrgizeLintRequest, OrgizeLintRunReport,
    OrgizeOrgElementReadModelReport, OrgizeOrgElementReadModelRequest, OrgizeOrgElementRow,
    OrgizeSddGraphDiffRequest, OrgizeSddStatusRequest, OrgizeSparseTreeRenderOptions,
    OrgizeSparseTreeRequest, OrgizeSparseTreeVisibility, OrgizeToolError, collect_agent_task_rows,
    collect_org_element_rows, count_sdd_graph_drift, count_sdd_status_issues, format_org_files,
    lint_org_files, render_agent_planning, render_eval_patch, render_eval_plan,
    render_sdd_graph_diff, render_sdd_status, render_sdd_status_json, render_sparse_tree,
};
pub use reference_core::ReferenceCore;
pub use references::{
    MarkdownReference, MarkdownReferenceKind, extract_references, parse_reference_literal,
};
pub use section_create::{
    BuildSectionOptions, InsertionInfo, ParsedHeadingLine, SiblingInfo,
    build_new_sections_content_with_options, compute_content_hash, find_insertion_point,
    generate_section_id, parse_heading_line,
};
pub use sections::{
    LogbookEntry, MarkdownSection, SectionCore, SectionMetadata, SectionScope, extract_sections,
};
pub use semantic_ssot::{
    SEMANTIC_PROJECTION_FRESHNESS_POLICY_ID, SEMANTIC_PROJECTION_POLICY_EVIDENCE_METADATA_KEY,
    SEMANTIC_SCOPE_BUNDLE_METADATA_KEY, SEMANTIC_SQL_GUARD_EVIDENCE_METADATA_KEY,
    SemanticBundleProvenance, SemanticChangeIntent, SemanticChangeIntentType, SemanticConfidence,
    SemanticConfidenceSource, SemanticObject, SemanticObjectKind, SemanticOwner,
    SemanticProjection, SemanticProjectionFreshnessPolicyEntry,
    SemanticProjectionFreshnessPolicyReport, SemanticProjectionPolicyStatus,
    SemanticProjectionRefreshPlanEntry, SemanticProjectionRefreshPlanReport,
    SemanticProjectionRefreshPlanStatus, SemanticProjectionStaleness, SemanticProjectionType,
    SemanticProvenance, SemanticRelation, SemanticRelationChange, SemanticRelationChangeAction,
    SemanticRelationEdge, SemanticRelationKind, SemanticRepository, SemanticScopeBundle,
    SemanticScopeMetadataEnvelope, SemanticScopeRequest, SemanticStatus, SemanticStatusTransition,
    SemanticValidationIssue, SemanticValidationReport, SemanticVerification,
    load_semantic_repository, parse_semantic_change_intent, parse_semantic_object,
    parse_semantic_projection, parse_semantic_scope_metadata_envelope_json,
    semantic_projection_freshness_policy_report, semantic_projection_refresh_plan_report,
    semantic_projection_source_revision, semantic_scope_bundle, semantic_scope_metadata_envelope,
    semantic_scope_metadata_envelope_to_vec,
};
pub use sourcepos::{LineColumnSpan, SourceByteRange, line_col_to_byte_range};
pub use targets::{
    MarkdownTargetOccurrence, MarkdownTargetOccurrenceKind, TargetOccurrenceCore, extract_targets,
};
pub use toc::{
    MarkdownOutlineDocument, MarkdownOutlineHeading, MarkdownTocDocument, TocDocument,
    parse_markdown_outline, parse_markdown_toc,
};
pub use wikilinks::{MarkdownWikiLink, extract_wikilinks, parse_wikilink_literal};

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config().with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public package API for cargo-test verification"),
        )
    }
);
