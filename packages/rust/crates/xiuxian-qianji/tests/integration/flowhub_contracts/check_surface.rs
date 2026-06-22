use super::{
    assert_common_diagnostic_shape, check_flowhub, create_flowhub_with_disconnected_mermaid_case,
    create_flowhub_with_invalid_bpmn_source_pair, create_flowhub_with_invalid_mermaid_case,
    create_flowhub_with_leaf_local_mermaid_case,
    create_flowhub_with_mermaid_presentation_directives_case,
    create_flowhub_with_topology_mismatch_case, create_flowhub_with_undeclared_mermaid_nodes_case,
    create_flowhub_with_unregistered_top_level_dir, create_invalid_flowhub,
    create_leaf_with_unregistered_child_dir_flowhub, create_missing_root_contract_flowhub,
    flowhub_root, real_flowhub_fixture_available, render_flowhub_check_markdown,
};
use tempfile::TempDir;

#[test]
fn check_flowhub_accepts_real_root() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let report = check_flowhub(flowhub_root())
        .unwrap_or_else(|error| panic!("real Flowhub root should check: {error}"));

    assert!(report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert!(rendered.contains("# Validation Passed"));
}

#[test]
fn check_flowhub_accepts_real_research_module() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let report = check_flowhub(flowhub_root().join("research/paper"))
        .unwrap_or_else(|error| panic!("real research module should check: {error}"));

    assert!(report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert!(rendered.contains("# Validation Passed"));
}

#[test]
fn check_flowhub_reports_invalid_mermaid_scenario_case() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_flowhub_with_invalid_mermaid_case(&temp_dir);

    let report = check_flowhub(&root)
        .unwrap_or_else(|error| panic!("invalid Mermaid case should still report: {error}"));

    assert!(!report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Invalid scenario-case graph"));
    assert!(rendered.contains("codex-plan.mmd"));
}

#[test]
fn check_flowhub_reports_disconnected_mermaid_module_backbone() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_flowhub_with_disconnected_mermaid_case(&temp_dir);

    let report = check_flowhub(&root)
        .unwrap_or_else(|error| panic!("disconnected Mermaid case should still report: {error}"));

    assert!(!report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Invalid scenario-case graph"));
    assert!(rendered.contains("disconnected Flowhub module backbone nodes"));
    assert!(rendered.contains("codex-plan.mmd"));
}

#[test]
fn check_flowhub_accepts_leaf_local_mermaid_case() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_flowhub_with_leaf_local_mermaid_case(&temp_dir);

    let report = check_flowhub(&root)
        .unwrap_or_else(|error| panic!("leaf-local Mermaid case should still report: {error}"));

    assert!(report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert!(rendered.contains("# Validation Passed"));
}

#[test]
fn check_flowhub_reports_invalid_required_bpmn_source_pair() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_flowhub_with_invalid_bpmn_source_pair(&temp_dir);

    let report = check_flowhub(&root)
        .unwrap_or_else(|error| panic!("invalid BPMN source pair should still report: {error}"));

    assert!(!report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Invalid Flowhub BPMN scenario source"));
    assert!(rendered.contains("docs-search.bpmn"));
}

#[test]
fn check_flowhub_reports_topology_mismatch_for_declared_graph_contract() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_flowhub_with_topology_mismatch_case(&temp_dir);

    let report = check_flowhub(&root)
        .unwrap_or_else(|error| panic!("topology-mismatch Flowhub should still report: {error}"));

    assert!(!report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Invalid scenario-case topology"));
    assert!(rendered.contains("topology `dag`"));
    assert!(rendered.contains("resolved `bounded_loop`"));
}

#[test]
fn check_flowhub_reports_undeclared_graph_nodes_in_mermaid_case() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_flowhub_with_undeclared_mermaid_nodes_case(&temp_dir);

    let report = check_flowhub(&root).unwrap_or_else(|error| {
        panic!("Mermaid case with undeclared graph nodes should still report: {error}")
    });

    assert!(!report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Invalid scenario-case graph"));
    assert!(rendered.contains("undeclared graph nodes"));
    assert!(rendered.contains("style"));
    assert!(rendered.contains("codex-plan.mmd"));
}

#[test]
fn check_flowhub_accepts_mermaid_case_with_presentation_directives() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_flowhub_with_mermaid_presentation_directives_case(&temp_dir);

    let report = check_flowhub(&root).unwrap_or_else(|error| {
        panic!("Mermaid case with presentation directives should still report: {error}")
    });

    assert!(report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert!(rendered.contains("# Validation Passed"));
}

#[test]
fn check_flowhub_reports_missing_required_module_paths() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_invalid_flowhub(&temp_dir);

    let report = check_flowhub(&root)
        .unwrap_or_else(|error| panic!("invalid Flowhub should still report: {error}"));

    assert!(!report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Missing contract path"));
    assert!(rendered.contains("broken-module"));
    assert!(!rendered.contains("## Follow-up Query"));
}

#[test]
fn check_flowhub_blocks_missing_root_contract() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_missing_root_contract_flowhub(&temp_dir);

    let report = check_flowhub(&root)
        .unwrap_or_else(|error| panic!("invalid Flowhub root should still report: {error}"));

    assert!(!report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Invalid Flowhub root contract"));
    assert!(rendered.contains("[contract]"));
}

#[test]
fn check_flowhub_blocks_unregistered_child_directory_under_leaf_node() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_leaf_with_unregistered_child_dir_flowhub(&temp_dir);

    let report = check_flowhub(&root)
        .unwrap_or_else(|error| panic!("Flowhub drift should still report: {error}"));

    assert!(!report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Unregistered child directory"));
    assert!(rendered.contains("style"));
    assert!(rendered.contains("contract.register"));
}

#[test]
fn check_flowhub_blocks_unregistered_top_level_directory() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let root = create_flowhub_with_unregistered_top_level_dir(&temp_dir);

    let report = check_flowhub(&root)
        .unwrap_or_else(|error| panic!("Flowhub top-level drift should still report: {error}"));

    assert!(!report.is_valid());
    let rendered = render_flowhub_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Unregistered Flowhub module"));
    assert!(rendered.contains("scratch"));
    assert!(rendered.contains("contract.register"));
}
