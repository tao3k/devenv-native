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
        assert!(
            !entry.docs_anchor.to_ascii_lowercase().contains("flowable")
                && !entry.docs_anchor.to_ascii_lowercase().contains("spiff"),
            "docs anchor must stay on canonical package docs for {}",
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
    assert_boundary("Global task catalogs", BpmnConformanceStatus::MetadataOnly);
    assert_boundary("Callable IO metadata", BpmnConformanceStatus::MetadataOnly);
    assert_boundary("BPMN DI", BpmnConformanceStatus::MetadataOnly);
}

#[test]
fn conformance_boundary_evidence_is_covered_by_lint_or_snapshot() {
    let complex_gateway = lint_fixture("invalid-unsupported-gateway.bpmn");
    let issue = single_issue(&complex_gateway, "bpmn.unsupported_element");
    assert!(issue.llm_fix_prompt.contains("complexGateway"));

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

    let diagram = lint_fixture("metadata-bpmn-diagram.bpmn");
    let issue = single_issue(&diagram, "bpmn.metadata_di_surface");
    assert_eq!(issue.evidence["snapshot"]["diagram_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["diagrams"][0]["shape_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["diagrams"][0]["edge_count"], 1);

    let snapshot = snapshot_bpmn_source(&fixture_source("metadata-bpmn-diagram.bpmn"))
        .unwrap_or_else(|error| panic!("diagram fixture should snapshot cleanly: {error}"));
    assert_eq!(snapshot.root.diagram_count, 1);
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
