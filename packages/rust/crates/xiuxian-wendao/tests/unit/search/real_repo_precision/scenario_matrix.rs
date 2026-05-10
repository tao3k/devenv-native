use crate::search::real_repo_precision::{
    RealRepoGoldQueryKind, RealRepoKnowledgeScenario,
    RealRepoKnowledgeScenarioAuthorityExpectation, RealRepoKnowledgeScenarioKind,
    RealRepoKnowledgeScenarioQueryVariant, RealRepoKnowledgeScenarioQueryVariantKind,
    RealRepoKnowledgeScenarioReceipt, RealRepoMarkdownKnowledgeSemanticGateReceipt,
    RealRepoMarkdownKnowledgeSemanticRelationPathReceipt, RealRepoPrecisionQueryReceipt,
    evaluate_knowledge_scenario_matrix,
};

#[test]
fn scenario_matrix_records_path_semantic_relation_and_authority_evidence() {
    let relation = relation_path(
        "decision.semantic-ssot.repo-native-first",
        "governs",
        "component.wendao.query-substrate",
    );
    let scenario = RealRepoKnowledgeScenario {
        id: "authority-repo-native-semantic-ssot".to_string(),
        kind: RealRepoKnowledgeScenarioKind::AuthorityOrdering,
        intent: "Prefer semantic SSOT authority.".to_string(),
        linked_query_ids: vec!["semantic-decision-repo-native-authority".to_string()],
        query_variants: query_variants(&[(
            "semantic-decision-repo-native-authority",
            RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
        )]),
        required_paths: vec![
            "semantic/objects/decision/semantic-ssot-repo-native-first.md".to_string(),
        ],
        required_semantic_object_ids: vec!["decision.semantic-ssot.repo-native-first".to_string()],
        required_relation_paths: vec![relation.clone()],
        authority: Some(RealRepoKnowledgeScenarioAuthorityExpectation {
            preferred_path: "semantic/objects/decision/semantic-ssot-repo-native-first.md"
                .to_string(),
            competing_paths: vec!["packages/rust/crates/xiuxian-wendao/README.md".to_string()],
        }),
        forbidden_paths: Vec::new(),
    };
    let query_receipts = vec![query_receipt(
        "semantic-decision-repo-native-authority",
        &[
            "semantic/objects/decision/semantic-ssot-repo-native-first.md",
            "packages/rust/crates/xiuxian-wendao/README.md",
        ],
        true,
    )];
    let gate = semantic_gate(
        &["decision.semantic-ssot.repo-native-first"],
        std::slice::from_ref(&relation),
    );

    let receipts = evaluate_knowledge_scenario_matrix(&[scenario], &query_receipts, Some(&gate));

    let receipt = &receipts[0];
    assert_authority_receipt_core(receipt, &relation);
    assert_reasoning_tree_receipt(receipt, &relation);
    assert_backend_frontier_receipt(receipt);
    assert_authority_order_receipt(receipt);
}

fn assert_authority_receipt_core(
    receipt: &RealRepoKnowledgeScenarioReceipt,
    relation: &RealRepoMarkdownKnowledgeSemanticRelationPathReceipt,
) {
    assert!(receipt.passed);
    assert_eq!(receipt.scenario_kind, "authority_ordering");
    assert_eq!(receipt.query_variant_count, 1);
    assert_eq!(receipt.failed_query_variant_count, 0);
    assert_eq!(receipt.intent_frame.task_kind, "authority_ordering");
    assert!(receipt.intent_frame.verifier_required);
    assert!(
        receipt
            .intent_frame
            .anchor_terms
            .contains(&"semantic".to_string())
    );
    assert!(
        receipt
            .intent_frame
            .required_evidence_kinds
            .contains(&"relation_path".to_string())
    );
    assert_eq!(
        receipt.intent_frame.relation_hypotheses,
        vec![relation.clone()]
    );
    assert_eq!(receipt.required_path_recall_bps, 10_000);
    assert_eq!(receipt.mean_required_path_reciprocal_rank_bps, 10_000);
    assert_eq!(receipt.best_required_path_rank, Some(0));
    assert_eq!(receipt.covered_required_path_count, 1);
}

fn assert_reasoning_tree_receipt(
    receipt: &RealRepoKnowledgeScenarioReceipt,
    relation: &RealRepoMarkdownKnowledgeSemanticRelationPathReceipt,
) {
    assert!(receipt.reasoning_tree.passed);
    assert_eq!(
        receipt.reasoning_tree.strategy,
        "graph_first_progressive_disclosure_v1"
    );
    assert_eq!(receipt.reasoning_tree.anchor_count, 1);
    assert_eq!(receipt.reasoning_tree.relation_step_count, 1);
    assert_eq!(receipt.reasoning_tree.page_index_step_count, 1);
    assert_eq!(receipt.reasoning_tree.source_step_count, 1);
    assert_eq!(receipt.reasoning_tree.max_disclosure_depth, 2);
    assert!(receipt.reasoning_tree.steps.iter().any(|step| {
        step.step_kind == "semantic_relation"
            && step.relation.as_ref() == Some(relation)
            && step.disclosure_depth == 1
            && step.passed
    }));
    assert!(receipt.reasoning_tree.steps.iter().any(|step| {
        step.step_kind == "source_evidence"
            && step.path.as_deref()
                == Some("semantic/objects/decision/semantic-ssot-repo-native-first.md")
            && step.zero_based_rank == Some(0)
            && step.passed
    }));
}

fn assert_backend_frontier_receipt(receipt: &RealRepoKnowledgeScenarioReceipt) {
    assert_eq!(
        receipt.backend_frontier.strategy,
        "rust_controlled_backend_frontier_v1"
    );
    assert_eq!(receipt.backend_frontier.control_plane_owner, "rust");
    assert_eq!(receipt.backend_frontier.graph_backend, "rust-baseline");
    assert_eq!(receipt.backend_frontier.expand_node_count, 1);
    assert_eq!(receipt.backend_frontier.subagent_judgement_node_count, 1);
    assert_eq!(receipt.backend_frontier.julia_candidate_node_count, 4);
    assert_eq!(receipt.backend_frontier.julia_dispatch_node_count, 4);
    assert_eq!(
        receipt.backend_frontier.strategy_flow_refinement_topology,
        "cyclic_refinement_allowed"
    );
    assert_eq!(
        receipt.backend_frontier.strategy_flow_frontier_node_count,
        receipt.reasoning_tree.disclosure_step_count
    );
    assert!(receipt.backend_frontier.nodes.iter().any(|node| {
        node.evidence_kind == "relation_path"
            && node.julia_algorithm_id.as_deref()
                == Some("relationship_search.ppr_like_relatedness")
            && node.julia_schedule_action.as_deref() == Some("dispatch")
    }));
}

fn assert_authority_order_receipt(receipt: &RealRepoKnowledgeScenarioReceipt) {
    assert_eq!(
        receipt.covered_semantic_object_ids,
        vec!["decision.semantic-ssot.repo-native-first".to_string()]
    );
    assert_eq!(receipt.covered_relation_paths.len(), 1);
    let authority = receipt
        .authority
        .as_ref()
        .unwrap_or_else(|| panic!("authority receipt should be present"));
    assert_eq!(authority.preferred_rank, Some(0));
    assert_eq!(authority.earliest_competing_rank, Some(1));
    assert!(authority.passed);
}

#[test]
fn scenario_matrix_fails_negative_guard_when_forbidden_path_is_observed() {
    let scenario = RealRepoKnowledgeScenario {
        id: "negative-llm-output-authority-guard".to_string(),
        kind: RealRepoKnowledgeScenarioKind::NegativeEvidence,
        intent: "Do not use package README as semantic authority.".to_string(),
        linked_query_ids: vec!["semantic-invariant-llm-output-not-authority".to_string()],
        query_variants: query_variants(&[(
            "semantic-invariant-llm-output-not-authority",
            RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
        )]),
        required_paths: vec![
            "semantic/objects/invariant/llm-output-is-not-authority.md".to_string(),
        ],
        required_semantic_object_ids: Vec::new(),
        required_relation_paths: Vec::new(),
        authority: None,
        forbidden_paths: vec!["packages/rust/crates/xiuxian-wendao/README.md".to_string()],
    };
    let query_receipts = vec![query_receipt(
        "semantic-invariant-llm-output-not-authority",
        &[
            "semantic/objects/invariant/llm-output-is-not-authority.md",
            "packages/rust/crates/xiuxian-wendao/README.md",
        ],
        true,
    )];

    let receipts = evaluate_knowledge_scenario_matrix(&[scenario], &query_receipts, None);

    let receipt = &receipts[0];
    assert!(!receipt.passed);
    assert_eq!(receipt.failure_reasons, vec!["negative_guard_failed"]);
    assert!(receipt.reasoning_tree.passed);
    assert_eq!(receipt.reasoning_tree.anchor_count, 1);
    assert_eq!(receipt.reasoning_tree.source_step_count, 1);
    assert!(
        receipt
            .backend_frontier
            .nodes
            .iter()
            .any(|node| node.evidence_kind == "negative_guard" && node.backend_action == "prune")
    );
    assert!(
        receipt
            .backend_frontier
            .nodes
            .iter()
            .filter(|node| node.evidence_kind == "negative_guard")
            .all(|node| node.julia_algorithm_id.is_none())
    );
    assert_eq!(receipt.backend_frontier.subagent_fanout_node_count, 1);
    let negative_guard = receipt
        .negative_guard
        .as_ref()
        .unwrap_or_else(|| panic!("negative guard receipt should be present"));
    assert_eq!(
        negative_guard.matched_forbidden_paths,
        vec!["packages/rust/crates/xiuxian-wendao/README.md".to_string()]
    );
}

#[test]
fn scenario_matrix_reports_missing_query_and_partial_recall() {
    let scenario = RealRepoKnowledgeScenario {
        id: "agent-task-polyglot-page-index-boundary".to_string(),
        kind: RealRepoKnowledgeScenarioKind::AgentTask,
        intent: "Gather boundary evidence.".to_string(),
        linked_query_ids: vec![
            "docs-polyglot-compute-orchestrator-rfc".to_string(),
            "wendao-page-index-reasoning".to_string(),
        ],
        query_variants: query_variants(&[
            (
                "docs-polyglot-compute-orchestrator-rfc",
                RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
            ),
            (
                "wendao-page-index-reasoning",
                RealRepoKnowledgeScenarioQueryVariantKind::Canonical,
            ),
        ]),
        required_paths: vec![
            "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md".to_string(),
            "packages/rust/crates/xiuxian-wendao/README.md".to_string(),
        ],
        required_semantic_object_ids: Vec::new(),
        required_relation_paths: Vec::new(),
        authority: None,
        forbidden_paths: Vec::new(),
    };
    let query_receipts = vec![query_receipt(
        "docs-polyglot-compute-orchestrator-rfc",
        &["docs/rfcs/2026-05-04-polyglot-compute-orchestrator-rfc.md"],
        true,
    )];

    let receipts = evaluate_knowledge_scenario_matrix(&[scenario], &query_receipts, None);

    let receipt = &receipts[0];
    assert!(!receipt.passed);
    assert_eq!(receipt.covered_required_path_count, 1);
    assert_eq!(receipt.required_path_recall_bps, 5_000);
    assert_eq!(receipt.required_path_recall_at_1_bps, 5_000);
    assert_eq!(receipt.required_path_recall_at_3_bps, 5_000);
    assert_eq!(receipt.mean_required_path_reciprocal_rank_bps, 5_000);
    assert_eq!(receipt.best_required_path_rank, Some(0));
    assert!(!receipt.reasoning_tree.passed);
    assert_eq!(receipt.reasoning_tree.anchor_count, 1);
    assert_eq!(receipt.reasoning_tree.source_step_count, 1);
    assert!(
        receipt
            .reasoning_tree
            .failure_reasons
            .contains(&"missing_source_evidence".to_string())
    );
    assert_eq!(
        receipt.missing_paths,
        vec!["packages/rust/crates/xiuxian-wendao/README.md".to_string()]
    );
    assert!(
        receipt
            .query_evidence
            .iter()
            .any(
                |evidence| evidence.query_id == "wendao-page-index-reasoning"
                    && evidence.failure_reason.as_deref() == Some("query receipt missing")
            )
    );
    assert_eq!(receipt.query_variant_count, 2);
    assert_eq!(receipt.failed_query_variant_count, 1);
    assert!(
        receipt
            .failure_reasons
            .contains(&"query_variants_failed".to_string())
    );
    assert!(
        receipt
            .failure_reasons
            .contains(&"reasoning_tree_failed".to_string())
    );
}

fn query_receipt(
    query_id: &str,
    observed_paths: &[&str],
    passed: bool,
) -> RealRepoPrecisionQueryReceipt {
    RealRepoPrecisionQueryReceipt {
        query_id: query_id.to_string(),
        query_kind: RealRepoGoldQueryKind::LinkGraph.as_str().to_string(),
        query: query_id.to_string(),
        limit: 10,
        query_ms: 1,
        passed,
        must_hit_paths: Vec::new(),
        missing_paths: Vec::new(),
        required_top_path: None,
        observed_top_path: observed_paths.first().map(|path| (*path).to_string()),
        required_path_ranks: observed_paths
            .iter()
            .take(1)
            .map(|path| crate::search::real_repo_precision::types::RealRepoPrecisionRequiredPathRankReceipt {
                path: (*path).to_string(),
                zero_based_rank: Some(0),
            })
            .collect(),
        required_path_recall_at_1_bps: if observed_paths.is_empty() { 0 } else { 10_000 },
        required_path_recall_at_3_bps: if observed_paths.is_empty() { 0 } else { 10_000 },
        required_path_recall_at_5_bps: if observed_paths.is_empty() { 0 } else { 10_000 },
        required_path_recall_at_10_bps: if observed_paths.is_empty() { 0 } else { 10_000 },
        mean_required_path_reciprocal_rank_bps: if observed_paths.is_empty() { 0 } else { 10_000 },
        best_required_path_rank: (!observed_paths.is_empty()).then_some(0),
        observed_paths: observed_paths
            .iter()
            .map(|path| (*path).to_string())
            .collect(),
    }
}

fn semantic_gate(
    object_ids: &[&str],
    relation_paths: &[RealRepoMarkdownKnowledgeSemanticRelationPathReceipt],
) -> RealRepoMarkdownKnowledgeSemanticGateReceipt {
    RealRepoMarkdownKnowledgeSemanticGateReceipt {
        schema: "test".to_string(),
        semantic_root: "semantic".to_string(),
        linked_query_ids: Vec::new(),
        required_markdown_paths: Vec::new(),
        covered_markdown_paths: Vec::new(),
        required_relation_paths: relation_paths.to_vec(),
        covered_relation_paths: relation_paths.to_vec(),
        knowledge_scenarios: Vec::new(),
        semantic_object_ids: object_ids
            .iter()
            .map(|object_id| (*object_id).to_string())
            .collect(),
        semantic_scope_object_count: object_ids.len(),
        semantic_scope_relation_count: relation_paths.len(),
        page_index_node_count: 0,
        page_index_edge_count: 0,
        page_index_seed_count: 0,
        required_validation_count: 0,
    }
}

fn relation_path(
    source: &str,
    kind: &str,
    target: &str,
) -> RealRepoMarkdownKnowledgeSemanticRelationPathReceipt {
    RealRepoMarkdownKnowledgeSemanticRelationPathReceipt {
        source: source.to_string(),
        kind: kind.to_string(),
        target: target.to_string(),
    }
}

fn query_variants(
    variants: &[(&str, RealRepoKnowledgeScenarioQueryVariantKind)],
) -> Vec<RealRepoKnowledgeScenarioQueryVariant> {
    variants
        .iter()
        .map(|(query_id, kind)| RealRepoKnowledgeScenarioQueryVariant {
            query_id: (*query_id).to_string(),
            kind: *kind,
        })
        .collect()
}
