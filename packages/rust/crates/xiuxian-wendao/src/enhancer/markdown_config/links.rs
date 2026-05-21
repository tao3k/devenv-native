//! `enhancer::markdown_config::links` owns Wendao enhancer markdown config links behavior.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use super::types::MarkdownConfigLinkTarget;

const CONFIG_TYPE_KEY: &str = "type";
const DEFAULT_CONFIG_TYPE: &str = "unknown";

/// Extracts normalized local or semantic link targets plus optional explicit
/// reference categories.
#[must_use]
pub fn extract_markdown_config_link_targets_by_id(
    markdown: &str,
    source_path: &str,
) -> HashMap<String, Vec<MarkdownConfigLinkTarget>> {
    let mut options = comrak::Options::default();
    options.extension.wikilinks_title_before_pipe = true;

    let arena = comrak::Arena::new();
    let root = comrak::parse_document(&arena, markdown, &options);

    let mut links_by_id: HashMap<String, Vec<MarkdownConfigLinkTarget>> = HashMap::new();
    let mut active_cursor: Option<MarkdownPropertyCursor> = None;

    for node in root.descendants() {
        match &node.data.borrow().value {
            comrak::nodes::NodeValue::Heading(heading) => {
                let heading_level = heading.level;
                if let Some(cursor) = &active_cursor
                    && heading_level <= cursor.heading_level
                {
                    active_cursor = None;
                }
                if let Some(next_cursor) = parse_cursor_from_heading(node, heading_level) {
                    active_cursor = Some(next_cursor);
                }
            }
            comrak::nodes::NodeValue::Link(link) => {
                let Some(cursor) = &active_cursor else {
                    continue;
                };
                insert_link_target(
                    &mut links_by_id,
                    &cursor.id,
                    link.url.as_str(),
                    source_path,
                    cursor.config_type.as_str(),
                );
            }
            comrak::nodes::NodeValue::Image(image) => {
                let Some(cursor) = &active_cursor else {
                    continue;
                };
                insert_link_target(
                    &mut links_by_id,
                    &cursor.id,
                    image.url.as_str(),
                    source_path,
                    cursor.config_type.as_str(),
                );
            }
            comrak::nodes::NodeValue::WikiLink(link) => {
                let Some(cursor) = &active_cursor else {
                    continue;
                };
                insert_link_target(
                    &mut links_by_id,
                    &cursor.id,
                    link.url.as_str(),
                    source_path,
                    cursor.config_type.as_str(),
                );
            }
            _ => {}
        }
    }

    links_by_id
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownPropertyCursor {
    id: String,
    config_type: String,
    heading_level: u8,
}

fn parse_cursor_from_heading<'a>(
    heading_node: &'a comrak::nodes::AstNode<'a>,
    heading_level: u8,
) -> Option<MarkdownPropertyCursor> {
    let sibling = heading_node.next_sibling()?;
    let comrak::nodes::NodeValue::HtmlBlock(html) = &sibling.data.borrow().value else {
        return None;
    };
    let tag = parse_property_tag(&html.literal)?;
    Some(MarkdownPropertyCursor {
        id: tag.id,
        config_type: tag.config_type,
        heading_level,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownPropertyTag {
    id: String,
    config_type: String,
}

fn parse_property_tag(html_block: &str) -> Option<MarkdownPropertyTag> {
    let body = html_block
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();

    let mut id: Option<String> = None;
    let mut config_type: Option<String> = None;
    for pair in body.split(',') {
        let Some((raw_key, raw_value)) = pair.split_once(':') else {
            continue;
        };
        let key = raw_key.trim().to_ascii_lowercase();
        let value = trim_quotes(raw_value.trim());
        if value.is_empty() {
            continue;
        }
        if key == "id" {
            id = Some(value.to_string());
        }
        if key == CONFIG_TYPE_KEY {
            config_type = Some(value.to_string());
        }
    }

    Some(MarkdownPropertyTag {
        id: id?,
        config_type: config_type.unwrap_or_else(|| DEFAULT_CONFIG_TYPE.to_string()),
    })
}

fn trim_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|rest| rest.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn insert_link_target(
    links_by_id: &mut HashMap<String, Vec<MarkdownConfigLinkTarget>>,
    id: &str,
    raw_target: &str,
    source_path: &str,
    config_type: &str,
) {
    let Some(target) = normalize_local_link_target(raw_target, source_path) else {
        return;
    };
    let normalized_type = normalize_reference_type(config_type, target.as_str());
    let links = links_by_id.entry(id.to_string()).or_default();
    if !links
        .iter()
        .any(|existing| existing.target == target && existing.reference_type == normalized_type)
    {
        links.push(MarkdownConfigLinkTarget {
            target,
            reference_type: normalized_type,
        });
    }
}

fn normalize_reference_type(config_type: &str, target: &str) -> Option<String> {
    infer_reference_type_from_target(target)
        .or_else(|| normalize_config_reference_type(config_type))
}

fn infer_reference_type_from_target(target: &str) -> Option<String> {
    let ext = extract_extension(target)?;
    if is_attachment_extension(ext) {
        return Some("attachment".to_string());
    }
    None
}

fn extract_extension(target: &str) -> Option<&str> {
    let without_fragment = strip_fragment_and_query(target);
    let leaf = without_fragment.rsplit('/').next()?;
    let (_, extension) = leaf.rsplit_once('.')?;
    let trimmed = extension.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn is_attachment_extension(extension: &str) -> bool {
    matches!(
        extension.trim().to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "pdf"
    )
}

fn normalize_config_reference_type(config_type: &str) -> Option<String> {
    let normalized = config_type.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "" | DEFAULT_CONFIG_TYPE => None,
        "prompt" | "template" | "tpl" => Some("template".to_string()),
        "persona" | "agent" => Some("persona".to_string()),
        "knowledge" | "doc" => Some("knowledge".to_string()),
        "workflow" | "flow" | "qianji-flow" => Some("qianji-flow".to_string()),
        _ => Some(normalized),
    }
}

fn normalize_local_link_target(raw_target: &str, source_path: &str) -> Option<String> {
    let target = strip_fragment_and_query(raw_target);
    if target.is_empty() || target.starts_with('#') {
        return None;
    }
    if is_wendao_resource_uri(target) {
        return Some(target.to_string());
    }
    if is_external_target(target) {
        return None;
    }

    let source_parent = Path::new(source_path).parent().unwrap_or(Path::new(""));
    let target_path = if target.starts_with('/') {
        PathBuf::from(target.trim_start_matches('/'))
    } else {
        source_parent.join(target)
    };
    normalize_relative_path(&target_path)
}

fn normalize_relative_path(path: &Path) -> Option<String> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(value) => normalized.push(value),
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }

    let candidate = normalized.to_string_lossy().replace('\\', "/");
    if candidate.is_empty() {
        None
    } else {
        Some(candidate)
    }
}

fn strip_fragment_and_query(raw: &str) -> &str {
    let mut end = raw.len();
    if let Some(index) = raw.find('#') {
        end = end.min(index);
    }
    if let Some(index) = raw.find('?') {
        end = end.min(index);
    }
    raw[..end].trim()
}

fn is_external_target(target: &str) -> bool {
    target.contains("://") || target.starts_with("mailto:") || target.starts_with("tel:")
}

fn is_wendao_resource_uri(target: &str) -> bool {
    target.trim().to_ascii_lowercase().starts_with("wendao://")
}
