use xiuxian_wendao_parsers::blocks::{
    BlockCoreRequest, MarkdownBlock, MarkdownBlockKind, compute_block_hash, extract_blocks,
    line_col_to_byte_range,
};
use xiuxian_wendao_parsers::{LineColumnSpan, SourceByteRange};

#[test]
fn block_kind_id_prefixes_match_markdown_contract() {
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
fn markdown_block_new_generates_stable_fields() {
    let block = MarkdownBlock::new(BlockCoreRequest {
        kind: MarkdownBlockKind::Paragraph,
        index: 0,
        byte_range: (0, 20),
        line_range: (1, 2),
        content: "Hello, world!".to_string(),
        structural_path: vec!["Section".to_string()],
    });

    assert_eq!(block.block_id, "block-para-0");
    assert_eq!(block.byte_range, (0, 20));
    assert_eq!(block.line_range, (1, 2));
    assert!(block.id.is_none());
    assert_eq!(block.structural_path, vec!["Section"]);
}

#[test]
fn markdown_block_with_explicit_id_replaces_generated_id() {
    let block = MarkdownBlock::new(BlockCoreRequest {
        kind: MarkdownBlockKind::CodeFence {
            language: "rust".into(),
        },
        index: 0,
        byte_range: (0, 100),
        line_range: (1, 10),
        content: "fn main() {}".to_string(),
        structural_path: vec!["Code".to_string(), "Examples".to_string()],
    })
    .with_explicit_id("my-snippet".into());

    assert_eq!(block.block_id, "my-snippet");
    assert_eq!(block.id, Some("my-snippet".to_string()));
    assert_eq!(block.structural_path, vec!["Code", "Examples"]);
}

#[test]
fn compute_block_hash_is_stable_and_compact() {
    let hash1 = compute_block_hash("test content");
    let hash2 = compute_block_hash("test content");
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 16);

    let hash3 = compute_block_hash("different content");
    assert_ne!(hash1, hash3);
}

#[test]
fn extract_blocks_parses_common_markdown_block_types() {
    let text = r#"Introduction paragraph.

```python
print("hello")
```

- List item 1
- List item 2

Conclusion paragraph.
"#;
    let blocks = extract_blocks(text, 0, 1, &["Section".to_string()]);

    assert_eq!(blocks.len(), 4);
    assert_eq!(blocks[0].kind, MarkdownBlockKind::Paragraph);
    assert!(
        matches!(&blocks[1].kind, MarkdownBlockKind::CodeFence { language } if language == "python")
    );
    assert!(matches!(
        &blocks[2].kind,
        MarkdownBlockKind::List { ordered: false }
    ));
    assert_eq!(blocks[3].kind, MarkdownBlockKind::Paragraph);
    for block in &blocks {
        assert_eq!(block.structural_path, vec!["Section"]);
    }
}

#[test]
fn extract_blocks_offsets_ranges_from_section_origin() {
    let blocks = extract_blocks(
        "Content",
        100,
        10,
        &["Root".to_string(), "Child".to_string()],
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].byte_range.0, 100);
    assert_eq!(blocks[0].line_range.0, 10);
    assert_eq!(blocks[0].structural_path, vec!["Root", "Child"]);
}

#[test]
fn line_col_to_byte_range_handles_single_and_multi_line_ranges() {
    let text = "Hello\nWorld";
    assert_eq!(
        line_col_to_byte_range(text, LineColumnSpan::new(1, 1, 1, 5)),
        Some(SourceByteRange::new(0, 5))
    );
    assert_eq!(
        line_col_to_byte_range(text, LineColumnSpan::new(2, 1, 2, 5)),
        Some(SourceByteRange::new(6, 11))
    );

    let multiline = "Line 1\nLine 2\nLine 3";
    let range = line_col_to_byte_range(multiline, LineColumnSpan::new(1, 1, 3, 6));
    assert_eq!(range, Some(SourceByteRange::new(0, multiline.len())));
}
