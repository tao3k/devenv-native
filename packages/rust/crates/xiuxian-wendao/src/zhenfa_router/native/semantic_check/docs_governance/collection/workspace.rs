//! Workspace-wide package documentation governance scanner.

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::hidden_links::collect_workspace_canonical_doc_issues;
use super::package_docs::collect_doc_governance_issues;
use crate::parsers::docs_governance::types::{LineSlice, LinksLine};
use crate::parsers::docs_governance::{
    collect_index_body_links, collect_lines, parse_footer_block, parse_relations_links_line,
};
use crate::zhenfa_router::native::semantic_check::docs_governance::rendering::{
    link_target, plan_index_footer_block_insertion, plan_index_relations_block_insertion,
    plan_index_section_link_insertion, render_package_docs_index, render_section_landing_page,
    standard_section_specs,
};
use crate::zhenfa_router::native::semantic_check::docs_governance::scope::{
    scope_matches, scope_matches_doc,
};
use crate::zhenfa_router::native::semantic_check::docs_governance::types::{
    MISSING_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE, MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_RELATIONS_BLOCK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_SECTION_LINK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_SECTION_LANDING_ISSUE_TYPE, MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE,
    SectionSpec,
};
use crate::zhenfa_router::native::semantic_check::{IssueLocation, SemanticIssue};

/// Collects workspace-wide doc governance issues.
#[must_use]
pub fn collect_workspace_doc_governance_issues(
    root: &Path,
    scope: Option<&str>,
) -> Vec<SemanticIssue> {
    let mut issues = collect_workspace_package_doc_governance_issues(root, scope);
    issues.extend(collect_workspace_canonical_doc_issues(root, scope));
    issues
}

fn collect_workspace_package_doc_governance_issues(
    root: &Path,
    scope: Option<&str>,
) -> Vec<SemanticIssue> {
    let crates_dir = root.join("packages").join("rust").join("crates");
    workspace_crate_dirs(&crates_dir)
        .into_iter()
        .flat_map(|package_dir| collect_package_docs_governance_issues(&package_dir, scope))
        .collect()
}

fn workspace_crate_dirs(crates_dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(crates_dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| is_workspace_crate_dir(path))
        .collect()
}

struct PackageDocsContext<'a> {
    package_dir: &'a Path,
    docs_dir: PathBuf,
    index_path: PathBuf,
    crate_name: String,
    scope: Option<&'a str>,
}

impl<'a> PackageDocsContext<'a> {
    fn new(package_dir: &'a Path, scope: Option<&'a str>) -> Self {
        let docs_dir = package_dir.join("docs");
        let index_path = docs_dir.join("index.md");
        let crate_name = package_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self {
            package_dir,
            docs_dir,
            index_path,
            crate_name,
            scope,
        }
    }

    fn scope_matches_index(&self) -> bool {
        scope_matches(
            self.scope,
            self.package_dir,
            &self.docs_dir,
            &self.index_path,
        )
    }

    fn scope_matches_doc(&self, path: &Path) -> bool {
        scope_matches_doc(self.scope, self.package_dir, &self.docs_dir, path)
    }
}

fn collect_package_docs_governance_issues(
    package_dir: &Path,
    scope: Option<&str>,
) -> Vec<SemanticIssue> {
    let context = PackageDocsContext::new(package_dir, scope);
    if !context.docs_dir.is_dir() {
        return missing_package_docs_tree_issue(&context)
            .into_iter()
            .collect();
    }

    let mut issues = collect_package_markdown_doc_issues(&context);
    if context.scope_matches_index() {
        issues.extend(collect_package_docs_index_issues(&context));
    }
    issues
}

fn missing_package_docs_tree_issue(context: &PackageDocsContext<'_>) -> Option<SemanticIssue> {
    context.scope_matches_index().then(|| SemanticIssue {
        severity: "warning".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE.to_string(),
        doc: context.index_path.to_string_lossy().into_owned(),
        node_id: context.crate_name.clone(),
        message: format!(
            "Missing documentation tree for package `{}`. Expected at `docs/`.",
            context.crate_name
        ),
        location: None,
        suggestion: Some(render_package_docs_index(
            &context.crate_name,
            &context.index_path.to_string_lossy(),
            &context.docs_dir,
        )),
        fuzzy_suggestion: None,
    })
}

fn collect_package_markdown_doc_issues(context: &PackageDocsContext<'_>) -> Vec<SemanticIssue> {
    markdown_doc_paths(&context.docs_dir)
        .into_iter()
        .filter(|path| context.scope_matches_doc(path))
        .filter_map(|path| read_doc_governance_issues(&path))
        .flatten()
        .collect()
}

fn markdown_doc_paths(docs_dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(docs_dir)
        .into_iter()
        .flatten()
        .map(walkdir::DirEntry::into_path)
        .filter(|path| is_markdown_file(path))
        .collect()
}

fn is_markdown_file(path: &Path) -> bool {
    path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("md")
}

fn read_doc_governance_issues(path: &Path) -> Option<Vec<SemanticIssue>> {
    let content = fs::read_to_string(path).ok()?;
    Some(collect_doc_governance_issues(
        &path.to_string_lossy(),
        &content,
    ))
}

fn collect_package_docs_index_issues(context: &PackageDocsContext<'_>) -> Vec<SemanticIssue> {
    if !context.index_path.is_file() {
        return vec![missing_package_docs_index_issue(context)];
    }

    let Ok(index_content) = fs::read_to_string(&context.index_path) else {
        return Vec::new();
    };
    collect_existing_package_docs_index_issues(context, &index_content)
}

fn missing_package_docs_index_issue(context: &PackageDocsContext<'_>) -> SemanticIssue {
    SemanticIssue {
        severity: "error".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE.to_string(),
        doc: context.index_path.to_string_lossy().into_owned(),
        node_id: context.crate_name.clone(),
        message: format!(
            "Missing documentation index for package `{}`. Expected at `docs/index.md`.",
            context.crate_name
        ),
        location: None,
        suggestion: Some(render_package_docs_index(
            &context.crate_name,
            &context.index_path.to_string_lossy(),
            &context.docs_dir,
        )),
        fuzzy_suggestion: None,
    }
}

fn collect_existing_package_docs_index_issues(
    context: &PackageDocsContext<'_>,
    index_content: &str,
) -> Vec<SemanticIssue> {
    let index_lines = collect_lines(index_content);
    let body_links = collect_index_body_links(&index_lines);
    let mut issues = collect_footer_block_issues(context, index_content, &index_lines);
    issues.extend(collect_relations_block_issues(
        context,
        index_content,
        &index_lines,
        &body_links,
    ));
    issues.extend(collect_standard_section_issues(
        context,
        index_content,
        &body_links,
    ));
    issues
}

fn collect_footer_block_issues(
    context: &PackageDocsContext<'_>,
    index_content: &str,
    index_lines: &[LineSlice<'_>],
) -> Vec<SemanticIssue> {
    if parse_footer_block(index_lines).is_some() {
        return Vec::new();
    }
    let (location, suggestion) = plan_index_footer_block_insertion(index_content);
    vec![SemanticIssue {
        severity: "warning".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE.to_string(),
        doc: context.index_path.to_string_lossy().into_owned(),
        node_id: context.crate_name.clone(),
        message: "Missing mandatory :FOOTER: block in documentation index".to_string(),
        location: Some(location),
        suggestion: Some(suggestion),
        fuzzy_suggestion: None,
    }]
}

fn collect_relations_block_issues(
    context: &PackageDocsContext<'_>,
    index_content: &str,
    index_lines: &[LineSlice<'_>],
    body_links: &[String],
) -> Vec<SemanticIssue> {
    if body_links.is_empty() {
        return Vec::new();
    }

    match parse_relations_links_line(index_lines) {
        None => vec![missing_relations_block_issue(
            context,
            index_content,
            body_links,
        )],
        Some(links) => missing_relation_link_issue(context, body_links, &links)
            .into_iter()
            .collect(),
    }
}

fn missing_relations_block_issue(
    context: &PackageDocsContext<'_>,
    index_content: &str,
    body_links: &[String],
) -> SemanticIssue {
    let (location, suggestion) = plan_index_relations_block_insertion(index_content, body_links);
    SemanticIssue {
        severity: "warning".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_INDEX_RELATIONS_BLOCK_ISSUE_TYPE.to_string(),
        doc: context.index_path.to_string_lossy().into_owned(),
        node_id: context.crate_name.clone(),
        message: format!(
            "Missing mandatory :RELATIONS: block in documentation index with body links: {}",
            format_wikilinks(body_links)
        ),
        location: Some(location),
        suggestion: Some(suggestion),
        fuzzy_suggestion: None,
    }
}

fn missing_relation_link_issue(
    context: &PackageDocsContext<'_>,
    body_links: &[String],
    links: &LinksLine<'_>,
) -> Option<SemanticIssue> {
    let missing_in_relations = body_links
        .iter()
        .filter(|body_link| !links.value.contains(&format!("[[{body_link}]]")))
        .cloned()
        .collect::<Vec<_>>();

    (!missing_in_relations.is_empty()).then(|| SemanticIssue {
        severity: "warning".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE.to_string(),
        doc: context.index_path.to_string_lossy().into_owned(),
        node_id: context.crate_name.clone(),
        message: format!(
            "Documentation links missing from :RELATIONS: block: {}",
            format_wikilinks(&missing_in_relations)
        ),
        location: Some(IssueLocation {
            line: links.line,
            heading_path: "Index Relations".to_string(),
            byte_range: Some((links.value_start, links.value_end)),
        }),
        suggestion: Some(format_wikilinks(body_links)),
        fuzzy_suggestion: None,
    })
}

fn collect_standard_section_issues(
    context: &PackageDocsContext<'_>,
    index_content: &str,
    body_links: &[String],
) -> Vec<SemanticIssue> {
    standard_section_specs(&context.crate_name)
        .iter()
        .flat_map(|spec| standard_section_issues(context, index_content, body_links, spec))
        .collect()
}

fn standard_section_issues(
    context: &PackageDocsContext<'_>,
    index_content: &str,
    body_links: &[String],
    spec: &SectionSpec,
) -> Vec<SemanticIssue> {
    let section_dir = context.docs_dir.join(spec.section_name);
    let section_path = context.docs_dir.join(&spec.relative_path);

    if !context.scope_matches_doc(&section_path) {
        return Vec::new();
    }

    [
        missing_section_landing_issue(context, spec, &section_path),
        missing_section_index_link_issue(context, index_content, body_links, spec),
        missing_section_directory_issue(context, spec, &section_dir),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn missing_section_landing_issue(
    context: &PackageDocsContext<'_>,
    spec: &SectionSpec,
    section_path: &Path,
) -> Option<SemanticIssue> {
    (!section_path.is_file()).then(|| SemanticIssue {
        severity: "warning".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_SECTION_LANDING_ISSUE_TYPE.to_string(),
        doc: section_path.to_string_lossy().into_owned(),
        node_id: context.crate_name.clone(),
        message: format!(
            "Missing mandatory section landing page for `{}` quadrant.",
            spec.section_name
        ),
        location: None,
        suggestion: Some(render_section_landing_page(
            &context.crate_name,
            context.package_dir,
            &section_path.to_string_lossy(),
            spec,
        )),
        fuzzy_suggestion: None,
    })
}

fn missing_section_index_link_issue(
    context: &PackageDocsContext<'_>,
    index_content: &str,
    body_links: &[String],
    spec: &SectionSpec,
) -> Option<SemanticIssue> {
    let target = link_target(&spec.relative_path);
    (!body_links.iter().any(|link| link == &target)).then(|| {
        let (location, suggestion) =
            plan_index_section_link_insertion(index_content, spec, &target);
        SemanticIssue {
            severity: "warning".to_string(),
            issue_type: MISSING_PACKAGE_DOCS_INDEX_SECTION_LINK_ISSUE_TYPE.to_string(),
            doc: context.index_path.to_string_lossy().into_owned(),
            node_id: context.crate_name.clone(),
            message: format!(
                "Mandatory section `{}` is not linked in documentation index.",
                spec.section_name
            ),
            location: Some(location),
            suggestion: Some(suggestion),
            fuzzy_suggestion: None,
        }
    })
}

fn missing_section_directory_issue(
    context: &PackageDocsContext<'_>,
    spec: &SectionSpec,
    section_dir: &Path,
) -> Option<SemanticIssue> {
    (!section_dir.is_dir()).then(|| SemanticIssue {
        severity: "warning".to_string(),
        issue_type: MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE.to_string(),
        doc: section_dir.to_string_lossy().into_owned(),
        node_id: context.crate_name.clone(),
        message: format!(
            "Missing directory tree for `{}` documentation quadrant.",
            spec.section_name
        ),
        location: None,
        suggestion: None,
        fuzzy_suggestion: None,
    })
}

fn format_wikilinks(links: &[String]) -> String {
    links
        .iter()
        .map(|link| format!("[[{link}]]"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn is_workspace_crate_dir(path: &Path) -> bool {
    path.join("Cargo.toml").is_file()
}
