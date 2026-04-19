use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use walkdir::{DirEntry, WalkDir};
use xiuxian_wendao_parsers::{
    MarkdownNote, MarkdownOutlineDocument, MarkdownSection, MarkdownTargetOccurrence,
    MarkdownTargetOccurrenceKind, parse_markdown_note, parse_markdown_outline,
};

use super::config::configured_ignore_dirs;
use super::types::{
    DocsPageIndexDocumentsResult, DocsPageIndexTreesResult, ProjectedPageIndexDocument,
    ProjectedPageIndexLink, ProjectedPageIndexNode, ProjectedPageIndexSection,
    ProjectedPageIndexTree, ProjectionPageKind,
};
use super::{GetCommand, GetScopeArgs};
use crate::{ClientContext, CommandOutcome, OutputFormat};

const LOCAL_SCOPE_REPO_ID: &str = "local";
const DEFAULT_LOCAL_IGNORED_DIRS: &[&str] = &[
    ".git",
    ".devenv",
    ".direnv",
    ".cache",
    ".config",
    ".data",
    ".run",
    ".bin",
    "node_modules",
    "target",
];

pub(crate) fn run_command(command: &GetCommand, context: &ClientContext) -> Result<CommandOutcome> {
    match command {
        GetCommand::Toc(args) => handle_toc(args, context),
        GetCommand::PageIndex(args) => handle_page_index(args, context),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeTargetKind {
    Directory,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalScopeTarget {
    path: PathBuf,
    kind: ScopeTargetKind,
}

impl CanonicalScopeTarget {
    fn display_base(&self) -> &Path {
        match self.kind {
            ScopeTargetKind::Directory => self.path.as_path(),
            ScopeTargetKind::File => self.path.parent().unwrap_or(self.path.as_path()),
        }
    }
}

fn handle_toc(args: &GetScopeArgs, context: &ClientContext) -> Result<CommandOutcome> {
    let scope = canonical_scope_target(context.root(), args.target.as_path())?;
    let ignore_dir_names = resolved_ignore_dir_names(args, context)?;
    let result =
        build_local_toc_documents_with_ignore(&scope, context.root(), ignore_dir_names.as_slice())?;
    emit_toc_output(&result, context.output())?;
    Ok(CommandOutcome::success())
}

fn handle_page_index(args: &GetScopeArgs, context: &ClientContext) -> Result<CommandOutcome> {
    let scope = canonical_scope_target(context.root(), args.target.as_path())?;
    let ignore_dir_names = resolved_ignore_dir_names(args, context)?;
    let result = build_local_page_index_trees_with_ignore(
        &scope,
        context.root(),
        ignore_dir_names.as_slice(),
    )?;
    emit_page_index_output(&result, context.output())?;
    Ok(CommandOutcome::success())
}

fn canonical_scope_target(cwd: &Path, target: &Path) -> Result<CanonicalScopeTarget> {
    let requested_target = if target.is_absolute() {
        target.to_path_buf()
    } else {
        cwd.join(target)
    };
    let canonical = fs::canonicalize(requested_target.as_path()).with_context(|| {
        format!(
            "failed to resolve target scope `{}`",
            requested_target.display()
        )
    })?;
    let kind = if canonical.is_dir() {
        ScopeTargetKind::Directory
    } else if canonical.is_file() {
        ScopeTargetKind::File
    } else {
        bail!(
            "target scope `{}` is neither a file nor a directory",
            canonical.display()
        );
    };
    Ok(CanonicalScopeTarget {
        path: canonical,
        kind,
    })
}

#[cfg(test)]
fn build_local_toc_documents(
    scope: &CanonicalScopeTarget,
    client_root: &Path,
) -> Result<DocsPageIndexDocumentsResult> {
    let default_ignored_dirs = default_ignore_dir_names();
    build_local_toc_documents_with_ignore(scope, client_root, default_ignored_dirs.as_slice())
}

fn build_local_toc_documents_with_ignore(
    scope: &CanonicalScopeTarget,
    client_root: &Path,
    ignore_dir_names: &[String],
) -> Result<DocsPageIndexDocumentsResult> {
    let canonical_client_root = fs::canonicalize(client_root).ok();
    let target_paths = collect_local_markdown_targets(scope, ignore_dir_names)?;
    let mut documents = materialize_local_targets_in_parallel(target_paths, |path| {
        build_local_projected_page_index_document(
            path.as_path(),
            scope,
            canonical_client_root.as_deref(),
        )
    })?;
    documents.sort_by(|left, right| left.doc_id.cmp(&right.doc_id));
    Ok(DocsPageIndexDocumentsResult {
        repo_id: LOCAL_SCOPE_REPO_ID.to_string(),
        documents,
    })
}

#[cfg(test)]
fn build_local_page_index_trees(
    scope: &CanonicalScopeTarget,
    client_root: &Path,
) -> Result<DocsPageIndexTreesResult> {
    let default_ignored_dirs = default_ignore_dir_names();
    build_local_page_index_trees_with_ignore(scope, client_root, default_ignored_dirs.as_slice())
}

fn build_local_page_index_trees_with_ignore(
    scope: &CanonicalScopeTarget,
    client_root: &Path,
    ignore_dir_names: &[String],
) -> Result<DocsPageIndexTreesResult> {
    let canonical_client_root = fs::canonicalize(client_root).ok();
    let target_paths = collect_local_markdown_targets(scope, ignore_dir_names)?;
    let mut trees = materialize_local_targets_in_parallel(target_paths, |path| {
        build_local_projected_page_index_tree(
            path.as_path(),
            scope,
            canonical_client_root.as_deref(),
        )
    })?;
    trees.sort_by(|left, right| left.doc_id.cmp(&right.doc_id));
    Ok(DocsPageIndexTreesResult {
        repo_id: LOCAL_SCOPE_REPO_ID.to_string(),
        trees,
    })
}

fn materialize_local_targets_in_parallel<T, F>(paths: Vec<PathBuf>, build: F) -> Result<Vec<T>>
where
    T: Send,
    F: Fn(&PathBuf) -> Result<T> + Sync + Send,
{
    paths
        .into_par_iter()
        .map(|path| build(&path))
        .collect::<Vec<_>>()
        .into_iter()
        .collect()
}

fn resolved_ignore_dir_names(args: &GetScopeArgs, context: &ClientContext) -> Result<Vec<String>> {
    let mut ignore_dir_names = default_ignore_dir_names();
    ignore_dir_names.extend(configured_ignore_dirs(context)?);
    ignore_dir_names.extend(
        args.ignore_dirs
            .iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
    );
    ignore_dir_names.sort();
    ignore_dir_names.dedup();
    Ok(ignore_dir_names)
}

fn default_ignore_dir_names() -> Vec<String> {
    DEFAULT_LOCAL_IGNORED_DIRS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

fn build_local_projected_page_index_document(
    file_path: &Path,
    scope: &CanonicalScopeTarget,
    canonical_client_root: Option<&Path>,
) -> Result<ProjectedPageIndexDocument> {
    let outline = load_local_outline(file_path)?;
    let path = display_absolute_path(file_path);
    let doc_id = display_scope_path(file_path, canonical_client_root, scope);
    Ok(ProjectedPageIndexDocument {
        repo_id: LOCAL_SCOPE_REPO_ID.to_string(),
        page_id: local_page_id(doc_id.as_str()),
        path,
        doc_id,
        title: outline.title.clone(),
        sections: build_local_outline_sections(&outline),
    })
}

fn build_local_projected_page_index_tree(
    file_path: &Path,
    scope: &CanonicalScopeTarget,
    canonical_client_root: Option<&Path>,
) -> Result<ProjectedPageIndexTree> {
    let note = load_local_note(file_path)?;
    let path = display_absolute_path(file_path);
    let doc_id = display_scope_path(file_path, canonical_client_root, scope);
    let page_id = local_page_id(doc_id.as_str());
    let title = note.document.core.title.clone();
    let roots = build_local_page_index_nodes(doc_id.as_str(), title.as_str(), &note);
    Ok(ProjectedPageIndexTree {
        repo_id: LOCAL_SCOPE_REPO_ID.to_string(),
        page_id,
        kind: local_projection_kind(doc_id.as_str(), note.document.core.doc_type.as_deref()),
        path,
        doc_id,
        title,
        root_count: roots.len(),
        roots,
    })
}

fn load_local_outline(file_path: &Path) -> Result<MarkdownOutlineDocument> {
    let markdown = fs::read_to_string(file_path)
        .with_context(|| format!("failed to read markdown document `{}`", file_path.display()))?;
    Ok(parse_markdown_outline(
        markdown.as_str(),
        fallback_title(file_path),
    ))
}

fn load_local_note(file_path: &Path) -> Result<MarkdownNote> {
    let markdown = fs::read_to_string(file_path)
        .with_context(|| format!("failed to read markdown document `{}`", file_path.display()))?;
    Ok(parse_markdown_note(
        markdown.as_str(),
        fallback_title(file_path),
    ))
}

fn build_local_outline_sections(
    outline: &MarkdownOutlineDocument,
) -> Vec<ProjectedPageIndexSection> {
    let mut sections = Vec::new();
    let mut heading_stack = Vec::<String>::new();

    if outline.headings.is_empty() {
        sections.push(ProjectedPageIndexSection {
            heading_path: outline.title.clone(),
            title: outline.title.clone(),
            level: 1,
            line_range: (1, outline.line_count.max(1)),
            attributes: Vec::new(),
        });
        return sections;
    }

    for heading in &outline.headings {
        if heading_stack.len() >= heading.level {
            heading_stack.truncate(heading.level.saturating_sub(1));
        }
        heading_stack.push(heading.title.clone());
        sections.push(ProjectedPageIndexSection {
            heading_path: heading_stack.join(" / "),
            title: heading.title.clone(),
            level: heading.level,
            line_range: heading.line_range,
            attributes: Vec::new(),
        });
    }

    sections
}

fn build_local_page_index_nodes(
    doc_id: &str,
    doc_title: &str,
    note: &MarkdownNote,
) -> Vec<ProjectedPageIndexNode> {
    let mut roots = Vec::new();
    let mut stack = Vec::new();
    let mut slug_counts = HashMap::new();
    for section in &note.core.sections {
        let level = effective_markdown_section_level(section);
        while stack
            .last()
            .is_some_and(|parent: &ProjectedPageIndexNode| parent.level >= level)
        {
            close_last_open_local_node(&mut roots, &mut stack);
        }

        stack.push(build_local_page_index_node(
            doc_id,
            effective_markdown_section_title(section, doc_title),
            effective_markdown_section_path(section, doc_title),
            (section.line_start(), section.line_end()),
            collect_section_links(section, note.core.targets.as_slice()),
            level,
            &mut slug_counts,
        ));
    }

    while !stack.is_empty() {
        close_last_open_local_node(&mut roots, &mut stack);
    }

    roots
}

fn build_local_page_index_node(
    doc_id: &str,
    title: String,
    structural_path: Vec<String>,
    line_range: (usize, usize),
    links: Vec<ProjectedPageIndexLink>,
    level: usize,
    slug_counts: &mut HashMap<String, usize>,
) -> ProjectedPageIndexNode {
    let slug = effective_outline_slug(structural_path.as_slice(), title.as_str());
    let sequence = slug_counts.entry(slug.clone()).or_insert(0);
    *sequence += 1;
    let node_id = if *sequence == 1 {
        format!("{doc_id}#{slug}")
    } else {
        format!("{doc_id}#{slug}-{}", *sequence - 1)
    };

    ProjectedPageIndexNode {
        node_id,
        title,
        level,
        structural_path,
        line_range,
        token_count: 0,
        is_thinned: true,
        text: String::new(),
        summary: None,
        links,
        children: Vec::new(),
    }
}

fn close_last_open_local_node(
    roots: &mut Vec<ProjectedPageIndexNode>,
    stack: &mut Vec<ProjectedPageIndexNode>,
) {
    let Some(node) = stack.pop() else {
        return;
    };
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        roots.push(node);
    }
}

fn effective_markdown_section_title(section: &MarkdownSection, doc_title: &str) -> String {
    if !section.heading_title().trim().is_empty() {
        return section.heading_title().to_string();
    }
    doc_title.to_string()
}

fn effective_markdown_section_level(section: &MarkdownSection) -> usize {
    effective_section_level(section.heading_level().max(1))
}

fn effective_markdown_section_path(section: &MarkdownSection, doc_title: &str) -> Vec<String> {
    if !section.heading_path().trim().is_empty() {
        return section
            .heading_path()
            .split(" / ")
            .map(ToString::to_string)
            .collect();
    }
    vec![doc_title.to_string()]
}

fn collect_section_links(
    section: &MarkdownSection,
    targets: &[MarkdownTargetOccurrence],
) -> Vec<ProjectedPageIndexLink> {
    let mut links = Vec::new();
    for occurrence in targets {
        if !matches!(
            occurrence.kind,
            MarkdownTargetOccurrenceKind::MarkdownLink
                | MarkdownTargetOccurrenceKind::MarkdownImage
                | MarkdownTargetOccurrenceKind::WikiLink
                | MarkdownTargetOccurrenceKind::WikiEmbed
        ) {
            continue;
        }
        if occurrence.line_range.0 < section.line_start()
            || occurrence.line_range.0 > section.line_end()
        {
            continue;
        }
        let link = ProjectedPageIndexLink {
            kind: local_target_kind_label(occurrence.kind).to_string(),
            target: occurrence.target.clone(),
            surface: local_target_surface(occurrence),
        };
        if !links.contains(&link) {
            links.push(link);
        }
    }
    links
}

fn local_target_kind_label(kind: MarkdownTargetOccurrenceKind) -> &'static str {
    match kind {
        MarkdownTargetOccurrenceKind::MarkdownLink => "markdown_link",
        MarkdownTargetOccurrenceKind::MarkdownImage => "markdown_image",
        MarkdownTargetOccurrenceKind::WikiLink => "wiki_link",
        MarkdownTargetOccurrenceKind::WikiEmbed => "wiki_embed",
    }
}

fn local_target_surface(occurrence: &MarkdownTargetOccurrence) -> String {
    if occurrence.surface.trim().is_empty() {
        occurrence.target.clone()
    } else {
        occurrence.surface.clone()
    }
}

fn effective_section_level(level: usize) -> usize {
    level.clamp(1, 6)
}

fn effective_outline_slug(structural_path: &[String], title: &str) -> String {
    let raw = if structural_path.is_empty() {
        title.to_string()
    } else {
        structural_path.join(" / ").to_ascii_lowercase()
    };
    let slug = raw
        .chars()
        .map(|ch| match ch {
            'a'..='z' | '0'..='9' => ch,
            'A'..='Z' => ch.to_ascii_lowercase(),
            _ => '-',
        })
        .collect::<String>();
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "overview".to_string()
    } else {
        slug
    }
}

fn collect_local_markdown_targets(
    scope: &CanonicalScopeTarget,
    ignore_dir_names: &[String],
) -> Result<Vec<PathBuf>> {
    match scope.kind {
        ScopeTargetKind::Directory => {
            collect_local_markdown_files(scope.path.as_path(), ignore_dir_names)
        }
        ScopeTargetKind::File => {
            if !is_markdown_path(scope.path.as_path()) {
                bail!(
                    "target scope `{}` is not a Markdown document",
                    scope.path.display()
                );
            }
            Ok(vec![scope.path.clone()])
        }
    }
}

fn collect_local_markdown_files(
    scope_root: &Path,
    ignore_dir_names: &[String],
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let walker = WalkDir::new(scope_root)
        .into_iter()
        .filter_entry(|entry| !should_skip_local_entry(entry, scope_root, ignore_dir_names));
    for entry in walker {
        let entry = entry
            .with_context(|| format!("failed to walk docs scope `{}`", scope_root.display()))?;
        if entry.file_type().is_file() && is_markdown_path(entry.path()) {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

fn should_skip_local_entry(
    entry: &DirEntry,
    scope_root: &Path,
    ignore_dir_names: &[String],
) -> bool {
    if entry.path() == scope_root {
        return false;
    }
    entry.file_type().is_dir()
        && entry
            .file_name()
            .to_str()
            .is_some_and(|name| ignore_dir_names.iter().any(|candidate| candidate == name))
}

fn is_markdown_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(std::ffi::OsStr::to_str),
        Some("md" | "markdown" | "mdx")
    )
}

fn display_scope_path(
    path: &Path,
    client_root: Option<&Path>,
    scope: &CanonicalScopeTarget,
) -> String {
    if let Some(canonical_client_root) = client_root
        && let Ok(stripped) = path.strip_prefix(canonical_client_root)
    {
        return stripped.to_string_lossy().replace('\\', "/");
    }
    path.strip_prefix(scope.display_base())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn display_absolute_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn fallback_title(path: &Path) -> &str {
    path.file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("document")
}

fn local_page_id(path: &str) -> String {
    format!("local:{path}")
}

fn local_projection_kind(path: &str, doc_type: Option<&str>) -> ProjectionPageKind {
    let normalized = format!(
        "{path} {}",
        doc_type.unwrap_or_default().trim().to_ascii_lowercase()
    )
    .to_ascii_lowercase();
    if normalized.contains("tutorial") {
        ProjectionPageKind::Tutorial
    } else if normalized.contains("reference") || normalized.contains("api") {
        ProjectionPageKind::Reference
    } else if normalized.contains("guide")
        || normalized.contains("howto")
        || normalized.contains("how-to")
    {
        ProjectionPageKind::HowTo
    } else {
        ProjectionPageKind::Explanation
    }
}

fn emit_toc_output(result: &DocsPageIndexDocumentsResult, output: OutputFormat) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => render_toc_markdown(result),
        OutputFormat::Json => {
            serde_json::to_string(result).context("failed to serialize get output as JSON")?
        }
        OutputFormat::Pretty => serde_json::to_string_pretty(result)
            .context("failed to serialize get output as JSON")?,
    };
    println!("{rendered}");
    Ok(())
}

fn emit_page_index_output(result: &DocsPageIndexTreesResult, output: OutputFormat) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => render_page_index_markdown(result),
        OutputFormat::Json => {
            serde_json::to_string(result).context("failed to serialize get output as JSON")?
        }
        OutputFormat::Pretty => serde_json::to_string_pretty(result)
            .context("failed to serialize get output as JSON")?,
    };
    println!("{rendered}");
    Ok(())
}

fn render_toc_markdown(result: &DocsPageIndexDocumentsResult) -> String {
    if result.documents.is_empty() {
        return "_No documents matched._".to_string();
    }

    let mut lines = Vec::new();
    for (index, document) in result.documents.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.push(format!("path: {}", document.path));
        lines.push(format!(
            "title: {} | sections: {}",
            document.title,
            document.sections.len()
        ));
        for section in &document.sections {
            let title = if section.title.trim().is_empty() {
                "(untitled)"
            } else {
                section.title.as_str()
            };
            lines.push(render_heading_with_range(
                section.level,
                title,
                section.line_range,
            ));
        }
    }
    lines.join("\n")
}

fn render_heading_with_range(level: usize, title: &str, line_range: (usize, usize)) -> String {
    let marker_count = effective_section_level(level);
    format!(
        "{} {} -> [L{} {}-{}]",
        "#".repeat(marker_count),
        title,
        effective_section_level(level),
        line_range.0,
        line_range.1
    )
}

fn render_page_index_markdown(result: &DocsPageIndexTreesResult) -> String {
    if result.trees.is_empty() {
        return "_No documents matched._".to_string();
    }

    let mut lines = Vec::new();
    for (index, tree) in result.trees.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.push(format!("path: {}", tree.path));
        lines.push(format!(
            "kind: {:?} | roots: {} | nodes: {} | links: {} | embeds: {}",
            tree.kind,
            tree.root_count,
            count_tree_nodes(tree.roots.as_slice()),
            count_tree_links(tree.roots.as_slice()),
            count_tree_embeds(tree.roots.as_slice())
        ));
        for root in &tree.roots {
            push_tree_markdown_lines(&mut lines, root);
        }
    }
    lines.join("\n")
}

fn push_tree_markdown_lines(lines: &mut Vec<String>, node: &ProjectedPageIndexNode) {
    lines.push(render_heading_with_range(
        node.level,
        node.title.as_str(),
        node.line_range,
    ));
    let section_links = node
        .links
        .iter()
        .filter(|link| !projected_page_index_link_is_embed(link))
        .cloned()
        .collect::<Vec<_>>();
    if !section_links.is_empty() {
        lines.push(format!(
            "links: {}",
            render_node_link_surfaces(section_links.as_slice())
        ));
    }
    let section_embeds = node
        .links
        .iter()
        .filter(|link| projected_page_index_link_is_embed(link))
        .cloned()
        .collect::<Vec<_>>();
    if !section_embeds.is_empty() {
        lines.push(format!(
            "embeds: {}",
            render_node_link_surfaces(section_embeds.as_slice())
        ));
    }
    for child in &node.children {
        push_tree_markdown_lines(lines, child);
    }
}

fn count_tree_nodes(nodes: &[ProjectedPageIndexNode]) -> usize {
    nodes
        .iter()
        .map(|node| 1 + count_tree_nodes(node.children.as_slice()))
        .sum()
}

fn count_tree_links(nodes: &[ProjectedPageIndexNode]) -> usize {
    nodes
        .iter()
        .map(|node| {
            node.links
                .iter()
                .filter(|link| !projected_page_index_link_is_embed(link))
                .count()
                + count_tree_links(node.children.as_slice())
        })
        .sum()
}

fn count_tree_embeds(nodes: &[ProjectedPageIndexNode]) -> usize {
    nodes
        .iter()
        .map(|node| {
            node.links
                .iter()
                .filter(|link| projected_page_index_link_is_embed(link))
                .count()
                + count_tree_embeds(node.children.as_slice())
        })
        .sum()
}

fn projected_page_index_link_is_embed(link: &ProjectedPageIndexLink) -> bool {
    matches!(link.kind.as_str(), "markdown_image" | "wiki_embed")
}

fn render_node_link_surfaces(links: &[ProjectedPageIndexLink]) -> String {
    links
        .iter()
        .map(|link| {
            if link.surface.trim().is_empty() {
                link.target.clone()
            } else {
                link.surface.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[path = "../../tests/unit/get_run.rs"]
mod tests;
