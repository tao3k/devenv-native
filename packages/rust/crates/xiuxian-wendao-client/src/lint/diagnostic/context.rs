use super::facts::DiagnosticFacts;
use super::link::{
    LinkIssueContext, TargetMetadata, split_target_path_and_fragment, split_target_path_and_heading,
};
use super::text::normalize_hint;
use crate::lint::discovery::first_transient_repo_dir;
use crate::lint::{MarkdownLintIssue, contract::diagnostic_contract};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use xiuxian_wendao_parsers::{
    MarkdownSection, MarkdownSyntaxLintIssue, MarkdownTargetOccurrenceKind, extract_blocks,
    parse_markdown_note,
};

pub(in crate::lint) struct DiagnosticContext<'a> {
    root: &'a Path,
    root_canonical: Option<PathBuf>,
    document_cache: HashMap<PathBuf, Option<LocalTargetDocumentIndex>>,
}

pub(in crate::lint) enum LocalTargetResolution {
    Resolved(PathBuf),
    OutsideRoot(PathBuf),
    Missing,
}

pub(in crate::lint) enum LocalTargetFragmentResolution {
    NotAddressed,
    Resolved,
    TransientDir,
    MissingHeading {
        fragment: String,
        target_title: Option<String>,
    },
    MissingBlock {
        fragment: String,
        target_title: Option<String>,
    },
    MissingTarget,
    OutsideRoot,
}

#[derive(Clone)]
struct LocalTargetDocumentIndex {
    title: Option<String>,
    heading_addresses: HashSet<String>,
    block_addresses: HashSet<String>,
}

impl<'a> DiagnosticContext<'a> {
    pub(in crate::lint) fn new(root: &'a Path) -> Self {
        Self {
            root,
            root_canonical: fs::canonicalize(root).ok(),
            document_cache: HashMap::new(),
        }
    }

    pub(in crate::lint) fn build_issue(
        &mut self,
        source_path: &Path,
        markdown: &str,
        issue: MarkdownSyntaxLintIssue,
    ) -> MarkdownLintIssue {
        let source = source_line(markdown, issue.line).map(ToOwned::to_owned);
        let link = source
            .as_deref()
            .and_then(|line| LinkIssueContext::from_source(issue.code, line, issue.column));
        let target_metadata = link
            .as_ref()
            .map(|context| self.resolve_target_metadata(source_path, context.target.as_str()));
        let duplicates_heading = link
            .as_ref()
            .and_then(|context| context.label.as_deref())
            .zip(
                target_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.heading.as_deref()),
            )
            .is_some_and(|(label, heading)| normalize_hint(label) == normalize_hint(heading));
        let facts = DiagnosticFacts::from_parser_issue(
            issue,
            source,
            link,
            target_metadata,
            duplicates_heading,
        );
        diagnostic_contract().render_issue(&facts)
    }

    pub(in crate::lint) fn inspect_local_target(
        &self,
        source_path: &Path,
        raw_target: &str,
        target_kind: MarkdownTargetOccurrenceKind,
    ) -> LocalTargetResolution {
        self.resolve_target_path(source_path, raw_target, target_kind)
    }

    pub(in crate::lint) fn root(&self) -> &Path {
        self.root
    }

    pub(in crate::lint) fn inspect_local_target_transient_dir(
        &self,
        resolved_path: &Path,
    ) -> Option<&'static str> {
        self.strip_root_prefix(resolved_path)
            .and_then(first_transient_repo_dir)
    }

    pub(in crate::lint) fn inspect_local_target_fragment(
        &mut self,
        source_path: &Path,
        source_markdown: &str,
        raw_target: &str,
        target_kind: MarkdownTargetOccurrenceKind,
    ) -> LocalTargetFragmentResolution {
        let (path_target, raw_fragment) = split_target_path_and_fragment(raw_target);
        let Some(fragment) = raw_fragment else {
            return LocalTargetFragmentResolution::NotAddressed;
        };
        let index = if path_target.is_empty() {
            build_document_index(
                source_markdown,
                source_path
                    .file_stem()
                    .and_then(std::ffi::OsStr::to_str)
                    .unwrap_or("document"),
            )
        } else {
            match self.resolve_target_path(source_path, path_target.as_str(), target_kind) {
                LocalTargetResolution::Resolved(path) => {
                    if self
                        .inspect_local_target_transient_dir(path.as_path())
                        .is_some()
                    {
                        return LocalTargetFragmentResolution::TransientDir;
                    }
                    let Some(index) = self.read_document_index(path.as_path()) else {
                        return LocalTargetFragmentResolution::MissingTarget;
                    };
                    index
                }
                LocalTargetResolution::OutsideRoot(_) => {
                    return LocalTargetFragmentResolution::OutsideRoot;
                }
                LocalTargetResolution::Missing => {
                    return LocalTargetFragmentResolution::MissingTarget;
                }
            }
        };

        if fragment.starts_with('^') {
            if index
                .block_addresses
                .contains(&normalize_block_fragment(&fragment))
            {
                LocalTargetFragmentResolution::Resolved
            } else {
                LocalTargetFragmentResolution::MissingBlock {
                    fragment,
                    target_title: index.title,
                }
            }
        } else if index
            .heading_addresses
            .contains(&normalize_heading_fragment(&fragment))
        {
            LocalTargetFragmentResolution::Resolved
        } else {
            LocalTargetFragmentResolution::MissingHeading {
                fragment,
                target_title: index.title,
            }
        }
    }

    fn resolve_target_metadata(&mut self, source_path: &Path, raw_target: &str) -> TargetMetadata {
        let (path_target, heading) = split_target_path_and_heading(raw_target);
        let title = self
            .resolve_target_path(
                source_path,
                path_target.as_str(),
                MarkdownTargetOccurrenceKind::WikiLink,
            )
            .resolved_path()
            .and_then(|path| self.read_document_index(path.as_path()))
            .and_then(|index| index.title);
        TargetMetadata {
            raw: raw_target.trim().to_string(),
            heading,
            title,
        }
    }

    fn resolve_target_path(
        &self,
        source_path: &Path,
        raw_target: &str,
        target_kind: MarkdownTargetOccurrenceKind,
    ) -> LocalTargetResolution {
        let trimmed = raw_target
            .trim()
            .trim_start_matches('/')
            .trim_end_matches('/');
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("id:")
            || trimmed.starts_with("obsidian://")
            || trimmed.starts_with("wendao://")
        {
            return LocalTargetResolution::Missing;
        }
        let target = Path::new(trimmed);
        candidate_base_dirs(source_path, self.root)
            .into_iter()
            .flat_map(|base| candidate_target_paths(base.as_path(), target, target_kind))
            .find_map(|candidate| self.resolve_existing_candidate(candidate))
            .unwrap_or(LocalTargetResolution::Missing)
    }

    fn resolve_existing_candidate(&self, candidate: PathBuf) -> Option<LocalTargetResolution> {
        candidate.is_file().then(|| {
            let canonical = fs::canonicalize(candidate.as_path()).unwrap_or(candidate);
            if self.is_within_root(canonical.as_path()) {
                LocalTargetResolution::Resolved(canonical)
            } else {
                LocalTargetResolution::OutsideRoot(canonical)
            }
        })
    }

    fn read_document_index(&mut self, path: &Path) -> Option<LocalTargetDocumentIndex> {
        if let Some(cached) = self.document_cache.get(path) {
            return cached.clone();
        }

        let index = fs::read_to_string(path).ok().map(|content| {
            build_document_index(
                &content,
                path.file_stem()
                    .and_then(std::ffi::OsStr::to_str)
                    .unwrap_or("document"),
            )
        });
        self.document_cache
            .insert(path.to_path_buf(), index.clone());
        index
    }

    fn is_within_root(&self, path: &Path) -> bool {
        self.root_canonical
            .as_deref()
            .is_some_and(|root| path.starts_with(root))
            || path.starts_with(self.root)
    }

    fn strip_root_prefix<'b>(&self, path: &'b Path) -> Option<&'b Path> {
        self.root_canonical
            .as_deref()
            .and_then(|root| path.strip_prefix(root).ok())
            .or_else(|| path.strip_prefix(self.root).ok())
    }
}

impl LocalTargetResolution {
    fn resolved_path(self) -> Option<PathBuf> {
        match self {
            Self::Resolved(path) => Some(path),
            Self::OutsideRoot(_) | Self::Missing => None,
        }
    }
}

fn build_document_index(markdown: &str, fallback_title: &str) -> LocalTargetDocumentIndex {
    let note = parse_markdown_note(markdown, fallback_title);
    let title = Some(note.document.core.title.trim().to_string()).filter(|value| !value.is_empty());
    let heading_addresses = collect_heading_addresses(note.core.sections.as_slice());
    let block_addresses = collect_block_addresses(note.core.sections.as_slice());
    LocalTargetDocumentIndex {
        title,
        heading_addresses,
        block_addresses,
    }
}

fn collect_heading_addresses(sections: &[MarkdownSection]) -> HashSet<String> {
    let mut addresses = HashSet::new();
    let mut slug_counts = HashMap::<String, usize>::new();
    for section in sections {
        let title = section.heading_title().trim();
        if title.is_empty() {
            continue;
        }
        let segments = if section.heading_path().trim().is_empty() {
            vec![title.to_string()]
        } else {
            section
                .heading_path()
                .split(" / ")
                .map(str::trim)
                .filter(|segment| !segment.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        };
        for start in 0..segments.len() {
            let candidate = normalize_heading_fragment(&segments[start..].join("#"));
            if !candidate.is_empty() {
                addresses.insert(candidate);
            }
        }
        let slug = markdown_heading_slug(title);
        let sequence = slug_counts.entry(slug.clone()).or_insert(0);
        *sequence += 1;
        if *sequence == 1 {
            addresses.insert(slug.clone());
        } else {
            addresses.insert(format!("{slug}-{}", *sequence - 1));
        }
    }
    addresses
}

fn collect_block_addresses(sections: &[MarkdownSection]) -> HashSet<String> {
    sections
        .iter()
        .flat_map(section_blocks)
        .filter(|block| !block.is_code())
        .flat_map(|block| {
            block
                .content
                .lines()
                .filter_map(block_address_candidate)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn section_blocks(section: &MarkdownSection) -> Vec<xiuxian_wendao_parsers::MarkdownBlock> {
    let structural_path = section_structural_path(section);
    extract_blocks(
        section.section_text.as_str(),
        section.byte_start(),
        section.line_start(),
        structural_path.as_slice(),
    )
}

fn section_structural_path(section: &MarkdownSection) -> Vec<String> {
    if section.heading_path().trim().is_empty() {
        return Vec::new();
    }
    section
        .heading_path()
        .split(" / ")
        .map(ToString::to_string)
        .collect()
}

fn block_address_candidate(line: &str) -> Option<String> {
    let trimmed = line.trim();
    (trimmed.starts_with('^')
        && trimmed.len() > 1
        && !trimmed[1..].chars().any(char::is_whitespace))
    .then(|| normalize_block_fragment(trimmed))
}

fn normalize_heading_fragment(fragment: &str) -> String {
    fragment
        .split('#')
        .map(|segment| segment.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("#")
}

fn normalize_block_fragment(fragment: &str) -> String {
    fragment.trim().to_ascii_lowercase()
}

fn markdown_heading_slug(title: &str) -> String {
    let slug = title
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

fn source_line(markdown: &str, line: usize) -> Option<&str> {
    markdown.lines().nth(line.saturating_sub(1))
}

fn candidate_base_dirs(source_path: &Path, root: &Path) -> Vec<PathBuf> {
    let source_dir = source_path
        .parent()
        .map_or_else(|| root.to_path_buf(), Path::to_path_buf);
    let mut bases = Vec::new();
    let mut cursor = Some(source_dir.as_path());
    while let Some(path) = cursor {
        bases.push(path.to_path_buf());
        if path == root {
            break;
        }
        cursor = path.parent();
    }
    if !bases.iter().any(|path| path == root) {
        bases.push(root.to_path_buf());
    }
    bases
}

fn candidate_target_paths(
    base: &Path,
    target: &Path,
    target_kind: MarkdownTargetOccurrenceKind,
) -> Vec<PathBuf> {
    let exact = base.join(target);
    if should_try_markdown_fallback(target, target_kind) {
        vec![
            exact.clone(),
            base.join(format!("{}.md", target.to_string_lossy())),
            exact.join("index.md"),
        ]
    } else {
        vec![exact]
    }
}

fn should_try_markdown_fallback(target: &Path, target_kind: MarkdownTargetOccurrenceKind) -> bool {
    match target_kind {
        MarkdownTargetOccurrenceKind::WikiLink | MarkdownTargetOccurrenceKind::WikiEmbed => true,
        MarkdownTargetOccurrenceKind::MarkdownLink => target.extension().is_none(),
        MarkdownTargetOccurrenceKind::MarkdownImage => false,
    }
}
