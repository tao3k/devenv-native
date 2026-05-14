use super::{
    SearchStrategyFlowCandidateInputBatch,
    run_wendaograph_search_strategy_flow_json_with_candidate_batch,
    run_wendaograph_search_strategy_flow_json_with_candidate_batch_and_branch_judgements,
};

#[test]
fn search_strategy_flow_rust_bridge_reserves_required_evidence_frontier() {
    let candidate_rows = [
        "packages/rust/crates/xiuxian-wendao-julia/README.md\t76-searchstrategyflow-flight-materialization-now-keeps-route-namespaces\tSearchStrategyFlow Flight materialization\t76\t90\t8\t1.0\t1.0\t0.97\t0.98\t0.02\tfalse\tsearch-strategy,authority",
        "docs/rfcs/2026-03-26-wendao-query-engine-rfc.md\t735-the-goal-is-not-to-freeze-an-api-immediately-the-goal-is-to-establish-the-ownership-boundary-between\tOwnership boundary\t735\t760\t8\t0.96\t0.95\t0.93\t0.92\t0.05\tfalse\tauthority,ownership",
        "docs/testing/README.md\t89-default-validation-path-both-local-just-validate-and-just-ci\tDefault validation path\t89\t105\t8\t0.95\t0.94\t0.90\t0.91\t0.05\tfalse\tvalidation,package-test",
        "packages/rust/crates/xiuxian-wendao-julia/tests/unit/integration_support/wendaograph/search_strategy.rs\t77-search-strategy-flow-link-graph-python-julia-toml\tSearch strategy flow LinkGraph path\t77\t120\t8\t0.94\t0.93\t0.88\t0.90\t0.06\tfalse\tlink-graph,relation",
    ];
    let batch = SearchStrategyFlowCandidateInputBatch {
        source: "rust-code-intelligence-inventory",
        row_count: candidate_rows.len(),
        tsv: candidate_rows.join("\n"),
        discovery_receipt_json: serde_json::json!({
            "receiptSource": "rust-code-intelligence-inventory",
            "candidateInputSource": "rust-code-intelligence-inventory",
            "candidateInputCount": candidate_rows.len(),
            "transport": "unit",
            "route": "unit-required-evidence-frontier",
            "attemptCount": 1,
            "mergedCandidateCount": candidate_rows.len()
        })
        .to_string(),
    };

    let trace = run_wendaograph_search_strategy_flow_json_with_candidate_batch(
        "find the SearchStrategyFlow ownership boundary and validation path",
        ".",
        batch,
    )
    .unwrap_or_else(|error| panic!("run required evidence frontier bridge trace: {error}"));
    let trace: serde_json::Value = serde_json::from_str(&trace)
        .unwrap_or_else(|error| panic!("parse required evidence frontier trace: {error}"));
    let validation = trace
        .get("validation")
        .unwrap_or_else(|| panic!("validation object must exist"));

    assert_eq!(
        validation
            .get("requiredEvidenceCovered")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        validation.get("selectedRequiredEvidence"),
        Some(&serde_json::json!([
            "ownership_boundary",
            "validation_path",
            "relation_path"
        ]))
    );
    assert_eq!(
        validation.get("missingRequiredEvidence"),
        Some(&serde_json::json!([]))
    );
    assert!(
        trace
            .get("frontier")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .any(|row| {
                row.get("selected").and_then(serde_json::Value::as_bool) == Some(true)
                    && row
                        .get("candidateId")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|candidate_id| {
                            candidate_id
                                .starts_with("docs/testing/README.md#89-default-validation-path")
                        })
            }),
        "validation-path candidate must be selected by the required-evidence frontier"
    );
}

#[test]
fn search_strategy_flow_rust_bridge_applies_agent_branch_judgements() {
    let candidate_rows = [
        "docs/a.md\towner\tOwner branch\t1\t12\t8\t0.88\t0.80\t0.60\t0.80\t0.10\tfalse\tgeneral",
        "docs/b.md\tvalidation\tValidation branch\t13\t24\t8\t0.86\t0.78\t0.60\t0.78\t0.10\tfalse\tgeneral",
        "docs/c.md\trelation\tRelation branch\t25\t36\t8\t0.84\t0.76\t0.60\t0.76\t0.10\tfalse\tgeneral",
        "docs/d.md\tblocked\tBlocked branch\t37\t48\t8\t0.90\t0.90\t0.90\t0.90\t0.02\tfalse\tgeneral",
    ];
    let batch = SearchStrategyFlowCandidateInputBatch {
        source: "rust-code-intelligence-inventory",
        row_count: candidate_rows.len(),
        tsv: candidate_rows.join("\n"),
        discovery_receipt_json: serde_json::json!({
            "receiptSource": "rust-code-intelligence-inventory",
            "candidateInputSource": "rust-code-intelligence-inventory",
            "candidateInputCount": candidate_rows.len(),
            "transport": "unit",
            "route": "unit-branch-judgement-frontier",
            "attemptCount": 1,
            "mergedCandidateCount": candidate_rows.len()
        })
        .to_string(),
    };
    let branch_judgements = [
        "pi-wendao-search-strategy-flow\tdocs/a.md#owner\tauthority\t0.950000\t0.900000\tkeep\tfalse\tAgent judged ownership boundary evidence.",
        "pi-wendao-search-strategy-flow\tdocs/b.md#validation\tvalidation\t0.940000\t0.900000\tkeep\tfalse\tAgent judged validation path evidence.",
        "pi-wendao-search-strategy-flow\tdocs/c.md#relation\tlink_graph\t0.930000\t0.900000\tkeep\tfalse\tAgent judged relation path evidence.",
        "pi-wendao-search-strategy-flow\tdocs/d.md#blocked\tsearch_strategy\t0.100000\t0.900000\treject\ttrue\tAgent rejected this branch.",
    ]
    .join("\n");

    let trace =
        run_wendaograph_search_strategy_flow_json_with_candidate_batch_and_branch_judgements(
            "find the SearchStrategyFlow ownership boundary and validation path",
            ".",
            batch,
            &branch_judgements,
        )
        .unwrap_or_else(|error| panic!("run branch judgement frontier bridge trace: {error}"));
    let trace: serde_json::Value = serde_json::from_str(&trace)
        .unwrap_or_else(|error| panic!("parse branch judgement frontier trace: {error}"));
    let selected_ids = trace
        .get("frontier")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| row.get("selected").and_then(serde_json::Value::as_bool) == Some(true))
        .filter_map(|row| row.get("candidateId").and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>();

    assert!(selected_ids.contains(&"docs/a.md#owner"));
    assert!(selected_ids.contains(&"docs/b.md#validation"));
    assert!(selected_ids.contains(&"docs/c.md#relation"));
    assert!(!selected_ids.contains(&"docs/d.md#blocked"));
    assert_eq!(
        trace
            .get("validation")
            .and_then(|validation| validation.get("requiredEvidenceCovered"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
}
