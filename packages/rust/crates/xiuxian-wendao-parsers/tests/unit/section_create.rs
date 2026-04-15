use xiuxian_wendao_parsers::section_create::{
    BuildSectionOptions, build_new_sections_content_with_options, compute_content_hash,
    find_insertion_point, generate_section_id, parse_heading_line,
};

#[test]
fn parse_heading_line_recognizes_markdown_headings() {
    assert_eq!(
        parse_heading_line("# Title"),
        Some((1, "Title".to_string()))
    );
    assert_eq!(
        parse_heading_line("## Sub Title"),
        Some((2, "Sub Title".to_string()))
    );
    assert_eq!(parse_heading_line("###Deep"), Some((3, "Deep".to_string())));
    assert_eq!(parse_heading_line("No heading"), None);
    assert_eq!(parse_heading_line("####### Too deep"), None);
}

#[test]
fn find_insertion_point_handles_empty_documents() {
    let result = find_insertion_point("", &["Section".to_string()]);
    assert_eq!(result.insertion_byte, 0);
    assert_eq!(result.start_level, 1);
    assert_eq!(result.remaining_path, vec!["Section".to_string()]);
}

#[test]
fn find_insertion_point_tracks_previous_sibling_context() {
    let doc = "# Main\n\nIntro.\n\n## Alpha\n\nAlpha content.\n\n## Beta\n\nBeta content.\n";
    let result = find_insertion_point(doc, &["Main".to_string(), "NewSection".to_string()]);

    assert_eq!(result.start_level, 2);
    assert!(result.next_sibling.is_none());
    let Some(prev_sibling) = result.prev_sibling else {
        panic!("expected previous sibling");
    };
    assert_eq!(prev_sibling.title, "Beta");
}

#[test]
fn build_new_sections_content_with_id_renders_heading_chain() {
    let content = build_new_sections_content_with_options(
        &["MySection".to_string()],
        2,
        "Content here",
        &BuildSectionOptions {
            generate_id: true,
            id_prefix: Some("sec".to_string()),
        },
    );

    assert!(content.contains("## MySection"));
    assert!(content.contains(":ID: sec-"));
    assert!(content.contains("Content here"));
}

#[test]
fn compute_content_hash_is_stable_for_identical_content() {
    let hash1 = compute_content_hash("test");
    let hash2 = compute_content_hash("test");

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 16);
}

#[test]
fn generate_section_id_supports_prefixed_and_plain_shapes() {
    let id1 = generate_section_id(None);
    let id2 = generate_section_id(Some("arch"));

    assert_eq!(id1.len(), 12);
    assert!(id2.starts_with("arch-"));
    assert_eq!(id2.len(), 13);
}
