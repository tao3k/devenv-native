use super::*;

struct GraphFixture {
    state: Arc<GatewayState>,
    _temp_dir: tempfile::TempDir,
}

#[derive(Debug, Deserialize)]
struct TestWendaoConfig {
    link_graph: Option<TestLinkGraphConfig>,
}

#[derive(Debug, Deserialize)]
struct TestLinkGraphConfig {
    projects: Option<std::collections::BTreeMap<String, TestProjectConfig>>,
}

#[derive(Debug, Deserialize)]
struct TestProjectConfig {
    root: String,
    #[serde(default)]
    dirs: Vec<String>,
}

pub(super) fn round_f32(value: f32) -> f32 {
    ((value as f64) * 10_000.0).round() as f32 / 10_000.0_f32
}

pub(super) fn make_graph_fixture(docs: Vec<(&str, &str)>) -> GraphFixture {
    let temp_dir =
        tempdir().unwrap_or_else(|err| panic!("failed to create graph fixture tempdir: {err}"));
    for (name, content) in docs {
        let absolute_path = temp_dir.path().join(name);
        if let Some(parent) = absolute_path.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("failed to create fixture doc parent {name}: {err}"));
        }
        std::fs::write(absolute_path, content)
            .unwrap_or_else(|err| panic!("failed to write fixture doc {name}: {err}"));
    }

    let mut studio_state = StudioState::new();
    studio_state.project_root = temp_dir.path().to_path_buf();
    studio_state.config_root = temp_dir.path().to_path_buf();

    GraphFixture {
        state: Arc::new(GatewayState {
            index: None,
            signal_tx: None,
            webhook_url: None,
            studio: Arc::new(studio_state),
        }),
        _temp_dir: temp_dir,
    }
}

pub(super) fn push_ui_config_from_toml(fixture: &GraphFixture, toml_content: &str) {
    let parsed: TestWendaoConfig = toml::from_str(toml_content)
        .unwrap_or_else(|err| panic!("failed to parse test wendao.toml: {err}"));
    let projects = parsed
        .link_graph
        .and_then(|link_graph| link_graph.projects)
        .unwrap_or_default()
        .into_iter()
        .map(
            |(name, project)| crate::gateway::studio::types::UiProjectConfig {
                name,
                root: project.root,
                dirs: project.dirs,
            },
        )
        .collect::<Vec<_>>();

    fixture
        .state
        .studio
        .seed_eager_configured_owners_for_tests(UiConfig {
            projects,
            repo_projects: Vec::new(),
        });
}

pub(super) fn sorted_graph_nodes_payload(nodes: Vec<GraphNode>) -> Vec<serde_json::Value> {
    let mut payload = nodes
        .into_iter()
        .map(|node| {
            json!({
                "id": node.id,
                "label": node.label,
                "path": node.path,
                "navigationTarget": node.navigation_target,
                "nodeType": node.node_type,
                "isCenter": node.is_center,
                "distance": node.distance,
            })
        })
        .collect::<Vec<_>>();
    payload.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
    payload
}

pub(super) fn sorted_graph_links_payload(links: Vec<GraphLink>) -> Vec<serde_json::Value> {
    let mut payload = links
        .into_iter()
        .map(|link| {
            json!({
                "source": link.source,
                "target": link.target,
                "direction": link.direction,
                "distance": link.distance,
            })
        })
        .collect::<Vec<_>>();
    payload.sort_by(|left, right| {
        left["source"]
            .as_str()
            .cmp(&right["source"].as_str())
            .then_with(|| left["target"].as_str().cmp(&right["target"].as_str()))
    });
    payload
}

pub(super) fn markdown_analysis_response_with_section_graph()
-> crate::gateway::studio::types::MarkdownAnalysisResponse {
    use crate::gateway::studio::types::MarkdownAnalysisResponse;

    MarkdownAnalysisResponse {
        path: "main/docs/index.md".to_string(),
        document_hash: "doc-hash".to_string(),
        node_count: 6,
        edge_count: 5,
        nodes: markdown_analysis_section_nodes(),
        edges: markdown_analysis_section_edges(),
        projections: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn markdown_analysis_section_nodes() -> Vec<crate::gateway::studio::types::AnalysisNode> {
    use crate::gateway::studio::types::{AnalysisNode, AnalysisNodeKind};

    vec![
        AnalysisNode {
            id: "main/docs/index.md#document".to_string(),
            kind: AnalysisNodeKind::Document,
            label: "index.md".to_string(),
            depth: 0,
            line_start: 1,
            line_end: 20,
            parent_id: None,
        },
        AnalysisNode {
            id: "main/docs/index.md#section:overview".to_string(),
            kind: AnalysisNodeKind::Section,
            label: "Overview".to_string(),
            depth: 1,
            line_start: 3,
            line_end: 12,
            parent_id: Some("main/docs/index.md#document".to_string()),
        },
        AnalysisNode {
            id: "main/docs/index.md#task:1".to_string(),
            kind: AnalysisNodeKind::Task,
            label: "Finish gateway fallback".to_string(),
            depth: 2,
            line_start: 8,
            line_end: 8,
            parent_id: Some("main/docs/index.md#section:overview".to_string()),
        },
        AnalysisNode {
            id: "main/docs/index.md#prop:id".to_string(),
            kind: AnalysisNodeKind::Property,
            label: ":ID: GraphProtocol".to_string(),
            depth: 2,
            line_start: 4,
            line_end: 4,
            parent_id: Some("main/docs/index.md#section:overview".to_string()),
        },
        AnalysisNode {
            id: "main/docs/index.md#observe:1".to_string(),
            kind: AnalysisNodeKind::Observation,
            label: ":OBSERVE: lang:rust \"fn compile() { $$$ }\"".to_string(),
            depth: 2,
            line_start: 5,
            line_end: 5,
            parent_id: Some("main/docs/index.md#section:overview".to_string()),
        },
        AnalysisNode {
            id: "main/docs/index.md#symbol:compile".to_string(),
            kind: AnalysisNodeKind::Symbol,
            label: "compile".to_string(),
            depth: 3,
            line_start: 5,
            line_end: 5,
            parent_id: Some("main/docs/index.md#observe:1".to_string()),
        },
    ]
}

fn markdown_analysis_section_edges() -> Vec<crate::gateway::studio::types::AnalysisEdge> {
    use crate::gateway::studio::types::{AnalysisEdge, AnalysisEdgeKind, AnalysisEvidence};

    vec![
        AnalysisEdge {
            id: "e1".to_string(),
            kind: AnalysisEdgeKind::Contains,
            source_id: "main/docs/index.md#document".to_string(),
            target_id: "main/docs/index.md#section:overview".to_string(),
            label: Some("contains".to_string()),
            evidence: AnalysisEvidence {
                path: "main/docs/index.md".to_string(),
                line_start: 3,
                line_end: 12,
                confidence: 1.0,
            },
        },
        AnalysisEdge {
            id: "e2".to_string(),
            kind: AnalysisEdgeKind::Contains,
            source_id: "main/docs/index.md#section:overview".to_string(),
            target_id: "main/docs/index.md#prop:id".to_string(),
            label: Some("contains".to_string()),
            evidence: AnalysisEvidence {
                path: "main/docs/index.md".to_string(),
                line_start: 4,
                line_end: 4,
                confidence: 1.0,
            },
        },
        AnalysisEdge {
            id: "e3".to_string(),
            kind: AnalysisEdgeKind::Contains,
            source_id: "main/docs/index.md#section:overview".to_string(),
            target_id: "main/docs/index.md#observe:1".to_string(),
            label: Some("contains".to_string()),
            evidence: AnalysisEvidence {
                path: "main/docs/index.md".to_string(),
                line_start: 5,
                line_end: 5,
                confidence: 1.0,
            },
        },
        AnalysisEdge {
            id: "e4".to_string(),
            kind: AnalysisEdgeKind::NextStep,
            source_id: "main/docs/index.md#section:overview".to_string(),
            target_id: "main/docs/index.md#task:1".to_string(),
            label: Some("next".to_string()),
            evidence: AnalysisEvidence {
                path: "main/docs/index.md".to_string(),
                line_start: 8,
                line_end: 8,
                confidence: 0.9,
            },
        },
        AnalysisEdge {
            id: "e5".to_string(),
            kind: AnalysisEdgeKind::References,
            source_id: "main/docs/index.md#observe:1".to_string(),
            target_id: "main/docs/index.md#symbol:compile".to_string(),
            label: Some("compile".to_string()),
            evidence: AnalysisEvidence {
                path: "main/docs/index.md".to_string(),
                line_start: 5,
                line_end: 5,
                confidence: 0.95,
            },
        },
    ]
}

pub(super) fn markdown_analysis_response_for_symbol(
    observe_label: &str,
) -> crate::gateway::studio::types::MarkdownAnalysisResponse {
    use crate::gateway::studio::types::{
        AnalysisEdge, AnalysisEdgeKind, AnalysisEvidence, AnalysisNode, AnalysisNodeKind,
        MarkdownAnalysisResponse,
    };

    MarkdownAnalysisResponse {
        path: "docs/index.md".to_string(),
        document_hash: "doc-hash".to_string(),
        node_count: 3,
        edge_count: 2,
        nodes: vec![
            AnalysisNode {
                id: "docs/index.md#document".to_string(),
                kind: AnalysisNodeKind::Document,
                label: "index.md".to_string(),
                depth: 0,
                line_start: 1,
                line_end: 3,
                parent_id: None,
            },
            AnalysisNode {
                id: "docs/index.md#observe:3:observe".to_string(),
                kind: AnalysisNodeKind::Observation,
                label: observe_label.to_string(),
                depth: 1,
                line_start: 3,
                line_end: 3,
                parent_id: Some("docs/index.md#document".to_string()),
            },
            AnalysisNode {
                id: "docs/index.md#symbol:alphaservice".to_string(),
                kind: AnalysisNodeKind::Symbol,
                label: "AlphaService".to_string(),
                depth: 2,
                line_start: 3,
                line_end: 3,
                parent_id: Some("docs/index.md#observe:3:observe".to_string()),
            },
        ],
        edges: vec![
            AnalysisEdge {
                id: "e1".to_string(),
                kind: AnalysisEdgeKind::Contains,
                source_id: "docs/index.md#document".to_string(),
                target_id: "docs/index.md#observe:3:observe".to_string(),
                label: Some("contains".to_string()),
                evidence: AnalysisEvidence {
                    path: "docs/index.md".to_string(),
                    line_start: 3,
                    line_end: 3,
                    confidence: 1.0,
                },
            },
            AnalysisEdge {
                id: "e2".to_string(),
                kind: AnalysisEdgeKind::References,
                source_id: "docs/index.md#observe:3:observe".to_string(),
                target_id: "docs/index.md#symbol:alphaservice".to_string(),
                label: Some("AlphaService".to_string()),
                evidence: AnalysisEvidence {
                    path: "docs/index.md".to_string(),
                    line_start: 3,
                    line_end: 3,
                    confidence: 1.0,
                },
            },
        ],
        projections: Vec::new(),
        diagnostics: Vec::new(),
    }
}

pub(super) use GraphFixture;
