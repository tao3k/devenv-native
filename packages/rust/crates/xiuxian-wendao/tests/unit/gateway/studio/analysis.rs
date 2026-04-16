use super::*;
use crate::gateway::studio::StudioState;
use crate::gateway::studio::analysis::service::AnalysisError;
use crate::gateway::studio::test_support::{assert_studio_json_snapshot, round_f64};
use crate::gateway::studio::types::{UiConfig, UiProjectConfig};
use crate::link_graph::LinkGraphIndex;
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;

struct AnalysisFixture {
    state: StudioState,
    temp_dir: tempfile::TempDir,
}

fn make_analysis_fixture() -> AnalysisFixture {
    let temp_dir =
        tempdir().unwrap_or_else(|err| panic!("failed to create analysis fixture tempdir: {err}"));
    let docs_dir = temp_dir.path().join("docs");
    std::fs::create_dir_all(&docs_dir)
        .unwrap_or_else(|err| panic!("failed to create docs fixture dir: {err}"));

    std::fs::write(
        docs_dir.join("analysis.md"),
        r#"# Analysis Kernel

## Inputs
- [ ] Parse markdown
- [x] Build IR

## Links
:PROPERTIES:
:ID: AnalysisKernel
:OBSERVE: lang:rust scope:"src/gateway/studio/**" "fn compile() { $$$ }"
:END:

Reference [[docs/guide.md]] and [[internal_skills/writer/SKILL.md]].

```rust
fn compile() {}
```
"#,
    )
    .unwrap_or_else(|err| panic!("failed to write markdown analysis fixture: {err}"));

    std::fs::write(
        docs_dir.join("deepwiki.md"),
        r#"# DeepWiki Kernel
:PROPERTIES:
:ID: deepwiki-kernel
:PARENT: [[index]]
:TAGS: feature, docs
:END:

## Overview
DeepWiki reads parser-owned metadata.

## References
:PROPERTIES:
:ID: DeepWikiKernel
:OBSERVE: lang:rust scope:"src/gateway/studio/**" "fn deepwiki() { $$$ }"
:RELATED: [[guide]]
:END:

Reference [[guide]].

:RELATIONS:
:LINKS: [[guide]], [[index]]
:END:
"#,
    )
    .unwrap_or_else(|err| panic!("failed to write deepwiki markdown fixture: {err}"));

    std::fs::write(
        docs_dir.join("guide.md"),
        "# Guide\n\nReference [[deepwiki]].\n",
    )
    .unwrap_or_else(|err| panic!("failed to write guide markdown fixture: {err}"));

    std::fs::write(docs_dir.join("index.md"), "# Index\n\n- [[deepwiki]]\n")
        .unwrap_or_else(|err| panic!("failed to write index markdown fixture: {err}"));

    std::fs::write(docs_dir.join("raw.rs"), "fn raw() {}\n")
        .unwrap_or_else(|err| panic!("failed to write non-markdown fixture: {err}"));

    let mut state = StudioState::new();
    state.project_root = temp_dir.path().to_path_buf();
    state.config_root = temp_dir.path().to_path_buf();
    state.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "main".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string()],
        }],
        repo_projects: Vec::new(),
    });
    let graph_index = LinkGraphIndex::build(temp_dir.path())
        .unwrap_or_else(|err| panic!("failed to build graph index for analysis fixture: {err}"));
    *state
        .graph_index
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::new(graph_index));

    AnalysisFixture { state, temp_dir }
}

#[tokio::test]
async fn analyze_markdown_returns_ir_and_projections() {
    let fixture = make_analysis_fixture();
    let payload = analyze_markdown(&fixture.state, "main/docs/deepwiki.md")
        .await
        .unwrap_or_else(|err| panic!("expected markdown analysis to succeed: {err:?}"));

    assert_studio_json_snapshot(
        "analysis_markdown_payload",
        json!({
            "path": payload.path,
            "documentHash": payload.document_hash,
            "nodeCount": payload.node_count,
            "edgeCount": payload.edge_count,
            "nodes": payload.nodes.into_iter().map(|node| {
                json!({
                    "id": node.id,
                    "kind": node.kind,
                    "label": node.label,
                    "depth": node.depth,
                    "lineStart": node.line_start,
                    "lineEnd": node.line_end,
                    "parentId": node.parent_id,
                })
            }).collect::<Vec<_>>(),
            "edges": payload.edges.into_iter().map(|edge| {
                json!({
                    "id": edge.id,
                    "kind": edge.kind,
                    "sourceId": edge.source_id,
                    "targetId": edge.target_id,
                    "label": edge.label,
                    "evidence": {
                        "path": edge.evidence.path,
                        "lineStart": edge.evidence.line_start,
                        "lineEnd": edge.evidence.line_end,
                        "confidence": round_f64(edge.evidence.confidence),
                    }
                })
            }).collect::<Vec<_>>(),
            "projections": payload.projections.into_iter().map(|projection| {
                json!({
                    "kind": projection.kind,
                    "source": projection.source,
                    "nodeCount": projection.node_count,
                    "edgeCount": projection.edge_count,
                })
            }).collect::<Vec<_>>(),
            "documentMetadata": payload.document_metadata.as_ref().map(|metadata| {
                json!({
                    "docId": metadata.doc_id,
                    "title": metadata.title,
                    "tags": metadata.tags,
                    "docType": metadata.doc_type,
                    "updated": metadata.updated,
                    "parent": metadata.parent.as_ref().map(|link| {
                        json!({
                            "label": link.label,
                            "kind": link.kind,
                            "literal": link.literal,
                            "docId": link.doc_id,
                            "path": link.path,
                            "title": link.title,
                            "targetAddress": link.target_address,
                        })
                    }),
                    "outgoingLinks": metadata.outgoing_links.iter().map(|link| {
                        json!({
                            "label": link.label,
                            "kind": link.kind,
                            "literal": link.literal,
                            "relationType": link.relation_type,
                            "metadataOwner": link.metadata_owner,
                            "docId": link.doc_id,
                            "path": link.path,
                            "title": link.title,
                            "targetAddress": link.target_address,
                        })
                    }).collect::<Vec<_>>(),
                    "backlinks": metadata.backlinks.iter().map(|link| {
                        json!({
                            "label": link.label,
                            "kind": link.kind,
                            "docId": link.doc_id,
                            "path": link.path,
                            "title": link.title,
                        })
                    }).collect::<Vec<_>>(),
                })
            }),
            "diagnostics": payload.diagnostics,
        }),
    );
}

#[tokio::test]
async fn analyze_markdown_emits_document_metadata_from_parser_and_graph_index() {
    let fixture = make_analysis_fixture();
    let payload = analyze_markdown(&fixture.state, "main/docs/deepwiki.md")
        .await
        .unwrap_or_else(|err| panic!("expected markdown analysis to succeed: {err:?}"));
    let metadata = payload
        .document_metadata
        .unwrap_or_else(|| panic!("expected markdown document metadata"));

    assert_eq!(metadata.doc_id.as_deref(), Some("docs/deepwiki"));
    assert_eq!(metadata.title, "DeepWiki Kernel");
    assert_eq!(
        metadata.tags,
        vec!["docs".to_string(), "feature".to_string()]
    );
    assert_eq!(
        metadata
            .parent
            .as_ref()
            .and_then(|row| row.doc_id.as_deref()),
        Some("docs/index")
    );
    assert_eq!(
        metadata.parent.as_ref().map(|row| row.kind),
        Some(crate::gateway::studio::types::MarkdownAnalysisDocumentLinkKind::Parent)
    );

    let relation_row = metadata
        .outgoing_links
        .iter()
        .find(|row| {
            row.kind == crate::gateway::studio::types::MarkdownAnalysisDocumentLinkKind::Relation
        })
        .unwrap_or_else(|| panic!("expected explicit relation row"));
    assert_eq!(relation_row.relation_type.as_deref(), Some("RELATED_TO"));
    assert_eq!(relation_row.doc_id.as_deref(), Some("docs/guide"));

    let index_row = metadata
        .outgoing_links
        .iter()
        .find(|row| {
            row.kind == crate::gateway::studio::types::MarkdownAnalysisDocumentLinkKind::Index
        })
        .unwrap_or_else(|| panic!("expected index relation row"));
    assert_eq!(index_row.doc_id.as_deref(), Some("docs/guide"));

    let backlink_labels = metadata
        .backlinks
        .iter()
        .map(|row| row.label.as_str())
        .collect::<Vec<_>>();
    assert!(backlink_labels.contains(&"Guide"));
    assert!(backlink_labels.contains(&"Index"));
}

#[tokio::test]
async fn analyze_markdown_rejects_non_markdown_content() {
    let fixture = make_analysis_fixture();
    let result = analyze_markdown(&fixture.state, "main/docs/raw.rs").await;
    let Err(error) = result else {
        panic!("expected non-markdown analysis request to fail");
    };

    match error {
        AnalysisError::UnsupportedContentType(content_type) => {
            assert_eq!(content_type, "text/x-rust");
        }
        AnalysisError::Vfs(vfs_error) => panic!("expected content-type failure, got {vfs_error}"),
    }
}

#[tokio::test]
async fn analyze_markdown_rejects_unqualified_vfs_paths() {
    let fixture = make_analysis_fixture();
    let result = analyze_markdown(&fixture.state, "docs/analysis.md").await;
    let Err(error) = result else {
        panic!("expected unqualified markdown analysis request to fail");
    };

    assert_studio_json_snapshot(
        "analysis_markdown_unqualified_path_error",
        json!({
            "error": error.to_string(),
        }),
    );
}

#[tokio::test]
async fn analyze_markdown_emits_retrieval_atoms_for_document_sections_and_code_blocks() {
    let fixture = make_analysis_fixture();
    let payload = analyze_markdown(&fixture.state, "main/docs/analysis.md")
        .await
        .unwrap_or_else(|err| panic!("expected markdown analysis to succeed: {err:?}"));

    let top_section = payload
        .nodes
        .iter()
        .find(|node| node.id == "sec:1")
        .unwrap_or_else(|| panic!("expected H1 section node"));
    assert_eq!(top_section.line_start, 1);
    assert_eq!(top_section.line_end, 2);

    let links_section = payload
        .nodes
        .iter()
        .find(|node| node.id == "sec:7")
        .unwrap_or_else(|| panic!("expected H2 section node"));
    assert_eq!(links_section.line_start, 7);
    assert_eq!(links_section.line_end, 17);

    assert!(payload.retrieval_atoms.iter().any(|atom| {
        atom.owner_id == "doc:0"
            && atom.chunk_id.starts_with("md:main-docs-analysis-md:doc-0")
            && atom.semantic_type == "document"
            && atom.line_start == Some(1)
            && atom.line_end == Some(17)
            && atom.fingerprint.starts_with("fp:")
            && atom.token_estimate > 0
    }));

    assert!(payload.retrieval_atoms.iter().any(|atom| {
        atom.owner_id == "sec:7"
            && atom.chunk_id.starts_with("md:main-docs-analysis-md:sec-7")
            && atom.semantic_type == "h2"
            && atom.line_start == Some(7)
            && atom.line_end == Some(17)
            && atom.fingerprint.starts_with("fp:")
            && atom.token_estimate > 0
    }));

    assert!(payload.retrieval_atoms.iter().any(|atom| {
        atom.owner_id == "code:15"
            && atom
                .chunk_id
                .starts_with("md:main-docs-analysis-md:code-15")
            && atom.semantic_type == "code:rust"
            && atom.line_start == Some(15)
            && atom.line_end == Some(17)
            && atom.fingerprint.starts_with("fp:")
            && atom.token_estimate > 0
    }));
}

#[tokio::test]
async fn analyze_markdown_emits_retrieval_atoms_for_tables() {
    let fixture = make_analysis_fixture();
    std::fs::write(
        fixture.temp_dir.path().join("docs/table.md"),
        r"# Performance

| Model | FP32 | INT8 |
| --- | --- | --- |
| BERT | 120 | 42 |
",
    )
    .unwrap_or_else(|err| panic!("failed to write table markdown fixture: {err}"));

    let payload = analyze_markdown(&fixture.state, "main/docs/table.md")
        .await
        .unwrap_or_else(|err| panic!("expected markdown analysis to succeed: {err:?}"));

    assert!(payload.retrieval_atoms.iter().any(|atom| {
        atom.owner_id == "table:3"
            && atom.chunk_id.starts_with("md:main-docs-table-md:table-3")
            && atom.semantic_type == "table"
            && atom.line_start == Some(3)
            && atom.line_end == Some(5)
            && atom.fingerprint.starts_with("fp:")
            && atom.token_estimate > 0
    }));
}

#[tokio::test]
async fn analyze_markdown_emits_retrieval_atoms_for_display_math() {
    let fixture = make_analysis_fixture();
    std::fs::write(
        fixture.temp_dir.path().join("docs/math.md"),
        "# Formula\n\n$$\nQ = clamp(round(R / S + Z), qmin, qmax)\n$$\n",
    )
    .unwrap_or_else(|err| panic!("failed to write math markdown fixture: {err}"));

    let payload = analyze_markdown(&fixture.state, "main/docs/math.md")
        .await
        .unwrap_or_else(|err| panic!("expected markdown analysis to succeed: {err:?}"));

    assert!(payload.retrieval_atoms.iter().any(|atom| {
        atom.owner_id.starts_with("math:")
            && atom.chunk_id.starts_with("md:main-docs-math-md:math-")
            && atom.semantic_type == "math:block"
            && atom.line_end >= atom.line_start
            && atom.fingerprint.starts_with("fp:")
            && atom.token_estimate > 0
    }));
}
