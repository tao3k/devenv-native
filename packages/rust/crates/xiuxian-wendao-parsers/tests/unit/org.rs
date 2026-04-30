use xiuxian_wendao_parsers::{
    DocumentFormat, extract_org_sections, parse_org_document, parse_org_note, parse_org_toc,
};

#[test]
fn parse_org_document_extracts_native_metadata() {
    let content = concat!(
        ":PROPERTIES:\n",
        ":ID: root-id\n",
        ":TYPE: knowledge\n",
        ":END:\n",
        "#+TITLE: Org Contract\n",
        "#+FILETAGS: :parser:org:\n",
        "\n",
        "* First\n",
        "Lead text.\n",
    );

    let document = parse_org_document(content, "fallback");

    assert_eq!(document.core.format, DocumentFormat::Org);
    assert_eq!(document.core.title, "Org Contract");
    assert_eq!(document.core.tags, vec!["org", "parser"]);
    assert_eq!(document.core.doc_type.as_deref(), Some("knowledge"));
    assert!(document.core.body.starts_with("* First\n"));
    assert_eq!(document.core.lead, "Lead text.");
    let Some(metadata) = document.raw_metadata else {
        panic!("org metadata should exist");
    };
    assert_eq!(
        metadata.properties.get("ID").map(String::as_str),
        Some("root-id")
    );
    assert_eq!(
        metadata
            .keywords
            .get("TITLE")
            .and_then(|values| values.first()),
        Some(&"Org Contract".to_string())
    );
}

#[test]
fn extract_org_sections_preserves_headline_property_drawers() {
    let body = concat!(
        "* TODO First :work:\n",
        ":PROPERTIES:\n",
        ":ID: first-id\n",
        ":RELATED: second\n",
        ":END:\n",
        "First body.\n",
        "** Child\n",
        ":PROPERTIES:\n",
        ":ID: child-id\n",
        ":END:\n",
        "Child body.\n",
    );

    let sections = extract_org_sections(body);

    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].heading_title(), "First");
    assert_eq!(sections[0].heading_level(), 1);
    assert_eq!(
        sections[0].attributes().get("ID").map(String::as_str),
        Some("first-id")
    );
    assert_eq!(
        sections[0].attributes().get("RELATED").map(String::as_str),
        Some("second")
    );
    assert_eq!(sections[1].heading_path(), "First / Child");
    assert_eq!(
        sections[1].attributes().get("ID").map(String::as_str),
        Some("child-id")
    );
}

#[test]
fn parse_org_note_and_toc_share_org_sections() {
    let content = concat!(
        "#+TITLE: Native Org\n",
        "\n",
        "* First\n",
        ":PROPERTIES:\n",
        ":ID: first-id\n",
        ":END:\n",
        "Body.\n",
    );

    let note = parse_org_note(content, "fallback");
    let toc = parse_org_toc(content, "fallback");

    assert_eq!(note.document.core.title, "Native Org");
    assert_eq!(note.core.sections, toc.sections);
    assert_eq!(toc.document.core.format, DocumentFormat::Org);
    assert_eq!(
        toc.sections[0].attributes().get("ID").map(String::as_str),
        Some("first-id")
    );
}
