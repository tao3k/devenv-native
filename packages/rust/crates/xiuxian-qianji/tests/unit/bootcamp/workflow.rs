use super::{parse_manifest, parsed_manifest_requires_link_graph};

#[test]
fn manifest_without_knowledge_nodes_skips_link_graph_requirement() {
    let manifest = parse_manifest(
        r#"
name = "AnnotationOnly"

[[nodes]]
id = "annotate"
task_type = "annotation"
weight = 1.0
params = { prompt = "noop" }
"#,
    )
    .unwrap_or_else(|error| panic!("manifest should parse: {error}"));

    assert!(!parsed_manifest_requires_link_graph(&manifest));
}

#[test]
fn manifest_with_knowledge_nodes_requires_link_graph() {
    let manifest = parse_manifest(
        r#"
name = "KnowledgeOnly"

[[nodes]]
id = "search"
task_type = "knowledge"
weight = 1.0
params = {}
"#,
    )
    .unwrap_or_else(|error| panic!("manifest should parse: {error}"));

    assert!(parsed_manifest_requires_link_graph(&manifest));
}
