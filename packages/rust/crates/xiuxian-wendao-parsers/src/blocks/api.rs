use comrak::{Arena, Options, nodes::AstNode, nodes::NodeValue, parse_document};

use super::counter::BlockIndexCounter;
use super::{MarkdownBlock, MarkdownBlockKind};
use crate::sourcepos::line_col_to_byte_range;

/// Extract top-level Markdown blocks from one section body.
///
/// The returned blocks preserve parser-visible ranges and raw content, while
/// leaving page-index addressing and other Wendao semantics to consumers.
#[must_use]
pub fn extract_blocks(
    section_text: &str,
    section_byte_offset: usize,
    section_line_offset: usize,
    structural_path: &[String],
) -> Vec<MarkdownBlock> {
    let arena = Arena::new();
    let root = parse_document(&arena, section_text, &Options::default());

    let mut blocks = Vec::new();
    let mut block_indices = BlockIndexCounter::default();

    for node in root.children() {
        if let Some(block) = node_to_block(
            node,
            section_text,
            section_byte_offset,
            section_line_offset,
            &mut block_indices,
            structural_path,
        ) {
            blocks.push(block);
        }
    }

    blocks
}

fn node_to_block(
    node: &AstNode<'_>,
    section_text: &str,
    section_byte_offset: usize,
    section_line_offset: usize,
    block_indices: &mut BlockIndexCounter,
    structural_path: &[String],
) -> Option<MarkdownBlock> {
    let ast = node.data.borrow();
    let sourcepos = ast.sourcepos;

    let start_line = sourcepos.start.line;
    let start_col = sourcepos.start.column;
    let end_line = sourcepos.end.line;
    let end_col = sourcepos.end.column;

    let byte_range =
        line_col_to_byte_range(section_text, start_line, start_col, end_line, end_col)?;

    let doc_line_range = (
        section_line_offset
            .saturating_add(start_line)
            .saturating_sub(1)
            .max(1),
        section_line_offset
            .saturating_add(end_line)
            .saturating_sub(1)
            .max(1),
    );

    let content = if byte_range.0 <= byte_range.1 && byte_range.1 <= section_text.len() {
        &section_text[byte_range.0..byte_range.1]
    } else {
        return None;
    };

    if content.trim().is_empty() {
        return None;
    }

    let kind = match &ast.value {
        NodeValue::Paragraph => MarkdownBlockKind::Paragraph,
        NodeValue::CodeBlock(block) => MarkdownBlockKind::CodeFence {
            language: block.info.trim().to_string(),
        },
        NodeValue::List(list) => MarkdownBlockKind::List {
            ordered: list.list_type == comrak::nodes::ListType::Ordered,
        },
        NodeValue::BlockQuote => MarkdownBlockKind::BlockQuote,
        NodeValue::ThematicBreak => MarkdownBlockKind::ThematicBreak,
        NodeValue::Table(_) => MarkdownBlockKind::Table,
        NodeValue::HtmlBlock(_) => MarkdownBlockKind::HtmlBlock,
        NodeValue::Heading(_)
        | NodeValue::Document
        | NodeValue::FrontMatter(_)
        | NodeValue::Text(_)
        | NodeValue::SoftBreak
        | NodeValue::LineBreak
        | NodeValue::Code(_)
        | NodeValue::Emph
        | NodeValue::Strong
        | NodeValue::Strikethrough
        | NodeValue::Superscript
        | NodeValue::Link(_)
        | NodeValue::Image(_)
        | NodeValue::FootnoteDefinition(_)
        | NodeValue::FootnoteReference(_)
        | NodeValue::DescriptionList
        | NodeValue::DescriptionItem(_)
        | NodeValue::DescriptionTerm
        | NodeValue::DescriptionDetails
        | NodeValue::Math(_)
        | NodeValue::Escaped
        | NodeValue::MultilineBlockQuote(_)
        | NodeValue::EscapedTag(_)
        | NodeValue::Raw(_)
        | NodeValue::Underline
        | NodeValue::Subscript
        | NodeValue::SpoileredText
        | NodeValue::Item(_)
        | NodeValue::TableRow(_)
        | NodeValue::TableCell
        | NodeValue::TaskItem(_)
        | NodeValue::HtmlInline(_)
        | NodeValue::Highlight
        | NodeValue::WikiLink(_)
        | NodeValue::Alert(_)
        | NodeValue::Subtext => {
            return None;
        }
    };

    let index = block_indices.next(&kind);
    Some(MarkdownBlock::new(
        kind,
        index,
        (
            byte_range.0 + section_byte_offset,
            byte_range.1 + section_byte_offset,
        ),
        doc_line_range,
        content,
        structural_path.to_vec(),
    ))
}
