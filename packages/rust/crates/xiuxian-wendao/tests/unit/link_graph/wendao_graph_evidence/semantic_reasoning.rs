use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serial_test::serial;
use xiuxian_wendao_julia::integration_support::probe_wendaograph_page_index_host_request_with_fixture;
use xiuxian_wendao_julia::validate_wendao_graph_page_index_reasoning_request_schema;
use xiuxian_wendao_parsers::semantic_ssot::{
    SemanticBundleProvenance, SemanticConfidence, SemanticConfidenceSource, SemanticObject,
    SemanticObjectKind, SemanticOwner, SemanticProvenance, SemanticRelation, SemanticRelationEdge,
    SemanticRelationKind, SemanticScopeBundle, SemanticStatus, SemanticVerification,
};

use super::support::{
    float_column, int64_column, page_index_edge_rows, page_index_node_rows, page_index_seed_rows,
    string_column,
};
use crate::link_graph::{
    LinkGraphWendaoGraphEvidenceError, WendaoGraphPageIndexReasoningRequestBundle,
    WendaoGraphPageIndexReasoningRequestOptions,
    build_semantic_scope_page_index_reasoning_request_bundle,
    build_semantic_scope_page_index_reasoning_request_bundle_with_options,
};

const RUN_WENDAOGRAPH_SEMANTIC_PAGE_INDEX_LIVE_PROOF_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_SEMANTIC_PAGE_INDEX_LIVE_PROOF_TEST";

fn semantic_object(
    id: &str,
    kind: SemanticObjectKind,
    title: &str,
    source_path: &str,
    relations: Vec<SemanticRelation>,
) -> SemanticObject {
    SemanticObject {
        id: id.to_string(),
        kind,
        title: title.to_string(),
        status: SemanticStatus::Active,
        confidence: SemanticConfidence {
            score: 0.98,
            source: SemanticConfidenceSource::Verified,
        },
        owners: vec![SemanticOwner {
            scope: "xiuxian-wendao".to_string(),
            role: "semantic-owner".to_string(),
        }],
        provenance: SemanticProvenance {
            source: "docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md".to_string(),
            recorded_by: "Artisan Workshop".to_string(),
            recorded_at: "2026-05-07".to_string(),
        },
        verification: SemanticVerification {
            required: vec!["cargo test -p xiuxian-wendao semantic_reasoning".to_string()],
            evidence: vec!["semantic-projection-fixture".to_string()],
        },
        relations,
        body: format!("{title} semantic reasoning fixture body."),
        source_path: PathBuf::from(source_path),
    }
}

fn fixture_scope() -> SemanticScopeBundle {
    let task = semantic_object(
        "task.semantic-reasoning-tree",
        SemanticObjectKind::Task,
        "Semantic Reasoning Tree",
        "semantic/tasks/reasoning-tree.md",
        vec![
            SemanticRelation {
                kind: SemanticRelationKind::Contains,
                target: "component.semantic-ssot".to_string(),
            },
            SemanticRelation {
                kind: SemanticRelationKind::Governs,
                target: "decision.julia-derived-evidence".to_string(),
            },
        ],
    );
    let component = semantic_object(
        "component.semantic-ssot",
        SemanticObjectKind::Component,
        "Semantic SSOT",
        "semantic/components/semantic-ssot.md",
        vec![SemanticRelation {
            kind: SemanticRelationKind::Constrains,
            target: "decision.julia-derived-evidence".to_string(),
        }],
    );
    let decision = semantic_object(
        "decision.julia-derived-evidence",
        SemanticObjectKind::Decision,
        "Julia Derived Evidence",
        "semantic/decisions/julia-derived-evidence.md",
        Vec::new(),
    );

    SemanticScopeBundle {
        task_id: Some(task.id.clone()),
        requested_object_ids: vec![component.id.clone()],
        relations: vec![
            SemanticRelationEdge {
                source: task.id.clone(),
                kind: SemanticRelationKind::Contains,
                target: component.id.clone(),
            },
            SemanticRelationEdge {
                source: task.id.clone(),
                kind: SemanticRelationKind::Governs,
                target: decision.id.clone(),
            },
            SemanticRelationEdge {
                source: component.id.clone(),
                kind: SemanticRelationKind::Constrains,
                target: decision.id.clone(),
            },
        ],
        affected_invariants: Vec::new(),
        required_validations: vec!["cargo test -p xiuxian-wendao semantic_reasoning".to_string()],
        projection_revision: "semantic-projection-fixture".to_string(),
        projection_source_revision: Some("semantic-source-fixture".to_string()),
        projection_staleness: None,
        provenance: vec![
            SemanticBundleProvenance {
                object_id: task.id.clone(),
                source_path: task.source_path.clone(),
                source: task.provenance.source.clone(),
            },
            SemanticBundleProvenance {
                object_id: component.id.clone(),
                source_path: component.source_path.clone(),
                source: component.provenance.source.clone(),
            },
            SemanticBundleProvenance {
                object_id: decision.id.clone(),
                source_path: decision.source_path.clone(),
                source: decision.provenance.source.clone(),
            },
        ],
        objects: vec![task, component, decision],
        change_intents: Vec::new(),
        unresolved_ids: Vec::new(),
    }
}

#[test]
fn semantic_scope_projects_to_page_index_reasoning_tables() {
    let bundle = build_semantic_scope_page_index_reasoning_request_bundle(&fixture_scope())
        .unwrap_or_else(|error| panic!("build semantic reasoning bundle: {error}"));

    validate_wendao_graph_page_index_reasoning_request_schema(
        "page_index_nodes",
        bundle.nodes.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate page_index_nodes schema: {error}"));
    validate_wendao_graph_page_index_reasoning_request_schema(
        "page_index_edges",
        bundle.edges.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate page_index_edges schema: {error}"));
    validate_wendao_graph_page_index_reasoning_request_schema(
        "page_index_seeds",
        bundle.seeds.schema().as_ref(),
    )
    .unwrap_or_else(|error| panic!("validate page_index_seeds schema: {error}"));

    assert_eq!(
        string_column(&bundle.nodes, 0),
        vec![
            "component.semantic-ssot".to_string(),
            "decision.julia-derived-evidence".to_string(),
            "task.semantic-reasoning-tree".to_string(),
        ]
    );
    assert_eq!(
        string_column(&bundle.nodes, 1),
        vec![
            "semantic/components/semantic-ssot.md".to_string(),
            "semantic/decisions/julia-derived-evidence.md".to_string(),
            "semantic/tasks/reasoning-tree.md".to_string(),
        ]
    );
    assert_eq!(
        string_column(&bundle.nodes, 2),
        vec![
            "task.semantic-reasoning-tree".to_string(),
            String::new(),
            String::new(),
        ]
    );
    assert_eq!(int64_column(&bundle.nodes, 3), vec![1, 0, 0]);

    let summaries = string_column(&bundle.nodes, 6);
    assert!(summaries[0].contains("semantic_kind=component"));
    assert!(summaries[0].contains("semantic_status=active"));
    assert!(summaries[0].contains("source_path=semantic/components/semantic-ssot.md"));

    assert_eq!(
        string_column(&bundle.edges, 0),
        vec![
            "task.semantic-reasoning-tree".to_string(),
            "task.semantic-reasoning-tree".to_string(),
            "component.semantic-ssot".to_string(),
        ]
    );
    assert_eq!(
        string_column(&bundle.edges, 2),
        vec![
            "contains".to_string(),
            "governs".to_string(),
            "constrains".to_string(),
        ]
    );
    assert_eq!(float_column(&bundle.edges, 3), vec![1.0, 0.95, 0.95]);

    assert_eq!(
        string_column(&bundle.seeds, 0),
        vec![
            "task.semantic-reasoning-tree".to_string(),
            "component.semantic-ssot".to_string(),
        ]
    );
    assert_eq!(float_column(&bundle.seeds, 1), vec![1.0, 0.8]);
    assert_eq!(
        string_column(&bundle.seeds, 2),
        vec![
            "semantic_task_anchor".to_string(),
            "semantic_requested_object".to_string(),
        ]
    );
}

#[test]
fn semantic_scope_accepts_explicit_seed_options() {
    let options = WendaoGraphPageIndexReasoningRequestOptions::default().with_seed(
        "decision.julia-derived-evidence",
        0.5,
        "agent_query",
    );
    let bundle = build_semantic_scope_page_index_reasoning_request_bundle_with_options(
        &fixture_scope(),
        &options,
    )
    .unwrap_or_else(|error| panic!("build explicit semantic reasoning bundle: {error}"));

    assert_eq!(
        string_column(&bundle.seeds, 0),
        vec!["decision.julia-derived-evidence".to_string()]
    );
    assert_eq!(float_column(&bundle.seeds, 1), vec![0.5]);
    assert_eq!(
        string_column(&bundle.seeds, 2),
        vec!["agent_query".to_string()]
    );
}

#[test]
#[serial]
fn semantic_scope_page_index_reasoning_live_proof_runs_real_wendaograph_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_SEMANTIC_PAGE_INDEX_LIVE_PROOF_TEST_ENV).is_none() {
        eprintln!(
            "skipping semantic PageIndex live proof; set {RUN_WENDAOGRAPH_SEMANTIC_PAGE_INDEX_LIVE_PROOF_TEST_ENV}=1 and WENDAOGRAPH_PACKAGE_DIR"
        );
        return;
    }

    let bundle = build_semantic_scope_page_index_reasoning_request_bundle(&fixture_scope())
        .unwrap_or_else(|error| panic!("build semantic reasoning bundle: {error}"));
    let temp = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("create semantic PageIndex fixture temp dir: {error}"));
    write_semantic_page_index_fixture(temp.path(), &bundle);

    let report = probe_wendaograph_page_index_host_request_with_fixture(temp.path(), 2)
        .unwrap_or_else(|error| panic!("run semantic PageIndex live proof: {error}"));

    assert_eq!(report.sample_count, 2);
    assert_eq!(bundle.nodes.num_rows(), 3);
    assert_eq!(bundle.edges.num_rows(), 3);
    assert_eq!(bundle.seeds.num_rows(), 2);
    assert!(report.frontier_rows > 0);
    assert!(report.trace_rows > 0);
    assert!(report.first_ms >= 0.0);
    assert!(report.warm_min_ms >= 0.0);
    assert!(report.warm_median_ms >= report.warm_min_ms);
    assert!(report.warm_p95_ms >= report.warm_median_ms);
    assert!(report.warm_max_ms >= report.warm_p95_ms);
    eprintln!(
        "wendaograph_semantic_page_index_live_proof_summary sample_count={} semantic_nodes={} semantic_edges={} semantic_seeds={} frontier_rows={} trace_rows={} first_ms={:.3} warm_median_ms={:.3} warm_p95_ms={:.3} warm_max_ms={:.3}",
        report.sample_count,
        bundle.nodes.num_rows(),
        bundle.edges.num_rows(),
        bundle.seeds.num_rows(),
        report.frontier_rows,
        report.trace_rows,
        report.first_ms,
        report.warm_median_ms,
        report.warm_p95_ms,
        report.warm_max_ms
    );
}

#[test]
fn semantic_scope_rejects_relation_outside_projected_scope() {
    let mut scope = fixture_scope();
    scope.relations.push(SemanticRelationEdge {
        source: "component.semantic-ssot".to_string(),
        kind: SemanticRelationKind::Affects,
        target: "component.missing".to_string(),
    });

    let Err(error) = build_semantic_scope_page_index_reasoning_request_bundle(&scope) else {
        panic!("missing semantic relation target should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::SemanticRelationMissingNode { .. }
    ));
}

fn write_semantic_page_index_fixture(
    fixture_dir: &Path,
    bundle: &WendaoGraphPageIndexReasoningRequestBundle,
) {
    write_tsv_file(
        &fixture_dir.join("page_index_nodes.tsv"),
        &[
            "node_id",
            "page_id",
            "parent_id",
            "depth",
            "rank",
            "title",
            "summary",
            "line_start",
            "line_end",
            "token_count",
        ],
        page_index_node_rows(&bundle.nodes),
    );
    write_tsv_file(
        &fixture_dir.join("page_index_edges.tsv"),
        &["source_id", "target_id", "edge_kind", "weight"],
        page_index_edge_rows(&bundle.edges),
    );
    write_tsv_file(
        &fixture_dir.join("page_index_seeds.tsv"),
        &["node_id", "weight", "seed_kind"],
        page_index_seed_rows(&bundle.seeds),
    );
}

fn write_tsv_file(path: &Path, header: &[&str], rows: Vec<Vec<String>>) {
    let mut content = header.join("\t");
    content.push('\n');
    for row in rows {
        let cells = row
            .iter()
            .map(|cell| cell.replace(['\t', '\n', '\r'], " "))
            .collect::<Vec<_>>();
        content.push_str(&cells.join("\t"));
        content.push('\n');
    }
    fs::write(path, content).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

#[test]
fn semantic_scope_rejects_containment_cycles() {
    let mut scope = fixture_scope();
    scope.relations.push(SemanticRelationEdge {
        source: "component.semantic-ssot".to_string(),
        kind: SemanticRelationKind::Contains,
        target: "task.semantic-reasoning-tree".to_string(),
    });

    let Err(error) = build_semantic_scope_page_index_reasoning_request_bundle(&scope) else {
        panic!("semantic containment cycle should fail");
    };

    assert!(matches!(
        error,
        LinkGraphWendaoGraphEvidenceError::SemanticContainmentCycle { .. }
    ));
}
