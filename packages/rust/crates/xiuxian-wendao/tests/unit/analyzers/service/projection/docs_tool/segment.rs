use super::{ProjectedMarkdownDocument, ProjectionPageKind, build_document_segment};

fn sample_document() -> ProjectedMarkdownDocument {
    ProjectedMarkdownDocument {
        repo_id: "repo-a".to_string(),
        page_id: "page-a".to_string(),
        kind: ProjectionPageKind::Reference,
        path: "reference/page-a.md".to_string(),
        title: "Page A".to_string(),
        markdown: "# Page A\n## Anchors\nBody line\n### Integrator\nChild body\n".to_string(),
    }
}

#[test]
fn build_document_segment_returns_requested_lines() {
    let segment = build_document_segment(&sample_document(), 2, 4)
        .unwrap_or_else(|error| panic!("segment: {error}"));

    assert_eq!(segment.line_count, 5);
    assert_eq!(segment.line_range, (2, 4));
    assert_eq!(segment.content, "## Anchors\nBody line\n### Integrator");
}

#[test]
fn build_document_segment_clamps_end_line() {
    let segment = build_document_segment(&sample_document(), 4, 99)
        .unwrap_or_else(|error| panic!("segment: {error}"));

    assert_eq!(segment.line_range, (4, 5));
    assert_eq!(segment.content, "### Integrator\nChild body");
}
