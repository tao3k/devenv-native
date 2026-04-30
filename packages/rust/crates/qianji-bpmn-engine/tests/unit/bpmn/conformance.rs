use std::collections::{BTreeMap, BTreeSet};

use qianji_bpmn_engine::{
    BpmnConformanceEntry, BpmnConformanceStatus, BpmnParseOptions, LintIssue, LintReport,
    bpmn_conformance_registry, lint_bpmn_source, parse_bpmn_package, snapshot_bpmn_source,
};

use super::fixture_source;

const COVERAGE_DOC: &str =
    include_str!("../../../docs/omg-alignment/bpmn/full-conformance-coverage.md");

#[test]
fn conformance_status_strings_match_coverage_vocabulary() {
    let statuses = [
        (BpmnConformanceStatus::Supported, "supported"),
        (
            BpmnConformanceStatus::BoundedExecutable,
            "bounded executable",
        ),
        (BpmnConformanceStatus::MetadataOnly, "metadata-only"),
        (BpmnConformanceStatus::LintDeferred, "lint-deferred"),
        (BpmnConformanceStatus::Missing, "missing"),
    ];

    for (status, expected) in statuses {
        assert_eq!(status.as_str(), expected);
        assert_eq!(status.to_string(), expected);
    }
}

#[test]
fn conformance_registry_matches_full_conformance_coverage_matrix() {
    let docs_rows = coverage_matrix_rows();
    let mut docs_by_family = BTreeMap::new();
    for (family, status) in docs_rows {
        let previous = docs_by_family.insert(family, status);
        assert!(
            previous.is_none(),
            "duplicate coverage docs row for {family}"
        );
    }

    let mut registry_by_family = BTreeMap::new();
    for entry in bpmn_conformance_registry() {
        let previous = registry_by_family.insert(entry.family, entry);
        assert!(
            previous.is_none(),
            "duplicate conformance registry entry for {}",
            entry.family
        );
    }

    assert_eq!(
        registry_by_family.len(),
        docs_by_family.len(),
        "registry and coverage docs must expose the same family count"
    );

    for (family, docs_status) in docs_by_family {
        let Some(entry) = registry_by_family.get(family) else {
            panic!("registry is missing coverage family {family}");
        };
        assert_eq!(
            entry.status.as_str(),
            docs_status,
            "coverage status drift for {family}"
        );
    }

    for entry in bpmn_conformance_registry() {
        assert!(
            registry_by_family.contains_key(entry.family),
            "coverage docs are missing registry family {}",
            entry.family
        );
    }
}

#[test]
fn conformance_registry_entries_have_canonical_tracking_fields() {
    let mut families = BTreeSet::new();
    let allowed_statuses = BTreeSet::from([
        BpmnConformanceStatus::Supported,
        BpmnConformanceStatus::BoundedExecutable,
        BpmnConformanceStatus::MetadataOnly,
        BpmnConformanceStatus::LintDeferred,
        BpmnConformanceStatus::Missing,
    ]);

    for entry in bpmn_conformance_registry() {
        assert!(
            families.insert(entry.family),
            "duplicate family {}",
            entry.family
        );
        assert!(allowed_statuses.contains(&entry.status));
        assert!(allowed_statuses.contains(&entry.parser));
        assert!(allowed_statuses.contains(&entry.snapshot));
        assert!(allowed_statuses.contains(&entry.lint));
        assert!(allowed_statuses.contains(&entry.runtime));
        assert!(allowed_statuses.contains(&entry.host_surface));
        assert!(
            !entry.docs_anchor.is_empty(),
            "docs anchor is required for {}",
            entry.family
        );
        assert!(
            !entry.next_milestone.is_empty(),
            "next milestone is required for {}",
            entry.family
        );
        assert!(
            !entry.docs_anchor.contains(".cache/")
                && !entry.docs_anchor.contains(".data/")
                && !entry.docs_anchor.contains(".run/"),
            "docs anchor must not point at hidden workspace paths for {}",
            entry.family
        );
    }
}

#[test]
fn conformance_registry_tracks_m4_boundary_families() {
    assert_boundary("Complex gateway", BpmnConformanceStatus::LintDeferred);
    assert_boundary("Event subprocess", BpmnConformanceStatus::BoundedExecutable);
    assert_boundary(
        "Collaboration and pools",
        BpmnConformanceStatus::MetadataOnly,
    );
    assert_boundary("Data objects", BpmnConformanceStatus::BoundedExecutable);
    assert_boundary("Data stores", BpmnConformanceStatus::LintDeferred);
    assert_boundary("Interfaces/operations", BpmnConformanceStatus::MetadataOnly);
    assert_boundary("Global task catalogs", BpmnConformanceStatus::MetadataOnly);
    assert_boundary("Callable IO metadata", BpmnConformanceStatus::MetadataOnly);
    assert_boundary(
        "Resource-role metadata",
        BpmnConformanceStatus::MetadataOnly,
    );
    assert_boundary("Flow-element metadata", BpmnConformanceStatus::MetadataOnly);
    assert_boundary("BPMN DI", BpmnConformanceStatus::MetadataOnly);
}

#[test]
fn conformance_boundary_evidence_is_covered_by_lint_or_snapshot() {
    let complex_gateway = lint_fixture("invalid-unsupported-gateway.bpmn");
    let issue = single_issue(&complex_gateway, "bpmn.unsupported_complex_gateway");
    assert!(issue.llm_fix_prompt.contains("exclusiveGateway"));
    assert!(issue.why_it_failed.contains("fan-in"));

    let event_subprocess = lint_fixture("invalid-compensation-event-subprocess.bpmn");
    let issue = single_issue(
        &event_subprocess,
        "bpmn.unsupported_subprocess_configuration",
    );
    assert!(
        issue
            .why_it_failed
            .contains("compensation event subprocesses")
    );

    let operation_binding = lint_fixture("invalid-task-operation-binding.bpmn");
    assert!(!operation_binding.ok);
    let issue = operation_binding
        .issues
        .iter()
        .find(|issue| issue.evidence["task_id"].as_str() == Some("invoke_service"))
        .unwrap_or_else(|| panic!("operation binding evidence should include service task"));
    assert_eq!(issue.code, "bpmn.unsupported_operation_binding");
    assert_eq!(issue.evidence["task_id"], "invoke_service");
    assert_eq!(issue.evidence["operation_ref"], "Operation_Invoke");
    assert!(
        issue
            .why_it_failed
            .contains("interface and operation catalogs as metadata")
    );

    let collaboration = lint_fixture("invalid-collaboration-participant.bpmn");
    let issue = single_issue(&collaboration, "bpmn.unsupported_collaboration_surface");
    assert_eq!(issue.evidence["snapshot"]["participant_count"], 2);
    assert_eq!(issue.evidence["snapshot"]["message_flow_count"], 1);

    let data_object = lint_fixture("invalid-data-object-reference.bpmn");
    assert!(data_object.ok);
    assert!(data_object.issues.is_empty());
    let package = parse_bpmn_package(
        &[fixture_source("invalid-data-object-reference.bpmn")],
        &BpmnParseOptions::default(),
    )
    .unwrap_or_else(|error| panic!("data object fixture should parse cleanly: {error}"));
    let process = package
        .find_process("data_flow")
        .unwrap_or_else(|| panic!("data object fixture should expose process"));
    assert_eq!(process.data_object_bindings.len(), 2);

    let data_store = lint_fixture("metadata-data-state.bpmn");
    let issue = single_issue(&data_store, "bpmn.unsupported_data_surface");
    assert_eq!(issue.evidence["snapshot"]["data_store_count"], 1);

    let callable_io = lint_fixture("metadata-callable-io.bpmn");
    let issue = single_issue(&callable_io, "bpmn.unsupported_collaboration_surface");
    assert_eq!(
        issue.evidence["snapshot"]["process_callable"]["global_task_io_specification_count"],
        1
    );

    let resource_role = lint_fixture("metadata-resource-role.bpmn");
    let issue = single_issue(&resource_role, "bpmn.unsupported_resource_role_metadata");
    assert_eq!(
        issue.evidence["snapshot"]["resource_roles"]["process_role_count"],
        2
    );
    assert!(issue.why_it_failed.contains("resource-parameter binding"));

    let flow_element_metadata = lint_fixture("metadata-flow-element.bpmn");
    let issue = single_issue(
        &flow_element_metadata,
        "bpmn.unsupported_flow_element_metadata",
    );
    assert_eq!(
        issue.evidence["snapshot"]["flow_element_metadata"]["element_count"],
        3
    );
    assert_eq!(
        issue.evidence["snapshot"]["flow_element_metadata"]["category_value_ref_count"],
        3
    );
    assert!(issue.why_it_failed.contains("monitoring telemetry"));

    assert_bpmn_di_boundary_evidence();
}

fn coverage_matrix_rows() -> Vec<(&'static str, &'static str)> {
    let mut rows = Vec::new();
    let mut in_matrix = false;

    for line in COVERAGE_DOC.lines() {
        if line == "## Coverage Matrix" {
            in_matrix = true;
            continue;
        }
        if in_matrix && line.starts_with("## ") {
            break;
        }
        if !in_matrix || !line.starts_with('|') {
            continue;
        }
        if line.contains("---") || line.contains("BPMN family") {
            continue;
        }

        let columns = line
            .split('|')
            .map(str::trim)
            .filter(|column| !column.is_empty())
            .collect::<Vec<_>>();
        if columns.len() < 2 {
            continue;
        }
        rows.push((columns[0], columns[1]));
    }

    rows
}

fn assert_boundary(family: &str, status: BpmnConformanceStatus) {
    let entry = registry_entry(family);
    assert_eq!(entry.status, status, "unexpected status for {family}");
}

fn registry_entry(family: &str) -> &'static BpmnConformanceEntry {
    for entry in bpmn_conformance_registry() {
        if entry.family == family {
            return entry;
        }
    }
    panic!("missing conformance registry entry for {family}");
}

fn lint_fixture(name: &str) -> LintReport {
    lint_bpmn_source(&fixture_source(name))
}

fn single_issue<'a>(report: &'a LintReport, code: &str) -> &'a LintIssue {
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, code);
    issue
}

fn assert_bpmn_di_boundary_evidence() {
    assert_bpmn_di_metadata_boundary_evidence();
    assert_bpmn_di_namespace_boundary_evidence();
    assert_bpmn_di_boolean_boundary_evidence();
    assert_bpmn_di_numeric_boundary_evidence();
    assert_bpmn_di_enum_boundary_evidence();
    assert_bpmn_di_topology_boundary_evidence();
    assert_bpmn_di_anchor_boundary_evidence();
    assert_bpmn_di_reference_boundary_evidence();
    assert_bpmn_di_identity_boundary_evidence();
    assert_bpmn_di_completeness_boundary_evidence();
}

fn assert_bpmn_di_metadata_boundary_evidence() {
    let diagram = lint_fixture("metadata-bpmn-diagram.bpmn");
    let issue = single_issue(&diagram, "bpmn.metadata_di_surface");
    assert_eq!(issue.evidence["snapshot"]["diagram_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["diagrams"][0]["shape_count"], 2);
    assert_eq!(issue.evidence["snapshot"]["diagrams"][0]["edge_count"], 1);

    let snapshot = snapshot_bpmn_source(&fixture_source("metadata-bpmn-diagram.bpmn"))
        .unwrap_or_else(|error| panic!("diagram fixture should snapshot cleanly: {error}"));
    assert_eq!(snapshot.root.diagram_count, 1);
}

fn assert_bpmn_di_namespace_boundary_evidence() {
    let invalid_di_namespace = lint_fixture("invalid-di-bpmndi-namespace.bpmn");
    let issue = single_issue(&invalid_di_namespace, "bpmn.invalid_di_namespace");
    assert_eq!(
        issue.evidence["invalid_namespaces"][0]["expected_namespace_uri"],
        "http://www.omg.org/spec/BPMN/20100524/DI"
    );
}

fn assert_bpmn_di_boolean_boundary_evidence() {
    let invalid_di_boolean = lint_fixture("invalid-di-boolean-values.bpmn");
    let issue = single_issue(&invalid_di_boolean, "bpmn.invalid_di_boolean");
    assert_eq!(
        issue.evidence["invalid_booleans"][0]["attribute"],
        "isHorizontal"
    );
    assert_eq!(issue.evidence["invalid_booleans"][0]["value"], "yes");
    assert_eq!(issue.evidence["invalid_booleans"][2]["attribute"], "isBold");
    assert_eq!(issue.evidence["invalid_booleans"][2]["value"], "sometimes");
}

fn assert_bpmn_di_numeric_boundary_evidence() {
    let invalid_di_numeric = lint_fixture("invalid-di-numeric-values.bpmn");
    let issue = single_issue(&invalid_di_numeric, "bpmn.invalid_di_numeric");
    assert_eq!(
        issue.evidence["invalid_numerics"][0]["attribute"],
        "resolution"
    );
    assert_eq!(issue.evidence["invalid_numerics"][0]["value"], "dense");
    assert_eq!(issue.evidence["invalid_numerics"][2]["attribute"], "width");
    assert_eq!(issue.evidence["invalid_numerics"][2]["value"], "NaN");
    assert_eq!(issue.evidence["invalid_numerics"][4]["attribute"], "size");
    assert_eq!(issue.evidence["invalid_numerics"][4]["value"], "huge");
}

fn assert_bpmn_di_enum_boundary_evidence() {
    let invalid_di_enum = lint_fixture("invalid-di-enum-values.bpmn");
    let issue = single_issue(&invalid_di_enum, "bpmn.invalid_di_enum");
    assert_eq!(
        issue.evidence["invalid_enums"][0]["attribute"],
        "participantBandKind"
    );
    assert_eq!(issue.evidence["invalid_enums"][0]["value"], "top_primary");
    assert_eq!(
        issue.evidence["invalid_enums"][1]["attribute"],
        "messageVisibleKind"
    );
    assert_eq!(issue.evidence["invalid_enums"][1]["value"], "both");
}

fn assert_bpmn_di_reference_boundary_evidence() {
    let invalid_di_reference = lint_fixture("invalid-di-reference.bpmn");
    let issue = single_issue(&invalid_di_reference, "bpmn.invalid_di_reference");
    assert_eq!(issue.evidence["invalid_reference_count"], 1);
    assert_eq!(
        issue.evidence["invalid_references"][0]["reference"],
        "missing_review"
    );

    let invalid_di_edge = lint_fixture("invalid-di-edge-reference.bpmn");
    let issue = single_issue(&invalid_di_edge, "bpmn.invalid_di_reference");
    assert_eq!(
        issue.evidence["invalid_references"][0]["attribute"],
        "sourceElement"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["reference"],
        "Missing_StartShape"
    );

    let invalid_di_label_style = lint_fixture("invalid-di-label-style-reference.bpmn");
    let issue = single_issue(&invalid_di_label_style, "bpmn.invalid_di_reference");
    assert_eq!(
        issue.evidence["invalid_references"][0]["attribute"],
        "labelStyle"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["reference"],
        "Missing_LabelStyle"
    );

    let invalid_choreography_shape = lint_fixture("invalid-di-choreography-shape-reference.bpmn");
    let issue = single_issue(&invalid_choreography_shape, "bpmn.invalid_di_reference");
    assert_eq!(
        issue.evidence["invalid_references"][0]["attribute"],
        "choreographyActivityShape"
    );
    assert_eq!(
        issue.evidence["invalid_references"][0]["reference"],
        "Missing_ChoreographyShape"
    );
}

fn assert_bpmn_di_topology_boundary_evidence() {
    let missing_di_plane = lint_fixture("invalid-di-missing-plane.bpmn");
    let issue = single_issue(&missing_di_plane, "bpmn.invalid_di_plane_topology");
    assert_eq!(
        issue.evidence["invalid_topology"][0]["reason"],
        "missing_direct_plane"
    );

    let multiple_di_planes = lint_fixture("invalid-di-multiple-planes.bpmn");
    let issue = single_issue(&multiple_di_planes, "bpmn.invalid_di_plane_topology");
    assert_eq!(
        issue.evidence["invalid_topology"][0]["reason"],
        "multiple_direct_planes"
    );

    let orphan_di_plane = lint_fixture("invalid-di-orphan-plane.bpmn");
    let issue = single_issue(&orphan_di_plane, "bpmn.invalid_di_plane_topology");
    assert_eq!(
        issue.evidence["invalid_topology"][0]["reason"],
        "plane_outside_diagram"
    );
}

fn assert_bpmn_di_anchor_boundary_evidence() {
    let missing_di_plane_anchor = lint_fixture("invalid-di-plane-missing-anchor.bpmn");
    let issue = single_issue(&missing_di_plane_anchor, "bpmn.missing_di_semantic_anchor");
    assert_eq!(issue.evidence["missing_anchors"][0]["element"], "BPMNPlane");

    let missing_di_shape_anchor = lint_fixture("invalid-di-shape-missing-anchor.bpmn");
    let issue = single_issue(&missing_di_shape_anchor, "bpmn.missing_di_semantic_anchor");
    assert_eq!(issue.evidence["missing_anchors"][0]["element"], "BPMNShape");

    let missing_di_edge_anchor = lint_fixture("invalid-di-edge-missing-anchor.bpmn");
    let issue = single_issue(&missing_di_edge_anchor, "bpmn.missing_di_semantic_anchor");
    assert_eq!(issue.evidence["missing_anchors"][0]["element"], "BPMNEdge");

    let invalid_di_shape_anchor_kind = lint_fixture("invalid-di-shape-anchor-kind.bpmn");
    let issue = single_issue(&invalid_di_shape_anchor_kind, "bpmn.invalid_di_anchor_kind");
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["actual_semantic_tag"],
        "sequenceFlow"
    );

    let invalid_di_edge_anchor_kind = lint_fixture("invalid-di-edge-anchor-kind.bpmn");
    let issue = single_issue(&invalid_di_edge_anchor_kind, "bpmn.invalid_di_anchor_kind");
    assert_eq!(
        issue.evidence["invalid_anchor_kinds"][0]["actual_semantic_tag"],
        "serviceTask"
    );
}

fn assert_bpmn_di_identity_boundary_evidence() {
    let duplicate_di_shape = lint_fixture("invalid-di-duplicate-shape-id.bpmn");
    let issue = single_issue(&duplicate_di_shape, "bpmn.duplicate_di_id");
    assert_eq!(
        issue.evidence["duplicate_di_ids"][0]["duplicate_id"],
        "Shape_Duplicate"
    );

    let duplicate_di_label_style = lint_fixture("invalid-di-duplicate-label-style-id.bpmn");
    let issue = single_issue(&duplicate_di_label_style, "bpmn.duplicate_di_id");
    assert_eq!(
        issue.evidence["duplicate_di_ids"][0]["duplicate_id"],
        "Style_Duplicate"
    );
}

fn assert_bpmn_di_completeness_boundary_evidence() {
    let incomplete_di_shape = lint_fixture("invalid-di-shape-missing-bounds.bpmn");
    let issue = single_issue(&incomplete_di_shape, "bpmn.incomplete_di_surface");
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["missing"],
        "dc:Bounds"
    );

    let incomplete_di_edge = lint_fixture("invalid-di-edge-missing-waypoints.bpmn");
    let issue = single_issue(&incomplete_di_edge, "bpmn.incomplete_di_surface");
    assert_eq!(
        issue.evidence["incomplete_surfaces"][0]["missing"],
        "di:waypoint[2]"
    );
}
