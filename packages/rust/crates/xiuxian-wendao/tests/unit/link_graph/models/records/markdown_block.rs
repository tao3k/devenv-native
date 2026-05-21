//! Unit tests for `markdown_block` module.

use super::{BlockAddress, BlockKindSpecifier, MarkdownBlock, MarkdownBlockKind};
use xiuxian_wendao_parsers::blocks::{BlockCoreRequest, compute_block_hash};

fn block(
    kind: MarkdownBlockKind,
    index: usize,
    byte_range: (usize, usize),
    line_range: (usize, usize),
    content: &str,
    structural_path: Vec<String>,
) -> MarkdownBlock {
    MarkdownBlock::new(BlockCoreRequest {
        kind,
        index,
        byte_range,
        line_range,
        content: content.to_string(),
        structural_path,
    })
}

#[test]
fn test_block_kind_id_prefix() {
    assert_eq!(MarkdownBlockKind::Paragraph.id_prefix(), "para");
    assert_eq!(
        MarkdownBlockKind::CodeFence {
            language: "rust".into()
        }
        .id_prefix(),
        "code"
    );
    assert_eq!(
        MarkdownBlockKind::List { ordered: true }.id_prefix(),
        "olist"
    );
    assert_eq!(
        MarkdownBlockKind::List { ordered: false }.id_prefix(),
        "ulist"
    );
    assert_eq!(MarkdownBlockKind::BlockQuote.id_prefix(), "quote");
    assert_eq!(MarkdownBlockKind::ThematicBreak.id_prefix(), "hr");
    assert_eq!(MarkdownBlockKind::Table.id_prefix(), "table");
    assert_eq!(MarkdownBlockKind::HtmlBlock.id_prefix(), "html");
}

#[test]
fn test_block_new() {
    let block = block(
        MarkdownBlockKind::Paragraph,
        0,
        (0, 20),
        (1, 2),
        "Hello, world!",
        vec!["Section".to_string()],
    );

    assert_eq!(block.block_id, "block-para-0");
    assert_eq!(block.byte_range, (0, 20));
    assert_eq!(block.line_range, (1, 2));
    assert!(block.id.is_none());
    assert_eq!(block.structural_path, vec!["Section"]);
}

#[test]
fn test_block_with_explicit_id() {
    let block = block(
        MarkdownBlockKind::CodeFence {
            language: "rust".into(),
        },
        0,
        (0, 100),
        (1, 10),
        "fn main() {}",
        vec!["Code".to_string(), "Examples".to_string()],
    )
    .with_explicit_id("my-snippet".to_string().into());

    assert_eq!(block.block_id, "my-snippet");
    assert_eq!(block.id, Some("my-snippet".to_string()));
    assert_eq!(block.structural_path, vec!["Code", "Examples"]);
}

#[test]
fn test_block_kind_specifier_parse() {
    assert_eq!(
        BlockKindSpecifier::parse("Paragraph"),
        Some(BlockKindSpecifier::Paragraph)
    );
    assert_eq!(
        BlockKindSpecifier::parse("para"),
        Some(BlockKindSpecifier::Paragraph)
    );
    assert_eq!(
        BlockKindSpecifier::parse("CodeFence"),
        Some(BlockKindSpecifier::CodeFence)
    );
    assert_eq!(
        BlockKindSpecifier::parse("code"),
        Some(BlockKindSpecifier::CodeFence)
    );
    assert_eq!(
        BlockKindSpecifier::parse("List"),
        Some(BlockKindSpecifier::List)
    );
    assert_eq!(BlockKindSpecifier::parse("unknown"), None);
}

#[test]
fn test_block_address_parse() {
    let addr = BlockAddress::parse("Paragraph[2]");
    assert!(addr.is_some());
    let Some(addr) = addr else {
        panic!("paragraph block address should parse");
    };
    assert_eq!(addr.kind, BlockKindSpecifier::Paragraph);
    assert_eq!(addr.index, 2);
    assert!(addr.sub_index.is_none());

    let addr = BlockAddress::parse("CodeFence[0]");
    assert!(addr.is_some());
    let Some(addr) = addr else {
        panic!("code fence block address should parse");
    };
    assert_eq!(addr.kind, BlockKindSpecifier::CodeFence);
    assert_eq!(addr.index, 0);
}

#[test]
fn test_block_address_parse_with_sub_index() {
    let addr = BlockAddress::parse("List[1]/Item[3]");
    assert!(addr.is_some());
    let Some(addr) = addr else {
        panic!("list item block address should parse");
    };
    assert_eq!(addr.kind, BlockKindSpecifier::List);
    assert_eq!(addr.index, 1);
    assert_eq!(addr.sub_index, Some(3));
}

#[test]
fn test_block_address_to_path_component() {
    let addr = BlockAddress::new(BlockKindSpecifier::Paragraph, 2);
    assert_eq!(addr.to_path_component(), "Paragraph[2]");

    let addr = BlockAddress::list_item(1, 3);
    assert_eq!(addr.to_path_component(), "List[1]/Item[3]");
}

#[test]
fn test_block_matches_kind() {
    let para = block(
        MarkdownBlockKind::Paragraph,
        0,
        (0, 10),
        (1, 1),
        "text",
        vec!["Section".to_string()],
    );
    assert!(super::matching::markdown_block_matches_kind(
        &para,
        &BlockKindSpecifier::Paragraph
    ));
    assert!(!super::matching::markdown_block_matches_kind(
        &para,
        &BlockKindSpecifier::CodeFence
    ));

    let code = block(
        MarkdownBlockKind::CodeFence {
            language: "rust".into(),
        },
        0,
        (0, 10),
        (1, 1),
        "fn main() {}",
        vec!["Section".to_string(), "Code".to_string()],
    );
    assert!(super::matching::markdown_block_matches_kind(
        &code,
        &BlockKindSpecifier::CodeFence
    ));
    assert!(!super::matching::markdown_block_matches_kind(
        &code,
        &BlockKindSpecifier::Paragraph
    ));
}

#[test]
fn test_compute_block_hash() {
    let hash1 = compute_block_hash("test content");
    let hash2 = compute_block_hash("test content");
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 16);

    let hash3 = compute_block_hash("different content");
    assert_ne!(hash1, hash3);
}
