//! Parser-owned docs governance parsing helpers.

use std::path::{Component, Path};
use std::sync::OnceLock;

use sha2::{Digest, Sha256};
use xiuxian_wendao_parsers::extract_wikilinks as extract_markdown_wikilinks;

use super::types::{
    FooterBlock, HiddenPathLink, IdLine, LineSlice, LinksLine, TopPropertiesDrawer,
};

/// Derives an opaque document ID from a doc path.
#[must_use]
pub fn derive_opaque_doc_id(doc_path: &str) -> String {
    let digest = Sha256::digest(normalize_doc_path(doc_path).as_bytes());
    let mut opaque_id = String::with_capacity(40);
    for byte in digest.iter().take(20) {
        use std::fmt::Write as _;

        let _ = write!(&mut opaque_id, "{byte:02x}");
    }
    opaque_id
}

/// Checks if a value is a valid opaque document ID.
#[must_use]
pub fn is_opaque_doc_id(value: &str) -> bool {
    value.len() == 40
        && value
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
}

fn normalize_doc_path(doc_path: &str) -> String {
    let normalized = doc_path.replace('\\', "/");
    normalized
        .find("packages/rust/crates/")
        .map_or(normalized.clone(), |idx| normalized[idx..].to_string())
}

/// Checks if a doc path is a package-local crate doc.
#[must_use]
pub fn is_package_local_crate_doc(doc_path: &str) -> bool {
    let path = Path::new(doc_path);
    if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return false;
    }

    let components: Vec<String> = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();

    components
        .windows(5)
        .any(|window| matches!(window, [a, b, c, _, d] if a == "packages" && b == "rust" && c == "crates" && d == "docs"))
}

/// Checks if a doc path belongs to the canonical documentation surface.
#[must_use]
pub fn is_canonical_repo_doc(doc_path: &str) -> bool {
    let path = Path::new(doc_path);
    if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return false;
    }

    let components = path_components(path);
    if components
        .iter()
        .any(|component| is_hidden_workspace_dir(component))
    {
        return false;
    }

    if is_package_local_crate_doc(doc_path) {
        return true;
    }

    if has_component_prefix(&components, &["docs"]) {
        return true;
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    match file_name {
        "AGENTS.md" | "CLAUDE.md" | "CHANGELOG.md" | "README.md" => true,
        "SKILL.md" => components.iter().any(|component| component == "skills"),
        _ => false,
    }
}

/// Collects line slices from content.
#[must_use]
pub fn collect_lines(content: &str) -> Vec<LineSlice<'_>> {
    let mut lines = content
        .split_inclusive('\n')
        .enumerate()
        .scan(0usize, |offset, (line_number, raw_line)| {
            let start_offset = *offset;
            let without_newline = raw_line.trim_end_matches(['\n', '\r']);
            let newline = &raw_line[without_newline.len()..];
            *offset += raw_line.len();
            Some(LineSlice {
                line_number: line_number + 1,
                start_offset,
                end_offset: *offset,
                trimmed: without_newline.trim(),
                without_newline,
                newline,
            })
        })
        .collect::<Vec<_>>();

    if !content.is_empty() && !content.ends_with('\n') {
        let without_newline = content.rsplit_once('\n').map_or(content, |(_, tail)| tail);
        if lines.is_empty()
            || lines
                .last()
                .is_some_and(|line| line.end_offset != content.len())
        {
            let start_offset = content.len() - without_newline.len();
            lines.push(LineSlice {
                line_number: lines.len() + 1,
                start_offset,
                end_offset: content.len(),
                trimmed: without_newline.trim(),
                without_newline,
                newline: "",
            });
        }
    }

    lines
}

/// Parses the top properties drawer from content.
#[must_use]
pub fn parse_top_properties_drawer(content: &str) -> Option<TopPropertiesDrawer<'_>> {
    let lines = collect_lines(content);

    let title_index = lines.iter().position(|line| !line.trimmed.is_empty())?;
    let title = lines.get(title_index)?;
    if !title.trimmed.starts_with('#') {
        return None;
    }

    let mut cursor = title_index + 1;
    while cursor < lines.len() && lines[cursor].trimmed.is_empty() {
        cursor += 1;
    }

    let properties = lines.get(cursor)?;
    if properties.trimmed != ":PROPERTIES:" {
        return None;
    }

    let newline = properties.newline;
    let insert_offset = properties.end_offset;

    let mut id_line = None;
    for line in lines.iter().skip(cursor + 1) {
        if line.trimmed == ":END:" {
            return Some(TopPropertiesDrawer {
                properties_line: properties.line_number,
                insert_offset,
                newline,
                id_line,
            });
        }

        if let Some(rest) = line.without_newline.strip_prefix(":ID:") {
            let leading_spaces = rest.chars().take_while(|ch| *ch == ' ').count();
            let value_start = line.start_offset + 4 + leading_spaces;
            let value = rest[leading_spaces..].trim();
            let value_end = value_start + value.len();
            id_line = Some(IdLine {
                line: line.line_number,
                value,
                value_start,
                value_end,
            });
        }
    }

    None
}

/// Parses the :LINKS: line from a relations block.
#[must_use]
pub fn parse_relations_links_line<'a>(lines: &'a [LineSlice<'a>]) -> Option<LinksLine<'a>> {
    let relations_idx = lines
        .iter()
        .position(|line| line.trimmed == ":RELATIONS:")?;
    for line in lines.iter().skip(relations_idx + 1) {
        if line.trimmed == ":END:" {
            break;
        }

        if let Some(rest) = line.without_newline.strip_prefix(":LINKS:") {
            let leading_spaces = rest.chars().take_while(|ch| *ch == ' ').count();
            let value = &rest[leading_spaces..];
            let value_start = line.start_offset + ":LINKS:".len() + leading_spaces;
            let value_end = line.start_offset + line.without_newline.len();
            return Some(LinksLine {
                line: line.line_number,
                value,
                value_start,
                value_end,
            });
        }
    }
    None
}

/// Parses the :FOOTER: block from lines.
#[must_use]
pub fn parse_footer_block<'a>(lines: &'a [LineSlice<'a>]) -> Option<FooterBlock<'a>> {
    let footer_idx = lines.iter().position(|line| line.trimmed == ":FOOTER:")?;
    let footer_line = &lines[footer_idx];
    let mut standards_value = None;
    let mut last_sync_value = None;

    for line in lines.iter().skip(footer_idx + 1) {
        if line.trimmed == ":END:" {
            return Some(FooterBlock {
                line: footer_line.line_number,
                start_offset: footer_line.start_offset,
                end_offset: line.end_offset,
                standards_value,
                last_sync_value,
            });
        }

        if let Some(rest) = line.without_newline.strip_prefix(":STANDARDS:") {
            standards_value = Some(rest.trim());
            continue;
        }

        if let Some(rest) = line.without_newline.strip_prefix(":LAST_SYNC:") {
            last_sync_value = Some(rest.trim());
        }
    }

    None
}

/// Extracts wikilinks from content.
#[must_use]
pub fn extract_wikilinks(content: &str) -> Vec<String> {
    extract_markdown_wikilinks(content)
        .into_iter()
        .filter_map(|link| wikilink_inner(link.original.as_str()))
        .collect()
}

/// Collects body links from an index document.
#[must_use]
pub fn collect_index_body_links(lines: &[LineSlice<'_>]) -> Vec<String> {
    let relations_start = lines
        .iter()
        .position(|line| line.trimmed == ":RELATIONS:")
        .unwrap_or(lines.len());

    let mut body_links = Vec::new();
    for line in &lines[..relations_start] {
        if !line.trimmed.starts_with("- ") {
            continue;
        }
        for link in extract_wikilinks(line.without_newline) {
            if !body_links.iter().any(|existing| existing == &link) {
                body_links.push(link);
            }
        }
    }
    body_links
}

/// Extract hidden workspace-path links from content.
#[must_use]
pub fn extract_hidden_path_links(content: &str) -> Vec<HiddenPathLink> {
    let lines = collect_lines(content);
    let mut hidden_links = Vec::new();

    for line in lines {
        collect_hidden_wikilinks_from_line(&line, &mut hidden_links);
        collect_hidden_markdown_links_from_line(&line, &mut hidden_links);
    }

    hidden_links
}

fn collect_hidden_wikilinks_from_line(line: &LineSlice<'_>, links: &mut Vec<HiddenPathLink>) {
    let mut cursor = 0usize;
    let text = line.without_newline;

    while let Some(start_rel) = text[cursor..].find("[[") {
        let start = cursor + start_rel;
        let after_start = &text[start + 2..];
        let Some(end_rel) = after_start.find("]]") else {
            break;
        };
        let end = start + 2 + end_rel + 2;
        let raw_target = &text[start + 2..start + 2 + end_rel];
        if let Some(target) = normalize_hidden_path_target(
            raw_target
                .split_once('|')
                .map_or(raw_target, |(target, _)| target),
        ) {
            links.push(HiddenPathLink {
                line: line.line_number,
                start_offset: line.start_offset + start,
                end_offset: line.start_offset + end,
                link_markup: text[start..end].to_string(),
                target,
            });
        }
        cursor = end;
    }
}

fn collect_hidden_markdown_links_from_line(line: &LineSlice<'_>, links: &mut Vec<HiddenPathLink>) {
    static RE_MARKDOWN_LINK: OnceLock<Option<regex::Regex>> = OnceLock::new();
    let Some(re_markdown_link) = RE_MARKDOWN_LINK
        .get_or_init(|| regex::Regex::new(r"!?\[[^\]]*\]\(([^)]+)\)").ok())
        .as_ref()
    else {
        return;
    };

    let text = line.without_newline;

    for captures in re_markdown_link.captures_iter(text) {
        let Some(link_match) = captures.get(0) else {
            continue;
        };
        let Some(target_match) = captures.get(1) else {
            continue;
        };
        let raw_target = normalize_markdown_target(target_match.as_str());
        if let Some(target) = normalize_hidden_path_target(raw_target) {
            links.push(HiddenPathLink {
                line: line.line_number,
                start_offset: line.start_offset + link_match.start(),
                end_offset: line.start_offset + link_match.end(),
                link_markup: link_match.as_str().to_string(),
                target,
            });
        }
    }
}

fn normalize_markdown_target(target: &str) -> &str {
    let trimmed = target.trim();
    let trimmed = trimmed
        .strip_prefix('<')
        .and_then(|inner| inner.strip_suffix('>'))
        .unwrap_or(trimmed);
    trimmed.split_whitespace().next().unwrap_or(trimmed)
}

fn wikilink_inner(original: &str) -> Option<String> {
    original
        .trim()
        .strip_prefix("[[")?
        .strip_suffix("]]")
        .map(ToString::to_string)
}

fn normalize_hidden_path_target(target: &str) -> Option<String> {
    let trimmed = target.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.contains("://")
        || trimmed.starts_with("mailto:")
        || trimmed.starts_with("javascript:")
    {
        return None;
    }

    let candidate = trimmed.split_once('#').map_or(trimmed, |(path, _)| path);
    let candidate = candidate
        .split_once('?')
        .map_or(candidate, |(path, _)| path)
        .replace('\\', "/");

    if candidate.split('/').any(is_hidden_path_component) {
        return Some(candidate);
    }

    None
}

fn path_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}

fn has_component_prefix(components: &[String], prefix: &[&str]) -> bool {
    components
        .windows(prefix.len())
        .any(|window| window.iter().map(String::as_str).eq(prefix.iter().copied()))
}

fn is_hidden_path_component(component: &str) -> bool {
    component.starts_with('.') && component != "." && component != ".."
}

fn is_hidden_workspace_dir(component: &str) -> bool {
    matches!(component, ".data" | ".cache" | ".run" | ".agent" | ".git")
}
