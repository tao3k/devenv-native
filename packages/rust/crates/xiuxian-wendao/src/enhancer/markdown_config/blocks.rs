//! `enhancer::markdown_config::blocks` owns Wendao enhancer markdown config blocks behavior.

use comrak::{
    Arena, Options,
    nodes::{AstNode, NodeValue},
    parse_document,
};

use super::types::MarkdownConfigBlock;

const DEFAULT_CONFIG_TYPE: &str = "unknown";
const TEMPLATE_CONFIG_TYPE: &str = "template";
const PERSONA_CONFIG_TYPE: &str = "persona";
const CONFIG_ID_KEY: &str = "id";
const CONFIG_TYPE_KEY: &str = "type";
const CONFIG_TARGET_KEY: &str = "target";

/// Extracts tagged configuration code blocks from markdown AST.
///
/// The parser scans heading nodes, checks their adjacent HTML property blocks,
/// keeps that property as a cursor, and extracts fenced `jinja2` or `toml`
/// code blocks under the active heading scope.
#[must_use]
pub fn extract_markdown_config_blocks(markdown: &str) -> Vec<MarkdownConfigBlock> {
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &Options::default());

    let mut extracted: Vec<MarkdownConfigBlock> = Vec::new();
    let mut active_cursor: Option<MarkdownPropertyCursor> = None;

    for node in root.descendants() {
        match &node.data.borrow().value {
            NodeValue::Heading(heading) => {
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
            NodeValue::CodeBlock(block) => {
                let Some(cursor) = &active_cursor else {
                    continue;
                };
                let Some(language) = parse_fence_language(&block.info) else {
                    continue;
                };
                if !is_extractable_config_code_block(&cursor.config_type, &language) {
                    continue;
                }
                extracted.push(MarkdownConfigBlock {
                    id: cursor.id.clone(),
                    config_type: cursor.config_type.clone(),
                    target: cursor.target.clone(),
                    heading: cursor.heading.clone(),
                    language,
                    content: block.literal.clone(),
                });
            }
            _ => {}
        }
    }

    extracted
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownPropertyCursor {
    id: String,
    config_type: String,
    target: Option<String>,
    heading: String,
    heading_level: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownPropertyTag {
    id: String,
    config_type: String,
    target: Option<String>,
}

fn parse_cursor_from_heading<'a>(
    heading_node: &'a AstNode<'a>,
    heading_level: u8,
) -> Option<MarkdownPropertyCursor> {
    let heading = collect_heading_text(heading_node);
    let sibling = heading_node.next_sibling()?;
    let NodeValue::HtmlBlock(html) = &sibling.data.borrow().value else {
        return None;
    };
    let tag = parse_property_tag(&html.literal)?;
    Some(MarkdownPropertyCursor {
        id: tag.id,
        config_type: tag.config_type,
        target: tag.target,
        heading,
        heading_level,
    })
}

fn collect_heading_text<'a>(heading_node: &'a AstNode<'a>) -> String {
    let mut heading = String::new();
    for child in heading_node.children() {
        push_text_from_node(child, &mut heading);
    }
    heading.trim().to_string()
}

fn push_text_from_node<'a>(node: &'a AstNode<'a>, out: &mut String) {
    match &node.data.borrow().value {
        NodeValue::Text(value) => out.push_str(value),
        NodeValue::Code(value) => out.push_str(&value.literal),
        NodeValue::SoftBreak | NodeValue::LineBreak => out.push(' '),
        _ => {
            for child in node.children() {
                push_text_from_node(child, out);
            }
        }
    }
}

fn parse_property_tag(html_block: &str) -> Option<MarkdownPropertyTag> {
    let body = html_block
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();

    let mut id: Option<String> = None;
    let mut config_type: Option<String> = None;
    let mut target: Option<String> = None;

    for pair in body.split(',') {
        let Some((raw_key, raw_value)) = pair.split_once(':') else {
            continue;
        };
        let key = raw_key.trim().to_ascii_lowercase();
        let value = trim_quotes(raw_value.trim());
        if value.is_empty() {
            continue;
        }
        match key.as_str() {
            CONFIG_ID_KEY => id = Some(value.to_string()),
            CONFIG_TYPE_KEY => config_type = Some(value.to_string()),
            CONFIG_TARGET_KEY => target = Some(value.to_string()),
            _ => {}
        }
    }

    Some(MarkdownPropertyTag {
        id: id?,
        config_type: config_type.unwrap_or_else(|| DEFAULT_CONFIG_TYPE.to_string()),
        target,
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

fn parse_fence_language(info: &str) -> Option<String> {
    info.split_whitespace().next().map(str::to_lowercase)
}

fn is_jinja2_fence(language: &str) -> bool {
    language == "jinja2" || language == "j2"
}

fn is_toml_fence(language: &str) -> bool {
    language == "toml"
}

fn is_extractable_config_code_block(config_type: &str, language: &str) -> bool {
    match config_type.trim().to_ascii_lowercase().as_str() {
        TEMPLATE_CONFIG_TYPE => is_jinja2_fence(language),
        PERSONA_CONFIG_TYPE => is_toml_fence(language),
        _ => false,
    }
}
