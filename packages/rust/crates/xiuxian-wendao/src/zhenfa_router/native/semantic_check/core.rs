//! Core semantic check orchestration.

use std::collections::HashMap;
use std::path::Path;

use xiuxian_zhenfa::{ZhenfaContext, ZhenfaError};

use crate::link_graph::{LinkGraphIndex, PageIndexNode, RegistryIndex};
use crate::parsers::docs_governance::is_package_local_crate_doc;
use crate::zhenfa_router::native::WendaoContextExt;
use crate::zhenfa_router::native::audit::{CodeLanguageId, SourceFile, resolve_source_files};

use super::checks::{
    check_code_observations, check_contracts, check_dead_links, check_deprecated_refs,
    check_hash_alignment, check_id_collisions, check_legacy_syntax, check_missing_identity,
};
use super::docs_governance;
use super::episteme::load_episteme_manifest;
use super::report::{build_file_reports, collect_report_doc_paths, format_result_as_xml};
use super::types::{CheckType, SemanticCheckResult, SemanticIssue, WendaoSemanticCheckArgs};

/// Perform semantic consistency check on the knowledge base.
///
/// # Errors
///
/// Returns `ZhenfaError` when the link graph index cannot be loaded or when the
/// underlying audit core cannot complete.
#[allow(clippy::needless_pass_by_value)]
#[allow(missing_docs)]
pub fn wendao_semantic_check(
    ctx: &ZhenfaContext,
    args: WendaoSemanticCheckArgs,
) -> Result<String, ZhenfaError> {
    let episteme = args
        .episteme_load
        .as_deref()
        .map(load_episteme_manifest)
        .transpose()
        .map_err(|error| {
            ZhenfaError::invalid_arguments(format!("episteme load failed: {error}"))
        })?;

    let (issues, file_contents) = run_audit_core(ctx, &args)?;
    let docs_list: Vec<String> = file_contents.keys().cloned().collect();
    let report_docs = collect_report_doc_paths(&docs_list, &issues);
    let docs_checked_count = report_docs.len();

    let error_count = issues.iter().filter(|i| i.severity == "error").count();
    let warning_count = issues.iter().filter(|i| i.severity == "warning").count();

    let status = if error_count > 0 {
        "fail"
    } else if warning_count > 0 {
        "warning"
    } else {
        "pass"
    };

    let summary = format!(
        "Found {error_count} errors and {warning_count} warnings across {docs_checked_count} documents"
    );

    let file_reports = build_file_reports(&issues, &report_docs);

    let result = SemanticCheckResult {
        status: status.to_string(),
        issue_count: issues.len(),
        issues,
        summary,
        file_reports,
        episteme,
    };

    Ok(format_result_as_xml(&result))
}

/// Run the core audit logic and return raw issues and file contents.
///
/// # Errors
///
/// Returns `ZhenfaError` when the link graph index cannot be queried.
pub fn run_audit_core(
    ctx: &ZhenfaContext,
    args: &WendaoSemanticCheckArgs,
) -> Result<(Vec<SemanticIssue>, HashMap<String, String>), ZhenfaError> {
    let index = ctx.link_graph_index()?;
    Ok(run_audit_core_with_index(&index, args))
}

fn run_audit_core_with_index(
    index: &LinkGraphIndex,
    args: &WendaoSemanticCheckArgs,
) -> (Vec<SemanticIssue>, HashMap<String, String>) {
    let include_warnings = args.include_warnings.unwrap_or(true);
    let mut file_contents = HashMap::new();
    let checks = resolved_checks(args);
    let source_files = resolved_source_files(args);
    let build_result = index.build_registry_index_with_collisions();
    let mut issues = Vec::new();
    collect_workspace_doc_governance(args, index, &checks, &mut file_contents, &mut issues);
    if checks.contains(&CheckType::IdCollisions) {
        check_id_collisions(&build_result, &mut issues);
    }

    let registry = build_result.registry;
    let trees = index.all_page_index_trees();
    let docs_to_check = docs_to_check(args.doc.as_deref(), trees);

    if let Some(explicit_doc) = args.doc.as_deref() {
        seed_explicit_doc_content(explicit_doc, &mut file_contents);
    }

    let audit_context = AuditCoreContext {
        trees,
        registry: &registry,
        checks: &checks,
        include_warnings,
        source_files: &source_files,
        fuzzy_threshold: args.fuzzy_confidence_threshold,
    };
    check_requested_docs(
        &docs_to_check,
        &audit_context,
        &mut file_contents,
        &mut issues,
    );
    collect_explicit_doc_governance(args, &checks, &docs_to_check, &file_contents, &mut issues);

    (issues, file_contents)
}

fn resolved_checks(args: &WendaoSemanticCheckArgs) -> Vec<CheckType> {
    args.checks.clone().unwrap_or_else(|| {
        vec![
            CheckType::DeadLinks,
            CheckType::DeprecatedRefs,
            CheckType::Contracts,
            CheckType::IdCollisions,
            CheckType::HashAlignment,
            CheckType::MissingIdentity,
            CheckType::LegacySyntax,
            CheckType::CodeObservations,
            CheckType::DocGovernance,
        ]
    })
}

fn resolved_source_files(args: &WendaoSemanticCheckArgs) -> Vec<SourceFile> {
    let Some(paths) = args.source_paths.as_ref() else {
        return Vec::new();
    };
    let path_refs: Vec<&std::path::Path> = paths.iter().map(std::path::Path::new).collect();
    audit_source_language_ids()
        .into_iter()
        .flat_map(|language_id| resolve_source_files(&path_refs, &language_id))
        .collect()
}

fn audit_source_language_ids() -> [CodeLanguageId; 5] {
    [
        CodeLanguageId::from("rust"),
        CodeLanguageId::from("python"),
        CodeLanguageId::from("typescript"),
        CodeLanguageId::from("javascript"),
        CodeLanguageId::from("go"),
    ]
}

fn collect_workspace_doc_governance(
    args: &WendaoSemanticCheckArgs,
    index: &LinkGraphIndex,
    checks: &[CheckType],
    file_contents: &mut HashMap<String, String>,
    issues: &mut Vec<SemanticIssue>,
) {
    if !checks.contains(&CheckType::DocGovernance) {
        return;
    }
    let workspace_issues =
        docs_governance::collect_workspace_doc_governance_issues(index.root(), args.doc.as_deref());
    for issue in &workspace_issues {
        seed_explicit_doc_content(&issue.doc, file_contents);
    }
    issues.extend(workspace_issues);
}

fn docs_to_check(
    explicit_doc: Option<&str>,
    trees: &HashMap<String, Vec<PageIndexNode>>,
) -> Vec<String> {
    let Some(doc) = explicit_doc else {
        return trees.keys().cloned().collect();
    };
    if doc == "." || doc.is_empty() {
        return trees.keys().cloned().collect();
    }
    if trees.contains_key(doc) {
        return vec![doc.to_string()];
    }
    trees
        .keys()
        .filter(|key| key.contains(doc))
        .cloned()
        .collect()
}

struct AuditCoreContext<'a> {
    trees: &'a HashMap<String, Vec<PageIndexNode>>,
    registry: &'a RegistryIndex,
    checks: &'a [CheckType],
    include_warnings: bool,
    source_files: &'a [SourceFile],
    fuzzy_threshold: Option<f32>,
}

fn check_requested_docs(
    docs_to_check: &[String],
    audit_context: &AuditCoreContext<'_>,
    file_contents: &mut HashMap<String, String>,
    issues: &mut Vec<SemanticIssue>,
) {
    for doc_id in docs_to_check {
        seed_doc_content_from_path(doc_id, file_contents);
        collect_doc_governance(doc_id, audit_context.checks, file_contents, issues);
        check_doc_trees(doc_id, audit_context, issues);
    }
}

fn seed_doc_content_from_path(doc_id: &str, file_contents: &mut HashMap<String, String>) {
    if let Ok(content) = std::fs::read_to_string(doc_id) {
        file_contents.insert(doc_id.to_string(), content);
    }
}

fn collect_doc_governance(
    doc_id: &str,
    checks: &[CheckType],
    file_contents: &HashMap<String, String>,
    issues: &mut Vec<SemanticIssue>,
) {
    if !checks.contains(&CheckType::DocGovernance) {
        return;
    }
    if let Some(content) = file_contents.get(doc_id) {
        issues.extend(docs_governance::collect_doc_governance_issues(
            doc_id, content,
        ));
    }
}

fn check_doc_trees(
    doc_id: &str,
    audit_context: &AuditCoreContext<'_>,
    issues: &mut Vec<SemanticIssue>,
) {
    let Some(doc_trees) = audit_context.trees.get(doc_id) else {
        return;
    };
    let audit_pass = AuditPass {
        doc_id,
        registry: audit_context.registry,
        checks: audit_context.checks,
        include_warnings: audit_context.include_warnings,
        source_files: audit_context.source_files,
        fuzzy_threshold: audit_context.fuzzy_threshold,
    };
    for root in doc_trees {
        check_node(root, &audit_pass, issues);
    }
}

fn collect_explicit_doc_governance(
    args: &WendaoSemanticCheckArgs,
    checks: &[CheckType],
    docs_to_check: &[String],
    file_contents: &HashMap<String, String>,
    issues: &mut Vec<SemanticIssue>,
) {
    let Some(explicit_doc) = args.doc.as_deref() else {
        return;
    };
    if !should_check_explicit_doc_governance(explicit_doc, checks, docs_to_check) {
        return;
    }
    if let Some(content) = resolve_explicit_doc_content(explicit_doc, file_contents) {
        issues.extend(docs_governance::collect_doc_governance_issues(
            explicit_doc,
            content,
        ));
    }
}

fn should_check_explicit_doc_governance(
    explicit_doc: &str,
    checks: &[CheckType],
    docs_to_check: &[String],
) -> bool {
    checks.contains(&CheckType::DocGovernance)
        && explicit_doc != "."
        && !explicit_doc.is_empty()
        && !is_package_local_crate_doc(explicit_doc)
        && !docs_to_check.iter().any(|doc_id| doc_id == explicit_doc)
}

fn seed_explicit_doc_content(doc: &str, file_contents: &mut HashMap<String, String>) {
    if doc.is_empty() || doc == "." {
        return;
    }

    let path = Path::new(doc);
    if !path.is_file() {
        return;
    }

    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };

    file_contents
        .entry(doc.to_string())
        .or_insert_with(|| content.clone());

    if let Ok(canonical_path) = path.canonicalize() {
        let canonical_key = canonical_path.to_string_lossy().to_string();
        file_contents.entry(canonical_key).or_insert(content);
    }
}

fn resolve_explicit_doc_content<'a>(
    doc: &str,
    file_contents: &'a HashMap<String, String>,
) -> Option<&'a String> {
    file_contents.get(doc).or_else(|| {
        Path::new(doc)
            .canonicalize()
            .ok()
            .and_then(|canonical_path| {
                file_contents.get(&canonical_path.to_string_lossy().to_string())
            })
    })
}

struct AuditPass<'a> {
    doc_id: &'a str,
    registry: &'a RegistryIndex,
    checks: &'a [CheckType],
    include_warnings: bool,
    source_files: &'a [SourceFile],
    fuzzy_threshold: Option<f32>,
}

fn check_node(node: &PageIndexNode, audit_pass: &AuditPass<'_>, issues: &mut Vec<SemanticIssue>) {
    if audit_pass.checks.contains(&CheckType::DeadLinks) {
        check_dead_links(node, audit_pass.doc_id, audit_pass.registry, issues);
    }

    if audit_pass.checks.contains(&CheckType::DeprecatedRefs) && audit_pass.include_warnings {
        check_deprecated_refs(node, audit_pass.doc_id, audit_pass.registry, issues);
    }

    if audit_pass.checks.contains(&CheckType::Contracts) {
        check_contracts(node, audit_pass.doc_id, issues);
    }

    if audit_pass.checks.contains(&CheckType::HashAlignment) {
        check_hash_alignment(node, audit_pass.doc_id, audit_pass.registry, issues);
    }

    if audit_pass.checks.contains(&CheckType::MissingIdentity) && audit_pass.include_warnings {
        check_missing_identity(node, audit_pass.doc_id, issues);
    }

    if audit_pass.checks.contains(&CheckType::LegacySyntax) && audit_pass.include_warnings {
        check_legacy_syntax(node, audit_pass.doc_id, issues);
    }

    if audit_pass.checks.contains(&CheckType::CodeObservations) {
        check_code_observations(
            node,
            audit_pass.doc_id,
            audit_pass.source_files,
            audit_pass.fuzzy_threshold,
            issues,
        );
    }

    for child in &node.children {
        check_node(child, audit_pass, issues);
    }
}
